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
  import { currentUser } from "$lib/stores/current_user";

  export let matchLog: string | undefined;
  export let matchData: object;
  let showSidebar = true;

  onMount(async () => {
    const apiClient = new ApiClient();
    matchLog = await apiClient.getText(`/api/matches/${matchData["id"]}/log`);
  });

  $: playersWithVisibleLog = matchData["players"]
    .map((player: any, index: number) => ({
      color: PLAYER_COLORS[index],
      value: index,
      playerId: index + 1, // stoopid player number + 1
      displayName: player["bot_name"] || "player",
      matchPlayer: player,
    }))
    .filter((item) => canSeePlayerLog($currentUser, item.matchPlayer));

  // TODO: refactor match logs so that users can no longer get match logs for other players.
  function canSeePlayerLog(user: object | null, matchPlayer: object): boolean {
    if (!matchPlayer["owner_id"]) {
      return true;
    }

    return matchPlayer["owner_id"] === user?.["user_id"];
  }

  function toggleSidebar() {
    showSidebar = !showSidebar;
  }

  // using the same value here causes svelte to freeze
  let dropdownSelectedPlayer: any;
  let selectedPlayer: any;
  $: if (playersWithVisibleLog.length == 1) {
    selectedPlayer = playersWithVisibleLog[0];
  } else {
    selectedPlayer = dropdownSelectedPlayer;
  }
</script>

<div class="container">
  <Visualizer {matchLog} {matchData}>
    <div slot="menu">
      {#if playersWithVisibleLog.length > 0}
        <div class="toggle-sidebar" on:click={toggleSidebar}>toggle sidebar</div>
      {/if}
    </div>
  </Visualizer>
  {#if showSidebar && playersWithVisibleLog.length > 0}
    <div class="output-pane">
      <div class="player-select">
        {#if playersWithVisibleLog.length == 1}
          <h3 class="player-log-header">
            player log for
            <span style:color={selectedPlayer["color"]}>{selectedPlayer.displayName}</span>
          </h3>
        {:else}
          <Select
            items={playersWithVisibleLog}
            label="displayName"
            clearable={false}
            searchable={false}
            bind:value={dropdownSelectedPlayer}
            placeholder="Select player to see logs"
          >
            <div slot="item" let:item>
              <span style:color={item.color}>{item.displayName}</span>
            </div>
          </Select>
        {/if}
      </div>
      <div class="player-log">
        <PlayerLog {matchLog} playerId={selectedPlayer?.["playerId"]} />
      </div>
    </div>
  {/if}
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
    padding: 0 20px;
  }

  .player-log-header {
    color: #eee;
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

  .toggle-sidebar:hover {
    cursor: pointer;
    color: #ccc;
  }
</style>
