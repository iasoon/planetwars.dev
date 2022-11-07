<script lang="ts">
  import { afterNavigate } from "$app/navigation";

  import "./style.css";
  import Fa from 'svelte-fa'
  import { faBars } from '@fortawesome/free-solid-svg-icons'

  import { get_session_token, clear_session_token } from "$lib/auth";
  import { currentUser } from "$lib/stores/current_user";
  

  import { onMount } from "svelte";

  // TODO: ideally we'd not store this in the user session,
  // and we'd be able to handle this in the load() function
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

  let navbarExpanded = false;

  function toggleExpanded() {
    navbarExpanded = !navbarExpanded;
  }

  afterNavigate(() => {
    navbarExpanded = false;
  });
</script>

<svelte:head>
  <title>Planetwars</title>
</svelte:head>

<div class="outer-container">
  <div class="navbar" class:expanded={navbarExpanded}>
    <a href="/" class="navbar-header">
      <img alt="logo" src="/ship.svg" height="32px'" />
      PlanetWars
    </a>
    <div class="navbar-expand" on:click={toggleExpanded}><Fa icon={faBars} /></div>
    <div class="navbar-items">
      <div class="navbar-item">
        <a href="/editor">Editor</a>
      </div>
      <div class="navbar-item">
        <a href="/leaderboard">Leaderboard</a>
      </div>
      <div class="navbar-item">
        <a href="/docs/rules">How to play</a>
      </div>
      <div class="navbar-divider" />
      {#if $currentUser}
        <div class="navbar-item">
          <a class="current-user-name" href="/users/{$currentUser['username']}">
            {$currentUser["username"]}
          </a>
        </div>
        <div class="navbar-item">
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <div class="sign-out" on:click={signOut}>Sign out</div>
        </div>
      {:else}
        <div class="navbar-item">
          <a class="account-href" href="/login">Sign in</a>
        </div>
        <div class="navbar-item">
          <a class="account-href" href="/register">Sign up</a>
        </div>
      {/if}
    </div>
  </div>
  <slot />
</div>

<style lang="scss" global>
  @import "src/styles/global.scss";

  $navbarHeight: 52px;

  .outer-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .navbar {
    height: $navbarHeight;
    // height: 52px;
    background-color: $bg-color;
    border-bottom: 1px solid;
    flex-shrink: 0;
    display: flex;
    justify-content: space-between;
    padding: 0 15px;
  }

  .navbar-items {
    display: flex;
    width: 100%;
    align-items: center;
  }

  .navbar-right {
    display: flex;
  }

  .navbar-divider {
    flex-grow: 1;
  }

  .navbar-header {
    height: $navbarHeight;
    padding-right: 24px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .navbar-header img {
    display: block;
  }

  .navbar-header {
    font-size: 20px;
    color: #fff;
    font-weight: 500;
    text-decoration: none;
  }
  .navbar-item {
    margin: auto 0;
    padding: 8px;
  }
  .navbar-item a {
    font-size: 14px;
    color: #fff;
    text-decoration: none;
    font-weight: 600;
  }

  .navbar-item a:hover {
    color: #ccc;
  }

  .current-user-name {
    text-decoration: none;
    color: #fff;
  }

  .sign-out {
    color: #ccc;
    cursor: pointer;
  }

  .sign-out:hover {
    color: #fff;
  }

  .navbar-expand {
    color: white;
    display: none;
    font-size: 16px;
    height: $navbarHeight;
    cursor: pointer;
  }
  @media screen and (max-width: 600px) {
    .navbar-items {
      display: none;
    }

    .navbar-divider {
      display: none;
    }

    .navbar.expanded {
      height: auto;
      flex-wrap: wrap;
    }

    .navbar.expanded .navbar-items {
      display: flex;
      flex-direction: column;
      align-items: flex-start;
      width: 100%;
    }

    .navbar-expand {
      display: flex;
      align-items: center;
    }
  }
</style>
