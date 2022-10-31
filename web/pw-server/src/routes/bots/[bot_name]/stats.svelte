<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  export async function load({ params, fetch }) {
    const apiClient = new ApiClient(fetch);

    try {
      const bot_name = params["bot_name"];
      const [botData, botStats, leaderboard] = await Promise.all([
        apiClient.get(`/api/bots/${bot_name}`),
        apiClient.get(`/api/bots/${bot_name}/stats`),
        apiClient.get("/api/leaderboard"),
      ]);

      const { bot } = botData;
      return {
        props: {
          bot,
          botStats,
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

  function calcMergedStats(rawStats: object) {
    return Object.fromEntries(
      Object.entries(rawStats).map(([opponent, ms]) => {
        const mapStats = ms as { k: MatchupStats };
        return [opponent, Object.values(mapStats).reduce(mergeStats)];
      })
    );
  }

  type MatchupStats = {
    win: number;
    tie: number;
    loss: number;
  };

  function winRate(stats: MatchupStats) {
    return (stats.win + 0.5 * stats.tie) / (stats.win + stats.tie + stats.loss);
  }

  function mergeStats(a: MatchupStats, b: MatchupStats): MatchupStats {
    return {
      win: a.win + b.win,
      tie: a.tie + b.tie,
      loss: a.loss + b.loss,
    };
  }
</script>

<script lang="ts">
  export let bot: object;
  export let botStats: object;
  export let leaderboard: object[];

  $: mergedStats = calcMergedStats(botStats);
</script>

<div class="container">
  <table class="leaderboard">
    <tr class="leaderboard-row leaderboard-header">
      <th class="leaderboard-rank">Rank</th>
      <th class="leaderboard-rating">Rating</th>
      <th class="leaderboard-bot">Bot</th>
      <th class="leaderboard-author">Author</th>
      <th>Winrate</th>
      <th>Matches</th>
    </tr>
    {#each leaderboard as entry, index}
      <tr class="leaderboard-row">
        <td class="leaderboard-rank">{index + 1}</td>
        <td class="leaderboard-rating">
          {entry["rating"].toFixed(0)}
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
        {#if mergedStats[entry["bot"]["name"]]}
          <td>
            {winRate(mergedStats[entry["bot"]["name"]]).toFixed(2)}
          </td>
          <td>
            <a href={`/matches?bot=${bot["name"]}&opponent=${entry["bot"]["name"]}`}>view matches</a
            >
          </td>
        {:else}
          <td />
          <td>no matches yet </td>{/if}
      </tr>
    {/each}
  </table>
</div>

<style lang="scss">
  .container {
    width: 800px;
    max-width: 80%;
    margin: 50px auto;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: 60px;
    border-bottom: 1px solid black;
  }

  $header-space-above-line: 12px;

  .bot-name {
    font-size: 24pt;
    margin-bottom: $header-space-above-line;
  }

  .owner-name {
    font-size: 14pt;
    text-decoration: none;
    color: #333;
    margin-bottom: $header-space-above-line;
  }

  .leaderboard {
    margin: 18px 10px;
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
