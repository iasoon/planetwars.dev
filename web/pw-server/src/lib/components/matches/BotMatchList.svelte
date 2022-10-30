<script lang="ts">
  import { ApiClient } from "$lib/api_client";
  import { apiMatchtoBotMatch, BotMatch } from "$lib/matches";
  import { onDestroy, onMount } from "svelte";
  import BotMatchCard from "./BotMatchCard.svelte";

  export let botName: string;
  export let opponentName: string | undefined;
  export let mapName: string | undefined;

  let botMatches: BotMatch[] = [];
  let nextCursor = undefined;
  let loading = false;
  let hasMore = true;

  onMount(async () => {
    window.addEventListener("scroll", onScroll);
    window.addEventListener("resize", onScroll);
  });

  onDestroy(() => {
    window.removeEventListener("scroll", onScroll);
    window.removeEventListener("resize", onScroll);
  });

  function onScroll(e: Event) {
    // const element = e.target as HTMLElement;
    const element = window.document.body;
    if (hasMore && element.scrollHeight - window.scrollY - element.clientHeight <= 300) {
      fetchNextPage();
    }
  }

  async function fetchNextPage() {
    if (loading) {
      return;
    }
    loading = true;
    const params = { bot: botName, count: "50" };
    if (opponentName) {
      params["opponent"] = opponentName;
    }
    if (mapName) {
      params["map"] = mapName;
    }
    if (nextCursor) {
      params["before"] = nextCursor;
    }
    const apiClient = new ApiClient();
    let { matches, has_next } = await apiClient.get("/api/matches", params);
    if (has_next) {
      nextCursor = matches[matches.length - 1]["timestamp"];
    } else {
      hasMore = false;
    }
    botMatches = botMatches.concat(matches.map((m) => apiMatchtoBotMatch(botName, m)));
    loading = false;
  }

  async function resetList(..._params: any[]) {
    botMatches = [];
    nextCursor = undefined;
    hasMore = true;
    await fetchNextPage();
  }

  $: resetList(botName, opponentName, mapName);
</script>

<div>
  {#each botMatches as botMatch}
    <BotMatchCard {botMatch} />
  {/each}
  {#if loading}
    <div class="loading-container">Loading...</div>
  {/if}
</div>

<style lang="scss">
  .loading-container {
    text-align: center;
    padding: 16px;
  }
</style>
