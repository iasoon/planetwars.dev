<script lang="ts">
  import type { PlayerLogTurn } from "$lib/log_parser";
  import Fa from "svelte-fa";
  import { faAngleRight, faAngleDown } from "@fortawesome/free-solid-svg-icons";

  export let turnNum: number;
  export let logTurn: PlayerLogTurn;
  let expanded = false;

  const PLURAL_MAP = {
    dispatch: "dispatches",
    ship: "ships",
  };

  function pluralize(num: number, word: string): string {
    if (num == 1) {
      return `1 ${word}`;
    } else {
      return `${num} ${PLURAL_MAP[word]}`;
    }
  }

  function toggleExpand() {
    expanded = !expanded;
  }
</script>

<div class="turn">
  <div class="turn-header" on:click={toggleExpand}>
    <span>
      <span class="turn-header-icon">
        {#if expanded}
          <Fa icon={faAngleDown} />
        {:else}
          <Fa icon={faAngleRight} />
        {/if}
      </span>
      <span class="turn-header-text">
        Turn {turnNum}
      </span>
    </span>
    {#if logTurn.action?.type === "dispatches"}
      {pluralize(logTurn.action.dispatches.length, "dispatch")}
    {:else if logTurn.action?.type === "timeout"}
      <span class="turn-error">timeout</span>
    {:else if logTurn.action?.type === "bad_command"}
      <span class="turn-error">invalid command</span>
    {/if}
  </div>
  {#if expanded}
    <div class="turn-content">
      {#if logTurn.action?.type === "dispatches"}
        <div class="dispatches-container">
          {#each logTurn.action.dispatches as dispatch}
            <div class="dispatch">
              <div class="dispatch-text">
                {pluralize(dispatch.ship_count, "ship")} from {dispatch.origin} to {dispatch.destination}
              </div>
              {#if dispatch.error}
                <span class="dispatch-error">{dispatch.error}</span>
              {/if}
            </div>
          {/each}
        </div>
      {:else if logTurn.action?.type === "bad_command"}
        <div class="bad-command-container">
          <div class="bad-command-text">{logTurn.action.command}</div>
          <div class="bad-command-error">Parse error: {logTurn.action.error}</div>
        </div>
      {/if}
      {#if logTurn.stderr.length > 0}
        <div class="stderr-header">stderr</div>
        <div class="stderr-text-box">
          {#each logTurn.stderr as stdErrMsg}
            <div class="stderr-text">{stdErrMsg}</div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style lang="scss">
  .turn {
    // padding: 4px 2px;
    color: #ccc;
  }

  .turn-header-icon {
    // padding-right: 0.2em;
    display: inline-block;
    width: 1em;
  }

  .turn-header {
    display: flex;
    justify-content: space-between;
  }

  .turn-header:hover {
    cursor: pointer;
    background-color: #333;
  }

  .turn-header-text {
    color: #eee;
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
  }

  .turn-content {
    margin-bottom: 12px;
  }

  .turn-error {
    color: red;
  }

  .dispatch {
    display: flex;
    justify-content: space-between;
  }

  .dispatch-error {
    color: red;
  }

  .bad-command-container {
    border-left: 1px solid red;
    margin-left: 4px;
    padding-left: 8px;
  }

  .bad-command-text {
    font-family: "Consolas", "Bitstream Vera Sans Mono", "Courier New", Courier, monospace;
    padding-bottom: 4px;
  }

  .bad-command-error {
    color: whitesmoke;
  }

  .stderr-text {
    // font-family: monospace;
    font-family: "Consolas", "Bitstream Vera Sans Mono", "Courier New", Courier, monospace;
    white-space: pre-wrap;
  }

  .stderr-header {
    color: #eee;
    padding-top: 4px;
  }

  .stderr-text-box {
    border-left: 1px solid #ccc;
    margin-left: 4px;
    padding-left: 8px;
  }
</style>
