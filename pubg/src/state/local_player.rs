use anyhow::Context;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    decrypt::StateDecrypt,
    state::StateWorld,
    StatePubgMemory,
};

pub struct StateLocalPlayerInfo {
    pub controller_address: u64,
    pub location: [f32; 3],
    pub rotation: [f32; 3],
}

impl State for StateLocalPlayerInfo {
    type Parameter = ();

    fn create(states: &StateRegistry, _: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StatePubgMemory>(())?;
        let decrypt = states.resolve::<StateDecrypt>(())?;
        let u_world = states.resolve::<StateWorld>(())?;

        let game_instance = u_world
            .game_instance()?
            .value_reference(memory.view_arc(), &decrypt)
            .context("game_instance nullptr")?;

        let local_player = game_instance
            .local_player()?
            .read_value(memory.view())?
            .context("local_player1 nullptr")?
            .value_reference(memory.view_arc(), &decrypt)
            .context("local_player2 nullptr")?;

        let local_player_controller = local_player
            .player_controller()?
            .value_reference(memory.view_arc(), &decrypt)
            .context("local_player_controller nullptr")?;

        let controller_address = local_player_controller.reference_address();

        let player_camera_manager = local_player_controller
            .player_camera_manager()?
            .value_reference(memory.view_arc())
            .context("player_camera_manager nullptr")?;

        let location = player_camera_manager.camera_pos()?;
        let rotation = player_camera_manager.camera_rot()?;

        Ok(Self {
            controller_address,
            location,
            rotation,
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
