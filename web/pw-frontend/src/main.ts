import App from './App.svelte'
import init_wasm_module from "planetwars-rs";

const wasm_url = new URL("../planetwars-rs/pkg/planetwars_rs_bg.wasm", import.meta.url)

init_wasm_module(wasm_url).then(() => {
  const app = new App({
    target: document.getElementById('app')
  })
})