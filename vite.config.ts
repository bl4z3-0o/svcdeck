import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

// Vite-Konfiguration fuer Tauri: fester Port, keine Bildschirm-Leerung, damit
// Rust-Fehlermeldungen im Terminal sichtbar bleiben.
export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: { $lib: fileURLToPath(new URL("./src/lib", import.meta.url)) },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Rust-Quellen nicht beobachten, die baut Tauri selbst neu
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    // WebKitGTK aktueller Arch-Versionen kann modernes JavaScript
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
  },
});
