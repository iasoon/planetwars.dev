<script lang="ts">
  import { ApiClient } from "$lib/api_client";

  import { get_session_token } from "$lib/auth";
  import { getBotName, saveBotName } from "$lib/bot_code";

  import { currentUser } from "$lib/stores/current_user";
  import { selectedOpponent, selectedMap } from "$lib/stores/editor_state";

  import { createEventDispatcher, onMount } from "svelte";
  import Select from "svelte-select";

  export let editSession;

  let availableBots: object[] = [];
  let maps: object[] = [];

  let botName: string | undefined = undefined;
  // whether to show the "save succesful" message
  let saveSuccesful = false;

  let saveErrors: string[] = [];

  onMount(async () => {
    botName = getBotName();
    const apiClient = new ApiClient();

    const [_bots, _maps] = await Promise.all([
      apiClient.get("/api/bots"),
      apiClient.get("/api/maps"),
    ]);

    availableBots = _bots;
    maps = _maps;

    if (!$selectedOpponent) {
      selectedOpponent.set(availableBots.find((b) => b["name"] === "simplebot"));
    }

    if (!$selectedMap) {
      selectedMap.set(maps.find((m) => m["name"] === "hex"));
    }
  });

  const dispatch = createEventDispatcher();

  async function submitBot() {
    let response = await fetch("/api/submit_bot", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        code: editSession.getDocument().getValue(),
        opponent_name: $selectedOpponent["name"],
        map_name: $selectedMap["name"],
      }),
    });

    let responseData = await response.json();

    if (response.ok) {
      // object has a "match" key containing the match data
      dispatch("matchCreated", responseData);
    } else {
      throw responseData;
    }
  }

  async function saveBot() {
    saveSuccesful = false;
    saveErrors = [];

    let response = await fetch("/api/save_bot", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${get_session_token()}`,
      },
      body: JSON.stringify({
        bot_name: botName,
        code: editSession.getDocument().getValue(),
      }),
    });

    let responseData = await response.json();
    if (response.ok) {
      dispatch("botSaved", responseData);
      saveBotName(botName);

      // make bot available in available bot list
      if (!availableBots.find((bot) => bot["id"] == responseData["id"])) {
        availableBots = [...availableBots, responseData];
      }
      saveSuccesful = true;
    } else {
      const error = responseData["error"];
      if (error["type"] === "validation_failed") {
        saveErrors = error["validation_errors"];
      } else if (error["type"] === "bot_name_taken") {
        saveErrors = ["Bot name is already taken"];
      } else {
        // unexpected error
        throw responseData;
      }
    }
  }
</script>

<div class="submit-pane">
  <div class="match-form">
    <h4>Play a match</h4>
    <div class="play-text">Opponent</div>
    <div class="opponent-select">
      <Select
        itemId="name"
        label="name"
        items={availableBots}
        bind:value={$selectedOpponent}
        clearable={false}
      />
    </div>
    <span>Map</span>
    <div class="map-select">
      <Select itemId="name" label="name" items={maps} bind:value={$selectedMap} clearable={false} />
    </div>
    <button class="submit-button play-button" on:click={submitBot}>Play</button>
  </div>
  <div class="save-form">
    <h4>Save your bot</h4>
    {#if $currentUser}
      <div>Add your bot to the opponents list</div>
      <input type="text" class="bot-name-input" placeholder="bot name" bind:value={botName} />
      {#if saveSuccesful}
        <div class="success-text">Bot saved succesfully</div>
      {:else if saveErrors.length > 0}
        <ul>
          {#each saveErrors as errorText}
            <li class="error-text">{errorText}</li>
          {/each}
        </ul>
      {/if}
      <button class="submit-button save-button" on:click={saveBot}>Save</button>
    {:else}
      Sign in to add your bot to the opponents list.
    {/if}
  </div>
</div>

<style lang="scss">
  .submit-pane {
    margin: 20px;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .submit-pane h4 {
    margin-bottom: 0.3em;
  }

  .opponent-select,
  .map-select {
    margin: 8px 0;
  }

  .save-form {
    margin-top: 8em;
  }

  .error-text {
    color: red;
  }

  .success-text {
    color: green;
    margin: 0 8px;
  }

  .submit-button {
    padding: 8px 16px;
    border-radius: 8px;
    border: 0;
    font-size: 18pt;
    display: block;
    margin: 10px auto;
    background-color: lightgreen;
    cursor: pointer;
  }

  .bot-name-input {
    width: 100%;
    font-size: 16px;
    padding: 8px 16px;
    box-sizing: border-box;
    margin: 10px 0;
    border: 1px solid rgb(216, 219, 223);
    border-radius: 3px;
  }
</style>
