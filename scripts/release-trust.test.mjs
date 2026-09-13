import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";

const root = new URL("../", import.meta.url);
const text = (path) => readFileSync(new URL(path, root), "utf8");
const json = (path) => JSON.parse(text(path));

test("project ownership and GPL metadata are consistent", () => {
  const pkg = json("package.json");
  const worker = json("worker/package.json");
  const cargo = text("src-tauri/Cargo.toml");
  const core = text("src-tauri/crates/overlay-core/Cargo.toml");
  const sidecar = text("sidecars/local-telemetry/islemap-thienvyma-telemetry.csproj");
  const tauri = json("src-tauri/tauri.conf.json");

  assert.equal(pkg.author, "thienvyma");
  assert.equal(pkg.license, "GPL-3.0-only");
  assert.equal(worker.license, "GPL-3.0-only");
  assert.match(cargo, /^authors\s*=\s*\["thienvyma"\]/m);
  assert.match(cargo, /^license\s*=\s*"GPL-3\.0-only"/m);
  assert.match(core, /^license\s*=\s*"GPL-3\.0-only"/m);
  assert.match(sidecar, /<Authors>thienvyma<\/Authors>/);
  assert.match(sidecar, /<Company>thienvyma<\/Company>/);
  assert.match(sidecar, /<Product>islemap-thienvyma<\/Product>/);
  const escapedVersion = pkg.version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  assert.match(sidecar, new RegExp(`<Version>${escapedVersion}<\\/Version>`));
  assert.equal(tauri.bundle.publisher, "thienvyma");
  assert.equal(tauri.bundle.license, "GPL-3.0-only");
  assert.equal(tauri.bundle.licenseFile, "../INSTALLER_NOTICE.txt");
  assert.match(text("LICENSE"), /GNU GENERAL PUBLIC LICENSE[\s\S]*Version 3/);
});

test("repository documents the signing and privacy policy required for OSS signing", () => {
  const policy = text("CODE_SIGNING_POLICY.md");
  const privacy = text("PRIVACY.md");
  const readme = text("README.md");
  const required = "Free code signing provided by SignPath.io, certificate by SignPath Foundation";

  assert.match(policy, new RegExp(required.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  assert.match(policy, /Committers and reviewers/i);
  assert.match(policy, /Approvers/i);
  assert.match(privacy, /Cloudflare/i);
  assert.match(privacy, /Npcap/i);
  assert.match(privacy, /DPAPI/i);
  assert.match(readme, /CODE_SIGNING_POLICY\.md/);
  assert.match(readme, /PRIVACY\.md/);
});

test("release input contains only project-owned bundled artwork", () => {
  for (const path of [
    "src/assets/dino-viewer-bg.jpg",
    "src/assets/guide-dino-1.jpg",
    "src/assets/guide-dino-2.jpg",
    "src/assets/guide-dino-3.jpg",
    "src/assets/qr_donate.png",
  ]) {
    assert.equal(existsSync(new URL(path, root)), false, `${path} must not be bundled`);
  }
  assert.doesNotMatch(text("src/lib/dino3d/DinoViewer3D.svelte"), /official overlay|copied verbatim/i);
  assert.doesNotMatch(text("src/lib/dino3d/registry.ts"), /extracted VERBATIM|official overlay/i);
  assert.doesNotMatch(text("src/lib/dino3d/skin.ts"), /official overlay/i);
});

test("release workflow verifies Authenticode before publishing when enforcement is enabled", () => {
  const workflow = text(".github/workflows/release.yml");
  const authenticode = text("scripts/check-authenticode.ps1");
  assert.match(workflow, /check-authenticode\.ps1/i);
  assert.match(workflow, /SIGNPATH_ENFORCE_AUTHENTICODE/);
  assert.match(workflow, /without trusted Windows Authenticode/);
  assert.match(authenticode, /Status\s+-ne\s+['"]Valid['"]/i);
  assert.match(authenticode, /TimeStamperCertificate/);
  assert.match(authenticode, /SignPath Foundation/);
  assert.ok(
    workflow.indexOf("check-authenticode.ps1") < workflow.indexOf("--draft=false"),
    "Authenticode must be verified before a release is made public",
  );
});

test("SignPath handoff documents every required repository setting", () => {
  const setup = text("docs/SIGNPATH_SETUP.md");
  for (const name of [
    "SIGNPATH_API_TOKEN",
    "SIGNPATH_ORGANIZATION_ID",
    "SIGNPATH_PROJECT_SLUG",
    "SIGNPATH_SIGNING_POLICY_SLUG",
    "SIGNPATH_APP_ARTIFACT_CONFIGURATION_SLUG",
    "SIGNPATH_INSTALLER_ARTIFACT_CONFIGURATION_SLUG",
    "SIGNPATH_ENFORCE_AUTHENTICODE",
  ]) {
    assert.match(setup, new RegExp(name));
  }
  assert.match(setup, /Tauri updater `\.sig`/);
});
