use directories::BaseDirs;

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
