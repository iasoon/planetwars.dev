<script lang="ts" context="module">
  import { get_session_token } from "$lib/auth";
  import { mount_component } from "svelte/internal";

  export async function load({ page }) {
    const token = get_session_token();
    const res = await fetch(`/api/bots/${page.params["bot_id"]}`, {
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
    });

    if (res.ok) {
      const data = await res.json();
      return {
        props: {
          bot: data["bot"],
          bundles: data["bundles"],
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
  import dayjs from "dayjs";

  export let bot: object;
  export let bundles: object[];

  let files;

  async function submitCode() {
    console.log("click");
    const token = get_session_token();

    const formData = new FormData();
    formData.append("File", files[0]);

    const res = await fetch(`/api/bots/${bot["id"]}/upload`, {
      method: "POST",
      headers: {
        // the content type header will be set by the browser
        Authorization: `Bearer ${token}`,
      },
      body: formData,
    });

    console.log(res.statusText);
  }
</script>

<div>
  {bot["name"]}
</div>

<div>Upload code</div>
<form on:submit|preventDefault={submitCode}>
  <input type="file" bind:files />
  <button type="submit">Submit</button>
</form>

<ul>
  {#each bundles as bundle}
    <li>
      bundle created at {dayjs(bundle["created_at"]).format("YYYY-MM-DD HH:mm")}
    </li>
  {/each}
</ul>
