<script lang="ts">
  let code = "";

  async function submitCode() {
    console.log("click");
    let response = await fetch("/api/submit_bot", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        "code": code,
      }),
    });

    if (!response.ok) {
      throw Error(response.statusText);
    }

    let responseData = await response.json()
    console.log(responseData);
  }
</script>

<h1>Planetwars</h1>
<textarea bind:value={code} />
<button on:click={submitCode}>Submit</button>