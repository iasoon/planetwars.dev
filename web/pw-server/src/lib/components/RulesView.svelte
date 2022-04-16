<div class="game-rules">
  <p>
    Every turn, your bot will receive a json-encoded line on stdin, representing the current game
    state.
  </p>

  Example game state:
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
    Every turn, you may send out expeditions to conquer other planets. You can do this by writing a
    json-encoded line to stdout:
  </p>

  Example command:
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

  The amount of turns an expedition will travel is equal to the ceiled euclidean distance between
  its origin and target planet.
</div>

<style lang="scss">
  .game-rules {
    padding: 15px;
    overflow-y: scroll;
    height: 100%;
    box-sizing: border-box;
  }
</style>
