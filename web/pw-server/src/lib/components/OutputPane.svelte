<script lang="ts">
  import PlayerLog from "./log_viewer/PlayerLog.svelte";

  export let matchData: object;
  export let matchLog: string;
</script>

<div class="output-pane">
  <h3 class="output-header">Player log</h3>
  <PlayerLog {matchData} {matchLog} playerId={1} />
</div>

<style lang="scss">
  .output-pane {
    width: 100%;
    overflow-y: scroll;
    background-color: rgb(41, 41, 41);
    padding: 15px;
  }

  .output-header {
    color: #eee;
    padding-bottom: 20px;
  }
</style>
