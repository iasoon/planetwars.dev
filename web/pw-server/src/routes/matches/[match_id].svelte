<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";
  export async function load({ params, fetch }) {
    try {
      const matchId = params["match_id"];
      const apiClient = new ApiClient(fetch);
      const [matchData, matchLog] = await Promise.all([
        apiClient.get(`/api/matches/${matchId}`),
        apiClient.getText(`/api/matches/${matchId}/log`),
      ]);
      return {
        props: {
          matchData: matchData,
          matchLog: matchLog,
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
  import Visualizer from "$lib/components/Visualizer.svelte";
  export let matchLog: string;
  export let matchData: object;
</script>

<div class="container">
  <Visualizer {matchLog} {matchData} />
</div>

<style lang="scss">
  .container {
    display: flex;
    // these are needed for making the visualizer fill the screen.
    min-height: 0;
    flex-grow: 1;
    overflow: hidden;
  }
</style>
