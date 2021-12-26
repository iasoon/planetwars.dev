<script lang="ts">
  import { onMount } from "svelte";
  import Visualizer from "./Visualizer.svelte";
  import moment from "moment";

  const PLAYER_COLORS = [
    "#FF8000",
    "#0080FF",
    "#FF6693",
    "#3FCB55",
    "#CBC33F",
    "#CF40E9",
    "#FF3F0D",
    "#1BEEF0",
    "#0DC5FF",
  ];

  let matches = [];
  let selectedMatch = null;
  let selectedMatchLog = null;

  onMount(() => {
    fetch("/api/matches")
      .then((response) => response.json())
      .then((m) => (matches = m));
  });

  function selectMatch(matchName) {
    selectedMatch = matchName;
    selectedMatchLog = null;
    fetch(`/api/matches/${matchName}`)
      .then((resp) => resp.text())
      .then((log) => {
        selectedMatchLog = log;
      });
  }

  function showTimestamp(dateStr: string): string {
    let t = moment(dateStr);
    if (t.day() == moment().day()) {
      return moment(dateStr).format("HH:mm");
    } else {
      return moment(dateStr).format("DD/MM");
    }
  }

  function playerColor(player_num: number): string {
    return PLAYER_COLORS[player_num % PLAYER_COLORS.length];
  }
</script>

<div class="container">
  <div class="sidebar">
    <div class="sidebar-header" />
    <ul class="match-list">
      {#each matches as match (match.name)}
        <li
          on:click={() => selectMatch(match.name)}
          class:selected={selectedMatch === match.name}
          class="match-card"
        >
          <div class="match-name">{match.name}</div>
          <span class="match-timestamp">{showTimestamp(match.timestamp)}</span>
          <span class="match-mapname">{match.map_name}</span>
          <ul class="match-player-list">
            {#each match.players as player, ix}
              <li class="match-player" style="color: {playerColor(ix)}">
                {player.name}
              </li>
            {/each}
          </ul>
        </li>
      {/each}
    </ul>
  </div>
  <Visualizer matchLog={selectedMatchLog} />
</div>

<style scoped>
  .container {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow-y: hidden;
    background-color: rgb(41, 41, 41);
  }

  .sidebar {
    width: 20%;
    width: 350px;
    color: #eee;
    overflow: hidden;
    overflow-y: scroll;
    height: 100%;
  }

  .sidebar-header {
    margin-top: 2em;
    text-transform: uppercase;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.7);
    font-size: 14px;
    font-family: "Open Sans", sans-serif;
    padding-left: 14px;
  }

  .match-list {
    list-style: none;
    padding: 0;
  }

  .match-card {
    padding: 0.5em;
    padding-left: 14px;
  }

  .match-card.selected {
    background-color: #333;
  }

  .match-card:hover {
    background-color: #333;
  }

  .match-timestamp {
    color: #ccc;
  }

  .match-mapname {
    padding: 0 0.5em;
    color: #ccc;
  }

  .match-player-list {
    list-style: none;
    overflow: hidden;
    text-overflow: ellipsis;
    padding-left: 18px;
  }
</style>
