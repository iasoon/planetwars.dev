use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

use serde_json;

use super::protocol as proto;
use super::rules::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub map_file: PathBuf,
    pub max_turns: u64,
}

impl Config {
    pub fn create_state(&self, num_players: usize) -> PwState {
        let planets = self.load_map(num_players);
        let players = (0..num_players)
            .map(|player_num| Player {
                id: player_num + 1,
                alive: true,
            })
            .collect();

        PwState {
            players,
            planets,
            expeditions: Vec::new(),
            expedition_num: 0,
            turn_num: 0,
            max_turns: self.max_turns,
        }
    }

    fn load_map(&self, num_players: usize) -> Vec<Planet> {
        let map = self.read_map().expect("[PLANET_WARS] reading map failed");

        map.planets
            .into_iter()
            .enumerate()
            .map(|(num, planet)| {
                let mut fleets = Vec::new();
                let owner = planet.owner.and_then(|owner_num| {
                    // in the current map format, player numbers start at 1.
                    // TODO: we might want to change this.
                    // ignore players that are not in the game
                    if owner_num > 0 && owner_num <= num_players {
                        Some(owner_num - 1)
                    } else {
                        None
                    }
                });
                if planet.ship_count > 0 {
                    fleets.push(Fleet {
                        owner,
                        ship_count: planet.ship_count,
                    });
                }
                Planet {
                    id: num,
                    name: planet.name,
                    x: planet.x,
                    y: planet.y,
                    fleets,
                }
            })
            .collect()
    }

    fn read_map(&self) -> io::Result<Map> {
        let mut file = File::open(&self.map_file)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(serde_json::from_str(&buf)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub planets: Vec<proto::Planet>,
}
