import { defineConfig } from 'vite'
import { viteCommonjs } from '@originjs/vite-plugin-commonjs'
import wasmPack from 'vite-plugin-wasm-pack';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    wasmPack([], ["planetwars-rs"]),
    viteCommonjs({
      transformMixedEsModules: true,
    }),
  ],
  build: {
    commonjsOptions: {
      transformMixedEsModules: true,
    },
  },
  server: {
    proxy: {
      "/api/": "http://localhost:5000",
      "/ws": "ws://localhost:5000/ws",
    },
  },
})
