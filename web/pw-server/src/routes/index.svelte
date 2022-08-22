<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  const NUM_MATCHES = "25";

  export async function load({ fetch }) {
    try {
      const apiClient = new ApiClient(fetch);

      let { matches, has_next } = await apiClient.get("/api/matches", {
        count: NUM_MATCHES,
      });

      return {
        props: {
          matches,
          hasNext: has_next,
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
  import LinkButton from "$lib/components/LinkButton.svelte";
  import MatchList from "$lib/components/matches/MatchList.svelte";

  export let matches;
  export let hasNext;

  $: viewMoreUrl = olderMatchesLink(matches);

  // TODO: deduplicate.
  // Maybe move to ApiClient logic?
  function olderMatchesLink(matches: object[]): string {
    if (matches.length == 0 || !hasNext) {
      return null;
    }
    const lastTimestamp = matches[matches.length - 1]["timestamp"];
    return `/matches?before=${lastTimestamp}`;
  }
</script>

<div class="container">
  <div class="introduction">
    <h2>Welcome to PlanetWars!</h2>
    <p>
      Planetwars is a game of galactic conquest for busy people. Your goal is to program a bot that
      will conquer the galaxy for you, while you take care of more important stuff.
    </p>
    <p>
      Feel free to watch some games below to see what it's all about. When you are ready to try
      writing your own bot, head over to
      <a href="/docs">How to play</a> for instructions. You can program your bot in the browser
      using the <a href="/editor">Editor</a>.
    </p>
  </div>
  <h2>Recent matches</h2>
  <MatchList {matches} />
  <div class="see-more-container">
    <LinkButton href={viewMoreUrl}>View more</LinkButton>
  </div>
</div>

<style scoped lang="scss">
  .container {
    max-width: 800px;
    margin: 0 auto;
  }

  .introduction {
    padding-top: 16px;
    a {
      color: rgb(9, 105, 218);
      text-decoration: none;
    }

    a:hover {
      text-decoration: underline;
    }
  }

  .see-more-container {
    padding: 24px;
    text-align: center;
  }
</style>
