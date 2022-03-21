<script lang="ts">
  import UserControls from "$lib/components/navbar/UserControls.svelte";

  import "./style.css";
</script>

<div class="outer-container">
  <div class="navbar">
    <div class="navbar-main">
      <a href="/">PlanetWars</a>
    </div>
    <UserControls />
  </div>
  <slot />
</div>

<style lang="scss">
  @import "src/styles/variables.scss";

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

  .navbar-main {
    margin: auto 0;
  }

  .navbar-main a {
    font-size: 20px;
    color: #eee;
    text-decoration: none;
  }
</style>
