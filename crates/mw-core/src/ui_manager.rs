use anyhow::Result;
use log::info;
use mw_sdk::types::{CustomNavButton, LobbyType};
use mw_sdk::LuaScriptGenerator;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub enable_vfs_redirection: bool,
    pub enable_lua_hooks: bool,
    pub custom_textures_folder: String,
    pub custom_buttons: Vec<CustomNavButton>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            enable_vfs_redirection: true,
            enable_lua_hooks: true,
            custom_textures_folder: "custom_assets/textures".into(),
            custom_buttons: vec![
                CustomNavButton {
                    id: "btn_goto_teamup".into(),
                    title: "Sảnh Chờ".into(),
                    icon_path: None,
                    pos_x: 20.0,
                    pos_y: 200.0,
                    width: 140.0,
                    height: 48.0,
                    target_lobby: Some(LobbyType::TeamUpWaitingRoom),
                    custom_lua_onclick: None,
                },
                CustomNavButton {
                    id: "btn_goto_multi".into(),
                    title: "Chơi Mạng".into(),
                    icon_path: None,
                    pos_x: 20.0,
                    pos_y: 260.0,
                    width: 140.0,
                    height: 48.0,
                    target_lobby: Some(LobbyType::MultiplayerLobby),
                    custom_lua_onclick: None,
                },
            ],
        }
    }
}

pub struct UIManager {
    config: ClientConfig,
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UIManager {
    pub fn new() -> Self {
        let config = Self::load_config().unwrap_or_default();
        Self { config }
    }

    pub fn load_config() -> Result<ClientConfig> {
        let path = PathBuf::from("mw_config.json");
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let cfg: ClientConfig = serde_json::from_str(&content)?;
            info!("[UIManager] Loaded config from mw_config.json");
            return Ok(cfg);
        }
        let default_cfg = ClientConfig::default();
        let json = serde_json::to_string_pretty(&default_cfg)?;
        let _ = std::fs::write(&path, json);
        info!("[UIManager] Created default mw_config.json");
        Ok(default_cfg)
    }

    pub fn get_navigation_buttons(&self) -> &[CustomNavButton] {
        &self.config.custom_buttons
    }

    pub fn build_injection_script(&self) -> String {
        LuaScriptGenerator::generate_custom_buttons_injector(&self.config.custom_buttons)
    }
}
