import test from "node:test";
import assert from "node:assert/strict";
import { get } from "svelte/store";
import { createUpdateController } from "../src/lib/update-controller.ts";

function deferred() {
  let resolve;
  const promise = new Promise(r => { resolve = r; });
  return { promise, resolve };
}
function update(version = "2.0.3", downloadAndInstall = async () => {}) {
  return { version, body: "Release notes", closed: 0, async close() { this.closed++; }, downloadAndInstall };
}
test("automatic checks stay quiet when current or unreachable; manual checks distinguish both", async () => {
  let failed = false;
  const c = createUpdateController(async () => { if (failed) throw Error("404"); return null; });
  await c.checkNow();
  assert.equal(get(c.state).visible, false);
  await c.checkNow(true);
  assert.equal(get(c.state).status, "current");
  assert.equal(get(c.state).visible, true);
  failed = true;
  await c.checkNow();
  assert.equal(get(c.state).visible, false);
  await c.checkNow(true);
  assert.equal(get(c.state).status, "check-error");
  assert.equal(get(c.state).visible, true);
  await c.dispose();
});
test("manual check joins an in-flight automatic check", async () => {
  const pending = deferred();
  let calls = 0;
  const c = createUpdateController(() => { calls++; return pending.promise; });
  const task = c.checkNow();
  await c.checkNow(true);
  pending.resolve(null);
  await task;
  assert.equal(calls, 1);
  assert.equal(get(c.state).visible, true);
  assert.equal(get(c.state).status, "current");
});
test("Later silences this version, a newer release notifies, manual check reopens it", async () => {
  let version = "2.0.3";
  const handles = [];
  const c = createUpdateController(async () => { const h = update(version); handles.push(h); return h; });
  await c.checkNow();
  assert.equal(get(c.state).visible, true);
  c.dismiss();
  await c.checkNow();
  assert.equal(get(c.state).visible, false);
  assert.equal(handles[0].closed, 1);
  await c.checkNow(true);
  assert.equal(get(c.state).visible, true);
  c.dismiss();
  version = "2.0.4";
  await c.checkNow();
  assert.equal(get(c.state).visible, true);
  await c.dispose();
  assert.ok(handles.every(h => h.closed === 1));
});
test("check result arriving after teardown is closed without publishing", async () => {
  const pending = deferred();
  const h = update();
  const c = createUpdateController(() => pending.promise);
  const task = c.checkNow();
  await c.dispose();
  pending.resolve(h);
  await task;
  assert.equal(h.closed, 1);
  assert.equal(get(c.state).version, null);
});
test("unknown download size stays indeterminate and install blocks duplicate actions", async () => {
  const pending = deferred();
  let downloads = 0, checks = 0;
  let progress;
  const h = update("2.0.3", async cb => { downloads++; progress = cb; await pending.promise; });
  const c = createUpdateController(async () => { checks++; return h; });
  await c.checkNow();
  const task = c.install();
  progress({ event: "Started", data: {} });
  progress({ event: "Progress", data: { chunkLength: 50 } });
  assert.equal(get(c.state).progress, null);
  await c.install();
  await c.checkNow(true);
  c.dismiss();
  assert.equal(get(c.state).visible, true);
  assert.equal(downloads, 1);
  assert.equal(checks, 1);
  progress({ event: "Finished" });
  assert.equal(get(c.state).status, "installing");
  pending.resolve();
  await task;
  await c.dispose();
});
test("failed download can retry with progress reset, then launch installer", async () => {
  let attempts = 0;
  const h = update("2.0.3", async cb => {
    attempts++;
    cb({ event: "Started", data: { contentLength: 100 } });
    cb({ event: "Progress", data: { chunkLength: 50 } });
    assert.equal(get(c.state).progress, 50);
    if (attempts === 1) throw Error("network");
    cb({ event: "Finished" });
  });
  const c = createUpdateController(async () => h);
  await c.checkNow();
  await c.install();
  assert.equal(get(c.state).status, "install-error");
  assert.equal(get(c.state).progress, null);
  await c.install();
  assert.equal(attempts, 2);
  assert.equal(get(c.state).status, "installing");
  await c.dispose();
});
