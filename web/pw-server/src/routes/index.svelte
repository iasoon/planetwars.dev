<script lang="ts">
  import { goto } from "$app/navigation";
  import Visualizer from "$lib/components/Visualizer.svelte";
  import { onMount } from "svelte";
  import "./style.css";

  let editor;
  let matches = [];

  let selectedMatchId: string | undefined = undefined;
  let selectedMatchLog: string | undefined = undefined;

  onMount(async () => {
    await load_editor();
  });

  async function load_editor() {
    const ace = await import("ace-builds");
    const python_mode = await import("ace-builds/src-noconflict/mode-python");
    const gh_theme = await import("ace-builds/src-noconflict/theme-github");

    editor = ace.edit("editor");
    editor.getSession().setMode(new python_mode.Mode());
    editor.setTheme(gh_theme);
  }

  async function submitCode() {
    if (editor === undefined) {
      return;
    }

    let response = await fetch("/api/submit_bot", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        code: editor.getValue(),
      }),
    });

    if (!response.ok) {
      throw Error(response.statusText);
    }

    let responseData = await response.json();

    let matchId = responseData["match_id"];
    // goto(`/submission_matches/${matchId}`);
    matches.push({ matchId: matchId });
    matches = matches;
  }

  async function selectMatch(matchId: string) {
    console.log("showing match " + matchId);
    let matchLog = await loadMatch(matchId);
    selectedMatchId = matchId;
    selectedMatchLog = matchLog;
  }

  async function loadMatch(matchId: string) {
    const res = await fetch(`/api/submission_match_log/${matchId}`, {
      headers: {
        "Content-Type": "application/json",
      },
    });

    let log = await res.text();
    return log;
  }

  function selectEditor() {
    selectedMatchId = undefined;
    selectedMatchLog = undefined;
    load_editor();
  }
</script>

<div class="outer-container">
  <div class="top-bar" />
  <div class="container">
    <div class="sidebar-left">
      <div
        class="sidebar-item"
        class:selected={selectedMatchId === undefined}
        on:click={selectEditor}
      >
        Editor
      </div>
      <ul class="match-list">
        {#each matches as match}
          <li class="match-card sidebar-item" on:click={() => selectMatch(match.matchId)}>
            <div class="match-name">{match.matchId}</div>
          </li>
        {/each}
      </ul>
    </div>
    <div class="editor-container">
      {#if selectedMatchLog !== undefined}
        <Visualizer matchLog={selectedMatchLog} />
      {:else}
        <div id="editor" />
      {/if}
    </div>
    <div class="sidebar-right">
      <button class="play-button" on:click={submitCode}>Submit</button>
    </div>
  </div>
</div>

<style lang="scss">
  $bg-color: rgb(41, 41, 41);

  .outer-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .top-bar {
    height: 40px;
    background-color: $bg-color;
    border-bottom: 1px solid;
  }

  .container {
    display: flex;
    flex-grow: 1;
  }

  .sidebar-left {
    width: 240px;
    background-color: $bg-color;
  }
  .sidebar-right {
    width: 400px;
    background-color: white;
    border-left: 1px solid;
    padding: 10px;
  }
  .editor-container {
    flex-grow: 1;
    overflow: hidden;
  }

  #editor {
    width: 100%;
    height: 100%;
  }

  .editor-container {
    height: 100%;
  }

  .play-button {
    padding: 8px 16px;
    border-radius: 8px;
    border: 0;
    font-size: 18pt;
    display: block;
    margin: 20px auto;
    background-color: lightgreen;
    cursor: pointer;
  }

  .sidebar-item {
    padding: 15px;
    color: #eee;
  }

  .sidebar-item:hover {
    background-color: #333;
  }

  .sidebar-item.selected {
    background-color: #333;
  }

  .toolbar-editor {
    padding: 15px;
    color: #eee;
  }

  .match-list {
    list-style: none;
    color: #eee;
  }
</style>
