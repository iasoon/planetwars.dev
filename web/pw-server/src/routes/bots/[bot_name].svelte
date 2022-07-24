<script lang="ts" context="module">
  import { get_session_token } from "$lib/auth";

  export async function load({ params, fetch }) {
    const token = get_session_token();
    const res = await fetch(`/api/bots/${params["bot_name"]}`, {
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
    });

    if (res.ok) {
      const { bot, owner, versions } = await res.json();
      // sort most recent first
      versions.sort((a: string, b: string) =>
        dayjs(a["created_at"]).isAfter(b["created_at"]) ? -1 : 1
      );
      return {
        props: {
          bot,
          owner,
          versions,
        },
      };
    }

    return {
      status: res.status,
      error: new Error("Could not find bot"),
    };
  }
</script>

<script lang="ts">
  import dayjs from "dayjs";

  import { currentUser } from "$lib/stores/current_user";

  export let bot: object;
  export let owner: object;
  export let versions: object[];

  // function last_updated() {
  //   versions.sort()
  // }

  // let files;

  // async function submitCode() {
  //   console.log("click");
  //   const token = get_session_token();

  //   const formData = new FormData();
  //   formData.append("File", files[0]);

  //   const res = await fetch(`/api/bots/${bot["id"]}/upload`, {
  //     method: "POST",
  //     headers: {
  //       // the content type header will be set by the browser
  //       Authorization: `Bearer ${token}`,
  //     },
  //     body: formData,
  //   });

  //   console.log(res.statusText);
  // }
</script>

<!-- 
<div>Upload code</div>
<form on:submit|preventDefault={submitCode}>
  <input type="file" bind:files />
  <button type="submit">Submit</button>
</form> -->

<div class="container">
  <div class="header">
    <h1 class="bot-name">{bot["name"]}</h1>
    {#if owner}
      <a class="owner-name" href="/users/{owner['username']}">
        {owner["username"]}
      </a>
    {/if}
  </div>

  {#if $currentUser && $currentUser["user_id"] === bot["owner_id"]}
    <div>
      <!-- TODO: can we avoid hardcoding the url? -->
      Publish a new version by pushing a docker container to
      <code>registry.planetwars.dev/{bot["name"]}:latest</code>, or using the web editor.
    </div>
  {/if}

  <div class="versions">
    <h4>Versions</h4>
    <ul class="version-list">
      {#each versions as version}
        <li>
          {dayjs(version["created_at"]).format("YYYY-MM-DD HH:mm")}
        </li>
      {/each}
    </ul>
    {#if versions.length == 0}
      This bot does not have any versions yet.
    {/if}
  </div>
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

  .versions {
    margin: 30px 0;
  }
</style>
