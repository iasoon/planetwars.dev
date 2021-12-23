import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { viteCommonjs } from '@originjs/vite-plugin-commonjs'
import wasmPack from 'vite-plugin-wasm-pack';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    svelte(),
    wasmPack(["./planetwars-rs"]),
    viteCommonjs({
      transformMixedEsModules: true,
    }),
  ],
  build: {
    commonjsOptions: {
      transformMixedEsModules: true,
    },
    minify: false,
    target: "modules",
  },
})
