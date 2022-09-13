<script>
  import TocEntry from "$lib/components/docs/TocEntry.svelte";
</script>

<div class="container">
  <div class="sidebar">
    <div class="sidebar-content">
      <h2>Docs</h2>
      <div class="sidebar-nav-group">
        <TocEntry href="/docs/rules" text="Rules" />
        <TocEntry href="/docs/local-development" text="Local development" />
      </div>
    </div>
  </div>
  <div class="content-container">
    <div class="content">
      <slot />
    </div>
  </div>
</div>

<style lang="scss">
  @use "src/styles/variables";

  .container {
    display: flex;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell,
      "Open Sans", "Helvetica Neue", sans-serif;
    flex-grow: 1;
  }

  .sidebar {
    width: 300px;
    line-height: 20px;
    border-right: 1px solid variables.$light-grey;
  }

  .sidebar-content {
    padding: 32px 16px;
    position: sticky;
    align-self: flex-start;
    top: 0px;
  }

  .sidebar-content h2 {
    margin: 16px 0;
  }

  .sidebar-nav-group {
    padding: 0 8px;
  }

  .content-container {
    padding: 16px 0;
    flex-grow: 1;
  }

  .content {
    max-width: 75%;
    margin: 0 auto;
  }
</style>
