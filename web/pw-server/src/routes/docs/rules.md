# How to play

## Protocol

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

You can dispatch as many expeditions as you like.
```

## Rules

All players send out their commands simultaneously, so there is no player order.

The amount of turns an expedition will travel is equal to the ceiled euclidean distance
between its origin and destination planet.

Each turn, one additional ship will be constructed on each player-owned planet.
Neutral planets do not construct ships.

Ships will only battle on planets. Combat resolution is simple: every ship destroys one enemy
ship, last man standing gets to keep the planet. When no player has ships remaining, the planet will turn neutral.

A turn progresses as follows:

1. Construct ships
2. Dispatch expeditions
3. Arrivals & combat resolution

It is not allowed for players to abandon a planet - at least one ship should remain at all times.
Note that you are still allowed to dispatch the full ship count you observe in the game state,
as an additional ship will be constructed before the ships depart.

The game will end when no enemy player ships remain (neutral ships may survive), or when the
turn limit is reached. When the turn limit is hit, the game will end it a tie.
Currently, the limit is set at 500 turns.

## Writing your bot

You can code a bot in python 3.10 using the [web editor](/editor). A working example bot is provided.
If you'd like to use a different programming language, or prefer coding on your own editor,
you can try [local development](/docs/local-development).

As logging to stdout will be interpreted as commands by the game server, we suggest you log to stderr.  
In python, you can do this using

```python
print("hello world", file=sys.stderr)
```

Output written to stderr will be displayed alongside the match replay.

Feel free to launch some test matches to get the hang of it!
