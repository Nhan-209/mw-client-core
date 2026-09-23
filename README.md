# mw-client-core

Runtime hook and SDK for Mini World UI customization and client modding.

## Architecture

- `crates/mw-core`: Runtime DLL (`cdylib`). Injected into game process.
  - VFS hook: Intercepts `CreateFileW` to redirect game asset requests to disk (`custom_assets/textures/`).
  - Lua hook: Intercepts `luaL_loadbuffer` in `liblua.dll` to execute custom UI scripts.
- `crates/mw-sdk`: Public crate for plugin developers. Provides `ClientPlugin` trait, lobby descriptors, and Lua script generators.
- `crates/mw-loader`: Standalone injector CLI using `CreateRemoteThread` + `LoadLibraryW`.

## Asset Redirection

Drop replacement images into `custom_assets/textures/`:

| File | Target UI |
| :--- | :--- |
| `bg_hall_mi.jpg` | Main Lobby V4 |
| `bg_hall_mi2.jpg` | Main Lobby V3 |
| `img_team_main_back.png` | Team-Up Waiting Room |
| `bg_garden_lottery.png` | Homeland Hall |
| `img_board_online.png` | Multiplayer Lobby button/banner |

## Configuration (`mw_config.json`)

Configure background overrides, custom navigation buttons, and folders:

```json
{
  "enable_vfs_redirection": true,
  "enable_lua_hooks": true,
  "custom_textures_folder": "custom_assets/textures",
  "custom_scripts_folder": "custom_assets/scripts",
  "background_overrides": [
    {
      "lobby": "MainLobbyV4",
      "image_path": "custom_assets/textures/bg_hall_mi.jpg"
    }
  ],
  "custom_buttons": [
    {
      "id": "btn_goto_teamup",
      "title": "Sảnh Chờ",
      "pos_x": 20.0,
      "pos_y": 200.0,
      "width": 140.0,
      "height": 48.0,
      "target_lobby": "TeamUpWaitingRoom"
    },
    {
      "id": "btn_goto_multi",
      "title": "Chơi Mạng",
      "pos_x": 20.0,
      "pos_y": 260.0,
      "width": 140.0,
      "height": 48.0,
      "target_lobby": "MultiplayerLobby"
    }
  ]
}
```

## Custom Scripts (`custom_assets/scripts/`)

Drop any `.lua` file into `custom_assets/scripts/` to run arbitrary FairyGUI / Mini World Lua code upon lobby initialization. All files are loaded and executed sequentially.

## Plugin Development

Add `mw-sdk` to `Cargo.toml`:

```toml
[dependencies]
mw-sdk = { git = "https://github.com/Nhan-209/mw-client-core" }
```

Implement `ClientPlugin`:

```rust
use mw_sdk::{ClientPlugin, CustomNavButton, LobbyType};

pub struct CustomNavigation;

impl ClientPlugin for CustomNavigation {
    fn name(&self) -> &'static str {
        "CustomNavigation"
    }

    fn custom_buttons(&self) -> Vec<CustomNavButton> {
        vec![CustomNavButton {
            id: "btn_social".into(),
            title: "Social Hall".into(),
            icon_path: None,
            pos_x: 20.0,
            pos_y: 320.0,
            width: 140.0,
            height: 48.0,
            target_lobby: Some(LobbyType::SocialHall),
            custom_lua_onclick: None,
        }]
    }
}
```

## CI / Verification

All builds and tests run on GitHub Actions (`.github/workflows/ci.yml`):
- Targets: `x86_64-pc-windows-msvc`, `i686-pc-windows-msvc`
- Checks: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo build --release`

## License

MIT
