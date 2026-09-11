import test from "node:test";
import assert from "node:assert/strict";
import { prepareManifest, releaseNotes } from "./prepare-updater-manifest.mjs";

const apiUrl = "https://api.github.com/repos/thienvyma/hubisle/releases/assets/123";
const release = { isDraft: true, tagName: "v2.1.0", assets: [
  { name: "Isle.Pulse.Overlay_2.1.0_x64-setup.exe", apiUrl, url: "https://github.com/thienvyma/hubisle/releases/download/untagged-123/setup.exe" },
  { name: "Isle.Pulse.Overlay_2.1.0_x64-setup.exe.sig" },
] };
const manifest = {version: "2.1.0", platforms: {"windows-x86_64": {url: apiUrl, signature: "signed-installer"}}};
test("updater notes include only the published version and require nonempty notes", () => {
  const changelog = "# Changes\r\n## [2.1.1] — today\r\n\r\n- Era fix\r\n\r\n## [2.1.0]\r\n- Old notes\r\n";
  assert.equal(releaseNotes(changelog, "2.1.1"), "- Era fix");
  assert.equal(releaseNotes(changelog, "2.1.0"), "- Old notes");
  assert.throws(() => releaseNotes(changelog, "2.2.0"));
  assert.throws(() => releaseNotes("## [2.1.1]\n\n## [2.1.0]\nold", "2.1.1"));
});
test("draft API URL becomes a stable public download without changing signature", () => {
  const result = prepareManifest(manifest, release, "2.1.0");
  assert.equal(result.platforms["windows-x86_64"].url, "https://github.com/thienvyma/hubisle/releases/download/v2.1.0/Isle.Pulse.Overlay_2.1.0_x64-setup.exe");
  assert.equal(result.platforms["windows-x86_64"].signature, "signed-installer");
  assert.equal(manifest.platforms["windows-x86_64"].url, apiUrl);
});
test("temporary browser URL is also resolved from the draft", () => {
  const input = structuredClone(manifest);
  input.platforms["windows-x86_64"].url = release.assets[0].url;
  assert.match(prepareManifest(input, release, "2.1.0").platforms["windows-x86_64"].url, /\/v2.1.0\//);
});
test("unknown assets, wrong repository, missing signature and published releases fail closed", () => {
  const foreign = structuredClone(manifest);
  foreign.platforms["windows-x86_64"].url = "https://example.com/setup.exe";
  assert.throws(() => prepareManifest(foreign, release, "2.1.0"));
  const otherRepo = structuredClone(release);
  otherRepo.assets[0].apiUrl = apiUrl.replace("thienvyma/hubisle", "someone/else");
  const otherManifest = structuredClone(manifest);
  otherManifest.platforms["windows-x86_64"].url = otherRepo.assets[0].apiUrl;
  assert.throws(() => prepareManifest(otherManifest, otherRepo, "2.1.0"));
  assert.throws(() => prepareManifest(manifest, {...release, isDraft: false}, "2.1.0"));
  assert.throws(() => prepareManifest(manifest, {...release, assets: [release.assets[0]]}, "2.1.0"));
  assert.throws(() => prepareManifest(manifest, {...release, tagName: "v2.0.2"}, "2.1.0"));
});
