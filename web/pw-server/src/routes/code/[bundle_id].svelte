<script lang="ts" context="module">
  import { ApiClient } from "$lib/api_client";

  export async function load({ params, fetch }) {
    const apiClient = new ApiClient(fetch);

    try {
      const code = await apiClient.getText(`/api/code/${params["bundle_id"]}`);
      return {
        props: {
          code,
        },
      };
    } catch (error) {
      return {
        status: error.status,
        error: error,
      };
    }
  }
</script>

<script lang="ts">
  export let code;
</script>

<pre class="bot-code">
  {code}
</pre>

<style lang="scss">
  .bot-code {
    margin: 24px 12px;
  }
</style>
