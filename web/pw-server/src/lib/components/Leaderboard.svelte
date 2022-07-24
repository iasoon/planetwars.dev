<script lang="ts">
  import { onMount } from "svelte";

  let leaderboard = [];

  onMount(async () => {
    const res = await fetch("/api/leaderboard", {
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (res.ok) {
      leaderboard = await res.json();
      console.log(leaderboard);
    }
  });

  function formatRating(entry: object): any {
    const rating = entry["rating"];
    if (rating != null) {
      return rating.toFixed(0);
    } else {
      // why does this happen?
      return "-inf";
    }
  }
</script>

<div class="container">
  <table class="leaderboard">
    <tr class="leaderboard-row leaderboard-header">
      <th class="leaderboard-rank" />
      <th class="leaderboard-rating">Rating</th>
      <th class="leaderboard-bot">Bot</th>
      <th class="leaderboard-author">Author</th>
    </tr>
    {#each leaderboard as entry, index}
      <tr class="leaderboard-row">
        <td class="leaderboard-rank">{index + 1}</td>
        <td class="leaderboard-rating">
          {formatRating(entry)}
        </td>
        <td class="leaderboard-bot">
          <a class="leaderboard-href" href="/bots/{entry['bot']['name']}"
            >{entry["bot"]["name"]}
          </a></td
        >
        <td class="leaderboard-author">
          {#if entry["author"]}
            <!-- TODO: remove duplication -->
            <a class="leaderboard-href" href="/users/{entry['author']['username']}"
              >{entry["author"]["username"]}</a
            >
          {/if}
        </td>
      </tr>
    {/each}
  </table>
</div>

<style lang="scss">
  .container {
    overflow-y: scroll;
    height: 100%;
  }
  .leaderboard {
    margin: 18px auto;
    text-align: center;
  }

  .leaderboard th,
  .leaderboard td {
    padding: 8px 16px;
  }
  .leaderboard-rank {
    color: #333;
  }

  .leaderboard-href {
    text-decoration: none;
    color: black;
  }
</style>
