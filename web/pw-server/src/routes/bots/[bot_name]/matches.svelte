<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  export async function load({ params, fetch }) {
    try {
      const apiClient = new ApiClient(fetch);
      const botName = params["bot_name"];

      const [allBots, allMaps] = await Promise.all([
        apiClient.get("/api/bots"),
        apiClient.get("/api/maps"),
      ]);

      return {
        props: {
          botName,
          allBots,
          allMaps,
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
  import BotMatchList from "$lib/components/matches/BotMatchList.svelte";
  import Select from "svelte-select";

  export let botName: string;
  export let allBots: object[];
  export let allMaps: object[];

  let opponentName: string | undefined;
  let mapName: string | undefined;
</script>

<div class="container">
  <div class="selections">
    <Select
      items={allBots}
      placeholder="Select opponent"
      label="name"
      itemId="name"
      bind:justValue={opponentName}
    />
    <Select
      items={allMaps}
      placeholder="Select map"
      label="name"
      itemId="name"
      bind:justValue={mapName}
    />
  </div>
  <BotMatchList {botName} {opponentName} {mapName} />
</div>

<style lang="scss">
  .container {
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
  }

  .selections {
    display: flex;
    padding: 16px 4px;
  }
</style>
