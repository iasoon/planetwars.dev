import adapter from "@sveltejs/adapter-auto";
import preprocess from "svelte-preprocess";
import { viteCommonjs } from "@originjs/vite-plugin-commonjs";
import wasmPack from "vite-plugin-wasm-pack";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    adapter: adapter(),

    // hydrate the <div id="svelte"> element in src/app.html
    target: "#svelte",
    ssr: false,
    vite: {
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
          "/api/": "http://localhost:9000",
          "/ws": "ws://localhost:9000/ws",
        },
        fs: {
          // Allow serving files from one level up to the project root
          allow: [".."],
        },
      },
    },
  },
};

export default config;
