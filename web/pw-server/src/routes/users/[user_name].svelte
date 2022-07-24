<script lang="ts" context="module">
  export async function load({ params, fetch }) {
    const userName = params["user_name"];
    const userBotsResponse = await fetch(`/api/users/${userName}/bots`);
    return {
      props: {
        userName,
        bots: await userBotsResponse.json(),
      },
    };

    // return {
    //   status: matchDataResponse.status,
    //   error: new Error("failed to load match"),
    // };
  }
</script>

<script lang="ts">
  export let userName: string;
  export let bots: object[];
</script>

<div class="container">
  <div class="header">
    <h1 class="user-name">{userName}</h1>
  </div>
  <h2>Bots</h2>
  <ul class="bot-list">
    {#each bots as bot}
      <li class="bot">
        <a class="bot-name" href="/bots/{bot['name']}">{bot["name"]}</a>
      </li>
    {/each}
  </ul>
</div>

<style lang="scss">
  .container {
    width: 800px;
    max-width: 80%;
    margin: 50px auto;
  }

  .header {
    margin-bottom: 60px;
    border-bottom: 1px solid black;
  }

  .user-name {
    margin-bottom: 0.5em;
  }

  .bot-list {
    list-style: none;
    padding: 0;
  }

  $border-color: #d0d7de;

  .bot {
    display: block;
    padding: 24px 0;
    border-bottom: 1px solid $border-color;
  }

  .bot-name {
    font-size: 20px;
    font-weight: 400;
    text-decoration: none;
    color: black;
  }

  .bot:first-child {
    border-top: 1px solid $border-color;
  }
</style>
