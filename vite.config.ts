import { defineConfig } from 'vite'
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { svelte } from '@sveltejs/vite-plugin-svelte';
import pkg from './package.json';

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte(), wasm(), topLevelAwait()],
  base: "/gb-web/",
  publicDir: "app/public",
  define: {
    "import.meta.env.PACKAGE_VERSION": JSON.stringify(pkg.version)
  }
})
