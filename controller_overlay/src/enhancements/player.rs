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
    players_data: Vec<(u32, u32, i32, u32)>,
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

        let mut players_data: Vec<(u32, u32, i32, u32)> = Vec::new();
        
        log::info!("Start");
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

            log::info!("Actor");

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

        //players_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // TODO: Try to find current team from local player and filter out teamaters during the loop
        //let first_team = players_data[0].3;
        //players_data.retain(|x| x.3 != first_team);

        // Hack: Filter some of the fog of war entries
        // TODO: Find a better way
        //players_data.dedup_by(|a, b| a.0 == b.0);

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

        self.players_data = players_data;

        Ok(())
    }

    fn render(&self, _states: &StateRegistry, _ui: &Ui, _unicode_text: &UnicodeTextRenderer) -> Result<()> {
        /* 

        // Skip if no players found
        if self.players_data.is_empty() {
            return Ok(());
        }

        let view_controller = _states.resolve::<ViewController>(())?;
        let local_player_info = _states.resolve::<StateLocalPlayerInfo>(())?;

        let draw_list = _ui.get_background_draw_list();

        for (_distance, health, _angle, _team_id) in &self.players_data {
            // Convert world position to screen coordinates
            let player_pos = Vector3::new(local_player_info.location[0], local_player_info.location[1], local_player_info.location[2]);
            let screen_pos = match view_controller.world_to_screen(&player_pos, false) {
                Some(screen_pos) => screen_pos,
                None => continue,
            };

            // Draw box around player
            let box_width = 40.0;
            let box_height = 80.0;
            let x = screen_pos.x - box_width / 2.0;
            let y = screen_pos.y - box_height / 2.0;

            // Draw red box with alpha based on health
            let alpha = (*health as f32 / 100.0 * 255.0) as u8;
            draw_list.add_rect(
                [x, y],
                [x + box_width, y + box_height],
                imgui::ImColor32::from_rgba(255, 0, 0, alpha)
            );

            // Draw health text
            draw_list.add_text(
                [x, y - 20.0],
                imgui::ImColor32::from_rgba(255, 255, 255, 255),
                &format!("HP: {}", health)
            );
        }
        */
        Ok(())
    }

    fn render_debug_window(&mut self, _states: &StateRegistry, _ui: &Ui, _unicode_text: &UnicodeTextRenderer) -> Result<()> {
        Ok(())
    }
}
