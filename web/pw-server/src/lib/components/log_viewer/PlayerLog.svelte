<script lang="ts">
  import { parsePlayerLog, PlayerLog } from "$lib/log_parser";
  import LogTurn from "./LogTurn.svelte";

  export let matchLog: string;
  export let playerId: number;

  let playerLog: PlayerLog;
  let showRawStderr = false;

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
    <!-- The log should be rerendered when playerId changes -->
    {#key playerId}
    <div class="log-contents">
      {#each playerLog as logTurn, turnNum}
        <LogTurn {logTurn} {turnNum} />
      {/each}
    </div>
    {/key}
  {/if}
</div>

<style lang="scss">
  .output {
    background-color: rgb(41, 41, 41);
  }
</style>
