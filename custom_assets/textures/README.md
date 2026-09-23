# Hướng Dẫn Thêm Ảnh Nền Custom Vào Sảnh

Thả tệp ảnh mới (định dạng JPG hoặc PNG) vào thư mục này với tên trùng khớp với texture mà game sử dụng:

| Tên Tệp Ảnh | Sảnh Tương Ứng Trong Mini World | Ghi Chú Kích Thước Khuyến Nghị |
| :--- | :--- | :--- |
| `bg_hall_mi.jpg` | Sảnh Chính V4 (Phiên bản mặc định mới nhất) | 1920x1080 hoặc 2560x1440 |
| `bg_hall_mi2.jpg` | Sảnh Chính V3 | 1920x1080 |
| `img_team_main_back.png` | Sảnh Chờ Ghép Đội (Team-Up Waiting Room) | 1920x1080 |
| `bg_garden_lottery.png` | Sảnh Gia Viên (Homeland) | 1280x720 |
| `img_board_online.png` | Nút/Banner Sảnh Chơi Mạng | Tự do |

Khi game khởi động, VFS Hook trong `mw_core.dll` sẽ tự động trỏ nạp ảnh từ thư mục này mà không cần sửa file `.pkg` của game!
