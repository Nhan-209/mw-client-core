use anyhow::Result;
use log::info;
use mw_sdk::types::{CustomNavButton, LobbyBgOverride, LobbyType};
use mw_sdk::LuaScriptGenerator;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub enable_vfs_redirection: bool,
    pub enable_lua_hooks: bool,
    pub custom_textures_folder: String,
    pub custom_scripts_folder: String,
    pub background_overrides: Vec<LobbyBgOverride>,
    pub custom_buttons: Vec<CustomNavButton>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            enable_vfs_redirection: true,
            enable_lua_hooks: true,
            custom_textures_folder: "custom_assets/textures".into(),
            custom_scripts_folder: "custom_assets/scripts".into(),
            background_overrides: vec![LobbyBgOverride {
                lobby: LobbyType::MainLobbyV4,
                image_path: "custom_assets/textures/bg_hall_mi.jpg".into(),
            }],
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

    pub fn load_custom_scripts(&self) -> Vec<(String, String)> {
        let mut scripts = Vec::new();
        let dir = Path::new(&self.config.custom_scripts_folder);
        let Ok(entries) = std::fs::read_dir(dir) else {
            return scripts;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("lua") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "custom.lua".into());
                info!("[UIManager] Loaded custom lua script: {}", name);
                scripts.push((name, content));
            }
        }
        scripts
    }

    pub fn build_injection_script(&self) -> String {
        let mut script = String::new();

        if !self.config.background_overrides.is_empty() {
            script.push_str(&LuaScriptGenerator::generate_bg_overrides_injector(
                &self.config.background_overrides,
            ));
            script.push('\n');
        }

        if !self.config.custom_buttons.is_empty() {
            script.push_str(&LuaScriptGenerator::generate_custom_buttons_injector(
                &self.config.custom_buttons,
            ));
            script.push('\n');
        }

        for (name, content) in self.load_custom_scripts() {
            script.push_str(&format!(
                "\n-- [MW-Client-Core Custom Script: {}]\n{}\n",
                name, content
            ));
        }

        script
    }
}
