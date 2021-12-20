use super::protocol as proto;
use super::rules::{Expedition, Planet, PwState};

/// Serialize given gamestate
pub fn serialize(state: &PwState) -> proto::State {
    serialize_rotated(state, 0)
}

/// Serialize given gamestate with player numbers rotated by given offset.
pub fn serialize_rotated(state: &PwState, offset: usize) -> proto::State {
    let serializer = Serializer::new(state, offset);
    serializer.serialize_state()
}

struct Serializer<'a> {
    state: &'a PwState,
    player_num_offset: usize,
}

impl<'a> Serializer<'a> {
    fn new(state: &'a PwState, offset: usize) -> Self {
        Serializer {
            state: state,
            player_num_offset: offset,
        }
    }

    fn serialize_state(&self) -> proto::State {
        proto::State {
            planets: self
                .state
                .planets
                .iter()
                .map(|planet| self.serialize_planet(planet))
                .collect(),
            expeditions: self
                .state
                .expeditions
                .iter()
                .map(|exp| self.serialize_expedition(exp))
                .collect(),
        }
    }

    /// Gets the player number for given player id.
    /// Player numbers are 1-based (as opposed to player ids), They will also be
    /// rotated based on the number offset for this serializer.
    fn player_num(&self, player_id: usize) -> usize {
        let num_players = self.state.players.len();
        let rotated_id =
            (player_id + num_players - self.player_num_offset) % num_players;
        // protocol player ids start at 1
        return rotated_id + 1;
    }

    fn serialize_planet(&self, planet: &Planet) -> proto::Planet {
        proto::Planet {
            name: planet.name.clone(),
            x: planet.x,
            y: planet.y,
            owner: planet.owner().map(|id| self.player_num(id)),
            ship_count: planet.ship_count(),
        }
    }

    fn serialize_expedition(&self, exp: &Expedition) -> proto::Expedition {
        proto::Expedition {
            id: exp.id,
            owner: self.player_num(exp.fleet.owner.unwrap()),
            ship_count: exp.fleet.ship_count,
            origin: self.state.planets[exp.origin as usize].name.clone(),
            destination: self.state.planets[exp.target as usize].name.clone(),
            turns_remaining: exp.turns_remaining,
        }
    }
}
