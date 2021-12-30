<script lang="ts">
	import { get_session_token, set_session_token } from '$lib/auth';
  import { goto } from '$app/navigation';

	let username: string | undefined;
	let password: string | undefined;

	const onSubmit = () => {
		fetch('/api/login', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				username,
				password
			})
		})
			.then((response) => {
        if (!response.ok) {
          throw Error(response.statusText);
        }
        return response.text();
      })
			.then((token) => {
				set_session_token(token);
        goto("/")
			});
	};

  function loggedIn(): boolean {
    return get_session_token() != null
  }
</script>

{#if loggedIn()}
  you are logged in 
{/if}

<form on:submit|preventDefault={onSubmit}>
	<label for="username">Username</label>
	<input name="username" bind:value={username} />
	<label for="password">Password</label>
	<input type="password" name="password" bind:value={password} />
	<button type="submit">Log in</button>
</form>
