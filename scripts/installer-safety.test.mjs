import { readFileSync } from "node:fs";
import { test } from "node:test";
import assert from "node:assert/strict";

const hooks = readFileSync(
  new URL("../src-tauri/windows/installer-hooks.nsh", import.meta.url),
  "utf8",
);

function macro(name) {
  const match = new RegExp(`!macro ${name}\\s+([\\s\\S]*?)!macroend`).exec(hooks);
  assert.ok(match, `${name} must exist`);
  return match[1];
}

test("preinstall never deletes the currently installed executable", () => {
  const preinstall = macro("NSIS_HOOK_PREINSTALL");
  assert.doesNotMatch(preinstall, /Delete[^\r\n]*islemap-thienvyma\.exe/i);
  assert.doesNotMatch(preinstall, /Delete[^\r\n]*islemap-thienvyma-telemetry\.exe/i);
});

test("legacy cleanup happens only after the new executable is present", () => {
  const preinstall = macro("NSIS_HOOK_PREINSTALL");
  const postinstall = macro("NSIS_HOOK_POSTINSTALL");
  assert.doesNotMatch(preinstall, /Isle Pulse Overlay/i);
  assert.match(postinstall, /FileExists[^\r\n]*islemap-thienvyma\.exe/i);
  assert.match(postinstall, /Delete[^\r\n]*theisle-overlay\.exe/i);
  assert.match(postinstall, /RMDir \/r \/REBOOTOK[^\r\n]*Isle Pulse Overlay/i);
});
