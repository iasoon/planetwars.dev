<script lang="ts">
	let username: string | undefined;
	let password: string | undefined;

	const onSubmit = () => {
		if (username === undefined || username.trim() === '') {
			return;
		}

    if (password === undefined || password.trim() === '') {
      return;
    }

		fetch('/api/register', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				username,
				password
			})
		})
			.then((resp) => resp.json())
			.then((data) => {
				console.log(data);
			});
	};
</script>

<h1>Register</h1>
<form on:submit|preventDefault={onSubmit}>
	<label for="username">Username</label>
	<input name="username" bind:value={username} />
	<label for="password">Password</label>
	<input type="password" name="password" bind:value={password} />
	<button type="submit">Register</button>
</form>
