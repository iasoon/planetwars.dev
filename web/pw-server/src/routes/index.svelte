<script lang="ts">
  import { goto } from "$app/navigation";

  let code = "";

  async function submitCode() {
    console.log("click");
    let response = await fetch("/api/submit_bot", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        code: code,
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
<textarea bind:value={code} />
<button on:click={submitCode}>Submit</button>
