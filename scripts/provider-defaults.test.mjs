import { test } from "node:test";
import assert from "node:assert/strict";
import {
  DEFAULT_PROVIDER_WEBSITE,
  connectionWebsite,
} from "../src/lib/provider-defaults.ts";

test("a fresh install starts with the central IslePilot website", () => {
  assert.equal(DEFAULT_PROVIDER_WEBSITE, "https://islepilot.eu");
  assert.equal(connectionWebsite(null), "https://islepilot.eu");
  assert.equal(connectionWebsite("   "), "https://islepilot.eu");
});

test("an existing provider website is preserved", () => {
  assert.equal(
    connectionWebsite("https://eragamingvn.net"),
    "https://eragamingvn.net",
  );
});
