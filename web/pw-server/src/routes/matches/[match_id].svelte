<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";
  export async function load({ params, fetch }) {
    try {
      const matchId = params["match_id"];
      const apiClient = new ApiClient(fetch);
      const matchData = await apiClient.get(`/api/matches/${matchId}`);
      return {
        props: {
          matchData,
        },
      };
    } catch (error) {
      return {
        status: error.status,
        error: error,
      };
    }
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import Visualizer from "$lib/components/Visualizer.svelte";
  import PlayerLog from "$lib/components/PlayerLog.svelte";
  import Select from "svelte-select";
  import { PLAYER_COLORS } from "$lib/constants";

  export let matchLog: string | undefined;
  export let matchData: object;

  onMount(async () => {
    const apiClient = new ApiClient();
    matchLog = await apiClient.getText(`/api/matches/${matchData["id"]}/log`);
  });

  let selectedPlayer;

  $: matchPlayerSelectItems = matchData["players"].map((player: any, index: number) => ({
    color: PLAYER_COLORS[index],
    value: index,
    playerId: index + 1, // stoopid player number + 1
    label: player["bot_name"],
  }));
</script>

<div class="container">
  <Visualizer {matchLog} {matchData} />
  <div class="output-pane">
    <div class="player-select">
      <Select items={matchPlayerSelectItems} clearable={false} bind:value={selectedPlayer}>
        <div slot="item" let:item>
          <span style:color={item.color}>{item.label}</span>
        </div>
      </Select>
    </div>
    <div class="player-log">
      <PlayerLog {matchLog} playerId={selectedPlayer?.playerId} />
    </div>
  </div>
</div>

<style lang="scss">
  @use "src/styles/variables";
  .container {
    display: flex;
    // these are needed for making the visualizer fill the screen.
    min-height: 0;
    flex-grow: 1;
    overflow: hidden;
  }

  .player-select {
    padding: 20px;
  }

  .player-log {
    padding: 15px;
    overflow-y: scroll;
  }

  .output-pane {
    width: 600px;
    // overflow: hidden;
    display: flex;
    flex-direction: column;
    background-color: variables.$bg-color;
  }
</style>
