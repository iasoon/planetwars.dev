<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import * as visualizer from "pw-visualizer";
  import init_wasm_module from "planetwars-rs";

  export let matchLog = null;
  export let matchData: object; // match object as returned by api

  let initialized = false;

  const PLAYER_COLORS = [
    "#ff8000",
    "#0080ff",
    "#ff6693",
    "#3fcb55",
    "#cbc33f",
    "#cf40e9",
    "#ff3f0d",
    "#1beef0",
    "#0dc5ff",
  ];

  onMount(async () => {
    await init_wasm_module();

    visualizer.init();
    initialized = true;
    visualizer.set_loading(false);
  });

  onDestroy(() => {
    // TODO: do a more thorough cleanup
    visualizer.stop();
  });

  $: if (initialized) {
    if (matchLog === null) {
      visualizer.set_loading(true);
    } else {
      console.log(matchLog);
      let instanceLog = extractGameStates(matchLog);
      visualizer.set_instance(instanceLog);
      visualizer.set_loading(false);
    }
  }

  function extractGameStates(matchLog: string): string {
    // TODO: find a better way to do this
    return matchLog
      .split("\n")
      .slice(0, -1)
      .filter((line) => JSON.parse(line)["type"] == "gamestate")
      .join("\n");
  }
</script>

<div id="main" class="loading">
  <canvas id="canvas" />
  <div id="name" />
  <ul class="player-labels">
    {#each matchData["players"] as player, i}
      <li style="color:{PLAYER_COLORS[i]}">{player["bot_name"] || "player"}</li>
    {/each}
  </ul>

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

  .player-labels {
    position: absolute;
    top: 10px;
    right: 10px;
    color: white;
    list-style: none;
  }

  .player-labels li {
    margin-bottom: 0.5em;
  }
</style>
