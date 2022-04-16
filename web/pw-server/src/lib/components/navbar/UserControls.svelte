<script lang="ts">
  import { get_session_token, clear_session_token } from "$lib/auth";
  import { currentUser } from "$lib/stores/current_user";

  import { onMount } from "svelte";

  onMount(async () => {
    // TODO: currentUser won't be set if the navbar component is not created.
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

    const user = await response.json();
    currentUser.set(user);
  });

  function signOut() {
    // TODO: destroy session on server
    currentUser.set(null);
    clear_session_token();
  }
</script>

<div class="user-controls">
  {#if $currentUser}
    <div class="current-user-name">
      {$currentUser["username"]}
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
