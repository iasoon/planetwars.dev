<script lang="ts">
  import type { BotMatch } from "$lib/matches";
  import dayjs from "dayjs";

  export let botMatch: BotMatch;
</script>

<a class="bot-match-card-wrapper" href={`/matches/${botMatch.id}`}>
  <div class="bot-match-card">
    <div class="bot-match-outcome">
      {botMatch.outcome}
    </div>
    <div class="bot-match-card-main">
      <div class="opponent-name">
        <a class="bot-link" href={`/bots/${botMatch.opponent.bot_name}`}
          >{botMatch.opponent.bot_name}</a
        >
      </div>
      <div class="map-name">
        {botMatch.map.name}
      </div>
    </div>
    <div class="bot-card-right">
      <div class="match-timestamp">
        {dayjs(botMatch.timestamp).format("YYYY-MM-DD HH:mm")}
      </div>
      <div class="match-errors">
        {#if botMatch.hadErrors}
          ! Had errors
        {/if}
      </div>
    </div>
  </div>
</a>

<style lang="scss">
  @use "src/styles/variables";

  .bot-match-card {
    margin: 4px;
    padding: 12px;
    display: flex;
    border: 1px solid variables.$light-grey;
  }

  .bot-match-outcome {
    font-size: 24pt;
    font-weight: 600;
    font-family: "Open Sans", sans-serif;
    text-transform: uppercase;
    color: #333;
    width: 75px;
    display: flex;
    justify-content: center;
  }

  .bot-match-card-main {
    flex-grow: 1;
    padding: 0 25px;
  }

  .bot-card-right {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: flex-end;
  }

  .opponent-name {
    font-size: 18pt;
    color: variables.$blue-primary;
    padding-bottom: 4px;
  }

  .match-errors {
    color: red;
    font-weight: 600;
    text-align: right;
  }

  .match-timestamp {
    text-align: right;
  }

  .bot-link {
    color: variables.$blue-primary;
    text-decoration: none;
  }

  .bot-link:hover {
    text-decoration: underline;
  }

  .bot-match-card:hover {
    background-color: rgb(246, 248, 250);
  }

  .bot-match-card-wrapper {
    text-decoration: none;
    color: black;
    display: block;
  }
</style>
