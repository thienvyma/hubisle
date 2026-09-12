# Lưu trữ nghiên cứu hướng camera từ la bàn Q

Cập nhật hiện tại: Isle Pulse 2.1.4 đã dùng sidecar Npcap riêng để giải mã góc
camera từ UDP chiều đi của game. Phần dưới là hồ sơ nghiên cứu lịch sử về phương
án nhận dạng la bàn Q trên nền 2.1.1; bộ nhận dạng ảnh Q không được đưa vào app.

Xem [phân tích lưu trữ và các sửa đổi ban đầu](runtime-research.md).

Ngày kiểm tra: 11/09/2026. Code ứng dụng được đối chiếu tại `e23cd3b` (2.1.1).

Đây là bộ thử phép tính offline, **không phải nguồn góc hiện tại của hub**.
Không chụp màn hình, không gửi phím Q, không đọc chuột, không kết nối game/server.
Ứng dụng đang cài trên máy không bị thay đổi. Chưa có profile hiệu chỉnh lấy từ
game thật, bộ nhận dạng pixel, hoặc kiểm chứng độ trễ trong game.

## Q thực sự làm gì?

- Cấu hình `GameUserSettings.ini` của người dùng ánh xạ `Q` tới `Scent`;
  `LeftAlt` tới `FreeLook`; `MouseX` tới `LookRight/Left`. Đây là bằng chứng
  trực tiếp về phím đang cấu hình, không phải phép đo hướng camera.
- Lập trình viên game mô tả giữ Q mở rộng vùng nhận dấu chân; bấm Q còn dùng
  bật/tắt và chọn dấu chân để theo dõi. Vì Q phụ thuộc ngữ cảnh, không nên
  suy ra trạng thái la bàn chỉ từ việc phím Q đang được giữ hay được nhả.
  Nguồn lịch sử: [DevBlog #18, 29/10/2021](https://store.steampowered.com/news/posts/?appgroupname=The+Isle&appids=376210&enddate=1646104136&feed=steam_community_announcements).
- [DevBlog #22, 28/02/2022](https://store.steampowered.com/news/app/376210/view/3128317225041424482)
  mô tả việc đưa biểu tượng mùi trở lại la bàn. Như vậy thanh này chứa cả
  phương hướng lẫn biểu tượng mùi; không thể coi mọi hình nhọn trên thanh là Bắc/Nam.
- Người dùng xác nhận phiên bản đang chơi hiển thị Bắc bằng đỉnh hướng lên,
  Nam bằng đỉnh hướng xuống. Phần HUD trong
  [hướng dẫn do ParaVixen viết](https://steamcommunity.com/sharedfiles/filedetails/?id=3440690332)
  cũng mô tả Q và hai dấu Bắc/Nam. Đây là nguồn cộng đồng, không phải đặc tả
  thuật toán hay xác nhận chính thức về hình học của thanh.
- Các thông tin cũ về giới hạn của scent không được áp dụng máy móc:
  [patch 0.13.18.17 ngày 19/12/2023](https://store.steampowered.com/news/posts/?appids=376210&enddate=1706029590&feed=steam_community_announcements)
  đã bổ sung scent khi ngồi nghỉ hoặc cúi. Bộ đọc nên kiểm tra HUD đang thấy
  thay vì suy đoán từ tư thế hoặc một danh sách trạng thái cũ.

Chưa xác minh trên bản game hiện tại: thanh Q bám camera hay thân khi giữ Alt;
thời gian hiện sau khi nhả Q; hình học theo FOV/tỉ lệ màn hình; có luôn thấy ít
nhất một dấu Bắc/Nam hay không. Không dùng nguồn Legacy để điền các khoảng trống
này. Không điều khiển hoặc chụp màn hình PC trong lượt nghiên cứu này theo yêu
cầu của người dùng. Các ảnh/video được liên kết trên trang hướng dẫn không tải
được qua công cụ web; không coi chúng là dữ liệu hình ảnh đã kiểm tra.

## Các điểm tìm thấy trong hub

| Code | Hành vi đã xác nhận | Hệ quả |
| --- | --- | --- |
| `src-tauri/src/providers/era.rs`: `normalize`, `retry_delay_secs` | Đọc `viewYaw` rồi các tên cũ, polling thành công cách 12 giây | Không thể tạo thêm góc quan sát mới giữa hai lần nhận dữ liệu |
| `src-tauri/src/providers/titan.rs`: `preferred_live_heading` | Ưu tiên `cam`, nhưng có thể trả về `body_heading_deg` | Hướng thân và hướng nhìn không luôn là một |
| `src-tauri/crates/overlay-core/src/tracker.rs`: `heading_with_source` | Ưu tiên local, provider, rồi hướng dịch chuyển; local/provider cùng hạn 15 giây | Cần hạn riêng ngắn hơn cho ảnh và nhãn nguồn không gây hiểu nhầm |
| `src-tauri/src/pipeline.rs`: `ingest_local_heading` | Có hàm nhận góc riêng nhưng chưa có caller sản xuất; vẫn phát `position://update` và cần có vị trí | Chưa có luồng đọc camera offline thật, hướng vẫn bị gắn với sự tồn tại của tọa độ |
| `src/minimap/main.ts` và `pipeline.rs` | Vẽ/phát góc theo sự kiện; không có bộ phát sự kiện riêng khi góc hết hạn | Khi nguồn im lặng, tính toán expiry trong tracker không tự làm mũi tên trên UI biến mất |
| `src-tauri/src/fetch.rs` | Bỏ qua tải lại ảnh nền nếu đã có bản cache và không force | Có thể dùng bản đồ nền offline sau lần tải đầu; ảnh bản đồ không tự cung cấp góc nhìn |
| `src/minimap/render.ts`: `render` | Return trước `drawCompass` nếu chưa có vị trí | Có thể cải thiện hướng dẫn Bắc/Đông/Nam/Tây ngay cả khi chưa có tọa độ |

Không được mặc định mọi `yaw` fallback đều là camera. Cần bảo toàn nguồn góc
từ provider đến UI; `viewYawSource` của Era hiện chưa được xử lý. Cũng không
được coi thời điểm nhận HTTP là bằng chứng góc đã được game đo vào thời điểm đó.
Chưa có bằng chứng rằng server chặn kết nối là nguyên nhân duy nhất.

## Phương án được chọn để phát triển tiếp

Tách hai luồng: tọa độ giữ nhịp server; góc lấy từ phần la bàn đang hiển thị của
cửa sổ game. Nếu HUD phản ánh camera tại chỗ, luồng góc này không cần chờ API Era.
Đây là kết luận về kiến trúc, chưa phải kết quả chạy thực tế.

1. Người dùng bật tính năng đọc la bàn. Capture riêng cửa sổ The Isle qua
   [Windows Graphics Capture / CreateForWindow](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createforwindow).
   Xử lý trên worker, chỉ phân tích vùng thanh Q trong bộ nhớ. Không lưu hoặc
   gửi ảnh ra mạng mặc định; không lấy ảnh toàn desktop và không tự gửi phím Q.
   Dừng khi đổi cửa sổ, mất game, thu nhỏ hoặc tắt tính năng. Dùng API hệ điều
   hành không đồng nghĩa đã được nhà phát triển game/anti-cheat phê duyệt.
2. Tìm dải màu/đường nền của HUD; nhận đỉnh hướng lên/xuống bằng hình dạng và
   liên kết với đường la bàn. Loại bỏ icon mùi nằm gần đó. Phải kiểm chứng
   với trời sáng, đêm, nước, cây, icon chồng lên và độ mờ của HUD.
3. Quy đổi vị trí dấu thành góc bằng bảng hiệu chỉnh đã đo. Không hardcode
   toàn thanh = 360°, cũng không mặc định phép chiếu tuyến tính. FOV, HUD scale
   hoặc tỉ lệ màn hình thay đổi phải làm profile cũ mất hiệu lực.
4. Dấu Bắc có bearing 0°, Nam 180°. Nếu dấu nằm lệch tâm một góc `a`, hướng
   nhìn là `wrap(bearing_dấu - a)`. Đây đã là bearing Bắc=0, Đông=90;
   **không cộng thêm 90° như dữ liệu yaw từ provider**.
5. Kết quả từ các dấu cùng khung phải đồng thuận. Nếu chỉ còn đường sóng/
   điểm Đông–Tây chưa phân biệt được, trả về chưa xác định; không tự chọn một
   trong hai hướng cách nhau 180°. Có thể nghiên cứu theo dõi nhiều khung để
   tăng độ bao phủ, nhưng phải có kiểm chứng riêng về sai lệch.
6. Phát sự kiện `heading://update` riêng, mang `source`, `capturedAt`, trạng
   thái đọc và góc; không đổi tọa độ, timestamp vị trí, đường đi hay gây gọi
   `nearest_waypoint`/pan bản đồ ở mỗi frame. Khi hết hạn phải chủ động phát
   trạng thái mất nguồn. UI phân biệt `La bàn Q`, `Góc server`, `Hướng di
   chuyển ≈`; không giữ nguyên nhãn camera trên một góc đã cũ.
7. Giữ bản đồ Bắc ở trên; bổ sung chữ đầy đủ Bắc/Đông/Nam/Tây cho người mới.
   La bàn độc lập vẫn có thể hiển thị khi chưa biết tọa độ; không đặt chấm vị
   trí giả trên bản đồ. Chế độ đọc Q sẽ là tùy chọn vì HUD có thể không đọc được.

[Tài liệu capture của Microsoft](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture)
cung cấp frame với `SystemRelativeTime`. Cần dùng tuổi frame capture thật,
không đóng dấu thời gian mới lên ảnh cache. Hai ảnh giống nhau không chứng
minh capture bị treo: người chơi có thể đứng yên và giữ nguyên camera.

Mục tiêu thử nghiệm ban đầu: phân tích 20–30 frame/giây, bỏ nguồn khi frame
cũ quá 250 ms. Đây là mục tiêu thiết kế, chưa phải benchmark. Khi thanh Q
biến mất hoặc không nhận dạng chắc chắn, không thể hứa góc luôn đúng chỉ dựa
trên Q. Bản đồ offline và làm mượt animation không bổ sung thông tin bị thiếu.

## Bộ thử hiện có

`solver.mjs` chỉ nhận các dấu **đã được nhận dạng** và bảng hiệu chỉnh, không
nhận ảnh. Hỗ trợ nội suy bảng phi tuyến, vòng 359°/0°, đối chiếu Bắc/Nam,
loại dữ liệu cũ, khung đổi kích thước và dấu ngoài vùng đã đo. Ngưỡng điểm
nhận dạng là tham số nghiên cứu; không phải xác suất chính xác đã được hiệu chuẩn.

Chạy từ gốc repo:

```sh
node --test tools/compass-lab/solver.test.mjs
```

Kết quả 11/09/2026: **11/11 test pass**. Dữ liệu test hoàn toàn tổng hợp, bao
gồm bảng tuyến tính và phi tuyến. Không chứng minh đã đọc được HUD game thật.
Không nối module nghiên cứu vào app hoặc xuất release mới chỉ dựa vào các test này.

Trước khi đưa vào bản cài cần kiểm chứng: camera quay khi thân đứng yên, giữ
Alt, xoay đủ vòng qua Bắc, Q bật/tắt, chết–respawn, đổi kích thước/HUD/FOV,
camera không chuyển động, capture mất frame, reconnect Era và icon che dấu.
Phải đo sai số góc và độ trễ với mốc độc lập đáng tin; không dùng chính hướng
di chuyển cập nhật chậm làm ground truth cho camera.
