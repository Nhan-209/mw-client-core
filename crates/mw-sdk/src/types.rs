use serde::{Deserialize, Serialize};

/// Định danh toàn bộ các sảnh trong Mini World
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LobbyType {
    /// Sảnh chính V4 (Phiên bản giao diện chuẩn mới nhất)
    MainLobbyV4,
    /// Sảnh chính V3
    MainLobbyV3,
    /// Sảnh chính V2
    MainLobbyV2,
    /// Sảnh chờ tổ đội / Ghép đội (Team-up waiting room)
    TeamUpWaitingRoom,
    /// Sảnh chơi nhiều người / Liên cơ (Multiplayer Lobby)
    MultiplayerLobby,
    /// Sảnh tiệc / Sảnh cộng đồng (Social Hall)
    SocialHall,
    /// Sảnh Gia viên (Homeland)
    Homeland,
    /// Sảnh thi đấu / Tranh tài (Race System Hall)
    RaceSystemHall,
    /// Sảnh đăng nhập & Trang chủ
    LoginScreen,
}

/// Thông tin chi tiết của từng sảnh để can thiệp texture và điều hướng
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyDescriptor {
    pub lobby_type: LobbyType,
    pub display_name: &'static str,
    /// Đường dẫn texture ảo mà Rainbow Engine / FairyGUI sử dụng
    pub virtual_texture_path: &'static str,
    /// Module UI FairyGUI hoặc Lua View tương ứng
    pub view_class: &'static str,
    /// Hàm gọi nhảy sảnh trong Lua script
    pub lua_jump_action: &'static str,
}

/// Danh sách mô tả tất cả các sảnh được trích xuất từ hệ thống Mini World
pub const KNOWN_LOBBIES: &[LobbyDescriptor] = &[
    LobbyDescriptor {
        lobby_type: LobbyType::MainLobbyV4,
        display_name: "Sảnh Chính V4",
        virtual_texture_path: "ui/mobile/texture0/bigtex/bg_hall_mi.jpg",
        view_class: "MainV4LobbyView",
        lua_jump_action: "GetInst('MainLobbyMgr'):ShowUI()",
    },
    LobbyDescriptor {
        lobby_type: LobbyType::MainLobbyV3,
        display_name: "Sảnh Chính V3",
        virtual_texture_path: "ui/mobile/texture0/bigtex/bg_hall_mi2.jpg",
        view_class: "MainV3LobbyView",
        lua_jump_action: "GetInst('MainLobbyMgr'):ShowUI()",
    },
    LobbyDescriptor {
        lobby_type: LobbyType::MainLobbyV2,
        display_name: "Sảnh Chính V2",
        virtual_texture_path: "ui/mobile/texture0/bigtex/bg_hall_mi.jpg",
        view_class: "MainV2LobbyView",
        lua_jump_action: "GetInst('MainLobbyMgr'):ShowUI()",
    },
    LobbyDescriptor {
        lobby_type: LobbyType::TeamUpWaitingRoom,
        display_name: "Sảnh Chờ Ghép Đội (Team-up)",
        virtual_texture_path: "ui/mobile/texture0/bigtex/img_team_main_back.png",
        view_class: "TeamupMainView",
        lua_jump_action: "GetInst('MiniUIManager'):OpenUI('TeamupMain', 'miniui/module/TeamupMain', 'TeamupMainAutoGen')",
    },
    LobbyDescriptor {
        lobby_type: LobbyType::MultiplayerLobby,
        display_name: "Sảnh Chơi Nhiều Người (Multiplayer)",
        virtual_texture_path: "ui/mobile/texture0/bigtex/img_board_online.png",
        view_class: "multiGameView",
        lua_jump_action: "G_MainLobbyJumpUtils:openMultiPlayer()",
    },
    LobbyDescriptor {
        lobby_type: LobbyType::SocialHall,
        display_name: "Sảnh Tiệc & Cộng Đồng (Social Hall)",
        virtual_texture_path: "ui/mobile/texture0/bigtex/ljdt_qukuaitu04.png",
        view_class: "SocialHallView",
        lua_jump_action: "G_MainLobbyJumpUtils:openSocialHall()",
    },
    LobbyDescriptor {
        lobby_type: LobbyType::Homeland,
        display_name: "Sảnh Gia Viên (Homeland)",
        virtual_texture_path: "ui/mobile/texture0/bigtex/bg_garden_lottery.png",
        view_class: "HomelandShopOpenBoxView",
        lua_jump_action: "G_MainLobbyJumpUtils:openHomeLand()",
    },
];

/// Định nghĩa nút bấm UI tùy biến để chèn vào giao diện sảnh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNavButton {
    pub id: String,
    pub title: String,
    pub icon_path: Option<String>,
    pub pos_x: f32,
    pub pos_y: f32,
    pub width: f32,
    pub height: f32,
    /// Sảnh mục tiêu khi click nút này
    pub target_lobby: Option<LobbyType>,
    /// Đoạn Lua code tùy biến thực thi khi click (nếu không dùng target_lobby mặc định)
    pub custom_lua_onclick: Option<String>,
}

impl LobbyType {
    pub fn descriptor(&self) -> Option<&'static LobbyDescriptor> {
        KNOWN_LOBBIES.iter().find(|desc| &desc.lobby_type == self)
    }

    pub fn virtual_texture(&self) -> Option<&'static str> {
        self.descriptor().map(|d| d.virtual_texture_path)
    }
}
