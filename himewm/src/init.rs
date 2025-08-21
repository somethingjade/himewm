use directories::BaseDirs;
use himewm_layout::*;

pub struct Directories {
    pub config_dir: std::path::PathBuf,
    pub layouts_dir: std::path::PathBuf,
}

impl Directories {
    pub fn new() -> Self {
        let base_dirs = BaseDirs::new().unwrap();
        let config_dir = base_dirs.config_dir().join("himewm");
        let layouts_dir = config_dir.join("layouts");
        return Self {
            config_dir,
            layouts_dir,
        };
    }
}

pub fn create_dirs() -> std::io::Result<()> {
    let dirs = Directories::new();
    let _config_dir = std::fs::create_dir(dirs.config_dir)?;
    let _layouts_dir = std::fs::create_dir(dirs.layouts_dir)?;
    return Ok(());
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
