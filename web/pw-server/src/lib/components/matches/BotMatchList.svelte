<script lang="ts">
  import type { BotMatch } from "$lib/matches";
  import BotMatchCard from "./BotMatchCard.svelte";

  export let botMatches: BotMatch[];

  function match_url(match: object) {
    return `/matches/${match["id"]}`;
  }
</script>

<div>
  {#each botMatches as botMatch}
    <BotMatchCard {botMatch} />
  {/each}
</div>

<style lang="scss">
</style>
