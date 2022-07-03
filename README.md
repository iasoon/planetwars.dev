# planetwars

Planetwars is a competitive programming game. You implement a bot that will be pitted against all other bots.

Currently a demo is available at https://demo.planetwars.dev.

current features:
- write your bot in python
- save your bot so that others can play against it
- saved bots are automatically ranked in the background.


At the moment only python is supported, but a more generic docker-based system is in development.


Project layout:
- `planetwars-server`: rust webserver
- `planetwars-matchrunner`: code for running matches
- `planetwars-rules`: implements the game rules
- `web/pw-server`: frontend
- `web/pw-visualizer`: code for the visualizer
