use crate::init::*;
use serde::{Deserialize, Serialize};
use windows::Win32::{Foundation::COLORREF, Graphics::Dwm::DWMWA_COLOR_DEFAULT};

#[derive(Deserialize, Serialize)]
pub struct LayoutSettings {
    default_layout: std::path::PathBuf,
    window_padding: i32,
    edge_padding: i32,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            default_layout: std::path::PathBuf::new(),
            window_padding: 0,
            edge_padding: 0,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct BorderSettings {
    disable_rounding: bool,
    disable_unfocused_border: bool,
    focused_border_colour: String,
    unfocused_border_colour: String,
}

impl Default for BorderSettings {
    fn default() -> Self {
        Self {
            disable_rounding: false,
            disable_unfocused_border: false,
            focused_border_colour: String::from("ffffff"),
            unfocused_border_colour: String::from(""),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct MiscSettings {
    floating_window_default_w_ratio: f64,
    floating_window_default_h_ratio: f64,
}

impl Default for MiscSettings {
    fn default() -> Self {
        Self {
            floating_window_default_w_ratio: 0.5,
            floating_window_default_h_ratio: 0.5,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AdvancedSettings {
    new_window_retries: i32,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            new_window_retries: 10000,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserSettings {
    layout_settings: LayoutSettings,
    border_settings: BorderSettings,
    misc_settings: MiscSettings,
    advanced_settings: AdvancedSettings,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            layout_settings: LayoutSettings::default(),
            border_settings: BorderSettings::default(),
            misc_settings: MiscSettings::default(),
            advanced_settings: AdvancedSettings::default(),
        }
    }
}

impl UserSettings {
    pub fn to_settings(
        &self,
        layouts: &Vec<(std::path::PathBuf, himewm_layout::Layout)>,
    ) -> crate::wm::Settings {
        let mut idx = 0;
        if self.layout_settings.default_layout != std::path::Path::new("") {
            for (i, (p, _)) in layouts.iter().enumerate() {
                if p == &self.layout_settings.default_layout {
                    idx = i;
                    break;
                }
            }
        }
        return crate::wm::Settings {
            default_layout_idx: idx,
            window_padding: self.layout_settings.window_padding,
            edge_padding: self.layout_settings.edge_padding,
            disable_rounding: self.border_settings.disable_rounding,
            disable_unfocused_border: self.border_settings.disable_unfocused_border,
            focused_border_colour: hex_string_to_colorref(
                self.border_settings.focused_border_colour.as_str(),
            ),
            unfocused_border_colour: parse_unfocused_border_colour(
                self.border_settings.unfocused_border_colour.as_str(),
            ),
            floating_window_default_w_ratio: self.misc_settings.floating_window_default_w_ratio,
            floating_window_default_h_ratio: self.misc_settings.floating_window_default_h_ratio,
            new_window_retries: self.advanced_settings.new_window_retries,
        };
    }
}

fn hex_to_decimal(c: u8) -> u8 {
    match c {
        48..58 => {
            return c - 48;
        }
        97..103 => {
            return c - 87;
        }
        _ => return 0,
    }
}

fn hex_string_to_colorref(s: &str) -> COLORREF {
    let lowercase = s.to_lowercase();
    let byte_slice = lowercase.as_bytes();
    let digits = byte_slice
        .iter()
        .map(|byte| hex_to_decimal(*byte))
        .collect::<Vec<u8>>();
    let r = digits[0] << 4 | digits[1];
    let g = digits[2] << 4 | digits[3];
    let b = digits[4] << 4 | digits[5];
    return COLORREF(r as u32 | (g as u32) << 8 | (b as u32) << 16);
}

fn parse_unfocused_border_colour(s: &str) -> COLORREF {
    if s.trim().len() == 0 {
        return COLORREF(DWMWA_COLOR_DEFAULT);
    } else {
        return hex_string_to_colorref(s);
    }
}

pub fn initialize_settings() -> UserSettings {
    let dirs = Directories::new();
    match std::fs::read(dirs.config_dir.join("settings.json")) {
        Ok(byte_vector) => match serde_json::from_slice::<UserSettings>(byte_vector.as_slice()) {
            Ok(settings) => {
                return settings;
            }
            Err(_) => {
                return UserSettings::default();
            }
        },
        Err(_) => {
            let settings_file =
                std::fs::File::create_new(dirs.config_dir.join("settings.json")).unwrap();
            let default_user_settings = UserSettings::default();
            let _ = serde_json::to_writer_pretty(settings_file, &default_user_settings);
            return default_user_settings;
        }
    }
}
