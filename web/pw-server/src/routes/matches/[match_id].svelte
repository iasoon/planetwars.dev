<script lang="ts" context="module">
  function fetchJson(url: string): Promise<Response> {
    return fetch(url, {
      headers: {
        "Content-Type": "application/json",
      },
    });
  }

  export async function load({ params }) {
    // TODO: handle failure cases better
    const matchId = params["match_id"];
    const matchDataResponse = await fetchJson(`/api/matches/${matchId}`);
    if (!matchDataResponse.ok) {
    }
    const matchLogResponse = await fetchJson(`/api/matches/${matchId}/log`);

    if (matchDataResponse.ok && matchLogResponse.ok) {
      return {
        props: {
          matchData: await matchDataResponse.json(),
          matchLog: await matchLogResponse.text(),
        },
      };
    }

    return {
      status: matchDataResponse.status,
      error: new Error("failed to load match"),
    };
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
