<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  export async function load({ fetch }) {
    try {
      const apiClient = new ApiClient(fetch);
      const matches = await apiClient.get("/api/matches");

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
</script>

<div class="container">
  <MatchList {matches} />
</div>

<style lang="scss">
  .container {
    width: 800px;
    margin: 0 auto;
  }
</style>
