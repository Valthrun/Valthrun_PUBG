use anyhow::Context;
use pubg::{
    schema::{
        AActor,
        ACharacter,
        APawn,
    },
    state::{
        StateActorLists,
        StateDecrypt,
        StateGNameCache,
        StateLocalPlayerInfo,
        StatePlayerInfo,
        StatePlayerInfoParams,
    },
    StatePubgHandle,
    StatePubgMemory,
};
use raw_struct::builtins::Ptr64;
use utils_state::StateRegistry;

use super::Enhancement;

pub struct PlayerSpyer {}

impl PlayerSpyer {
    pub fn collect_players_info(
        &self,
        states: &StateRegistry,
        actor_list: &Vec<(u64, Ptr64<dyn AActor>)>,
        players_data: &mut Vec<(StatePlayerInfo, u32, u32)>,
    ) -> anyhow::Result<()> {
        let decrypt = states.resolve::<StateDecrypt>(())?;
        let pubg_handle = states.resolve::<StatePubgHandle>(())?;
        let mut gname_cache = states.resolve_mut::<StateGNameCache>(())?;
        let memory = states.resolve::<StatePubgMemory>(())?;
        let local_player_info = states.resolve::<StateLocalPlayerInfo>(())?;

        for (_actor_address, actor_ptr) in actor_list.iter() {
            if actor_ptr.is_null() {
                continue;
            }

            let actor = actor_ptr
                .value_reference(memory.view_arc())
                .context("actor nullptr")?;

            let name = gname_cache.get_gname_by_id(&decrypt, &pubg_handle, &memory, actor.id()?)?;

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
            let team_id = character.last_team_num()?;

            let player_info = states.resolve::<StatePlayerInfo>(StatePlayerInfoParams {
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

            players_data.push((player_info.clone(), distance, team_id));
        }
        Ok(())
    }
}

impl Enhancement for PlayerSpyer {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        let actor_lists = ctx.states.resolve::<StateActorLists>(())?;

        let mut players_data: Vec<(StatePlayerInfo, u32, u32)> = Vec::new();

        let cached_actors = actor_lists.cached_actors();
        for (_actor_id, actor_list) in cached_actors {
            self.collect_players_info(&ctx.states, actor_list, &mut players_data)?;
        }

        if players_data.is_empty() {
            return Ok(());
        }

        players_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let first_team = players_data[0].2;
        players_data.retain(|x| x.2 != first_team);

        players_data.dedup_by(|a, b| a.1 == b.1);

        for (player_info, distance, team_id) in players_data {
            log::info!(
                "Distance: {} Health: {} Angle: {}",
                distance,
                player_info.health,
                team_id
            );
        }

        Ok(())
    }
}
