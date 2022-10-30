import type { Match as ApiMatch, MatchPlayer as ApiMatchPlayer, Map } from "./api_types";

// match from the perspective of a bot
export type BotMatch = {
  id: number;
  opponent: BotMatchOpponent;
  outcome: BotMatchOutcome;
  timestamp: string;
  map: Map;
  hadErrors?: boolean;
};

export type BotMatchOutcome = "win" | "loss" | "tie";

export type BotMatchOpponent = {
  bot_id: number;
  bot_name: string;
  owner_id?: number;
};

export function apiMatchtoBotMatch(bot_name: string, apiMatch: ApiMatch): BotMatch {
  let player: ApiMatchPlayer;
  let playerIndex: number;
  let opponent: ApiMatchPlayer;
  apiMatch.players.forEach((matchPlayer, index) => {
    if (matchPlayer.bot_name === bot_name) {
      player = matchPlayer;
      playerIndex = index;
    } else {
      opponent = matchPlayer;
    }
  });

  if (player === undefined || opponent === undefined || playerIndex === undefined) {
    throw "could not assign player and opponent";
  }

  let outcome: BotMatchOutcome;
  if (apiMatch.winner === playerIndex) {
    outcome = "win";
  } else if (apiMatch.winner) {
    outcome = "loss";
  } else {
    outcome = "tie";
  }

  return {
    id: apiMatch.id,
    opponent,
    outcome,
    timestamp: apiMatch.timestamp,
    map: apiMatch.map,
    hadErrors: player.had_errors,
  };
}
