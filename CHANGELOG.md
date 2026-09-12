# Changelog

## [Unreleased]

## [2.1.4] — 2026-09-12

- Kiểm tra Npcap bằng thiết bị capture thật khi hub mở, phát hiện cả trường
  hợp DLL còn nhưng driver bị dừng; bộ cài Npcap đặt chế độ nạp cùng Windows
  và cho phép hub chạy bằng tài khoản thường.
- Lịch sử giảm máu hoạt động chung với Era, Titan và IslePilot, đồng thời vẫn
  chỉ hiện danh tính đối phương khi server gửi sự kiện xác thực.
- Chuẩn hóa đăng nhập IslePilot token về API trung tâm để theo người chơi qua
  mọi server; Garage báo rõ online/Dino hiện tại, tab Dino báo số bạn bè và
  Skin giải thích khi server tắt Live Skin.
- Thêm nút kiểm tra cập nhật cố định trong Cài đặt, ghi lỗi updater vào log và
  tự kiểm tra lại mỗi 5 phút.
- Popup Prime trên minimap hoạt động chung cho Era, Titan và IslePilot: báo từng
  điều kiện vừa xong, báo hoàn tất toàn bộ và không phát lại tiến độ cũ sau khi
  mở app hoặc kết nối lại.
- Bộ cài dừng cả hub lẫn sidecar, xóa các executable cũ rồi bắt buộc ghi đè;
  nếu file còn bị khóa thì dừng cài đặt thay vì báo thành công nhưng vẫn chạy
  phiên bản cũ.

## [2.1.3] — 2026-09-12

- Tích hợp sidecar telemetry riêng của Isle Pulse, đọc gói UDP chiều đi của
  tiến trình game qua Npcap để lấy vị trí và góc camera realtime trên mọi
  server; không cần cài hoặc chạy IsleLiveMap.
- Ưu tiên góc camera cục bộ với hạn dùng 250 ms, tự mất hiệu lực và khởi động
  lại sidecar khi nguồn capture gián đoạn, tránh giữ mũi tên ở góc cũ.
- Đóng gói sidecar .NET tự chứa vào installer Windows và bổ sung pipeline CI
  để mọi bản phát hành đều kiểm tra, ký và phân phối thành phần này cùng app.

## [2.1.2] — 2026-09-11

- Tách sự kiện góc quay khỏi tọa độ; góc cục bộ không còn tạo sự kiện vị trí hoặc gọi tính waypoint theo mỗi frame.
- Chủ động phát trạng thái hết hạn góc từ luồng Rust kể cả khi server im lặng; ngăn phản hồi cũ khôi phục góc đã hết hạn.
- Phân biệt góc server với hướng di chuyển ước lượng; hiển thị đầy đủ Bắc/Đông/Nam/Tây trên minimap, kể cả khi chưa có vị trí.
- Thêm kiểm thử luồng góc và pipeline CI Windows. Bộ đọc hình ảnh la bàn Q vẫn chưa tích hợp; chưa có camera offline realtime trong bản này.
- Nối dữ liệu IslePilot token-mode vào provider snapshot: vị trí, hướng server và danh sách bạn bè được publish cho full map/minimap thay vì chỉ nằm trong tab Dino.
- Bật skin IslePilot bằng endpoint overlay mới `/api/overlay/skin`; tab Skin đọc danh sách vùng màu động từ server DinoVietnam và chỉ gửi lệnh apply khi người dùng bấm áp dụng.
- Dò endpoint DinoVietnam VIP: `/api/overlay/friends`, `/api/overlay/garage`, `/api/overlay/skin` đều trả dữ liệu hợp lệ. Bạn bè chỉ hiện trên bản đồ khi server trả kèm tọa độ; mẫu hiện tại chỉ có quan hệ bạn bè, chưa có vị trí.

Mọi thay đổi đáng chú ý của Isle Pulse Overlay được ghi tại đây, theo định dạng
[Keep a Changelog](https://keepachangelog.com/vi/1.1.0/) và đánh số phiên bản
[SemVer](https://semver.org/lang/vi/). Mã trong ngoặc là commit tương ứng.

## [2.1.1] — 2026-09-11

- Đọc trường góc nhìn `viewYaw` mới của Era; ưu tiên góc nhìn này khi có giá trị hợp lệ. Khi server không gửi góc nhìn, hub vẫn dùng hướng suy từ di chuyển (ký hiệu ≈).
- Tăng thời gian chờ phản hồi Era, tái sử dụng kết nối HTTP và điều chỉnh nhịp tải theo trang bản đồ chính thức. Khi server lỗi hoặc giới hạn yêu cầu, hub giãn thời gian thử lại.
- Giữ dữ liệu gần nhất trong gián đoạn ngắn, kèm cảnh báo dữ liệu cũ trên bản đồ, minimap và tab Dino. Lỗi kéo dài, đăng xuất hoặc nhân vật offline vẫn xóa dữ liệu đang theo dõi.
- Hiển thị rõ trạng thái Prime chưa được Era cung cấp thay vì làm biến mất mục Prime. Phản hồi được kiểm tra đang trả `prime: null`; hub không biến dữ liệu thiếu thành tiến độ 0/10.
- Đồng bộ nội dung thông báo cập nhật trong app với ghi chú của đúng phiên bản trên GitHub.

Góc nhìn Era vẫn phụ thuộc nhịp dữ liệu do server cung cấp, không phải luồng camera từng khung hình. Bản này chưa được xác nhận trực tiếp trong game khi respawn/Alt-Tab.

## [2.1.0] — 2026-09-10

- Bổ sung làm mới minimap từ luồng Windows khi overlay đang hiển thị, khôi
  phục controller bị treo và tái xác lập vị trí nổi trên game.
- Titan tự nối lại khi stream im lặng quá 5 giây; xóa góc camera cũ khi nối
  lại và chờ nhân vật mới sau khi offline. Góc camera thiếu quá 1 giây dùng
  hướng fallback thay vì giữ vĩnh viễn hướng của nhân vật trước.
- Reset hướng suy từ di chuyển khi respawn/teleport để không nối hai đời.
- Thêm lịch sử chiến đấu Era và thông báo sự kiện trên minimap. Mất máu chỉ
  được ghi là biến động máu nếu server không xác nhận người tấn công.

- Hoàn thiện kênh cập nhật chính thức thienvyma/hubisle: kiểm tra thủ công ở
  chân cửa sổ, kiểm tra mỗi giờ và khi máy có mạng trở lại.
- Thông báo phân biệt lỗi kiểm tra, chưa có bản mới và lỗi cài đặt; hiển thị
  nội dung phát hành, tiến trình tải và nút mở GitHub.
- Nút Để sau ẩn cùng phiên bản trong phiên chạy; bản mới hơn vẫn được báo.
- Workflow kiểm tra cấu hình, kiểm thử cập nhật và xác nhận latest.json của
  bản nháp trước khi công bố release.

## [2.0.2] — 2026-09-10

- Ẩn cửa sổ terminal trong cả bộ cài thử và bản phát hành; log chẩn đoán vẫn
  được ghi vào thư mục dữ liệu ứng dụng.
- Thêm kiểm tra PE subsystem để ngăn bộ cài Windows vô tình trở lại chế độ console.

## [2.0.1] — 2026-09-09

- Tách WebView2 của minimap khỏi bảng điều khiển để sửa lỗi overlay không render trong game.
- Tự hủy cửa sổ minimap lỗi và tạo lại bằng nhãn dự phòng mà không làm ứng dụng chính thoát.
- Gửi đủ ngữ cảnh same-origin cho thao tác Garage Era, gồm cả slot 4 và 5.

## [2.0.0] — 2026-09-09

### Thêm

- Tự kiểm tra bản phát hành mới từ `thienvyma/hubisle`, hiển thị tiến trình tải
  và cài đặt gói đã xác minh chữ ký ngay trong ứng dụng.
- Thêm workflow Windows để GitHub Release tự tạo installer, chữ ký và
  `latest.json` cho kênh cập nhật.
- Bấm chuột trái trên bản đồ lớn để đặt hoặc thay một cờ đích `🚩`; minimap
  hiển thị cờ khi ở gần và chỉ hướng cùng khoảng cách khi cờ nằm ngoài khung.
- Phím `` ` `` mở chat, nhập `/unstuck` và tự gửi khi The Isle đang ở
  foreground. Chuỗi cố định dừng ngay nếu người chơi chuyển sang cửa sổ khác.
- Hiển thị bạn bè đã chấp nhận và đang online từ Era hoặc Titan trên bản đồ
  lớn và minimap; bạn ở xa được ghim bằng số ở mép vòng để chỉ hướng.
- Đưa Dino Garage vào hub chung: giữ Garage 3D của IslePilot, đồng thời thêm
  cất/lấy/xóa theo slot bằng API chính thức của Era và Titan, gồm trạng thái
  cầu nối, khóa slot, tiến trình bất đồng bộ và hủy đếm ngược Titan.
- Thêm tab Đổi skin cho Era và Titan với 7 vùng màu, bảng màu theo quyền
  Free/VIP của Era, preset, biến thể Titan và đồng hồ cooldown của server.

### Sửa

- Chuyển toàn bộ metadata, liên kết GitHub và thông tin phát hành sang Huỳnh Vỹ.
- Cờ đích được ưu tiên cho bộ tính hướng; waypoint thường vẫn được lưu và hiện
  trên cả bản đồ lớn lẫn minimap.
- Sửa thứ tự trục tọa độ Titan theo đúng phép chiếu của live map chính thức,
  tránh đánh dấu sai vị trí trên cả hai bản đồ.
- Mũi tên camera vẽ ngay theo từng gói realtime và cửa sổ minimap tắt cơ chế
  WebView2 occlusion/throttling, không còn chờ Alt-Tab mới cập nhật hình.
- Kiểm tra chặt slot Garage, state hash xóa của Era và đúng 7 mã màu trước khi
  gửi; mọi request chỉ đi đến endpoint cố định trên domain provider đã xác minh.
