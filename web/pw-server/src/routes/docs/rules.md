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
```

You can dispatch as many expeditions as you like.

## Rules

All players decide on their commands simultaneously, so there is no player order.

After all players have issued their commands, these steps happen in order:

1.  **Construct ships**  
    One new ship is constructed on every planet that is owned by a player.  
    Neutral planets do not construct ships.

2.  **Dispatch expeditions**  
    All ordered expeditions will depart.

    Note: The amount of ships that can be dispatched from a planet is the planet ship count you received in your gamestate. It is not allowed to dispatch the ship constructed in the previous step, as one ship should remain on the planet to maintain control.

3.  **Ship movement**  
    All in-flight expeditions will move one step towards their destination.

    Once an expedition has travelled for an amount of turns equal to the _ceiled euclidean distance between its origin and destination planet_, it will arrive at its destination planet.

    Note that expeditions with a travel time of one turn will arrive immediately.

4.  **Combat**  
    When multiple owners have fleets at a planet, they will enter combat.
    - First, fleets belonging to the same owner will be merged, so that each owner has one fleet.
    - The largest fleet will win the combat. Its owner will gain control of the planet, losing an amount of ships equal to the second-largest fleet. All other fleets will be destroyed.
    - When there is a tie for the largest fleet, all fleets will be destroyed, and the planet will be neutral.

The game will end either when no enemy player ships remain (neutral ships may survive), or when the turn limit is reached. If the turn limit was reached, the game ends in a tie.  
Currently, the turn limit is set at 500 turns.

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
