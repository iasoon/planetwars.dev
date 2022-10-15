<script lang="ts">
  import { parsePlayerLog, PlayerLog } from "$lib/log_parser";

  export let matchLog: string;
  export let playerId: number;

  let playerLog: PlayerLog;

  let showRawStderr = false;

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

  $: if (matchLog) {
    playerLog = parsePlayerLog(playerId, matchLog);
  } else {
    playerLog = [];
  }
</script>

<div class="output">
  {#if showRawStderr}
    <div class="output-text stderr-text">
      {playerLog.flatMap((turn) => turn.stderr).join("\n")}
    </div>
  {:else}
    <div class="output-text">
      {#each playerLog as logTurn, i}
        <div class="turn">
          <div class="turn-header">
            <span class="turn-header-text">Turn {i}</span>
            {#if logTurn.action?.type === "dispatches"}
              {pluralize(logTurn.action.dispatches.length, "dispatch")}
            {:else if logTurn.action?.type === "timeout"}
              <span class="turn-error">timeout</span>
            {:else if logTurn.action?.type === "bad_command"}
              <span class="turn-error">invalid command</span>
            {/if}
          </div>
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
      {/each}
    </div>
  {/if}
</div>

<style lang="scss">
  .output {
    background-color: rgb(41, 41, 41);
  }

  .turn {
    margin: 16px 4px;
  }

  .output-text {
    color: #ccc;
  }

  .turn-header {
    display: flex;
    justify-content: space-between;
  }

  .turn-header-text {
    color: #eee;
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
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
