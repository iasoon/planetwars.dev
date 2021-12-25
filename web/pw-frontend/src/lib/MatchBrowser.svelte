<script lang="ts">
  import { onMount } from "svelte";
  import Visualizer from "./Visualizer.svelte";

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
          {match.name}
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
</style>
