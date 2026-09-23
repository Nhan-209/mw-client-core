use crate::types::{CustomNavButton, LobbyBgOverride, LobbyType};

/// Trình sinh mã kịch bản Lua 5.1 an toàn để inject vào Mini World
pub struct LuaScriptGenerator;

impl LuaScriptGenerator {
    /// Sinh mã Lua để ghi đè hàng loạt ảnh nền sảnh từ cấu hình
    pub fn generate_bg_overrides_injector(overrides: &[LobbyBgOverride]) -> String {
        let mut script = String::from("-- [MW-Client-Core] Batch Background Overrides\n");
        for item in overrides {
            script.push_str(&Self::generate_background_override(
                item.lobby,
                &item.image_path,
            ));
        }
        script
    }

    /// Sinh mã Lua để thay đổi ảnh nền của một sảnh cụ thể tại runtime
    pub fn generate_background_override(lobby: LobbyType, custom_abs_path: &str) -> String {
        let normalized_path = custom_abs_path.replace('\\', "/");
        match lobby {
            LobbyType::MainLobbyV4 | LobbyType::MainLobbyV3 | LobbyType::MainLobbyV2 => {
                format!(
                    r#"
-- [MW-Client-Core] Override Main Lobby Background
local function _apply_custom_lobby_bg()
    local lobbyUI = MainLobbyMgr:GetLobbyUI()
    if lobbyUI and lobbyUI.view and lobbyUI.view.img_bg then
        lobbyUI.view.img_bg:setURL("{path}")
        print("[MW-Client-Core] Overrode Main Lobby Background: {path}")
    end
end
_apply_custom_lobby_bg()
"#,
                    path = normalized_path
                )
            }
            LobbyType::TeamUpWaitingRoom => {
                format!(
                    r#"
-- [MW-Client-Core] Override TeamUp Waiting Room Background
local function _apply_custom_teamup_bg()
    local curUI = GetInst("MiniUIManager"):GetUI("TeamupMainAutoGen")
    if curUI and curUI.ctrl and curUI.ctrl.view and curUI.ctrl.view.bkg then
        curUI.ctrl.view.bkg:setURL("{path}")
        print("[MW-Client-Core] Overrode TeamUp Background: {path}")
    end
end
_apply_custom_teamup_bg()
"#,
                    path = normalized_path
                )
            }
            _ => {
                format!(
                    r#"
print("[MW-Client-Core] Background override scheduled for {lobby:?} -> {path}")
"#,
                    lobby = lobby,
                    path = normalized_path
                )
            }
        }
    }

    /// Sinh mã Lua để chèn các nút bấm UI chuyển giao diện tùy thích
    pub fn generate_custom_buttons_injector(buttons: &[CustomNavButton]) -> String {
        let mut script = String::from(
            r#"
-- [MW-Client-Core] Inject Custom Navigation Buttons
local function _inject_custom_nav_buttons()
    local currentSceneRoot = GetInst("MiniUISceneMgr"):getCurrentSceneRootNode()
    if not currentSceneRoot then return end
"#,
        );

        for btn in buttons {
            let action_code = if let Some(custom) = &btn.custom_lua_onclick {
                custom.clone()
            } else if let Some(target) = btn.target_lobby {
                match target {
                    LobbyType::MainLobbyV4 | LobbyType::MainLobbyV3 | LobbyType::MainLobbyV2 => {
                        "GetInst('MainLobbyMgr'):ShowUI()".to_string()
                    }
                    LobbyType::TeamUpWaitingRoom => {
                        "GetInst('MiniUIManager'):OpenUI('TeamupMain', 'miniui/module/TeamupMain', 'TeamupMainAutoGen')".to_string()
                    }
                    LobbyType::MultiplayerLobby => {
                        "if G_MainLobbyJumpUtils then G_MainLobbyJumpUtils:openMultiPlayer() end".to_string()
                    }
                    LobbyType::SocialHall => {
                        "if G_MainLobbyJumpUtils then G_MainLobbyJumpUtils:openSocialHall() end".to_string()
                    }
                    LobbyType::Homeland => {
                        "if G_MainLobbyJumpUtils then G_MainLobbyJumpUtils:openHomeLand() end".to_string()
                    }
                    _ => "print('[MW-Client-Core] Custom button clicked')".to_string(),
                }
            } else {
                "print('[MW-Client-Core] No action assigned to button')".to_string()
            };

            script.push_str(&format!(
                r#"
    -- Button: {id} ("{title}")
    local btn_{id} = fairygui.UIPackage.createObject("common", "Button")
    if btn_{id} then
        btn_{id}:setText("{title}")
        btn_{id}:setXY({x}, {y})
        btn_{id}:setSize({w}, {h})
        btn_{id}:addClickListener(function()
            {action}
        end)
        currentSceneRoot:addChild(btn_{id})
        print("[MW-Client-Core] Attached button: {title}")
    end
"#,
                id = btn.id,
                title = btn.title,
                x = btn.pos_x,
                y = btn.pos_y,
                w = btn.width,
                h = btn.height,
                action = action_code
            ));
        }

        script.push_str(
            r#"
end
_inject_custom_nav_buttons()
"#,
        );

        script
    }
}
