import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { extname, join } from "node:path";
import test from "node:test";

const root = new URL("../", import.meta.url);
const atRoot = (path) => new URL(path, root);

function sourceFiles(path) {
  const absolute = atRoot(path);
  if (!existsSync(absolute)) return [];
  const pending = [absolute];
  const files = [];
  while (pending.length) {
    const current = pending.pop();
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      const next = join(entry.parentPath, entry.name);
      if (entry.isDirectory()) pending.push(next);
      else if ([".rs", ".ts", ".svelte"].includes(extname(entry.name))) files.push(next);
    }
  }
  return files;
}

const activeFiles = [
  ...sourceFiles("src/main/voice"),
  ...sourceFiles("src-tauri/src/voice"),
  atRoot("src/main/App.svelte"),
  atRoot("src/lib/api.ts"),
  atRoot("src-tauri/src/lib.rs"),
].filter(existsSync);

const voiceCopy = ["src/lib/i18n/en.ts", "src/lib/i18n/vi.ts"]
  .flatMap((path) => readFileSync(atRoot(path), "utf8").split(/\r?\n/))
  .filter((line) => line.includes('"voice.'))
  .join("\n");
const activeSource = activeFiles.map((path) => readFileSync(path, "utf8")).join("\n") + voiceCopy;

test("Voice uses the generic provider surface and has no external-client bridge", () => {
  assert.ok(existsSync(atRoot("src/main/voice/VoiceTab.svelte")), "generic Voice tab is required");
  assert.ok(!existsSync(atRoot("src-tauri/src/dinovoice.rs")), "old executable bridge must be removed");
  assert.ok(!existsSync(atRoot("src/main/voice/DinoVoiceTab.svelte")), "old service-specific tab must be removed");
  for (const forbidden of [
    "dinovoice",
    "DinoVietnam",
    "DinoVietNam.exe",
    "DINOVN_VOICE_TOKEN",
    "x-api-key",
    "ricktanker/dinovietnam-overlay",
  ]) {
    assert.ok(!activeSource.toLowerCase().includes(forbidden.toLowerCase()), `forbidden Voice dependency: ${forbidden}`);
  }
});

test("Voice frontend exposes normalized provider states without grant secrets", () => {
  const api = readFileSync(atRoot("src/lib/api.ts"), "utf8");
  for (const state of [
    "not-configured",
    "login-required",
    "authorizing",
    "ready",
    "unavailable",
    "blocked",
    "error",
  ]) {
    assert.ok(api.includes(`\"${state}\"`), `missing Voice state: ${state}`);
  }
  assert.ok(api.includes('invoke<VoiceStatus>("voice_status")'));
  assert.ok(!api.includes('invoke<VoiceGrant>("voice_request_grant")'), "grant tokens stay in Rust");
});
