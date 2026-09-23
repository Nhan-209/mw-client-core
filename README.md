# Mini World Custom Client Core (`mw-client-core`)

> **Bộ khung mã nguồn mở (Rust) hỗ trợ tùy biến UI tự do, thay đổi ảnh nền các sảnh, chèn nút điều hướng giao diện và mở rộng tính năng client cho game Mini World.**

---

## 🌟 Tính Năng Nổi Bật

* **Tùy biến UI & Thay đổi ảnh nền không cần sửa `.pkg`**:
  * Sử dụng **Virtual File System (VFS) Redirection**: Tự động chuyển hướng nạp ảnh từ thư mục `custom_assets/textures/`.
  * Khi game cập nhật bản vá hay tải lại file `.pkg`, **ảnh nền tùy biến của bạn không bao giờ bị mất hoặc bị ghi đè**.
* **Chèn nút bấm & Chuyển giao diện tự do**:
  * Can thiệp máy ảo **Lua 5.1** (`liblua.dll`) qua Detour Hook.
  * Tự động sinh mã kịch bản FairyGUI để thêm nút bấm điều hướng nhanh qua lại giữa tất cả các sảnh (Sảnh V4, Sảnh Tổ đội, Sảnh Chơi mạng, Sảnh Gia viên,...).
* **Linh hoạt phương thức nạp**:
  * Hỗ trợ tiêm qua **mọi bộ Injector bên ngoài** (Standard DLL Injection thông qua `DllMain`).
  * Tích hợp sẵn injector mẫu nhẹ bằng Rust (`mw-loader`).
* **Hỗ trợ phát triển Plugin cho Coder khác (`mw-sdk`)**:
  * Cung cấp crate `mw-sdk` chuẩn hóa với trait `ClientPlugin` để cộng đồng lập trình viên có thể viết thêm mod, custom giao diện, hoặc xây dựng client riêng.
* **Tự động hóa hoàn toàn trên GitHub Actions CI**:
  * 100% quy trình Format, Lint (Clippy), Unit Test và Build Release đều chạy trên GitHub Actions (Windows MSVC x86_64 và i686).

---

## 📁 Cấu Trúc Mã Nguồn

```text
mw-client-core/
├── .github/workflows/ci.yml    # CI/CD tự động build release artifact trên GitHub
├── crates/
│   ├── mw-sdk/                 # SDK thư viện công khai cho Coder viết Plugin
│   │   ├── src/types.rs        # Danh mục tất cả sảnh & Virtual Texture Path
│   │   ├── src/plugin.rs       # ClientPlugin trait interface
│   │   └── src/lua_api.rs      # Trình tạo kịch bản Lua can thiệp FairyGUI
│   ├── mw-core/                # DLL Runtime (mw_core.dll) tiêm vào game
│   │   ├── src/lib.rs          # DllMain & Background worker thread
│   │   ├── src/hook/           # VFS Redirection Hook & Lua 5.1 Hook
│   │   └── src/ui_manager.rs   # Quản lý cấu hình mw_config.json
│   └── mw-loader/              # CLI Injector độc lập bằng Rust
└── custom_assets/
    ├── textures/               # Thư mục thả ảnh nền tùy biến
    └── scripts/                # Thư mục chứa script Lua custom
```

---

## 🎨 Hướng Dẫn Tùy Biến Giao Diện

### 1. Thay Đổi Ảnh Nền Sảnh
Chỉ cần thả ảnh JPG/PNG của bạn vào thư mục `custom_assets/textures/` với tên tệp tương ứng:
* `bg_hall_mi.jpg`: Ảnh nền Sảnh Chính V4.
* `bg_hall_mi2.jpg`: Ảnh nền Sảnh Chính V3.
* `img_team_main_back.png`: Ảnh nền Sảnh Chờ Ghép Đội (Team-up).
* `bg_garden_lottery.png`: Ảnh nền Sảnh Gia Viên.

### 2. Tùy Biến Nút Bấm Điều Hướng (`mw_config.json`)
Chỉnh sửa file `mw_config.json` nằm cùng thư mục game:
```json
{
  "enable_vfs_redirection": true,
  "enable_lua_hooks": true,
  "custom_textures_folder": "custom_assets/textures",
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

---

## 🚀 Hướng Dẫn Biên Dịch & Kiểm Thử Trên GitHub Actions

1. Đẩy (push) mã nguồn dự án này lên repository GitHub Public của bạn:
   ```bash
   git init
   git add .
   git commit -m "feat: initial commit mw-client-core"
   git remote add origin https://github.com/<your-username>/<your-repo>.git
   git push -u origin main
   ```
2. Mở tab **Actions** trên GitHub Repo:
   * Hệ thống sẽ tự động chạy pipeline `.github/workflows/ci.yml`.
   * Kiểm tra định dạng code (`cargo fmt`), lint cảnh báo (`cargo clippy`), và biên dịch cho cả Windows x86_64 lẫn i686.
3. Tải file nhị phân hoàn chỉnh từ mục **Artifacts** của Action:
   * `mw_core.dll`: File DLL runtime để tiêm vào game.
   * `mw_loader.exe`: File thực thi injector.

---

## 💻 Dành Cho Coder: Viết Plugin Mở Rộng Bằng `mw-sdk`

Thêm `mw-sdk` vào `Cargo.toml`:
```toml
[dependencies]
mw-sdk = { git = "https://github.com/example/mw-client-core" }
```

Viết Plugin tùy biến:
```rust
use mw_sdk::{ClientPlugin, LobbyType, CustomNavButton};

pub struct MyCustomLobbyPlugin;

impl ClientPlugin for MyCustomLobbyPlugin {
    fn name(&self) -> &'static str {
        "MySuperLobby"
    }

    fn custom_buttons(&self) -> Vec<CustomNavButton> {
        vec![
            CustomNavButton {
                id: "btn_my_feature".into(),
                title: "Menu Nhanh".into(),
                icon_path: None,
                pos_x: 50.0,
                pos_y: 100.0,
                width: 120.0,
                height: 40.0,
                target_lobby: Some(LobbyType::SocialHall),
                custom_lua_onclick: None,
            }
        ]
    }
}
```

---

## 📜 Giấy Phép (License)
Dự án được phát hành dưới giấy phép [MIT](LICENSE).
Mã nguồn này được viết hoàn toàn độc lập, không tái phân phối bất kỳ asset đồ họa hay nhị phân độc quyền nào của nhà phát hành game.
