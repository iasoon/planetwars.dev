<script lang="ts">
  export let matchLog: string;

  function getStdErr(log: string, botId: number): string {
    let output = [];
    log
      .split("\n")
      .slice(0, -1)
      .forEach((line) => {
        let message = JSON.parse(line);
        if (message["type"] === "stderr" && message["player_id"] === botId) {
          output.push(message["message"]);
        }
      });
    return output.join("\n");
  }

  $: botStdErr = getStdErr(matchLog, 1);
</script>

<div class="output">
  {#if botStdErr.length > 0}
    <h3 class="output-header">stderr:</h3>
    <div class="output-text">
      {botStdErr}
    </div>
  {/if}
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
    font-family: monospace;
    white-space: pre-wrap;
  }

  .output-header {
    color: #eee;
    padding-bottom: 20px;
  }
</style>
