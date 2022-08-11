<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  const PAGE_SIZE = "50";

  export async function load({ url, fetch }) {
    try {
      const apiClient = new ApiClient(fetch);

      let query = {
        count: PAGE_SIZE,
        before: url.searchParams.get("before"),
        after: url.searchParams.get("after"),
      };

      let matches = await apiClient.get("/api/matches", removeUndefined(query));

      // TODO: should this be done client-side?
      if (query["after"]) {
        matches = matches.reverse();
      }

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

  function removeUndefined(obj: Record<string, string>): Record<string, string> {
    Object.keys(obj).forEach((key) => {
      if (obj[key] === undefined || obj[key] === null) {
        delete obj[key];
      }
    });
    return obj;
  }
</script>

<script lang="ts">
  import { goto } from "$app/navigation";

  import MatchList from "$lib/components/matches/MatchList.svelte";

  export let matches: object[];

  async function loadNewer() {
    if (matches.length == 0) {
      return;
    }
    const firstTimestamp = matches[0]["timestamp"];
    goto(`?after=${firstTimestamp}`);
  }

  async function loadOlder() {
    if (matches.length == 0) {
      return;
    }
    const lastTimestamp = matches[matches.length - 1]["timestamp"];
    goto(`?before=${lastTimestamp}`);
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
