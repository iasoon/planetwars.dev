# planetwars

Planetwars is a competitive programming game. You implement a bot that will be pitted against all other bots.

Try it out at https://planetwars.dev !

Current features:
- write and publish a python bot in the demo web interface
- develop a bot locally and publish it as a docker container
- published bots will be ranked in the background


## Creating a bot locally
For development, you can play a game with a locally running bot using [`planetwars-client`](https://github.com/iasoon/planetwars.dev/tree/main/planetwars-client). \
Once you are happy with your bot, you can publish it to the planetwars server as a docker container.

1. Register your bot. In order to publish a bot version, you first have to register a bot name. You can do this by navigating to your profile after logging in (click your name in the navbar).
2. Bake your bot into a docker container. If you'd like to test whether your container works, you can try running it using `planetwars-client` by using `docker run -it my-bot-tag` as the run command.
3. Log in to the planetwars docker registry: `docker login registry.planetwars.dev`
4. Tag and push your bot to `registry.planetwars.dev/my-bot-name:latest`.
5. Your bot should be up and running now! Feel free to play a game against  it to test whether all is well. Shortly, your bot should appear in the rankings.


## Project
The repository contains these components:
- `planetwars-server`: rust webserver
- `planetwars-matchrunner`: code for running matches
- `planetwars-rules`: implements the game rules
- `planetwars-client`: for running your bot locally
- `web/pw-server`: frontend
- `web/pw-visualizer`: code for the visualizer
