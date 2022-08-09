<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  const PAGE_SIZE = "50";

  export async function load({ fetch }) {
    try {
      const apiClient = new ApiClient(fetch);
      const matches = await apiClient.get("/api/matches", {
        count: PAGE_SIZE,
      });

      return {
        props: {
          matches,
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
  import MatchList from "$lib/components/matches/MatchList.svelte";

  export let matches: object[];
  let loading = false;

  async function loadNewer() {
    if (matches.length == 0) {
      return;
    }
    const firstTimestamp = matches[0]["timestamp"];
    const apiClient = new ApiClient();
    const reversedNewPage = await apiClient.get("/api/matches", {
      count: PAGE_SIZE,
      after: firstTimestamp,
    });

    if (reversedNewPage.length > 0) {
      matches = reversedNewPage.reverse();
    }
  }

  async function loadOlder() {
    if (matches.length == 0) {
      return;
    }
    const lastTimestamp = matches[matches.length - 1]["timestamp"];
    const apiClient = new ApiClient();
    const newPage = await apiClient.get("/api/matches", {
      count: PAGE_SIZE,
      before: lastTimestamp,
    });
    if (newPage.length > 0) {
      matches = newPage;
    }
  }
</script>

<div class="container">
  <MatchList {matches} />
  <div class="page-controls">
    <button on:click={loadNewer}>newer</button>
    <button on:click={loadOlder}>older</button>
  </div>
</div>

<style lang="scss">
  .container {
    width: 800px;
    margin: 0 auto;
  }

  .page-controls {
    display: flex;
    justify-content: space-between;
    margin: 12px;
  }
</style>
