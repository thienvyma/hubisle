import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath } from "node:url";
import pkg from "./package.json";

// Overlay entries stay separate from the main app so their WebViews remain
// small and event-driven while the game is running.
export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  define: {
    // Compile-time so the footer needs no IPC (and no permission) to show it.
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
  build: {
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL("./index.html", import.meta.url)),
        minimap: fileURLToPath(new URL("./minimap.html", import.meta.url)),
        mutationOverlay: fileURLToPath(new URL("./mutation-overlay.html", import.meta.url)),
      },
    },
  },
  // Tauri dev server conventions.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Cargo creates and locks executables under src-tauri/target while Tauri
    // dev is running. Vite must not try to watch those Windows build outputs;
    // doing so can fail with EBUSY and terminate beforeDevCommand.
    watch: {
      ignored: ["**/src-tauri/target/**"],
    },
  },
})
