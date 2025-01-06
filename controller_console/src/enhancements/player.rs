use anyhow::Context;
use pubg::{
    decrypt::StateDecrypt,
    decrypt::StateGNameCache,
    schema::ACharacter,
    state::{
        StateActorLists,
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
        let actor_array = ctx.states.resolve::<StateActorLists>(())?;
        let decrypt = ctx.states.resolve::<StateDecrypt>(())?;
        let local_player_info = ctx.states.resolve::<StateLocalPlayerInfo>(())?;
        let actor_count = actor_array.count()?;

        let mut players_data: Vec<(u32, u32, i32, u32)> = Vec::new();

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

            let name = decrypt.get_gname_by_id(&ctx.states, actor.id()?)?;

            if name != "PlayerFemale_A_C" && name != "PlayerMale_A_C" {
                continue;
            }

            let root_component = match actor
                .root_component()?
                .value_reference(memory.view_arc(), &decrypt)
            {
                Some(root_component) => root_component,
                None => {
                    continue;
                }
            };

            let character = actor.cast::<dyn ACharacter>();
            let team_id = character.team()?;

            let player_info = ctx
                .states
                .resolve::<StatePlayerInfo>(StatePlayerInfoParams {
                    character,
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

            players_data.push((distance, player_info.health, angle_diff, team_id));
        }

        if players_data.is_empty() {
            return Ok(());
        }

        players_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // TODO: Try to find current team from local player and filter out teamaters during the loop
        let first_team = players_data[0].3;
        players_data.retain(|x| x.3 != first_team);

        // Hack: Filter some of the fog of war entries
        // TODO: Find a better way
        players_data.dedup_by(|a, b| a.0 == b.0);

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
