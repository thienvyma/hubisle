import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

// GitHub draft assets have temporary untagged/API URLs. Resolve only assets
// owned by this draft, then use their public URL after the tag is published.
export function prepareManifest(manifest, release, version) {
  assert.equal(release.isDraft, true, "Only prepare a draft release");
  assert.equal(release.tagName, "v" + version);
  assert.equal(manifest.version.replace(/^v/, ""), version);
  const result = structuredClone(manifest);
  for (const platform of Object.values(result.platforms)) {
    const asset = release.assets.find(item => item.apiUrl === platform.url || item.url === platform.url);
    assert.ok(asset, "Manifest refers to an asset outside this draft");
    const api = new URL(asset.apiUrl);
    assert.equal(api.origin, "https://api.github.com");
    assert.ok(/^\/repos\/thienvyma\/hubisle\/releases\/assets\/\d+$/.test(api.pathname));
    assert.ok(/\.exe$/i.test(asset.name), "Expected a Windows installer");
    assert.ok(release.assets.some(item => item.name === asset.name + ".sig"), "Signature asset missing");
    platform.url = "https://github.com/thienvyma/hubisle/releases/download/v" + version + "/" + encodeURIComponent(asset.name);
  }
  return result;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [manifestPath, releasePath] = process.argv.slice(2);
  const config = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
  const result = prepareManifest(JSON.parse(readFileSync(manifestPath, "utf8")), JSON.parse(readFileSync(releasePath, "utf8")), config.version);
  writeFileSync(manifestPath, JSON.stringify(result, null, 2) + "\n");
  console.log("Prepared public updater URLs for v" + config.version);
}
