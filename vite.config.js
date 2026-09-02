import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [sveltekit(), tailwindcss()],
  clearScreen: false,
  build: {
    target: "esnext",
    reportCompressedSize: false,
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  test: {
    // Pure logic only — the units under test take plain values, so there is
    // no DOM to stand up. `resolve()` reads four fields off a KeyboardEvent,
    // which a literal covers.
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
}));
