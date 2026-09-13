import { test } from "node:test";
import assert from "node:assert/strict";
import { runGaragePark } from "../src/main/garage/park-flow.ts";

test("delayed IslePilot parking runs start, countdown, finalize, then status poll", async () => {
  const calls = [];
  const progress = [];
  await runGaragePark({
    start: async () => {
      calls.push("start");
      return { pending: true, delaySec: 2, commandId: null };
    },
    finalize: async () => {
      calls.push("finalize");
      return "command-7";
    },
    cancel: async () => calls.push("cancel"),
    wait: async (id) => calls.push(`wait:${id}`),
    sleep: async (ms) => calls.push(`sleep:${ms}`),
    isCancelled: () => false,
    onProgress: (state) => progress.push(state),
  });

  assert.deepEqual(calls, [
    "start",
    "sleep:1000",
    "sleep:1000",
    "finalize",
    "wait:command-7",
  ]);
  assert.deepEqual(
    progress.map(({ phase, remainingSec }) => [phase, remainingSec]),
    [
      ["starting", 0],
      ["countdown", 2],
      ["countdown", 1],
      ["finalizing", 0],
      ["polling", 0],
      ["done", 0],
    ],
  );
});

test("an immediate command id skips finalize and polls directly", async () => {
  const calls = [];
  await runGaragePark({
    start: async () => ({ pending: false, delaySec: 0, commandId: 81 }),
    finalize: async () => {
      calls.push("finalize");
      return "unused";
    },
    cancel: async () => calls.push("cancel"),
    wait: async (id) => calls.push(`wait:${id}`),
    sleep: async () => {},
    isCancelled: () => false,
    onProgress: () => {},
  });
  assert.deepEqual(calls, ["wait:81"]);
});

test("cancel during the countdown tells the server and never finalizes", async () => {
  const calls = [];
  let cancelled = false;
  const result = await runGaragePark({
    start: async () => ({ pending: true, delaySec: 3, commandId: null }),
    finalize: async () => {
      calls.push("finalize");
      return "unused";
    },
    cancel: async () => calls.push("cancel"),
    wait: async (id) => calls.push(`wait:${id}`),
    sleep: async () => {
      cancelled = true;
    },
    isCancelled: () => cancelled,
    onProgress: () => {},
  });
  assert.equal(result.cancelled, true);
  assert.deepEqual(calls, ["cancel"]);
});

test("finalize retries temporary connection failures", async () => {
  let attempts = 0;
  const sleeps = [];
  await runGaragePark({
    start: async () => ({ pending: true, delaySec: 1, commandId: null }),
    finalize: async () => {
      attempts += 1;
      if (attempts < 3) throw new Error("connection reset");
      return "command-after-retry";
    },
    cancel: async () => {},
    wait: async () => {},
    sleep: async (ms) => sleeps.push(ms),
    isCancelled: () => false,
    onProgress: () => {},
  });
  assert.equal(attempts, 3);
  assert.deepEqual(sleeps, [1000, 1200, 1200]);
});
