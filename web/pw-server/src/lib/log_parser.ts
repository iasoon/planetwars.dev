export type PlayerLog = PlayerLogTurn[];

export type PlayerLogTurn = {
  action?: PlayerAction;
  stderr: string[];
};

type PlayerAction = Timeout | BadCommand | Dispatches;

type Timeout = {
  type: "timeout";
};

type BadCommand = {
  type: "bad_command";
  command: string;
  error: string;
};

type Dispatches = {
  type: "dispatches";
  dispatches: Dispatch[];
};

type Dispatch = {
  origin: string;
  destination: string;
  ship_count: number;
  error?: string;
};

function createEmptyLogTurn(): PlayerLogTurn {
  return {
    stderr: [],
  };
}

export function parsePlayerLog(playerId: number, logText: string): PlayerLog {
  const logLines = logText.split("\n").slice(0, -1);

  const playerLog: PlayerLog = [];

  let turn = null;

  logLines.forEach((logLine) => {
    const logMessage = JSON.parse(logLine);

    if (logMessage["type"] === "gamestate") {
      if (turn) {
        playerLog.push(turn);
        turn = createEmptyLogTurn();
      }
    } else if (logMessage["player_id"] === playerId) {
      if (!turn) {
        // older match logs don't have an initial game state due to a bug.
        turn = createEmptyLogTurn();
      }
      switch (logMessage["type"]) {
        case "stderr": {
          let msg = logMessage["message"];
          turn.stderr.push(msg);
          break;
        }
        case "timeout":
        case "bad_command":
        case "dispatches": {
          turn.action = logMessage;
          break;
        }
      }
    }
  });

  return playerLog;
}
