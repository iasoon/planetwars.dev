<div class="container">
  <slot />
</div>

<style scoped lang="scss">
  @use "src/styles/variables";
  .container {
    max-width: 800px;
    color: rgb(36, 41, 47);
  }

  .container a {
    color: variables.$blue-primary;
  }
</style>
