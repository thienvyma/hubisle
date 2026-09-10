# Isle Pulse Overlay

**Tiếng Việt** · [English](README.en.md)

**Isle Pulse Overlay 2.1.0** do **Huỳnh Vỹ** phát triển. Minimap hiện đè lên
game và tự lấy tọa độ nhân vật từ website quản lý của server. Ứng dụng có adapter
cho **Era Gaming VN**, **The Real Server VN (Titan)** và **IslePilot**.

Lần đầu mở app, nhập website server và đăng nhập trong cửa sổ riêng. Phiên đăng
nhập được mã hóa trên máy; app tiếp tục cập nhật vị trí, hướng đi và chỉ số trên
minimap khi chơi. Website chưa được hỗ trợ cần thêm adapter xác định — app không
gửi cookie sang domain lạ hoặc đoán cấu trúc API.

![Minimap có chấm bạn bè, thanh chỉ số, nhiệm vụ Prime và thanh Thoại đè lên game — con dino đang mang skin vừa đổi trong app](docs/screenshot-ingame.jpg)

![Bản đồ lớn với tên địa danh và các lớp POI](docs/screenshot-fullmap.png)

![Tab Khủng long của bạn với chỉ số và Prime progress](docs/screenshot-dino.png)

![Tab Garage (Gacha): dino đã gửi kho, xem 3D, growth và chỉ số, nút Restore](docs/screenshot-garage.png)

![Bảng phối màu skin theo từng vùng và áp trực tiếp lên dino](docs/screenshot-skin.png)

![Tab Thoại: nói theo khoảng cách qua IsleVOIP, chọn micro và người trong tầm nghe](docs/screenshot-voip.png)

## Tính năng

- **Minimap tròn** bám góc cửa sổ game, chuột bấm xuyên qua, không cản trở lúc chơi.
  Hướng Bắc luôn ở trên, có mũi tên chỉ hướng đang đi.
- **Bản đồ lớn**: phóng to/thu nhỏ mượt, 12 lớp bật/tắt được (nước ngọt, nguồn
  nước, mỏ muối, vũng bùn, khu bảo tồn, vùng di cư, vùng tuần tra AI, khu thức
  ăn, động vật với biểu tượng riêng từng loài 🐗🦌🐢, tên vùng, địa điểm, và
  lớp **POI server** sống từ IslePilot), tên địa danh hiện trực tiếp trên bản
  đồ; danh sách lớp thu gọn được; nút xóa đường đi cho đỡ rối mắt giữa trận.
  Mở bản đồ bằng phím tắt là app tự về tab Bản đồ.
- **3 kiểu nền bản đồ**: ảnh chụp Vulnona (mặc định) hoặc bản vẽ tay
  [IsleMaps](https://www.islemaps.com/) sáng/tối — đổi trong Cài đặt, áp dụng
  cho cả bản đồ lớn lẫn minimap. Nền IsleMaps vẽ theo phiên bản game mới hơn,
  thấy cả quần đảo đông nam (Hell's Mouth).
- **Điểm đánh dấu (waypoint)**: bấm chuột phải để cắm, đổi tên/đổi màu, xóa,
  biểu tượng nhanh (💀 chỗ chết, 🏠 hang…); minimap có mũi tên rìa đĩa chỉ
  hướng + khoảng cách tới điểm gần nhất.
- **Tìm kiếm & điều hướng**: ô tìm địa danh/waypoint, dán tọa độ để nhảy tới,
  chế độ bám vị trí với mũi tên mép màn hình dẫn về chỗ đứng.
- **Đường đã đi**: tự ghi theo phiên, khôi phục lại đường đi của phiên trước.
- **Bạn bè cùng server**: tự hiển thị những người đã chấp nhận kết bạn và đang
  online khi Era hoặc Titan cấp vị trí; minimap giữ số bạn ở mép vòng khi ở xa.
- **Vị trí và chỉ số realtime**: tọa độ, growth, máu, đói, khát và thể lực được
  chuẩn hóa từ provider đã chọn rồi đưa vào cùng minimap. Trường nào server không
  cung cấp sẽ để trống, không dựng dữ liệu giả.
- **Garage đa server**: IslePilot giữ card/model 3D cùng Park/Restore/Đổi tên/Bán;
  Era và Titan có 5 slot cất/lấy/xóa theo API chính thức, khóa thao tác theo
  trạng thái online/cầu nối và theo dõi tiến trình xử lý của server.
- **Đổi skin trong hub**: 7 vùng màu theo giao thức hiện hành của Era/Titan,
  preset, lưu bản nháp trên máy, quyền 16 màu/đủ màu của Era, biến thể Titan và
  cooldown do server xác nhận. IslePilot chưa có API overlay chính thức cho mục này.
- **Phím tắt toàn cục** đổi được trong app, giao diện song ngữ Việt/Anh.

## Cài đặt

Chạy file cài đặt NSIS được tạo từ nhánh tùy chỉnh này.
Lần đầu mở app sẽ tải dữ liệu bản đồ (~3 MB) về máy.

Yêu cầu: **Windows 10/11 64-bit**. WebView2 (thường đã có sẵn trên Windows 11;
nếu thiếu, installer tự tải về).

> Windows có thể hiện cảnh báo SmartScreen vì installer chưa ký số.
> Bấm **More info → Run anyway**.

### Cập nhật tự động

Ứng dụng kiểm tra bản phát hành tại [github.com/thienvyma/hubisle](https://github.com/thienvyma/hubisle).
Khi có phiên bản cao hơn, một bảng thông báo sẽ cho phép tải và cài ngay trong
ứng dụng. Mỗi gói cập nhật phải có chữ ký của Huỳnh Vỹ; gói bị sửa hoặc phát
hành từ nguồn khác sẽ bị từ chối.

App kiểm tra khi mở, mỗi giờ và khi mạng kết nối lại. Bạn cũng có thể bấm
**Kiểm tra cập nhật** ở chân cửa sổ để xem trạng thái hoặc thử lại khi có lỗi.
Thông báo có nội dung bản phát hành, tiến trình tải và nút **Để sau**.

## Kết nối "Khủng long của bạn" (IslePilot)

Tab **Khủng long** đọc chỉ số dino của chính bạn (growth, máu, đói, khát, thể
lực, dinh dưỡng, Prime progress) từ hệ thống IslePilot. Có 2 cách kết nối:

**Cách 1 — Đăng nhập Steam qua IslePilot (khuyên dùng):** mở tab Khủng long →
bấm **Đăng nhập Steam** → đăng nhập trong cửa sổ islepilot.eu hiện ra, cửa sổ
tự đóng khi xong. Chỉ cần làm **một lần duy nhất** — không cần nhập link
server, dùng cho **mọi server IslePilot**, đổi server trong game là dữ liệu tự
đổi theo. Cách này còn mở thêm tab **Garage (Gacha)** và lớp **POI server**
trên bản đồ. Nếu cửa sổ không tự bắt được token, mở mục *"Hoặc dán token thủ
công"* và dán token (hoặc nguyên link `theisle-overlay://…`).

**Cách 2 — Cách cũ: nhập server + cookie** (chỉ khi cách 1 không hoạt động;
cookie lưu riêng từng server, đổi server phải làm lại). Mở mục **"Cách cũ"**
trong phần đăng nhập, nhập link server rồi bấm Đăng nhập Steam trong mục đó;
vẫn không được thì dán cookie thủ công:

1. Mở trang server trong trình duyệt và đăng nhập Steam ở đó. Bấm **F12**
   (hoặc chuột phải → **Inspect**) rồi chọn tab **Application** (Chrome) /
   **Storage** (Firefox).

   ![Mở DevTools và chọn tab Application](docs/guide-dino-1-devtools.png)

2. Chọn **Cookies** → domain của server → bấm cookie tên **`islepilot_player`**
   → copy toàn bộ **Value**.

   ![Copy giá trị cookie islepilot_player](docs/guide-dino-2-copy-cookie.jpg)

3. Trong app: dán vào ô cookie → bấm **Kiểm tra & lưu cookie**.

   ![Nhập link server, dán cookie và lưu trong app](docs/guide-dino-3-paste-app.jpg)

Server có **live map** thì app tự nhận và bật "lấy vị trí tự động" — khỏi cần
copy tọa độ thủ công; server tắt live map thì tùy chọn này tự khóa.

**Một số server dùng IslePilot** (tham khảo — mọi server chạy IslePilot đều
dùng được):

- https://mixi.islepilot.eu
- https://hoho.islepilot.eu
- https://sdvn.islepilot.eu
- https://sdvn2.islepilot.eu
- https://khunglong.islepilot.eu
- https://islepilot.eu/p/sbtcisland

## Nhẹ cỡ nào?

Số đo thật trên máy: **Intel Core i5-14400F (10 nhân/16 luồng), 32 GB RAM,
RTX 3060 Ti, Windows 11 Pro build 26200, độ phân giải 100%** — bản release v1.0.0:

| Hạng mục | Dung lượng |
|---|---|
| File cài đặt | **4,3 MB** |
| File chạy sau khi cài | 17,8 MB |
| Dữ liệu bản đồ tải lần đầu | 2,9 MB (ảnh nền 2,6 MB + dữ liệu điểm 0,3 MB) |
| **Tổng chiếm ổ cứng** | **~21 MB** |

| Lúc chạy | RAM (working set) | CPU lúc rảnh |
|---|---|---|
| Mở cả bản đồ lớn + minimap | **522 MB** (8 tiến trình) | 0,18% |
| Ẩn bản đồ lớn bằng `Ctrl+Alt+F` (kịch bản khi đang chơi) | **448 MB** | 0,08% |

**CPU gần như bằng 0** vì app không có vòng lặp vẽ lại — chỉ vẽ khi có dữ liệu mới.

## Lưu ý cần biết

1. **Chế độ hình ảnh của game**: overlay không thể hiện đè lên **Toàn màn hình
   độc quyền (Exclusive Fullscreen)** — đây là giới hạn của Windows với mọi
   overlay ngoài tiến trình. Hãy dùng **Cửa sổ** hoặc **Toàn màn hình không viền**.
   App tự đọc cấu hình game và cảnh báo nếu bạn đang để sai chế độ.
2. **Vị trí không tự động cập nhật**: bạn phải tự bấm `Tab` → **Asset Location**
   trong game mỗi khi muốn cập nhật vị trí. Đây là *chủ đích*, không phải thiếu sót
   — xem mục an toàn anti-cheat bên dưới.
3. **Hướng đi cần hai lần chép tọa độ** cách nhau ít nhất 20 m; mẫu cũ quá 10 phút
   thì hướng hết hạn để tránh chỉ sai.
4. **Không mở được hai bản cùng lúc** — phím tắt toàn cục mang tính độc quyền,
   hai bản chạy song song sẽ tranh nhau.
5. **Máy ít RAM**: ẩn bản đồ lớn bằng `Ctrl+Alt+F` khi vào game — app tự giảm
   bộ nhớ của cửa sổ ẩn. Bấm nút X thì app thu về **khay hệ thống** (icon cạnh
   đồng hồ) như Steam/Discord — chuột trái icon để mở lại, chuột phải → Thoát
   để tắt hẳn.
6. **Phím tắt bị ứng dụng khác chiếm**: app báo ngay khi khởi động, đổi lại trong
   tab Cài đặt.
7. **Tính năng "Khủng long của bạn"** hỗ trợ server dùng nền tảng IslePilot —
   xem mục [Kết nối "Khủng long của bạn"](#kết-nối-khủng-long-của-bạn-islepilot).
   Chế độ đăng nhập Steam (khuyên dùng) đọc qua API JSON ổn định; riêng cách cũ
   server + cookie phân tích HTML trang web của server nên **có thể hỏng khi
   IslePilot đổi giao diện** — app sẽ báo khi phát hiện server vừa cập nhật.
   Nếu phần này lỗi, các tính năng bản đồ **không bị ảnh hưởng**.
8. **Nên hỏi admin server** trước khi dùng thường xuyên — một số server có luật
   riêng về công cụ bên thứ ba. Tùy chọn lấy vị trí tự động chỉ bật khi app dò
   thấy server có live map; server tắt live map thì tùy chọn tự khóa, và lựa
   chọn tắt/bật thủ công của bạn luôn được tôn trọng.
9. **Token/cookie đăng nhập** được mã hóa bằng Windows DPAPI, chỉ giải được
   bằng tài khoản Windows của bạn trên chính máy đó.
10. **SmartScreen** sẽ cảnh báo ở lần cài đầu vì installer chưa ký số (chứng chỉ ký
   số tốn phí hằng năm). Bản cập nhật tự động về sau không bị hỏi lại.

## An toàn với anti-cheat

Game chạy Easy Anti-Cheat cấp kernel. App này an toàn vì **không bao giờ đụng
vào tiến trình game**:

- Vị trí realtime lấy qua **HTTPS từ website quản lý đã đăng nhập của server**;
  clipboard chỉ còn là cách dự phòng thủ công khi provider không cấp vị trí.
- Phím tắt dùng `RegisterHotKey` (API hợp tác của Windows), **không phải**
  keyboard hook.
- Chỉ số, vị trí, bạn bè, Garage và skin lấy qua **HTTPS tới đúng provider đã
  xác minh** (Era, Titan hoặc IslePilot), không đọc bộ nhớ tiến trình game.
- Phím `` ` `` chỉ dùng `SendInput` để gửi chuỗi cố định `/unstuck` sau khi xác
  nhận The Isle đang là cửa sổ foreground. App không inject DLL, hook DirectX
  hay bắt gói mạng.

CI có bước grep chặn mọi call site API cấm (`scripts/check-forbidden-apis.ps1`).
Danh sách API được phép nằm ở đầu `src-tauri/src/win/mod.rs`.

## Phát triển

Yêu cầu: Node 22+, Rust stable (MSVC), WebView2.

```powershell
npm install
npx tauri dev                        # chạy dev

# Lái UI không cần mở game:
$env:THEISLE_REPLAY = "path\to\replay_sample.txt"; npx tauri dev

# Test
npm run check                        # svelte-check
cd src-tauri; cargo test --workspace # toàn bộ test Rust
cargo clippy --workspace -- -D warnings
..\scripts\check-forbidden-apis.ps1

# Sau mỗi lần map game cập nhật (chạy cho từng nền đã tải):
cargo run --bin verify_data --features devtools -- --source vulnona
cargo run --bin verify_data --features devtools -- --source islemaps-light
cargo run --bin verify_data --features devtools -- --source islemaps-dark
cargo test -p theisle-overlay --lib -- --ignored parse_real_cache
```

Lưu ý: `.cargo/config.toml` đặt `target-dir` ra ngoài thư mục OneDrive.

## Nguồn dữ liệu

Dữ liệu bản đồ **tải khi chạy lần đầu, không đóng gói sẵn** — đây là bản sao
cá nhân trên máy bạn, không phải bản phát hành lại.

- Basemap: [VulnonaMAP](https://vulnona.com/game/map/) (Coco.N) — ghép từ ảnh
  chụp trong game. Bản quyền hình ảnh: Afterthought LLC (The Isle).
- Nền IsleMaps (tùy chọn, chỉ tải khi bạn chọn trong Cài đặt) và điểm spawn
  động vật: [islemaps.com](https://www.islemaps.com/) (Pont & Emeara).
- POI: [myislemap.com](https://myislemap.com/), VulnonaMAP, hướng dẫn Steam
  của wiredredman.

Không liên kết với Afterthought LLC.

## Ghi công

Được phát triển bởi **Huỳnh Vỹ**.

- 💬 Facebook: https://www.facebook.com/thienvyma
- 💻 GitHub: https://github.com/thienvyma/hubisle
