<script lang="ts">
  import { goto } from "$app/navigation";
  import dayjs from "dayjs";

  export let matches: object[];

  function match_url(match: object) {
    return `/matches/${match["id"]}`;
  }
</script>

<table class="matches-table">
  <tr>
    <th class="header-timestamp">timestamp</th>
    <th class="col-player-1">player 1</th>
    <th />
    <th />
    <th class="col-player-2">player 2</th>
  </tr>
  {#each matches as match}
    <tr class="match-table-row" on:click={() => goto(match_url(match))}>
      <td class="col-timestamp">
        {dayjs(match["timestamp"]).format("YYYY-MM-DD HH:mm")}
      </td>
      <td class="col-player-1">
        {match["players"][0]["bot_name"]}
      </td>
      {#if match["winner"] == null}
        <td class="col-score-player-1"> TIE </td>
        <td class="col-score-player-2"> TIE </td>
      {:else if match["winner"] == 0}
        <td class="col-score-player-1"> WIN </td>
        <td class="col-score-player-2"> LOSS </td>
      {:else if match["winner"] == 1}
        <td class="col-score-player-1"> LOSS </td>
        <td class="col-score-player-2"> WIN </td>
      {/if}
      <td class="col-player-2">
        {match["players"][1]["bot_name"]}
      </td>
    </tr>
  {/each}
</table>

<style lang="scss">
  .matches-table td,
  .matches-table th {
    padding: 8px 16px;
    // width: 100%;
  }

  .header-timestamp {
    text-align: left;
  }

  .col-timestamp {
    color: #555;
  }

  .col-player-1 {
    text-align: left;
  }

  .col-player-2 {
    text-align: right;
  }

  @mixin col-player-score {
    text-transform: uppercase;
    font-weight: 600;
    font-size: 14px;
    font-family: "Open Sans", sans-serif;
  }

  .col-score-player-1 {
    @include col-player-score;
    text-align: right;
  }

  .col-score-player-2 {
    @include col-player-score;
    text-align: left;
  }

  .matches-table {
    margin: 12px auto;
    border-collapse: collapse;
  }

  .match-table-row:hover {
    cursor: pointer;
    background-color: #eee;
  }
</style>
