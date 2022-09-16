<script lang="ts">
  import { parsePlayerLog, PlayerLog } from "$lib/log_parser";

  export let matchLog: string;
  let playerLog: PlayerLog;

  $: if (matchLog) {
    playerLog = parsePlayerLog(1, matchLog);
  } else {
    playerLog = [];
  }
</script>

<div class="output">
  <h3 class="output-header">Player log</h3>
  <div class="output-text stderr-text">
    {playerLog.flatMap((turn) => turn.stderr).join("\n")}
  </div>
</div>

<style lang="scss">
  .output {
    width: 100%;
    overflow-y: scroll;
    background-color: rgb(41, 41, 41);
    padding: 15px;
  }

  .output-text {
    color: #ccc;
  }

  .stderr-text {
    font-family: monospace;
    white-space: pre-wrap;
  }

  .output-header {
    color: #eee;
    padding-bottom: 20px;
  }
</style>
