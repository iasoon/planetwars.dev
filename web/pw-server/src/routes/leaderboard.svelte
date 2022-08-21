<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  export async function load({ fetch }) {
    try {
      const apiClient = new ApiClient(fetch);
      const leaderboard = await apiClient.get("/api/leaderboard");
      return {
        props: {
          leaderboard,
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
  import Leaderboard from "$lib/components/Leaderboard.svelte";

  export let leaderboard: object[];
</script>

<Leaderboard {leaderboard} />
