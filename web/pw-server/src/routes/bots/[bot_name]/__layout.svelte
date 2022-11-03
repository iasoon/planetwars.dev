<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  export async function load({ params, fetch }) {
    const apiClient = new ApiClient(fetch);
    const botName = params["bot_name"];
    const { bot, owner } = await apiClient.get(`/api/bots/${botName}`);

    return {
      props: {
        bot,
        owner,
      },
    };
  }
</script>

<script lang="ts">
  import NavTab from "$lib/components/NavTab.svelte";
  import { currentUser } from "$lib/stores/current_user";

  export let bot;
  export let owner;
</script>

<svelte:head>
  <title>{bot["name"]}</title>
</svelte:head>

<div class="header">
  <div class="header-title-line">
    <h1 class="bot-name">{bot["name"]}</h1>
    {#if owner}
      <span class="title-line-owner">
        by
        <a class="owner-name" href="/users/{owner['username']}">
          {owner["username"]}
        </a>
      </span>
    {/if}
  </div>
  <div class="bot-tabs">
    <NavTab href={`/bots/${bot.name}`}>index</NavTab>
    <NavTab href={`/bots/${bot.name}/matches`}>matches</NavTab>
    <NavTab href={`/bots/${bot.name}/stats`}>stats</NavTab>
    {#if $currentUser && $currentUser["user_id"] === bot["owner_id"]}
      <NavTab href={`/bots/${bot.name}/versions`}>versions</NavTab>
    {/if}
  </div>
</div>
<slot />

<style lang="scss">
  .header {
    padding: 0 16px;
    border-bottom: 1px solid black;
  }

  .header-title-line {
    display: flex;
    align-items: baseline;
  }

  .title-line-owner {
    padding: 0 16px;
    color: #333;
  }

  $header-space-above-line: 12px;

  .bot-name {
    font-size: 24pt;
    margin-bottom: $header-space-above-line;
  }

  .owner-name {
    font-size: 14pt;
    // font-weight: 600;
    text-decoration: none;
    color: black;
    margin-bottom: $header-space-above-line;
  }

  .bot-tabs {
    display: flex;
  }
</style>
