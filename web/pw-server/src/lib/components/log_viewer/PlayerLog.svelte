<script lang="ts">
  import { parsePlayerLog, PlayerLog } from "$lib/log_parser";
  import LogTurn from "./LogTurn.svelte";

  export let matchLog: string;
  export let matchData: object;
  export let playerId: number;

  let playerLog: PlayerLog;
  let showRawStderr = false;

  async function copyTurn(turnNum: number) {
    // find state for turnNum
    let gamestate = matchLog
      .split("\n")
      .slice(0, -1)
      .map((line) => JSON.parse(line))
      .filter((json) => json["type"] == "gamestate")
      .at(turnNum);

    let numPlayers = matchData["players"].length;
    let rotatePlayerNum = (playerNum: number | null) => {
      if (playerNum === null) {
        return null;
      }
      return ((numPlayers + playerNum - playerId) % numPlayers) + 1;
    };

    gamestate["planets"].forEach((planet) => {
      planet["owner"] = rotatePlayerNum(planet["owner"]);
    });
    gamestate["expeditions"].forEach((expedition) => {
      expedition["owner"] = rotatePlayerNum(expedition["owner"]);
    });

    await navigator.clipboard.writeText(JSON.stringify(gamestate));
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
    <!-- The log should be rerendered when playerId changes -->
    {#key playerId}
      <div class="log-contents">
        {#each playerLog as logTurn, turnNum}
          <LogTurn {logTurn} {turnNum} copyTurn={() => copyTurn(turnNum)} />
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
