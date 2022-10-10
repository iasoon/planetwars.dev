<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";
  export async function load({ params, fetch }) {
    try {
      const matchId = params["match_id"];
      const apiClient = new ApiClient(fetch);
      const matchData = await apiClient.get(`/api/matches/${matchId}`);
      return {
        props: {
          matchData,
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
  import { onMount } from "svelte";
  import Visualizer from "$lib/components/Visualizer.svelte";

  export let matchLog: string | undefined;
  export let matchData: object;

  onMount(async () => {
    const apiClient = new ApiClient();
    matchLog = await apiClient.getText(`/api/matches/${matchData["id"]}/log`);
  });
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
