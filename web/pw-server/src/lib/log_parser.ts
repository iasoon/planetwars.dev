export type PlayerLog = PlayerLogTurn[];

export type PlayerLogTurn = {
  action?: PlayerAction;
  stderr: string[];
};

type PlayerAction = BadCommand;

type BadCommand = {
  type: "bad_command";
  command: string;
  error: string;
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
        }
        case "bad_command": {
          turn.action = logMessage;
        }
      }
    }
  });

  return playerLog;
}
