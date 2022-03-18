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
    <form class="register-form" on:submit|preventDefault={onSubmit}>
      <label for="username">Username</label>
      <input name="username" bind:value={username} />
      <label for="password">Password</label>
      <input type="password" name="password" bind:value={password} />
      <button type="submit">Sign up</button>
    </form>
  </div>
</div>

<style lang="scss">
  .page-card {
    margin: 50px auto;
    width: 40%;
    max-width: 600px;
    border: 1px solid #b5b5b5;
    box-sizing: border-box;
    border-radius: 0px;
  }

  .page-card-content {
    margin: 20px 50px;
  }

  .page-card-header {
    padding-top: .5em;
    padding-bottom: 1em;
    text-align: center;
  }

  .register-form {
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    font-size: 18px;
  }

  .register-form label {
    margin: 10px 5px;
    font-weight: 500;
  }

  .register-form input {
    margin: 10px 5px;
    font-size: 1rem;
    // height: 2.5em;
    display: block;
    border-radius: 4px;
    border: 1px solid #b5b5b5;
    padding: .75rem 1rem;
  }

  .register-form button {
    background-color: lightgreen;
    padding: 8px 16px;
    border-radius: 8px;
    border: 0;
    font-size: 18pt;
    display: block;
    margin: 10px auto;
    margin-top: 16px;

  }
</style>