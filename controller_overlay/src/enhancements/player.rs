use anyhow::{Context, Result};
use imgui::Ui;
use nalgebra::Vector3;
use overlay::UnicodeTextRenderer;
use pubg::schema::ACharacter;
use pubg::state::{StatePlayerInfo, StateLocalPlayerInfo, StatePlayerInfoParams, StateActorList};
use pubg::{StateDecrypt, StatePubgMemory};
use utils_state::StateRegistry;

use crate::{
    app::types::UpdateContext,
    settings::AppSettings,
    view::ViewController,
};

use super::Enhancement;

pub struct PlayerSpyer {
    players_data: Vec<StatePlayerInfo>,
}

impl Default for PlayerSpyer {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerSpyer {
    pub fn new() -> Self {
        Self {
            players_data: Vec::new(),
        }
    }
}

impl Enhancement for PlayerSpyer {
    fn update_settings(&mut self, _ui: &Ui, _settings: &mut AppSettings) -> Result<bool> {
        Ok(false)
    }

    fn update(&mut self, context: &UpdateContext) -> Result<()> {
        let memory = context.states.resolve::<StatePubgMemory>(())?;
        let actor_array = context.states.resolve::<StateActorList>(())?;
        let decrypt = context.states.resolve::<StateDecrypt>(())?;
        let local_player_info = context.states.resolve::<StateLocalPlayerInfo>(())?;
        let actor_count = actor_array.count()?;

        let mut players_data: Vec<(StatePlayerInfo, u32, u32)> = Vec::new();
        
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

            let name = decrypt.get_gname_by_id(&context.states, actor.id()?)?;

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

            let player_info = context
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

            players_data.push((player_info.clone(), distance, team_id));
        }

        if players_data.is_empty() {
            return Ok(());
        }

        players_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // TODO: Try to find current team from local player and filter out teamaters during the loop
        let first_team = players_data[0].2;
        players_data.retain(|x| x.2 != first_team);

        // Hack: Filter some of the fog of war entries
        // TODO: Find a better way
        players_data.dedup_by(|a, b| a.1 == b.1);

        let players_data = players_data.iter().map(|x| x.0.clone()).collect();

        self.players_data = players_data;

        Ok(())
    }

    fn render(&self, _states: &StateRegistry, _ui: &Ui, _unicode_text: &UnicodeTextRenderer) -> Result<()> {
        // Skip if no players found
        if self.players_data.is_empty() {
            return Ok(());
        }

        let view_controller = _states.resolve::<ViewController>(())?;
        let local_player_info = _states.resolve::<StateLocalPlayerInfo>(())?;

        let draw_list = _ui.get_window_draw_list();

        for player_info in &self.players_data {
            let player_pos = Vector3::new(player_info.position[0], player_info.position[1], player_info.position[2]);
            
            let screen_pos = match view_controller.world_to_screen(&player_pos, false) {
                Some(screen_pos) => screen_pos,
                None => continue,
            };

            let base_width = 40.0;
            let base_height = 80.0;
            
            let distance = ((player_info.position[0] - local_player_info.location[0]).powi(2)
                + (player_info.position[1] - local_player_info.location[1]).powi(2)
                + (player_info.position[2] - local_player_info.location[2]).powi(2))
            .sqrt();

            let scale = 3000.0 / distance.max(100.0);
            let box_width = base_width * scale;
            let box_height = base_height * scale;
            
            let x = screen_pos.x - box_width / 2.0;
            let y = screen_pos.y - box_height / 2.0;

            // Draw red box
            draw_list.add_rect(
                [x, y],
                [x + box_width, y + box_height],
                imgui::ImColor32::from_rgba(255, 0, 0, 255)
            ).build();

            // Draw health text
            draw_list.add_text(
                [x, y - 20.0],
                imgui::ImColor32::from_rgba(255, 255, 255, 255),
                &format!("HP: {}", player_info.health)
            );
        }

        Ok(())
    }

    fn render_debug_window(&mut self, _states: &StateRegistry, _ui: &Ui, _unicode_text: &UnicodeTextRenderer) -> Result<()> {
        Ok(())
    }
}
