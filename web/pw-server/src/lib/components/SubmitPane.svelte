<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import Select from "svelte-select";

  let availableBots: object[] = [];
  let selectedOpponent = "simplebot";

  const optionIdentifier = "name";
  const labelIdentifier = "name";

  onMount(async () => {
    const res = await fetch("/api/bots", {
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (res.ok) {
      availableBots = await res.json();
      console.log(availableBots);
    }
  });

  const dispatch = createEventDispatcher();

  function submit() {
    dispatch("submit");
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
    <button class="play-button" on:click={submit}>Play</button>
  </div>
</div>

<style lang="scss">
  .submit-pane {
    margin: 20px;
    flex: 1;
  }

  .opponentSelect {
    margin: 20px 0;
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
</style>
