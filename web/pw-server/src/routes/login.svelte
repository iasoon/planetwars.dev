<script lang="ts">
  import { get_session_token, set_session_token } from "$lib/auth";
  import { goto } from "$app/navigation";

  let username: string | undefined;
  let password: string | undefined;

  async function login() {
    let response = await fetch("/api/login", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        username,
        password,
      }),
    });

    if (!response.ok) {
      throw Error(response.statusText);
    }

    let token = response.headers.get("Token");
    set_session_token(token);

    let user = await response.json();

    goto("/");
  }

  function loggedIn(): boolean {
    let session = get_session_token();
    return session !== null && session !== undefined;
  }
</script>

{#if loggedIn()}
  you are logged in
{/if}

<form on:submit|preventDefault={login}>
  <label for="username">Username</label>
  <input name="username" bind:value={username} />
  <label for="password">Password</label>
  <input type="password" name="password" bind:value={password} />
  <button type="submit">Log in</button>
</form>
