use mw_sdk::types::{CustomNavButton, LobbyType, KNOWN_LOBBIES};
use mw_sdk::LuaScriptGenerator;

#[test]
fn test_known_lobbies_not_empty() {
    assert!(!KNOWN_LOBBIES.is_empty());
}

#[test]
fn test_lobby_descriptor_lookup() {
    let desc = LobbyType::MainLobbyV4.descriptor();
    assert!(desc.is_some());
    let desc = desc.unwrap();
    assert_eq!(
        desc.virtual_texture_path,
        "ui/mobile/texture0/bigtex/bg_hall_mi.jpg"
    );
    assert_eq!(desc.view_class, "MainV4LobbyView");
}

#[test]
fn test_lua_override_generation() {
    let script = LuaScriptGenerator::generate_background_override(
        LobbyType::MainLobbyV4,
        "D:/custom_ui/my_bg.png",
    );
    assert!(script.contains("img_bg:setURL"));
    assert!(script.contains("D:/custom_ui/my_bg.png"));
}

#[test]
fn test_lua_button_generation() {
    let buttons = vec![CustomNavButton {
        id: "btn_test".into(),
        title: "Test Button".into(),
        icon_path: None,
        pos_x: 100.0,
        pos_y: 200.0,
        width: 120.0,
        height: 40.0,
        target_lobby: Some(LobbyType::TeamUpWaitingRoom),
        custom_lua_onclick: None,
    }];
    let script = LuaScriptGenerator::generate_custom_buttons_injector(&buttons);
    assert!(script.contains("btn_btn_test"));
    assert!(script.contains("Test Button"));
    assert!(script.contains("TeamupMain"));
}

#[test]
fn test_lua_bg_overrides_batch_generation() {
    use mw_sdk::types::LobbyBgOverride;

    let overrides = vec![
        LobbyBgOverride {
            lobby: LobbyType::MainLobbyV4,
            image_path: "D:/assets/bg1.png".into(),
        },
        LobbyBgOverride {
            lobby: LobbyType::TeamUpWaitingRoom,
            image_path: "D:/assets/bg2.png".into(),
        },
    ];
    let script = LuaScriptGenerator::generate_bg_overrides_injector(&overrides);
    assert!(script.contains("D:/assets/bg1.png"));
    assert!(script.contains("D:/assets/bg2.png"));
    assert!(script.contains("TeamupMainAutoGen"));
}

