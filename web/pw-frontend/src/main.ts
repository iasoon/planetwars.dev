import App from './App.svelte'
import init_wasm_module from "planetwars-rs";

init_wasm_module().then(() => {
  const app = new App({
    target: document.getElementById('app')
  })
})