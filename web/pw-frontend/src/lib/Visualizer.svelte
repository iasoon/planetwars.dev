<script lang="ts">
  import { onMount } from 'svelte';
  import * as visualizer from '../lib/visualizer/index';

  onMount(() => {
    visualizer.init();
    fetch("match.log")
      .then(response => response.text())
      .then(data => visualizer.set_instance(data));
  });
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
      <input
        type="range"
        min="0"
        max="1"
        value="1"
        class="slider"
        id="turnSlider"
      />
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
  @import 'visualizer/style.css';
</style>
