use osbot_api::api::domain::character::{Character, CharacterImpl};
use osbot_api::api::domain::entity::Entity;
use osbot_api::api::scene::coords::position::Position;
use osbot_api::api::scene::npcs::{npcs_find_closest_by_name, npcs_find_closest_conditional};
use osbot_api::api::scene::players::players_get_local_player;
use osbot_api::api::script::script::Script;
use osbot_api::api::script::script_metadata::ScriptMetadata;
use osbot_api::api::ui::tab::inventory::{inventory_drop_all_by_id, inventory_is_empty, inventory_is_full};
use osbot_api::api::ui::tab::skills::{skills_get_experience, Skill};
use osbot_api::api::util::utils::{utils_calculate_per_hour, utils_current_time_millis, utils_format_runtime, utils_sleep_conditional};
use osbot_api::api::web_walking::{web_walking_traverse, WebWalkingArgs};
use osbot_api::log::error;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[no_mangle]
pub extern "C" fn metadata() -> ScriptMetadata {
    ScriptMetadata {
        name: "OSBot Native Fisher".to_string(),
        author: "Patrick".to_string(),
        version: 1.0,
        info: "Fishes".to_string(),
        logo: "https://i.gyazo.com/cff84f44847c548c9024b5a06384a73d.png".to_string(),
    }
}

struct Configuration {
    spot_action: String,
    spot_name: String,
    editing: bool,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            spot_action: String::from("Net"),
            spot_name: String::from("Fishing spot"),
            editing: true,
        }
    }
}

struct Progress {
    start_time: Instant,
    start_time_ms: u128,
    start_exp: i32,
    status: String
}
impl Progress {
    pub fn set_status(&mut self, status: &str) {
        self.status = String::from(status);
    }
}

static mut PROGRESS: Option<Progress> = None;
static mut CONFIGURATION: Option<Configuration> = None;

fn get_progress() -> &'static mut Progress {
    unsafe {
        if PROGRESS.is_none() {
            PROGRESS.replace(Progress {
                start_time: Instant::now(),
                start_time_ms: utils_current_time_millis(),
                start_exp: 0,
                status: String::new()
            });
        }

        PROGRESS.as_mut().unwrap()
    }
}
fn get_configuration() -> &'static mut Configuration {
    unsafe {
        if CONFIGURATION.is_none() {
            CONFIGURATION.replace(Configuration::default());
        }

        CONFIGURATION.as_mut().unwrap()
    }
}

#[osbot_api::script_exports]
pub struct FishingScript;
impl Script for FishingScript {
    fn new() -> Self {
        Self { }
    }

    fn on_start(&mut self, _: Option<String>) {
        get_progress().start_exp = skills_get_experience(&Skill::Fishing);

        get_progress().set_status("Started");
    }

    fn on_loop(&mut self) -> i32 {
        if get_configuration().editing {
            return 1000;
        }

        if inventory_is_full() {
            get_progress().set_status("Dropping");
            inventory_drop_all_by_id(vec![317, 321, 335, 331, 359, 371]);
        } else {
            if let Some(local_player) = players_get_local_player() {
                if !local_player.is_moving() && !local_player.is_animating() {
                    if let Some(fishing_spot) = npcs_find_closest_by_name(get_configuration().spot_name.as_str()) {
                        get_progress().set_status("Interacting");

                        if fishing_spot.interact(get_configuration().spot_action.as_str()) {
                            utils_sleep_conditional(6000, 100, || { local_player.is_moving() || local_player.is_animating() });
                            utils_sleep_conditional(3000, 100, || { local_player.is_animating() });
                        }
                    }
                } else {
                    get_progress().set_status("Fishing");
                }
            }
        }

        100
    }

    fn on_render(&self, ui: &mut egui::Ui) {
        ui.heading("OSBot Native Fisher");

        if get_configuration().editing {
            ui.columns(4, |columns| {
                columns[0].label("Spot action");
                columns[1].text_edit_singleline(&mut get_configuration().spot_action);
            });

            ui.columns(4, |columns| {
                columns[0].label("Spot name");
                columns[1].text_edit_singleline(&mut get_configuration().spot_name);
            });

            ui.add_space(8.0);
            if ui.button("Complete").clicked() {
                get_configuration().editing = false;
            }
        } else {
            let progress: &Progress = get_progress();

            let exp_gained: i32 = skills_get_experience(&Skill::Fishing) - progress.start_exp;

            ui.label(&format!("Runtime: {}", utils_format_runtime(progress.start_time.elapsed())));
            ui.label(&format!("Status: {}", progress.status));
            ui.label(&format!("Exp: {} ({}/hr)", exp_gained, utils_calculate_per_hour(exp_gained, progress.start_time_ms)));

            ui.add_space(8.0);
            if ui.button("Edit configuration").clicked() {
                get_configuration().editing = true;
            }
        }
    }
}