<script lang="ts">
  let username: string | undefined;
  let password: string | undefined;

  let registrationErrors: string[] = [];

  const onSubmit = async () => {
    if (username === undefined || username.trim() === "") {
      return;
    }

    if (password === undefined || password.trim() === "") {
      return;
    }

    registrationErrors = [];

    const response = await fetch("/api/register", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        username,
        password,
      }),
    });

    if (!response.ok) {
      const resp = await response.json();
      const error = resp["error"];
      if (response.status == 422 && error["type"] === "validation_failed") {
        registrationErrors = error["validation_errors"];
      }
    }
  };
</script>

<div class="page-card">
  <div class="page-card-content">
    <h1 class="page-card-header">Create account</h1>
    {#if registrationErrors.length > 0}
      <ul class="error-message-list">
        {#each registrationErrors as errorMessage}
          <li class="error-message">{errorMessage}</li>
        {/each}
      </ul>
    {/if}

    <form class="account-form" on:submit|preventDefault={onSubmit}>
      <label for="username">Username</label>
      <input name="username" bind:value={username} />
      <label for="password">Password</label>
      <input type="password" name="password" bind:value={password} />
      <button type="submit">Submit</button>
    </form>
  </div>
</div>

<style lang="scss">
  @import "src/styles/account_forms.scss";

  .error-message-list {
    color: red;
  }
</style>
