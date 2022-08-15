<script lang="ts">
  import { goto } from "$app/navigation";
  import { get_session_token } from "$lib/auth";
  import { currentUser } from "$lib/stores/current_user";
  import { onMount } from "svelte";
  let botName: string | undefined = undefined;
  let saveErrors: string[] = [];

  onMount(() => {
    // ensure user is logged in
    if (!$currentUser) {
      goto("/login");
    }
  });

  async function createBot() {
    saveErrors = [];

    // TODO: how can we handle this with the new ApiClient?
    let response = await fetch("/api/bots", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${get_session_token()}`,
      },
      body: JSON.stringify({
        name: botName,
      }),
    });

    let responseData = await response.json();
    if (response.ok) {
      let bot = responseData;
      goto(`/bots/${bot["name"]}`);
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

<div class="container">
  <div class="create-bot-form">
    <h4>Create new bot</h4>
    <input type="text" class="bot-name-input" placeholder="bot name" bind:value={botName} />
    {#if saveErrors.length > 0}
      <ul>
        {#each saveErrors as errorText}
          <li class="error-text">{errorText}</li>
        {/each}
      </ul>
    {/if}
    <button class="submit-button save-button" on:click={createBot}>Save</button>
  </div>
</div>

<style lang="scss">
  .container {
    width: 400px;
    max-width: 80%;
    margin: 50px auto;
  }

  .create-bot-form h4 {
    margin-bottom: 0.3em;
  }

  .error-text {
    color: red;
  }

  .submit-button {
    padding: 6px 16px;
    border-radius: 4px;
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
