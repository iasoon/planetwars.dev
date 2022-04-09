<script lang="ts">
  import * as auth from "$lib/auth";
  import { goto } from "$app/navigation";

  let username: string | undefined;
  let password: string | undefined;

  let error: string | undefined;

  async function submitLogin() {
    try {
      error = undefined;
      await auth.login({ username, password });
      goto("/");
    } catch (e) {
      error = e.message;
    }
  }

  function loggedIn(): boolean {
    let session = auth.get_session_token();
    return session !== null && session !== undefined;
  }
</script>

<div class="page-card">
  <div class="page-card-content">
    <h1 class="page-card-header">Sign in</h1>
    {#if error}
      <div class="error-message">{error}</div>
    {/if}
    <form class="account-form" on:submit|preventDefault={submitLogin}>
      <label for="username">Username</label>
      <input name="username" bind:value={username} />
      <label for="password">Password</label>
      <input type="password" name="password" bind:value={password} />
      <button type="submit">Submit</button>
    </form>
  </div>
</div>

<style lang="scss">
  @import "src/styles/account_forms.scss";
  .error-message {
    color: red;
  }
</style>
