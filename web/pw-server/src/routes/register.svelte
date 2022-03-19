<script lang="ts">
  let username: string | undefined;
  let password: string | undefined;

  const onSubmit = () => {
    if (username === undefined || username.trim() === "") {
      return;
    }

    if (password === undefined || password.trim() === "") {
      return;
    }

    fetch("/api/register", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        username,
        password,
      }),
    })
      .then((resp) => resp.json())
      .then((data) => {
        console.log(data);
      });
  };
</script>

<div class="page-card">
  <div class="page-card-content">
    <h1 class="page-card-header">Create account</h1>
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
</style>