<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  export async function load({ params, fetch }) {
    const apiClient = new ApiClient(fetch);

    try {
      const [botData, matches] = await Promise.all([
        apiClient.get(`/api/bots/${params["bot_name"]}`),
        apiClient.get("/api/matches", { bot: params["bot_name"], count: "20" }),
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
          matches,
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

  export let bot: object;
  export let owner: object;
  export let versions: object[];
  export let matches: object[];

  // function last_updated() {
  //   versions.sort()
  // }

  // let files;

  // async function submitCode() {
  //   console.log("click");
  //   const token = get_session_token();

  //   const formData = new FormData();
  //   formData.append("File", files[0]);

  //   const res = await fetch(`/api/bots/${bot["id"]}/upload`, {
  //     method: "POST",
  //     headers: {
  //       // the content type header will be set by the browser
  //       Authorization: `Bearer ${token}`,
  //     },
  //     body: formData,
  //   });

  //   console.log(res.statusText);
  // }
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
  {/if}

  <div class="matches">
    <h3>Recent matches</h3>
    <MatchList {matches} />
    {#if matches.length > 0}
      <div class="btn-container">
        <a class="btn-view-more" href={`/matches?bot=${bot["name"]}`}>All matches</a>
      </div>
    {/if}
  </div>

  <!-- <div class="versions">
    <h4>Versions</h4>
    <ul class="version-list">
      {#each versions as version}
        <li>
          {dayjs(version["created_at"]).format("YYYY-MM-DD HH:mm")}
        </li>
      {/each}
    </ul>
    {#if versions.length == 0}
      This bot does not have any versions yet.
    {/if}
  </div> -->
</div>

<style lang="scss">
  @import "src/styles/variables.scss";
  .container {
    width: 800px;
    max-width: 80%;
    margin: 50px auto;
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
  .btn-view-more {
    color: $btn-text-color;
    font-size: 14px;
    text-decoration: none;
    padding: 6px 16px;
    border: 1px solid $btn-border-color;
    border-radius: 5px;
  }

  .versions {
    margin: 30px 0;
  }
</style>
