#[macro_use]
extern crate serde;
extern crate serde_json;

pub mod config;
pub mod protocol;
pub mod rules;
pub mod serializer;

pub use config::Config as PwConfig;
pub use protocol::CommandError;
pub use rules::{Dispatch, PwState};
use std::collections::HashMap;

pub struct PlanetWars {
    /// Game state
    state: rules::PwState,
    /// Map planet names to their ids
    planet_map: HashMap<String, usize>,
}

impl PlanetWars {
    pub fn create(config: PwConfig, num_players: usize) -> Self {
        let state = config.create_state(num_players);

        let planet_map = state
            .planets
            .iter()
            .map(|p| (p.name.clone(), p.id))
            .collect();

        PlanetWars { state, planet_map }
    }

    /// Proceed to next turn
    pub fn step(&mut self) {
        self.state.repopulate();
        self.state.step();
    }

    pub fn is_finished(&self) -> bool {
        self.state.is_finished()
    }

    pub fn serialize_state(&self) -> protocol::State {
        serializer::serialize(&self.state)
    }

    pub fn serialize_player_state(&self, player_id: usize) -> protocol::State {
        serializer::serialize_rotated(&self.state, player_id - 1)
    }

    pub fn state(&self) -> &PwState {
        &self.state
    }

    /// Execute a command
    pub fn execute_command(
        &mut self,
        player_num: usize,
        cmd: &protocol::Command,
    ) -> Result<(), CommandError> {
        let dispatch = self.parse_command(player_num, cmd)?;
        self.state.dispatch(&dispatch);
        Ok(())
    }

    /// Check the given command for validity.
    /// If it is valid, return an internal representation of the dispatch
    /// described by the command.
    pub fn parse_command(
        &self,
        player_id: usize,
        cmd: &protocol::Command,
    ) -> Result<Dispatch, CommandError> {
        let origin_id = *self
            .planet_map
            .get(&cmd.origin)
            .ok_or(CommandError::OriginDoesNotExist)?;

        let target_id = *self
            .planet_map
            .get(&cmd.destination)
            .ok_or(CommandError::DestinationDoesNotExist)?;

        if self.state.planets[origin_id].owner() != Some(player_id - 1) {
            return Err(CommandError::OriginNotOwned);
        }

        if self.state.planets[origin_id].ship_count() < cmd.ship_count {
            return Err(CommandError::NotEnoughShips);
        }

        if cmd.ship_count == 0 {
            return Err(CommandError::ZeroShipMove);
        }

        Ok(Dispatch {
            origin: origin_id,
            target: target_id,
            ship_count: cmd.ship_count,
        })
    }

    /// Execute a dispatch.
    /// This assumes the dispatch is valid. You should check this yourself
    /// or use `parse_command` to obtain a valid dispatch.
    pub fn execute_dispatch(&mut self, dispatch: &Dispatch) {
        self.state.dispatch(dispatch);
    }
}
