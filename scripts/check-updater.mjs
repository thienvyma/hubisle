import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const root = new URL("../", import.meta.url);
const json = (path) => JSON.parse(readFileSync(new URL(path, root), "utf8"));
const repo = "https://github.com/thienvyma/hubisle";
const pkg = json("package.json");
const config = json("src-tauri/tauri.conf.json");
const capabilities = json("src-tauri/capabilities/default.json");
assert.equal(pkg.homepage, repo);
assert.equal(pkg.repository.url, "git+" + repo + ".git");
assert.equal(config.version, pkg.version);
assert.deepEqual(config.plugins.updater.endpoints, [repo + "/releases/latest/download/latest.json"]);
assert.ok(config.plugins.updater.pubkey?.trim(), "Updater public signing key is required");
assert.equal(config.bundle.createUpdaterArtifacts, true);
assert.ok(config.bundle.targets.includes("nsis"));
for (const permission of ["updater:allow-check", "updater:allow-download-and-install"]) {
  assert.ok(capabilities.permissions.includes(permission), "Missing " + permission);
}
if (process.env.GITHUB_REPOSITORY) {
  assert.equal(process.env.GITHUB_REPOSITORY, "thienvyma/hubisle", "Wrong release repository");
}
if (process.argv[2]) {
  const manifest = JSON.parse(readFileSync(process.argv[2], "utf8"));
  assert.equal(manifest.version.replace(/^v/, ""), pkg.version);
  const platform = manifest.platforms?.["windows-x86_64"];
  assert.ok(platform, "Missing windows-x86_64 update");
  const url = new URL(platform.url);
  assert.equal(url.origin, "https://github.com");
  assert.ok(url.pathname.startsWith("/thienvyma/hubisle/releases/download/v" + pkg.version + "/"), "Installer must belong to this release");
  assert.ok(/\.exe$/i.test(decodeURIComponent(url.pathname)), "Expected NSIS installer");
  assert.ok(typeof platform.signature === "string" && platform.signature.trim().length > 0, "Missing updater signature");
}
console.log("Updater configuration passed: thienvyma/hubisle " + pkg.version);
