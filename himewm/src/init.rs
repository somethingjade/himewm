use directories::BaseDirs;

use himewm_layout::*;

use serde::{Deserialize, Serialize};

use windows::Win32::Foundation::COLORREF;

struct Directories {
    config_dir: std::path::PathBuf,
    layouts_dir: std::path::PathBuf,
}

impl Directories {
    fn new() -> Self {
        let base_dirs = BaseDirs::new().unwrap();

        let config_dir = base_dirs.config_dir().join("himewm");

        let layouts_dir = config_dir.join("layouts");

        return Self {
            config_dir,
            layouts_dir,
        };
    }
}

#[derive(Deserialize, Serialize)]
struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    fn as_colorref(&self) -> COLORREF {
        COLORREF(self.r as u32 | (self.g as u32) << 8 | (self.b as u32) << 16)
    }
}

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
    focused_border_colour: Colour,
}

impl Default for BorderSettings {
    fn default() -> Self {
        Self {
            disable_rounding: false,
            disable_unfocused_border: false,
            focused_border_colour: Colour {
                r: 255,
                g: 255,
                b: 255,
            },
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
    ) -> himewm::Settings {
        let mut idx = 0;
        if self.layout_settings.default_layout != std::path::Path::new("") {
            for (i, (p, _)) in layouts.iter().enumerate() {
                if p == &self.layout_settings.default_layout {
                    idx = i;
                    break;
                }
            }
        }
        return himewm::Settings {
            default_layout_idx: idx,
            window_padding: self.layout_settings.window_padding,
            edge_padding: self.layout_settings.edge_padding,
            disable_rounding: self.border_settings.disable_rounding,
            disable_unfocused_border: self.border_settings.disable_unfocused_border,
            focused_border_colour: self.border_settings.focused_border_colour.as_colorref(),
            floating_window_default_w_ratio: self.misc_settings.floating_window_default_w_ratio,
            floating_window_default_h_ratio: self.misc_settings.floating_window_default_h_ratio,
            new_window_retries: self.advanced_settings.new_window_retries,
        };
    }
}

pub fn create_dirs() -> std::io::Result<()> {
    let dirs = Directories::new();

    let _config_dir = std::fs::create_dir(dirs.config_dir)?;

    let _layouts_dir = std::fs::create_dir(dirs.layouts_dir)?;

    return Ok(());
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

pub fn initialize_layouts() -> Option<Vec<(std::path::PathBuf, Layout)>> {
    let mut ret = Vec::new();

    let dirs = Directories::new();

    for entry_result in std::fs::read_dir(dirs.layouts_dir).unwrap() {
        match entry_result {
            Ok(entry) => match std::fs::read(entry.path()) {
                Ok(byte_vector) => {
                    let layout: Layout = match serde_json::from_slice(byte_vector.as_slice()) {
                        Ok(val) => val,

                        Err(_) => continue,
                    };

                    let layout_name = std::path::Path::new(&entry.file_name()).with_extension("");

                    ret.push((layout_name, layout));
                }

                Err(_) => continue,
            },

            Err(_) => continue,
        }
    }

    if ret.is_empty() {
        return None;
    } else {
        return Some(ret);
    }
}
