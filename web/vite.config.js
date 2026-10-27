import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  base: "./",
  plugins: [vue()],
  build: {
    outDir: "dist",
    assetsInlineLimit: 0
  },
  clearScreen: false,
  server: {
    port: 5183,
    strictPort: true
  }
});
