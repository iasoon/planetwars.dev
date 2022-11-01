<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  export async function load({ params, fetch }) {
    const apiClient = new ApiClient(fetch);

    try {
      const bot_name = params["bot_name"];
      const [botData, botStats, matchesPage, errorMatchesPage] = await Promise.all([
        apiClient.get(`/api/bots/${bot_name}`),
        apiClient.get(`/api/bots/${bot_name}/stats`),
        apiClient.get("/api/matches", { bot: params["bot_name"], count: "20" }),
        apiClient.get("/api/matches", { bot: params["bot_name"], count: "10", had_errors: "true" }),
      ]);

      const { bot, owner, versions } = botData;
      versions.sort((a: string, b: string) =>
        dayjs(a["created_at"]).isAfter(b["created_at"]) ? -1 : 1
      );
      return {
        props: {
          bot,
          botStats,
          matches: matchesPage["matches"],
          errorMatches: errorMatchesPage["matches"],
        },
      };
    } catch (error) {
      return {
        status: error.status,
        error: error,
      };
    }
  }
</script>

<script lang="ts">
  import dayjs from "dayjs";
  import { currentUser } from "$lib/stores/current_user";
  import MatchList from "$lib/components/matches/MatchList.svelte";
  import LinkButton from "$lib/components/LinkButton.svelte";

  export let bot: object;
  export let matches: object[];
  export let errorMatches: object[];
</script>

<!-- 
<div>Upload code</div>
<form on:submit|preventDefault={submitCode}>
  <input type="file" bind:files />
  <button type="submit">Submit</button>
</form> -->

<div class="container">
  {#if $currentUser && $currentUser["user_id"] === bot["owner_id"]}
    <div class="matches">
      <h3>Matches with errors</h3>
      <MatchList matches={errorMatches} />
      {#if errorMatches.length > 0}
        <div class="btn-container">
          <LinkButton href={`/matches?bot=${bot["name"]}&had_errors=true`}>View all</LinkButton>
        </div>
      {:else}
        <div class="table-placeholder">Nothing here yet</div>
      {/if}
    </div>
  {/if}

  <div class="matches">
    <h3>Recent matches</h3>
    <MatchList {matches} />
    {#if matches.length > 0}
      <div class="btn-container">
        <LinkButton href={`/matches?bot=${bot["name"]}`}>All matches</LinkButton>
      </div>
    {:else}
      <div class="table-placeholder">No matches played yet</div>
    {/if}
  </div>
</div>

<style lang="scss">
  .container {
    width: 800px;
    max-width: 80%;
    margin: 50px auto;
    padding-bottom: 24px;
  }

  .btn-container {
    padding: 24px;
    text-align: center;
  }

  .table-placeholder {
    padding: 12px;
    text-align: center;
  }
</style>
