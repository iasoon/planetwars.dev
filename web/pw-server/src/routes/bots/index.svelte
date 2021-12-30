<script lang="ts" context="module">
	import { get_session_token } from '$lib/auth';

	/** @type {import('@sveltejs/kit').Load} */
	export async function load({ params, fetch, session, stuff }) {
		const token = get_session_token();
		const res = await fetch('/api/bots/my_bots', {
			headers: {
				'Content-Type': 'application/json',
				Authorization: `Bearer ${token}`
			}
		});

		if (res.ok) {
			return {
				props: {
					bots: await res.json()
				}
			};
		}

		return {
			status: res.status,
			error: new Error('Could not load bots')
		};
	}
</script>

<script lang="ts">
	import { goto } from '$app/navigation';

	export let bots: object[];
	let name: string | undefined;

	async function createBot() {
		const token = get_session_token();
		const res = await fetch('/api/bots', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				Authorization: `Bearer ${token}`
			},
			body: JSON.stringify({
				name: name
			})
		});

		if (res.ok) {
			let bot = await res.json();
			goto(`/bots/${bot['id']}`);
		} else {
			new Error('creation failed');
		}
	}
</script>

<form on:submit|preventDefault={createBot}>
	<label for="name">Name</label>
	<input name="name" bind:value={name}/>
	<button type="submit">Create</button>
</form>

<ul>
	{#each bots as bot}
		<li>
			<a target="_blank" href={`bots/${bot['id']}`}>
				{bot['name']}
			</a>
		</li>
	{/each}
</ul>
