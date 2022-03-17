<script lang="ts">
  import "./style.css";
</script>

<div class="outer-container">
  <div class="navbar">
    <div class="navbar-main">
      <a href="/">PlanetWars</a>
    </div>
    <div class="navbar-user">
      <a class="navbar-href" href="login">Sign in</a>
      <a class="navbar-href" href="register">Sign up</a>
    </div>
  </div>
  <slot />
</div>

<style lang="scss">
  @import "../lib/variables.scss";

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

  .navbar-user a {
    font-size: 18px;
    color: #eee;
    font-family: Helvetica, sans-serif;
    padding: 4px 8px;
    text-decoration: none;
  }

  .navbar-main {
    margin: auto 0;
  }

  .navbar-main a {
    font-size: 20px;
    color: #eee;
    text-decoration: none;
  }

  .navbar-user {
    margin: auto 0;
  }
</style>
