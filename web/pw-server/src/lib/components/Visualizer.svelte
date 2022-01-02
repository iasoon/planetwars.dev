<script lang="ts">
  import { onMount } from "svelte";
  import * as visualizer from "pw-visualizer";
  import init_wasm_module from "planetwars-rs";

  export let matchLog = null;

  let initialized = false;

  onMount(async () => {
    await init_wasm_module();

    visualizer.init();
    initialized = true;
    visualizer.set_loading(false);
  });

  $: if (initialized) {
    if (matchLog === null) {
      visualizer.set_loading(true);
    } else {
      visualizer.set_instance(matchLog);
      visualizer.set_loading(false);
    }
  }
</script>

<div id="main" class="loading">
  <canvas id="canvas" />
  <div id="name" />
  <div id="addbutton" class="button" />

  <div id="meta">
    <div id="turnCounter">0 / 0</div>
    <div>
      <span>Ms per frame:&nbsp;</span>
      <input type="number" id="speed" value="300" />
    </div>
    <div class="slidecontainer">
      <input type="range" min="0" max="1" value="1" class="slider" id="turnSlider" />
    </div>
  </div>
  <div class="lds-roller">
    <div />
    <div />
    <div />
    <div />
    <div />
    <div />
    <div />
    <div />
  </div>
</div>

<style scoped>
  @import "pw-visualizer/src/style.css";
</style>
