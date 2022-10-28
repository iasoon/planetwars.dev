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
        apiClient.get("/api/matches", { bot: params["bot_name"], count: "10", had_errors: true }),
      ]);

      const { bot, owner, versions } = botData;
      versions.sort((a: string, b: string) =>
        dayjs(a["created_at"]).isAfter(b["created_at"]) ? -1 : 1
      );
      return {
        props: {
          bot,
          owner,
          versions,
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
  export let owner: object;
  export let versions: object[];
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
  <div class="header">
    <h1 class="bot-name">{bot["name"]}</h1>
    {#if owner}
      <a class="owner-name" href="/users/{owner['username']}">
        {owner["username"]}
      </a>
    {/if}
  </div>

  {#if $currentUser && $currentUser["user_id"] === bot["owner_id"]}
    <div>
      <!-- TODO: can we avoid hardcoding the url? -->
      Publish a new version by pushing a docker container to
      <code>registry.planetwars.dev/{bot["name"]}:latest</code>, or using the web editor.
    </div>

    <div class="versions">
      <h3>Versions</h3>
      <ul class="version-list">
        {#each versions.slice(0, 10) as version}
          <li class="bot-version">
            {dayjs(version["created_at"]).format("YYYY-MM-DD HH:mm")}
            {#if version["container_digest"]}
              <span class="container-digest">{version["container_digest"]}</span>
            {:else}
              <a href={`/code/${version["id"]}`}>view code</a>
            {/if}
          </li>
        {/each}
      </ul>
      {#if versions.length == 0}
        This bot does not have any versions yet.
      {/if}
    </div>

    <div class="matches">
      <h3>Matches with errors</h3>
      <MatchList matches={errorMatches} />
      {#if errorMatches.length > 0}
        <div class="btn-container">
          <LinkButton href={`/matches?bot=${bot["name"]}&had_errors=true`}>View all</LinkButton>
        </div>
      {:else}
        <div class="table-placeholder">
          Nothing here yet
        </div>
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
      <div class="table-placeholder">
        No matches played yet
      </div>
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

  .header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 60px;
    border-bottom: 1px solid black;
  }

  $header-space-above-line: 12px;

  .bot-name {
    font-size: 24pt;
    margin-bottom: $header-space-above-line;
  }

  .owner-name {
    font-size: 14pt;
    text-decoration: none;
    color: #333;
    margin-bottom: $header-space-above-line;
  }

  .btn-container {
    padding: 24px;
    text-align: center;
  }

  .table-placeholder {
    padding: 12px;
    text-align: center;
  }

  .versions {
    margin: 30px 0;
  }

  .version-list {
    padding: 0;
  }

  .bot-version {
    display: flex;
    justify-content: space-between;
    padding: 4px 24px;
  }
</style>
