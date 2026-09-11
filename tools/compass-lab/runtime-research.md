# Góc camera độc lập với vị trí trong Isle Pulse Overlay

La bàn Q là ứng viên phù hợp để lấy hướng tại máy khi API vị trí cập nhật chậm.
Tuy nhiên, một bộ đọc đáng tin cần chứng minh ba việc riêng biệt: nhận đúng dấu
phương hướng, quy đổi đúng hình học của thanh sang góc, và xác định khung hình
vẫn còn mới. Việc vẽ mũi tên mượt hoặc chạy một bộ thử toán học không chứng
minh ba điều đó đã đạt được.

Phần code ứng dụng đã được sửa để nhận và làm hết hạn góc độc lập với tọa độ.
Phần nhận dạng hình ảnh Q chưa được tích hợp. Phiên bản phát hành vẫn là 2.1.1;
các thay đổi được ghi trong mục Unreleased để tránh mô tả chúng như một bản
camera offline đã hoàn tất.

## Chức năng Q và mức độ chắc chắn

Cấu hình game trên máy ánh xạ Q tới `Scent`, LeftAlt tới `FreeLook` và MouseX
tới `LookRight/Left`. Cấu hình xác nhận chức năng phím nhưng không xuất góc
camera. DevBlog #18 mô tả giữ Q để mở rộng vùng phát hiện dấu chân và bấm Q
để bật/tắt hoặc chọn dấu chân theo ngữ cảnh. Đây là mô tả lịch sử năm 2021;
không nên dùng nó để giả định mọi hành vi HUD của phiên bản hiện tại.[1]

DevBlog #22 xác nhận biểu tượng mùi được đưa lên la bàn. Các điểm Bắc/Nam và
biểu tượng mùi cùng xuất hiện trong một khu vực, nên bộ nhận dạng phải phân
biệt đường la bàn với icon thức ăn, nước hoặc các vùng đặc biệt. Hướng dẫn
HUD của ParaVixen là lời mô tả trực tiếp của người chơi về Q và hai dấu Bắc/Nam,
không phải đặc tả hình học của nhà phát triển game.[2][3]

Các nguồn công khai được đối chiếu chưa đủ để xác nhận cách thanh Q phản ứng
khi thân đứng yên nhưng camera xoay tự do với Alt. Cũng chưa có bảng ánh xạ
pixel–góc được kiểm chứng cho FOV, HUD scale và tỉ lệ màn hình hiện tại. Thiết
kế chỉ dựa vào giả định toàn thanh bằng 360° hoặc 180° có thể quay mượt nhưng
vẫn sai hướng. Vùng trời, cây hoặc icon cùng màu có thể làm thuật toán nhận
dấu nhầm ngay cả khi phép tính góc hoàn toàn đúng.

## Nguyên nhân trong luồng dữ liệu

Trong Era, ứng dụng đọc `viewYaw` rồi các tên trường cũ, với nhịp nghỉ 12 giây
sau một lần polling thành công. Nếu không có góc do provider gửi, tracker suy
hướng từ hai tọa độ. Camera xoay tại cùng một tọa độ không tạo ra một vector
dịch chuyển mới; không thể suy được góc camera mới từ dữ liệu đó.[4]

Titan có luồng SSE riêng. Hàm `preferred_live_heading` chọn góc camera nếu
có, giữ tạm trong một giây rồi có thể dùng góc thân. Do đó tên nội bộ
`provider-camera` không đảm bảo mọi giá trị fallback đều là camera. Giao diện
được đổi sang nhãn trung tính “Góc server”, trong khi hướng suy từ tọa độ mang
nhãn “Hướng di chuyển ≈”. Chưa thay đổi hợp đồng dữ liệu của provider.[4]

Một lỗi riêng đã được xác định ở vòng đời góc. Tracker biết góc nguồn đã quá
15 giây, nhưng kết quả này chỉ được tính khi có bên gọi. Giao diện vẽ theo
sự kiện, nên khi nguồn hoàn toàn im lặng, không có sự kiện buộc góc cũ bị gỡ.
Đây là lỗi phát trạng thái hết hạn, khác với vấn đề API chỉ gửi góc mỗi 12 giây.[4]

## Thay đổi đã thực hiện

| Thành phần | Hành vi mới | Giới hạn |
| --- | --- | --- |
| `pipeline.rs`, `events.rs` | `heading://update` và `get_current_heading` không chứa tọa độ | Chưa có bộ đọc Q gọi nguồn local |
| Watchdog Rust | Kiểm tra 100 ms/lần; chỉ phát khi góc/nguồn thay đổi, gồm cả chuyển sang chưa xác định | Không bảo đảm thời gian trình bày của WebView khi OS hoặc GPU gặp sự cố |
| Tracker | Nguồn local hết hạn sau 250 ms; góc provider giữ hạn 15 giây; có thể xóa riêng local | 250 ms là ngưỡng thiết kế, chưa phải đo độ trễ trong game |
| Thời gian góc | Kèm thời điểm quan sát trên đồng hồ đơn điệu | Đây là thứ tự trạng thái của hub, không phải timestamp đo góc của server |
| Frontend | Bỏ payload góc cũ hơn trạng thái đã nhận | Không làm dữ liệu góc server mới hơn thực tế |
| Minimap | Vẽ hướng độc lập; ghi đầy đủ Bắc/Đông/Nam/Tây cả khi thiếu tọa độ | Không đặt một tọa độ giả khi chưa biết vị trí |
| Full map | Sự kiện góc chỉ xoay hình mũi tên; không pan map hoặc tính lại waypoint; áp dụng góc mới khi tab hiện lại | Gói vị trí thực vẫn đi qua luồng vị trí |

Góc local phải mang thời điểm chụp trên cùng hệ thời gian với AppState. Hàm
nhận từ chối mẫu đã quá cũ hoặc ở tương lai; tracker không cho mẫu local cũ
ghi đè mẫu mới. Cách này tránh việc khung hình bị giữ trong bộ nhớ được gán
thời gian nhận mới rồi trông như dữ liệu realtime. Sau 250 ms, watchdog sẽ
phát kết quả fallback/mất hướng ở lần kiểm tra kế tiếp; trong điều kiện luồng
chạy bình thường, độ trễ phát sinh do tick tối đa xấp xỉ 100 ms.

Các thay đổi này sửa cơ chế phân phối và hết hạn góc. Chúng không tạo thêm
các góc camera mới giữa hai lần polling. Việc tiếp tục hiển thị một hướng
dịch chuyển có nhãn ước lượng vẫn đúng với nguồn dữ liệu hiện có.

## Phương án lấy khung hình

Windows Graphics Capture có thể lấy frame từ một cửa sổ ứng dụng.
`CreateForWindow` định danh trực tiếp cửa sổ qua HWND và yêu cầu Windows 10
1903 trở lên. Kiến trúc phù hợp là capture đúng cửa sổ game, phân tích phần
thanh Q, giữ pixel trong bộ nhớ và chỉ gửi kết quả hướng sang giao diện.[5]

Microsoft cung cấp timestamp QPC trong `SystemRelativeTime`. Worker cần chuyển
nó sang đồng hồ của ứng dụng; timestamp khi callback đến không thay thế được
timestamp của frame. Tài liệu cũng yêu cầu xử lý đổi kích thước và mất thiết
bị đồ họa, đồng thời lưu ý màu HDR có thể không phù hợp với pipeline SDR.[6]

Thư viện Rust `windows-capture` 2.0.1 là một lựa chọn triển khai cần đánh giá.
API có callback frame, đọc timestamp và cắt buffer trước khi phân tích.
Giới hạn tần suất cập nhật là khoảng cách tối thiểu giữa các frame đủ điều
kiện, không phải lời hứa có frame mới đều đặn. Không dùng việc hai ảnh giống
nhau để kết luận bị treo: camera có thể đang đứng yên.[7]

So với capture, tích phân delta chuột không cung cấp hướng tuyệt đối khi
chưa hiệu chỉnh. Nhìn tự do, menu, sensitivity, mất input hoặc thay đổi nhân
vật khiến hướng tích phân cần được neo lại. Vì mục tiêu là đúng hướng, không
chọn phương án delta chuột làm nguồn camera chính ở giai đoạn này. Đây là
đánh giá thiết kế; không phải tuyên bố rằng một thuật toán thị giác đã được
kiểm chứng tốt hơn trên máy thật.

## Nhận dạng và hiệu chỉnh

Bộ đọc dự kiến cần xác định vùng HUD trước, rồi tìm đường nền và hình dạng
đỉnh. Một dấu nhọn chỉ đáng tin khi có liên hệ hình học với thanh la bàn;
độ giống màu đơn thuần không đủ. Nếu nhiều dấu cùng frame suy ra các góc mâu
thuẫn, frame phải bị loại thay vì lấy trung bình để che sai số.

Khi dấu Bắc có bearing 0° và Nam 180°, góc nhìn được tính từ bearing của dấu
trừ góc lệch của dấu so với tâm. Bộ thử đã hỗ trợ bảng hiệu chỉnh phi tuyến,
không ngoại suy ngoài vùng được đo, và xử lý vòng 359°/0°. Giá trị kết quả là
bearing theo Bắc=0, Đông=90, nên không được cộng 90° thêm như dữ liệu yaw API.[4]

Một clip liên tục thể hiện thanh Q khi xoay camera đủ vòng là dữ liệu tối
thiểu để khảo sát chuyển động của các dấu. Cần thêm đoạn nhìn tự do bằng Alt
trong khi thân đứng yên để xác định chính xác đại lượng đang theo dõi. Clip
giúp xây giả thuyết hình học; đo sai số tuyệt đối vẫn cần các mốc hướng độc
lập đáng tin. Một ảnh tĩnh chỉ có đường sóng không đủ xác định tỉ lệ pixel–góc.

Chỉ tích hợp nguồn Q vào bản cài sau khi có dữ liệu cho các trường hợp:
quay qua Bắc, đứng yên, chuyển thân/camera, Q hiện/ẩn, icon chồng lên, respawn,
thay đổi HUD/FOV, game mất focus, frame cũ và reconnect provider. Khi không
đọc được HUD, app phải hiển thị nguồn fallback hoặc chưa xác định. Không có
cơ sở hứa luôn chính xác khi nguồn hình ảnh không còn quan sát được.

## Bản đồ offline

Ảnh nền đã được lưu cache và được dùng lại nếu tồn tại. Do đó không cần một
dịch vụ bản đồ mới để giải quyết góc quay. Bản đồ có thể giữ Bắc cố định ở trên
và dùng tên hướng đầy đủ để người mới dễ đọc; tọa độ mới vẫn cần nguồn riêng.
Bản đồ offline cung cấp hệ quy chiếu, không đo được hướng nhìn hiện tại.[4]

## Kiểm chứng

Các kiểm tra tại thời điểm viết: 17 test JavaScript cho thứ tự sự kiện, hết
hạn và phép tính la bàn; 53 test Rust phần lõi; kiểm tra Svelte không có lỗi
hay cảnh báo; build frontend thành công với cảnh báo kích thước chunk Three.js
có từ trước. Build toàn bộ workspace trên máy dừng vì Windows Application
Control chặn build helper của `httparse` (4551), không phải do lỗi biên dịch
được báo trong phần sửa. Workflow `Check Windows app` thực hiện kiểm tra đầy
đủ trên runner Windows và là nơi xác nhận biên dịch tích hợp.[4]

Các test la bàn dùng dữ liệu tổng hợp. Chưa đo độ chính xác nhận dạng ảnh,
độ trễ từ camera đến overlay hoặc hiệu năng capture trong game. Chưa phát
hành bộ cài mới mang tính năng đọc Q.

## Nguồn

1. Afterthought LLC, lập trình viên dmIV. [DevBlog #18](https://store.steampowered.com/news/posts/?appgroupname=The+Isle&appids=376210&enddate=1646104136&feed=steam_community_announcements), 29/10/2021. Hành vi Q và theo dấu.
2. Afterthought LLC, dmIV. [DevBlog #22](https://store.steampowered.com/news/app/376210/view/3128317225041424482), 28/02/2022. Biểu tượng scent trên la bàn.
3. ParaVixen. [The Isle Evrima: Ultimate Survival Guide, HUD Elements](https://steamcommunity.com/sharedfiles/filedetails/?id=3440690332), đăng 09/03/2025, truy cập 11/09/2026. Mô tả gameplay của người chơi; không phải đặc tả kỹ thuật.
4. [Mã nguồn hubisle](https://github.com/thienvyma/hubisle), nền 2.1.1 `e23cd3b` và thay đổi Unreleased: `tracker.rs`, `pipeline.rs`, `events.rs`, `era.rs`, `titan.rs`, `fetch.rs`, `heading.ts`, minimap và FullMap. Kết quả test tại workspace ngày 11/09/2026.
5. Microsoft. [IGraphicsCaptureItemInterop::CreateForWindow](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createforwindow), truy cập 11/09/2026.
6. Microsoft. [Screen capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture), cập nhật 23/08/2026. Timestamp, HDR và vòng đời frame.
7. NiiightmareXD. [windows-capture README và source](https://github.com/NiiightmareXD/windows-capture), phiên bản được README công bố 2.0.1, truy cập 11/09/2026. Callback, timestamp, buffer và giới hạn nhịp cập nhật.
