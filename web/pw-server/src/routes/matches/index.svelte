<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  const PAGE_SIZE = "50";

  export async function load({ url, fetch }) {
    try {
      const apiClient = new ApiClient(fetch);
      const botName = url.searchParams.get("bot");

      let query = {
        count: PAGE_SIZE,
        before: url.searchParams.get("before"),
        after: url.searchParams.get("after"),
        bot: botName,
      };

      let { matches, has_next } = await apiClient.get("/api/matches", removeUndefined(query));

      // TODO: should this be done client-side?
      if (query["after"]) {
        matches = matches.reverse();
      }

      return {
        props: {
          matches,
          botName,
          hasNext: has_next,
          query,
        },
      };
    } catch (error) {
      return {
        status: error.status,
        error: new Error("failed to load matches"),
      };
    }
  }

  function removeUndefined(obj: Record<string, string>): Record<string, string> {
    Object.keys(obj).forEach((key) => {
      if (obj[key] === undefined || obj[key] === null) {
        delete obj[key];
      }
    });
    return obj;
  }
</script>

<script lang="ts">
  import MatchList from "$lib/components/matches/MatchList.svelte";

  export let matches: object[];
  export let botName: string | null;
  // whether a next page exists in the current iteration direction (before/after)
  export let hasNext: boolean;
  export let query: object;

  type Cursor = {
    before?: string;
    after?: string;
  };

  function pageLink(cursor: Cursor) {
    let paramsObj = {
      ...cursor,
    };
    if (botName) {
      paramsObj["bot"] = botName;
    }
    const params = new URLSearchParams(paramsObj);
    return `?${params}`;
  }

  function olderMatchesLink(matches: object[]): string {
    if (matches.length == 0 || (query["before"] && !hasNext)) {
      return null;
    }
    const lastTimestamp = matches[matches.length - 1]["timestamp"];
    return pageLink({ before: lastTimestamp });
  }

  function newerMatchesLink(matches: object[]): string {
    if (
      matches.length == 0 ||
      (query["after"] && !hasNext) ||
      // we are viewing the first page, so there should be no newer matches.
      // alternatively, we could show a "refresh" here.
      (!query["before"] && !query["after"])
    ) {
      return null;
    }
    const firstTimestamp = matches[0]["timestamp"];
    return pageLink({ after: firstTimestamp });
  }
</script>

<div class="container">
  <MatchList {matches} />
  <div class="page-controls">
    <div class="btn-group">
      <a class="btn btn-page-prev" href={newerMatchesLink(matches)}>newer</a>
      <a class="btn btn-page-next" href={olderMatchesLink(matches)}>older</a>
    </div>
  </div>
</div>

<style lang="scss">
  .container {
    width: 800px;
    margin: 0 auto;
  }

  .page-controls {
    display: flex;
    justify-content: center;
    margin: 24px 0;
  }
</style>
