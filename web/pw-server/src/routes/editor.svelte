<script lang="ts">
  import Visualizer from "$lib/components/Visualizer.svelte";
  import EditorView from "$lib/components/EditorView.svelte";
  import { onMount } from "svelte";
  import { DateTime } from "luxon";

  import type { Ace } from "ace-builds";
  import ace from "ace-builds/src-noconflict/ace?client";
  import * as AcePythonMode from "ace-builds/src-noconflict/mode-python?client";
  import { getBotCode, saveBotCode } from "$lib/bot_code";
  import { matchHistory } from "$lib/stores/editor_state";
  import { debounce } from "$lib/utils";
  import SubmitPane from "$lib/components/SubmitPane.svelte";
  import OutputPane from "$lib/components/OutputPane.svelte";

  enum ViewMode {
    Editor,
    MatchVisualizer,
  }

  let viewMode = ViewMode.Editor;
  let selectedMatchId: string | undefined = undefined;
  let selectedMatchLog: string | undefined = undefined;

  let editSession: Ace.EditSession;

  onMount(() => {
    init_editor();
  });

  function init_editor() {
    editSession = new ace.EditSession(getBotCode());
    editSession.setMode(new AcePythonMode.Mode());

    const saveCode = () => {
      const code = editSession.getDocument().getValue();
      saveBotCode(code);
    };

    // cast to any because the type annotations are wrong here
    (editSession as any).on("change", debounce(saveCode, 2000));
  }

  async function onMatchCreated(e: CustomEvent) {
    const matchData = e.detail["match"];
    matchHistory.pushMatch(matchData);
    await selectMatch(matchData["id"]);
  }

  async function selectMatch(matchId: string) {
    selectedMatchId = matchId;
    selectedMatchLog = null;
    fetchSelectedMatchLog(matchId);

    viewMode = ViewMode.MatchVisualizer;
  }

  async function fetchSelectedMatchLog(matchId: string) {
    if (matchId !== selectedMatchId) {
      return;
    }

    let matchLog = await getMatchLog(matchId);

    if (matchLog) {
      selectedMatchLog = matchLog;
    } else {
      // try again in 1 second
      setTimeout(fetchSelectedMatchLog, 1000, matchId);
    }
  }

  async function getMatchData(matchId: string) {
    let response = await fetch(`/api/matches/${matchId}`, {
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (!response.ok) {
      throw Error(response.statusText);
    }

    let matchData = await response.json();
    return matchData;
  }

  async function getMatchLog(matchId: string) {
    const matchData = await getMatchData(matchId);
    console.log(matchData);
    if (matchData["state"] !== "Finished") {
      // log is not available yet
      return null;
    }

    const res = await fetch(`/api/matches/${matchId}/log`, {
      headers: {
        "Content-Type": "application/json",
      },
    });

    let log = await res.text();
    return log;
  }

  function setViewMode(viewMode_: ViewMode) {
    selectedMatchId = undefined;
    selectedMatchLog = undefined;
    viewMode = viewMode_;
  }

  function formatMatchTimestamp(timestampString: string): string {
    let timestamp = DateTime.fromISO(timestampString, { zone: "utc" }).toLocal();
    if (timestamp.startOf("day").equals(DateTime.now().startOf("day"))) {
      return timestamp.toFormat("HH:mm");
    } else {
      return timestamp.toFormat("dd/MM");
    }
  }

  $: selectedMatch = $matchHistory.find((m) => m["id"] === selectedMatchId);
</script>

<div class="container">
  <div class="sidebar-left">
    <div
      class="editor-button sidebar-item"
      class:selected={viewMode === ViewMode.Editor}
      on:click={() => setViewMode(ViewMode.Editor)}
    >
      Code
    </div>
    <div class="sidebar-header">match history</div>
    <ul class="match-list">
      {#each $matchHistory as match}
        <li
          class="match-card sidebar-item"
          on:click={() => selectMatch(match.id)}
          class:selected={match.id === selectedMatchId}
        >
          <div class="match-timestamp">{formatMatchTimestamp(match.timestamp)}</div>
          <div class="match-card-body">
            <!-- ugly temporary hardcode -->
            <div class="match-opponent">{match["players"][1]["bot_name"]}</div>
            <div class="match-map">{match["map"]?.name}</div>
          </div>
        </li>
      {/each}
    </ul>
  </div>
  <div class="editor-container">
    {#if viewMode === ViewMode.MatchVisualizer}
      <Visualizer matchData={selectedMatch} matchLog={selectedMatchLog} />
    {:else if viewMode === ViewMode.Editor}
      <EditorView {editSession} />
    {/if}
  </div>
  <div class="sidebar-right">
    {#if viewMode === ViewMode.MatchVisualizer}
      <OutputPane matchLog={selectedMatchLog} />
    {:else if viewMode === ViewMode.Editor}
      <SubmitPane {editSession} on:matchCreated={onMatchCreated} />
    {/if}
  </div>
</div>

<style lang="scss">
  @import "src/styles/variables.scss";

  .container {
    display: flex;
    flex-grow: 1;
    min-height: 0;
  }

  .sidebar-left {
    width: 240px;
    background-color: $bg-color;
    display: flex;
    flex-direction: column;
  }
  .sidebar-right {
    width: 400px;
    background-color: white;
    border-left: 1px solid;
    padding: 0;
    display: flex;
    overflow: hidden;
  }
  .editor-container {
    flex-grow: 1;
    flex-shrink: 1;
    overflow: hidden;
    background-color: white;
  }

  .editor-container {
    height: 100%;
  }

  .sidebar-item {
    color: #eee;
    padding: 15px;
  }

  .sidebar-item:hover {
    background-color: #333;
  }

  .sidebar-item.selected {
    background-color: #333;
  }

  .match-list {
    list-style: none;
    color: #eee;
    padding-top: 15px;
    overflow-y: scroll;
    padding-left: 0px;
  }

  .match-card {
    padding: 10px 15px;
    font-size: 11pt;
    display: flex;
  }

  .match-timestamp {
    color: #ccc;
  }

  .match-card-body {
    margin: 0 8px;
  }

  .match-opponent {
    font-weight: 600;
    color: #eee;
  }

  .match-map {
    color: #ccc;
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
</style>
