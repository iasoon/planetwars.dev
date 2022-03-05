<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import Select from "svelte-select";

  let availableBots: object[] = [];
  let selectedOpponent = undefined;
  let botName: string | undefined = undefined;

  onMount(async () => {
    const res = await fetch("/api/bots", {
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (res.ok) {
      availableBots = await res.json();
      selectedOpponent = availableBots.find((b) => b["name"] === "simplebot");
    }
  });

  const dispatch = createEventDispatcher();

  function submitBot() {
    dispatch("submitBot", {
      opponentName: selectedOpponent["name"],
    });
  }

  function saveBot() {
    dispatch("saveBot", {
      botName: botName,
    });
  }
</script>

<div class="submit-pane">
  <div class="match-form">
    <div class="play-text">Select an opponent to test your bot</div>
    <div class="opponentSelect">
      <Select
        optionIdentifier="name"
        labelIdentifier="name"
        items={availableBots}
        bind:value={selectedOpponent}
      />
    </div>
    <button class="submit-button play-button" on:click={submitBot}>Play</button>
  </div>
  <div class="save-form">
    <h4>Save your bot</h4>
    <input type="text" class="bot-name-input" placeholder="bot name" bind:value={botName} />
    <button class="submit-button save-button" on:click={saveBot}>Save</button>
  </div>
</div>

<style lang="scss">
  .submit-pane {
    margin: 20px;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .opponentSelect {
    margin: 20px 0;
  }

  .save-form {
    margin-top: 8em;
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

  .play-button {
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
  }

  .save-button {
    background-color: lightgreen;
    cursor: pointer;
    padding: 8px 16px;
    border: 0;
  }
</style>
