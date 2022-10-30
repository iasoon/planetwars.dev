export type Match = {
  id: number;
  timestamp: string;
  state: string;
  players: MatchPlayer[];
  winner: number;
  map: Map;
};

export type MatchPlayer = {
  bot_id: number;
  bot_version_id: number;
  bot_name: string;
  owner_id?: number;
  had_errors?: boolean;
};

export type Map = {
  name: string;
};
