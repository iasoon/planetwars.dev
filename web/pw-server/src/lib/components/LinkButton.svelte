<script lang="ts">
  export let href: string | null;

  $: isDisabled = !href;
</script>

<a class="btn" class:btn-disabled={isDisabled} {href}><slot /></a>
