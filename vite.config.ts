/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import packageJson from "./package.json";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
const pagesBase = "/ObsidianOS/";
const appVersion = packageJson.version;
const appVersionDisplay = `V.${appVersion.replace(/(^|[-.])([a-z])/g, (_, prefix, letter) => `${prefix}${letter.toUpperCase()}`)}`;
const appCodename = "Bixbite";
const appProductName = "ObsidianOS";
const appCopyright = "Copyright (c) Max Staneker";

// https://vite.dev/config/
export default defineConfig(async ({ mode }) => ({
  base: mode === "tauri" ? "./" : pagesBase,
  build: {
    outDir: mode === "tauri" ? "dist" : "docs",
    emptyOutDir: true,
  },
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
    __APP_VERSION_DISPLAY__: JSON.stringify(appVersionDisplay),
    __APP_CODENAME__: JSON.stringify(appCodename),
    __APP_PRODUCT_NAME__: JSON.stringify(appProductName),
    __APP_COPYRIGHT__: JSON.stringify(appCopyright),
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1430,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1431,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
