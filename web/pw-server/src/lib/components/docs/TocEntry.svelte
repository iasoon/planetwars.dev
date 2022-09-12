<script lang="ts">
  export let href: string;
  export let text: string;
</script>

<div class="toc-entry">
  <a {href}>{text}</a>
</div>

<style scoped lang="scss">
  @use "src/styles/variables";

  .toc-entry {
    font-size: 16px;
    padding: 4px;
  }

  .toc-entry a {
    color: variables.$blue-primary;
  }
</style>
