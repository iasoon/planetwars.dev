<!-- TODO: should we prevent users who are not the bot owner
from seeing this page? -->
<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";
  import dayjs from "dayjs";

  export async function load({ params, fetch }) {
    const apiClient = new ApiClient(fetch);

    try {
      const botName = params["bot_name"];
      const { bot, versions } = await apiClient.get(`/api/bots/${botName}`);

      versions.sort((a: string, b: string) =>
        dayjs(a["created_at"]).isAfter(b["created_at"]) ? -1 : 1
      );
      return {
        props: {
          bot,
          versions,
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
  export let bot;
  export let versions;
</script>

<div class="container">
  <div>
    <!-- TODO: can we avoid hardcoding the url? -->
    Publish a new version by pushing a docker container to
    <code>registry.planetwars.dev/{bot["name"]}:latest</code>, or using the web editor.
  </div>

  <div class="versions">
    <h3>Versions</h3>
    <ul class="version-list">
      {#each versions as version}
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
</div>

<style lang="scss">
  .container {
    width: 800px;
    max-width: 80%;
    margin: 50px auto;
    padding-bottom: 24px;
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
