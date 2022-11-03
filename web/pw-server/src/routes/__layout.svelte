<script lang="ts">
    import { afterNavigate, beforeNavigate } from "$app/navigation";
  import UserControls from "$lib/components/navbar/UserControls.svelte";

  import "./style.css";

  let isExpanded = false;

  function toggleExpanded() {
    isExpanded = !isExpanded;
  }

  afterNavigate(() => {
    isExpanded = false;
  });
</script>

<div class="outer-container">
  <div class="navbar" class:expanded={isExpanded}>
    <div class="navbar-left">
      <div class="navbar-header">
        <a href="/">PlanetWars</a>
      </div>
      <div class="navbar-item">
        <a href="/editor">Editor</a>
      </div>
      <div class="navbar-item">
        <a href="/leaderboard">Leaderboard</a>
      </div>
      <div class="navbar-item">
        <a href="/docs/rules">How to play</a>
      </div>
      <div class="navbar-divider"></div>
      <div class="navbar-item">
        <UserControls />
      </div>

    </div>
    <!-- <div class="navbar-right">
    </div> -->
    <div class="navbar-expand" on:click={toggleExpanded}>
      expand
    </div>
  </div>
  <slot />
</div>

<style lang="scss" global>
  @import "src/styles/global.scss";

  $navbarHeight: 48px;

  .outer-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .navbar {
    height: $navbarHeight;
    background-color: $bg-color;
    border-bottom: 1px solid;
    flex-shrink: 0;
    display: flex;
    justify-content: space-between;
    padding: 0 15px;
  }

  .navbar-left {
    display: flex;
    width: 100%;
  }

  .navbar-right {
    display: flex;
  }

  .navbar-divider {
    flex-grow: 1;
  }

  .navbar-header {
    height: $navbarHeight;
    padding-top: 12px;
    padding-right: 24px;
  }

  .navbar-header a {
    font-size: 20px;
    color: #fff;
    text-decoration: none;
  }
  .navbar-item {
    margin: auto 0;
    padding: 0 8px;
  }
  .navbar-item a {
    font-size: 14px;
    color: #fff;
    text-decoration: none;
    font-weight: 600;
  }

  .navbar-expand {
    color: white;
    // margin: auto 0px;
    display: none;
    font-size: 16px;
    padding-top: 14px;
  }
  @media screen and (max-width: 600px) {
    .navbar-item {
      display: none;
    }

    .navbar-divider {
      display: none;
    }

    .navbar.expanded {
      height: auto;
    }

    .navbar.expanded .navbar-left {
      flex-direction: column;
    }

    .navbar.expanded .navbar-item {
      display: block;
      padding: 8px;
    }

    .navbar-right {
      display: none;
    }

    .navbar.expanded .navbar-right {
      display: flex;
    }

    .navbar-expand {
      display: block;
    }
  }
</style>
