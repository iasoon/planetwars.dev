<script lang="ts">
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  function submit() {
    dispatch("submit");
  }
</script>

<div class="submit-pane">
  Submit your bot to play a match
  <button class="play-button" on:click={submit}>Submit</button>
</div>

<style lang="scss">
  .submit-pane {
    margin: 20px auto;
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
</style>
