use crate::types::{CustomNavButton, LobbyType};

/// Plugin Interface dành cho coder bên ngoài phát triển tính năng tùy biến UI / Client
pub trait ClientPlugin: Send + Sync {
    /// Tên định danh của plugin
    fn name(&self) -> &'static str;

    /// Phiên bản plugin
    fn version(&self) -> &'static str {
        "1.0.0"
    }

    /// Khởi tạo khi Runtime Core được inject vào game
    fn on_initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Kích hoạt khi game chuyển sang một sảnh cụ thể
    fn on_lobby_enter(&mut self, _lobby: LobbyType) {}

    /// Kích hoạt khi rời khỏi sảnh
    fn on_lobby_exit(&mut self, _lobby: LobbyType) {}

    /// Danh sách các nút bấm tùy biến cần chèn vào giao diện
    fn custom_buttons(&self) -> Vec<CustomNavButton> {
        Vec::new()
    }

    /// Danh sách texture thay thế ảnh nền sảnh (LobbyType -> đường dẫn tệp ảnh ngoài đĩa)
    fn custom_textures(&self) -> Vec<(LobbyType, String)> {
        Vec::new()
    }

    /// Đoạn Lua script bổ sung muốn inject lúc khởi động máy ảo Lua
    fn custom_lua_scripts(&self) -> Vec<String> {
        Vec::new()
    }
}
