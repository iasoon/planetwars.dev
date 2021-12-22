import App from './App.svelte'
import load_wasm_module from "planetwars-rs";

load_wasm_module().then(() => {
  const app = new App({
    target: document.getElementById('app')
  })
});

