<script lang="ts" context="module">
  import { get_session_token } from "$lib/auth";

  export async function load({ page }) {
    const token = get_session_token();
    const res = await fetch(`/api/bots/${page.params["bot_id"]}`, {
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
    });

    if (res.ok) {
      return {
        props: {
          bot: await res.json(),
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
  export let bot: object;
</script>

<div>
  {bot["name"]}
</div>
