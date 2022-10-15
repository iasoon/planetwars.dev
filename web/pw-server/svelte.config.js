import adapter from "@sveltejs/adapter-node";
import sveltePreprocess from "svelte-preprocess";
import { viteCommonjs } from "@originjs/vite-plugin-commonjs";
import wasmPack from "vite-plugin-wasm-pack";
import { isoImport } from "vite-plugin-iso-import";
import { mdsvex } from "mdsvex";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: [
    sveltePreprocess(),
    mdsvex({
      extensions: [".md"],
      layout: {
        docs: "src/routes/docs/doc.svelte",
      },
    }),
  ],
  extensions: [".svelte", ".md"],
  kit: {
    adapter: adapter(),

    // hydrate the <div id="svelte"> element in src/app.html
    // target: "#svelte",
    vite: {
      plugins: [
        isoImport(),
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
          "/api/": "http://127.0.0.1:9000",
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
