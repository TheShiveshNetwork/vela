use anyhow::{Result, anyhow};
use std::fs;

static CFG_PATH: &str = "storage-config.toml";

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeConfig {
    selected_mount: String,
}

pub fn save_selection(path: String) -> Result<()> {
    let cfg = NodeConfig { selected_mount: path.clone() };
    let content = toml::to_string(&cfg)?;
    fs::write(CFG_PATH, content)?;
    println!("Selected mount path saved: {}", path);
    Ok(())
}

pub fn load_selection() -> Result<String> {
    let raw = fs::read_to_string(CFG_PATH)
        .map_err(|_| anyhow!("No device selected. Use storage-node select <path>"))?;
    let cfg: NodeConfig = toml::from_str(&raw)?;
    Ok(cfg.selected_mount)
}

