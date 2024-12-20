use anyhow::Context;
use pubg::{
    decrypt::StateDecrypt,
    schema::{
        AActor,
        APlayerController,
    },
    state::{
        StateActorList,
        StateLocalPlayerInfo,
        StatePlayerInfo,
        StatePlayerInfoParams,
    },
    StatePubgMemory,
};

use super::Enhancement;

pub struct PlayerSpyer {}

impl Enhancement for PlayerSpyer {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let memory = ctx.states.resolve::<StatePubgMemory>(())?;
        let actor_array = ctx.states.resolve::<StateActorList>(())?;
        let decrypt = ctx.states.resolve::<StateDecrypt>(())?;
        let local_player_info = ctx.states.resolve::<StateLocalPlayerInfo>(())?;
        let actor_count = actor_array.count()?;

        let mut players_data: Vec<(u32, u32, i32)> = Vec::new();

        for actor_ptr in actor_array
            .data()?
            .elements(memory.view(), 0..actor_count as usize)?
        {
            if actor_ptr.is_null() {
                continue;
            }

            let actor = actor_ptr
                .value_reference(memory.view_arc())
                .context("actor nullptr")?;

            let root_component = match actor
                .root_component()?
                .value_reference(memory.view_arc(), &decrypt)
            {
                Some(root_component) => root_component,
                None => {
                    continue;
                }
            };

            let player_controller = actor.cast::<dyn APlayerController>();

            if player_controller.player_state()?.is_null() {
                /* Actor is not a player controller */
                continue;
            }

            if !player_controller.acknowledged_pawn()?.is_null() {
                continue;
            }

            if player_controller.reference_address() == local_player_info.controller_address {
                log::info!("Skipping local player");
                continue;
            }

            let mesh = match player_controller.mesh()?.read_value(memory.view()) {
                Ok(mesh) => mesh,
                _ => {
                    continue;
                }
            };

            let mesh = match mesh {
                Some(mesh) => mesh,
                None => {
                    continue;
                }
            };

            if mesh <= 0x10000 {
                continue;
            }

            let player_info = ctx
                .states
                .resolve::<StatePlayerInfo>(StatePlayerInfoParams {
                    actor,
                    root_component,
                })?;

            let distance = ((player_info.position[0] - local_player_info.location[0]).powi(2)
                + (player_info.position[1] - local_player_info.location[1]).powi(2)
                + (player_info.position[2] - local_player_info.location[2]).powi(2))
            .sqrt() as u32;

            if player_info.health < 1 || player_info.health > 100 {
                continue;
            }

            let difference = [
                player_info.position[0] - local_player_info.location[0],
                player_info.position[1] - local_player_info.location[1],
                player_info.position[2] - local_player_info.location[2],
            ];

            // Calculate horizontal angle to target (atan2 gives us angle in radians)
            let target_angle = difference[1].atan2(difference[0]);

            // Get player's horizontal angle (z rotation)
            let player_angle = local_player_info.rotation[1].to_radians();

            // Calculate the difference and convert to degrees
            let angle_diff = (target_angle - player_angle).to_degrees();

            // Normalize angle to -180 to 180 degrees
            let angle_diff = if angle_diff > 180.0 {
                (angle_diff - 360.0) as i32
            } else if angle_diff < -180.0 {
                (angle_diff + 360.0) as i32
            } else {
                angle_diff as i32
            };

            players_data.push((distance, player_info.health, angle_diff));
        }

        players_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut count = players_data.len();
        if count > 25 {
            count = 25;
        }
        for i in 0..count {
            log::info!(
                "Distance: {} Health: {} Angle: {}",
                players_data[i].0,
                players_data[i].1,
                players_data[i].2
            );
        }

        Ok(())
    }
}
