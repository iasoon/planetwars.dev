<script lang="ts">
  import { get_session_token } from "$lib/auth";

  import { onMount } from "svelte";

  let user = null;

  onMount(async () => {
    const session_token = get_session_token();
    if (!session_token) {
      return;
    }

    let response = await fetch("/api/users/me", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${session_token}`,
      },
    });

    if (!response.ok) {
      throw response.statusText;
    }

    user = await response.json();
  });

  function signOut() {
    // TODO: destroy session on server
    user = null;
  }
</script>

<div class="user-controls">
  {#if user}
    <div class="current-user-name">
      {user["username"]}
    </div>
    <div class="sign-out" on:click={signOut}>Sign out</div>
  {:else}
    <a class="account-href" href="login">Sign in</a>
    <a class="account-href" href="register">Sign up</a>
  {/if}
</div>

<style lang="scss">
  @mixin navbar-item {
    font-size: 18px;
    font-family: Helvetica, sans-serif;
    padding: 4px 8px;
  }

  .account-href {
    @include navbar-item;
    color: #eee;
    text-decoration: none;
  }

  .current-user-name {
    @include navbar-item;
    color: #fff;
  }

  .sign-out {
    @include navbar-item;
    color: #ccc;
    cursor: pointer;
  }

  .user-controls {
    display: flex;
    align-items: center;
  }
</style>
