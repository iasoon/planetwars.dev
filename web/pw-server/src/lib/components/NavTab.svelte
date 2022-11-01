<script lang="ts">
  import { page } from '$app/stores';
  export let href: string;

  $: isActive = $page.url.pathname == href;
</script>

<a class="nav-tab" {href} class:active={isActive}>
  <slot></slot>
</a>

<style lang="scss">
  .nav-tab {
    padding: 8px;
    text-decoration: none;
    color: black;
  }

  .nav-tab:hover {
    background-color: rgb(246, 248, 250);
  }

  .nav-tab.active {
    border-bottom: 2px solid #0080ff;
  }
</style>
