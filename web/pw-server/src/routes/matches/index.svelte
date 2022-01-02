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
  import dayjs from "dayjs";
  export let matches;
</script>

<a href="/matches/new">new match</a>
<ul>
  {#each matches as match}
    <li>
      <a href="/matches/{match['id']}">{dayjs(match["created_at"]).format("YYYY-MM-DD HH:mm")}</a>
    </li>
  {/each}
</ul>
