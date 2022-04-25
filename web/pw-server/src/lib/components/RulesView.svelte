<div class="container">
  <div class="game-rules">
    <h2 class="title">Welcome to planetwars!</h2>

    <p>
      Planetwars is a game of galactic conquest for busy people. Your goal is to program a bot that
      will conquer the galaxy for you, while you take care of more important stuff.
    </p>
    <p>
      In every game turn, your bot will receive a json-encoded line on stdin, describing the current
      state of the game. Each state will hold a set of planets, and a set of spaceship fleets
      traveling between the planets (<em>expeditions</em>).
    </p>
    <p>Example game state:</p>
    <pre>{`
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
  `}</pre>

    <p>
      The <code>owner</code> field holds a player number when the planet is held by a player, and is
      <code>null</code> otherwise. Your bot is always referred to as player 1.<br />
      Each turn, every player-owned planet will gain one additional ship. <br />
      Planets will never move during the game.
    </p>

    <p>
      Every turn, you may send out expeditions to conquer other planets. You can do this by writing
      a json-encoded line to stdout:
    </p>

    <p>Example command:</p>
    <pre>{`
  {
    "moves": [
      {
        "origin": "your planet",
        "target": "enemy planet",
        "ship_count": 2
      }
    ]
  }
  `}
  </pre>
    <p>
      All players send out their commands simultaneously, so there is no turn order. You may send as
      many commands as you please.
    </p>

    <p>
      The amount of turns an expedition will travel is equal to the ceiled euclidean distance
      between its origin and target planet.
    </p>

    <p>
      Ships will only battle on planets. Combat resolution is simple: every ship destroys one enemy
      ship, last man standing gets to keep the planet.
    </p>

    <p>
      The game will end when no enemy player ships remain (neutral ships may survive), or when the
      turn limit is reached. The default limit is 100 turns.
    </p>

    <p>
      You can code your bot in python 3.10. You have the entire stdlib at your disposal. <br />
      If you'd like additional libraries or a different programming language, feel free to nag the administrator.
    </p>

    <h3 class="tldr">TL;DR</h3>
    <p>
      Head over to the editor view to get started - a working example is provided. <br />
      Feel free to just hit the play button to see how it works!
    </p>
  </div>
</div>

<style lang="scss">
  .container {
    overflow-y: scroll;
    height: 100%;
    box-sizing: border-box;
  }
  .game-rules {
    padding: 15px 30px;
    max-width: 800px;
  }

  .game-rules p {
    padding-top: 1.5em;
  }

  .game-rules .tldr {
    padding-top: 3em;
  }
</style>
