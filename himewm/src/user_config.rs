use crate::{directories, keybinds, layouts, settings, util, window_rules};
use himewm_layout::layout::Layout;
use serde::{Deserialize, Serialize};

pub struct Config {
    pub settings: settings::Settings,
    pub window_rules: window_rules::WindowRules,
    pub layouts: Vec<Layout>,
    pub keybinds: keybinds::Keybinds,
}

pub struct UserConfig {
    pub config: Config,
    pub warnings: String,
    pub errors: String,
}

fn get_from_file<T>(file_name: &str) -> serde_json::Result<T>
where
    for<'a> T: Default + Deserialize<'a> + Serialize,
{
    let dirs = directories::Directories::new();
    let config_path = dirs.config_dir.join(format!("{file_name}"));
    match std::fs::read(&config_path) {
        Ok(byte_vector) => {
            return serde_json::from_slice::<T>(byte_vector.as_slice());
        }
        Err(_) => {
            let file = std::fs::File::create_new(config_path).unwrap();
            let default_user_config = T::default();
            let _ = serde_json::to_writer_pretty(&file, &default_user_config);
            return Ok(default_user_config);
        }
    }
}

pub fn get_user_config() -> UserConfig {
    let mut warnings = String::new();
    let mut errors = String::new();
    let layouts_with_names = match layouts::initialize_layouts(&mut warnings) {
        Some(val) => val,
        None => {
            let dirs = directories::Directories::new();
            let layouts_dir = dirs.layouts_dir;
            util::add_to_message(
                &mut errors,
                &format!(
                    "Error: No layouts found\nPlease add layouts to {}",
                    layouts_dir.display()
                ),
            );
            Vec::new()
        }
    };
    let layout_idx_map = layouts::get_layout_idx_map(&layouts_with_names);
    let settings = match get_from_file::<settings::UserSettings>("settings.json") {
        Ok(user_settings) => user_settings.to_settings(&layout_idx_map),
        Err(e) => {
            util::add_to_message(&mut warnings, &format!("Warning: An error occurred when parsing settings.json:\n{}\nProceeding with default settings", e));
            settings::UserSettings::default().to_settings(&layout_idx_map)
        }
    };
    let window_rules = match get_from_file::<Vec<window_rules::UserWindowRule>>("window_rules.json")
    {
        Ok(user_window_rules) => {
            window_rules::get_window_rules(&user_window_rules, &layout_idx_map)
        }
        Err(e) => {
            util::add_to_message(&mut warnings, &format!("Warning: An error occurred when parsing window_rules.json:\n{}\nProceeding with default settings", e));
            window_rules::WindowRules::default()
        }
    };
    let keybinds = match get_from_file("keybinds.json") {
        Ok(user_keybinds) => keybinds::Keybinds::from(&user_keybinds),
        Err(e) => {
            util::add_to_message(&mut warnings, &format!("Warning: An error occurred when parsing keybinds.json:\n{}\nProceeding with default settings", e));
            let default_user_keybinds = keybinds::UserKeybinds::default();
            keybinds::Keybinds::from(&default_user_keybinds)
        }
    };
    let config = Config {
        settings,
        window_rules,
        layouts: layouts_with_names
            .into_iter()
            .map(|(_, layout)| layout)
            .collect(),
        keybinds,
    };
    return UserConfig {
        config,
        warnings,
        errors,
    };
}
