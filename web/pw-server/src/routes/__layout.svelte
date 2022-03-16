<script lang="ts">
  import "./style.css";
</script>

<div class="outer-container">
  <div class="top-bar" />
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

  .top-bar {
    height: 40px;
    background-color: $bg-color;
    border-bottom: 1px solid;
    flex-shrink: 0;
  }
</style>
