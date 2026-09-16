import { mount } from "svelte";
import MutationOverlay from "./MutationOverlay.svelte";

const app = mount(MutationOverlay, {
  target: document.getElementById("app")!,
});

export default app;
