
## How to play
In every game turn, your bot will receive a json-encoded line on stdin, describing the current
state of the game. Each state will hold a set of planets, and a set of spaceship fleets
traveling between the planets (_expeditions_).

Example game state:

```json
  {
    "planets": [
      {
        "ship_count": 2,
        "x": -2.0,
        "y": 0.0,
        "owner": 1,
        "name": "your planet"
      },
      {
        "ship_count": 4,
        "x": 2.0,
        "y": 0.0,
        "owner": 2,
        "name": "enemy planet"
      },
      {
        "ship_count": 2,
        "x": 0.0,
        "y": 2.0,
        "owner": null,
        "name": "neutral planet"
      }
    ],
    "expeditions": [
      {
        "id": 169,
        "ship_count": 8,
        "origin": "your planet",
        "destination": "enemy planet",
        "owner": 1,
        "turns_remaining": 2
      }
    ]
  }
```

The `owner` field holds a player number when the planet is held by a player, and is
`null` otherwise. Your bot is always referred to as player 1.  
Each turn, every player-owned planet will gain one additional ship.  
Planets will never move during the game.

Every turn, you may send out expeditions to conquer other planets. You can do this by writing
a json-encoded line to stdout:

Example command:
```json
  {
    "moves": [
      {
        "origin": "your planet",
        "destination": "enemy planet",
        "ship_count": 2
      }
    ]
  }
```

All players send out their commands simultaneously, so there is no turn order. You may send as
many commands as you please.

The amount of turns an expedition will travel is equal to the ceiled euclidean distance
between its origin and destination planet.

Ships will only battle on planets. Combat resolution is simple: every ship destroys one enemy
ship, last man standing gets to keep the planet.

The game will end when no enemy player ships remain (neutral ships may survive), or when the
turn limit is reached. The default limit is 100 turns.

You can code your bot in python 3.10. You have the entire stdlib at your disposal.  
If you'd like additional libraries or a different programming language, feel free to nag the administrator.

### TL;DR
Head over to the editor view to get started - a working example is provided.  
Feel free to just hit the play button to see how it works!
