<script lang="ts" context="module">
  export async function load() {
    const res = await fetch("/api/matches", {
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (res.ok) {
      return {
        props: {
          matches: await res.json(),
        },
      };
    }

    return {
      status: res.status,
      error: new Error("failed to load matches"),
    };
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
