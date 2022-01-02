<script lang="ts" context="module">
  import { get_session_token } from "$lib/auth";
  import { mount_component } from "svelte/internal";

  export async function load({ page }) {
    const token = get_session_token();
    const res = await fetch("/api/bots", {
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
    });

    if (res.ok) {
      return {
        props: {
          bots: await res.json(),
        },
      };
    }

    return {
      status: res.status,
      error: new Error("Could not load bot"),
    };
  }
</script>

<script lang="ts">
  import Select from "svelte-select";
import { goto } from "$app/navigation";
  export let bots: object[];
  let items: object[];
  let players: object[] = [];
  let selected: object | null = null;

  $: items = bots.map((bot) => {
    return {
      value: bot["id"],
      label: bot["name"],
    };
  });

  function addPlayer() {
    if (selected === null) {
      return;
    }

    players.push(selected);
    players = players;
  }

  async function submitMatch() {
    const token = get_session_token();
    const res = await fetch("/api/matches", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({
        players: players.map((player) => player["value"]),
      }),
    });

    if (res.ok) {
      // TODO
      goto("/matches")
    } else {
      alert(res.statusText);
    }
  }
</script>

Select players:
<Select {items} bind:value={selected} />
<button on:click={addPlayer}>add</button>
<h3>Selected Players</h3>
<ol>
  {#each players as player}
    <li>{player["label"]}</li>
  {/each}
</ol>
<button on:click={submitMatch}>Play</button>
