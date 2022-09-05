<script lang="ts">
  import UserControls from "$lib/components/navbar/UserControls.svelte";

  import "./style.css";
</script>

<div class="outer-container">
  <div class="navbar">
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
    </div>
    <div class="navbar-right">
      <UserControls />
    </div>
  </div>
  <slot />
</div>

<style lang="scss" global>
  @import "src/styles/global.scss";

  .outer-container {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .navbar {
    height: 48px;
    background-color: $bg-color;
    border-bottom: 1px solid;
    flex-shrink: 0;
    display: flex;
    justify-content: space-between;
    padding: 0 15px;
  }

  .navbar-left {
    display: flex;
  }

  .navbar-right {
    display: flex;
  }

  .navbar-header {
    margin: auto 0;
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
</style>
