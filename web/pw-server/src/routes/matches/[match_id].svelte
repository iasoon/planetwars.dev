<script lang="ts" context="module">
  export async function load({ page }) {
    const res = await fetch(`/api/matches/${page.params["match_id"]}`, {
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (res.ok) {
      return {
        props: {
          matchLog: await res.text(),
        },
      };
    }

    return {
      status: res.status,
      error: new Error("failed to load match"),
    };
  }
</script>

<script lang="ts">
  import Visualizer from "$lib/components/Visualizer.svelte";
  export let matchLog: string;
</script>

<div>
  <Visualizer {matchLog} />
</div>
