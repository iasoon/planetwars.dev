<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";
  import type { Match } from "$lib/api_types";

  const PAGE_SIZE = "50";

  export async function load({ params, fetch }) {
    try {
      const apiClient = new ApiClient(fetch);
      const botName = params["bot_name"];

      let { matches, has_next } = await apiClient.get("/api/matches", { bot: botName });

      // TODO: should this be done client-side?
      // if (query["after"]) {
      //   matches = matches.reverse();
      // }

      return {
        props: {
          matches,
          botName,
          hasNext: has_next,
        },
      };
    } catch (error) {
      return {
        status: error.status,
        error: new Error("failed to load matches"),
      };
    }
  }
</script>

<script lang="ts">
  import LinkButton from "$lib/components/LinkButton.svelte";
  import BotMatchList from "$lib/components/matches/BotMatchList.svelte";
  import { apiMatchtoBotMatch } from "$lib/matches";

  export let matches: Match[];
  export let botName: string | null;
  // whether a next page exists in the current iteration direction (before/after)
  // export let hasNext: boolean;

  $: botMatches = matches.map((match) => apiMatchtoBotMatch(botName, match));
</script>

<div class="container">
  <BotMatchList {botMatches} />
</div>

<style lang="scss">
  .container {
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
  }

  .page-controls {
    display: flex;
    justify-content: center;
    margin: 24px 0;
  }
</style>
