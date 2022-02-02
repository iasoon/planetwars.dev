<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  let editor;

  onMount(async () => {
    const ace = await import("ace-builds");
    editor = ace.edit("editor");
  });

  async function submitCode() {
    if (editor === undefined) {
      return;
    }

    let response = await fetch("/api/submit_bot", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        code: editor.getValue(),
      }),
    });

    if (!response.ok) {
      throw Error(response.statusText);
    }

    let responseData = await response.json();
    let matchId = responseData["match_id"];
    goto(`/submission_matches/${matchId}`);
  }
</script>

<h1>Planetwars</h1>
<div id="editor" />
<button on:click={submitCode}>Submit</button>

<style scoped>
  #editor {
    width: 100%;
    max-width: 800px;
    height: 600px;
  }
</style>
