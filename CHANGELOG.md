# Changelog

## [Unreleased]

## [2.4.9] — 2026-09-14

- Bổ sung các skin mẫu xanh đậm gân đen, đen, trắng, rừng, sa mạc, bóng tối,
  tuyết, đại dương và dung nham trên cả IslePilot, Titan và Era.
- Thêm nút Ngẫu nhiên để tạo bảng màu hài hòa mới; Titan random cả biến thể
  hoa văn, còn các vùng màu bị server khóa luôn được giữ nguyên.
- Mỗi skin mẫu có dải màu xem nhanh và có thể lưu lại bằng thư viện skin như
  bảng màu tự chọn.

## [2.4.8] — 2026-09-14

- Sửa màu skin trong mô hình 3D bị tối sai do bảng màu từng bị giảm còn 55%;
  giữ nguyên giá trị sRGB đã chọn, cache normal map theo loài và giảm khối lượng
  compositing để đổi màu/tải preview nhanh hơn.
- Chuẩn hóa màu Live Skin IslePilot trước khi gửi và dùng `#000001` cho màu đen
  trực quan, tránh kênh float bằng 0 bị game bridge hiểu như giá trị chưa đặt.
- Minimap hiển thị tên ingame cạnh từng dấu bạn bè, tự đặt nhãn về phía tâm để
  tên của bạn ở ngoài bán kính vẫn không bị mất tại rìa bản đồ.
- Cài chồng không xóa bản đang hoạt động trước khi xác nhận executable và
  sidecar mới đã được ghi đầy đủ.
- Hoàn tất metadata `thienvyma`, giấy phép GPL-3.0-only, chính sách quyền riêng
  tư/ký mã; CI từ chối phát hành nếu installer, app hoặc sidecar thiếu chữ ký
  Authenticode hợp lệ có dấu thời gian.
- Tắt số liệu sử dụng theo mặc định và bỏ dịch Prime trực tuyến; chuỗi Prime
  chưa có trong từ điển cục bộ giữ nguyên tiếng Anh.
- Phát hành chuyển tiếp vẫn có chữ ký cập nhật Tauri để app cũ xác minh gói;
  Authenticode đáng tin cậy sẽ được bật sau khi SignPath duyệt đơn mã nguồn mở.

## [2.4.7] — 2026-09-13

- Sửa lỗi đổi kết nối giữa các server IslePilot nhưng hub vẫn giữ server cũ:
  địa chỉ panel đã chọn như DinoVietnam hoặc Titan nay được giữ qua lần khởi
  động tiếp theo thay vì bị thay bằng địa chỉ IslePilot trung tâm.
- Xóa ngay snapshot, vị trí, bạn bè và chỉ số của server trước khỏi mọi cửa sổ
  khi đổi kết nối. Dữ liệu IslePilot có tên server không khớp với panel đã chọn
  sẽ bị chặn và hub hiện trạng thái chờ server đúng xác nhận.
- Nút đổi kết nối nay giữ phiên Steam IslePilot đã mã hóa trên máy, vì vậy có
  thể chuyển từ Titan sang DinoVietnam mà không phải đăng nhập Steam lại khi
  token hiện tại còn hợp lệ.

## [2.4.6] — 2026-09-13

- Bổ sung nút tự sát Dino đang chơi cho Titan và DinoVietnam qua đúng endpoint
  `/api/overlay/garage/slay`, chỉ bật khi IslePilot xác nhận server cho phép và
  người chơi đang online với một Dino hoạt động.
- Luôn hiển thị thao tác xóa/bán Dino đã cất. Hub gọi đúng API `/sell` của
  IslePilot khi server bật `sellingEnabled`; nếu server tắt quyền này, nút được
  khóa kèm giải thích thay vì biến mất.
- Giảm relay vị trí bạn bè từ 20 xuống một lần mỗi 90 giây và giới hạn Worker
  còn hai request/phút/token. Với 500 người dùng trung bình bốn giờ/ngày, phần
  relay dùng khoảng 80.000 request và dòng ghi D1/ngày, nằm trong free tier.

## [2.4.5] — 2026-09-13

- Đổi website gợi ý mặc định trên màn hình kết nối của bản cài mới từ EraGaming
  sang IslePilot trung tâm (`https://islepilot.eu`), đồng thời giữ nguyên server
  đã lưu của người dùng hiện tại.
- Bỏ từ “Gacha” khỏi tiêu đề Garage và tài liệu giao diện liên quan.
- Bổ sung tương thích `titan.islepilot.eu`: dùng tọa độ cá nhân từ `/me` khi
  server tắt Live Map, mở đúng trang tìm bạn theo tên của Titan, và không thay
  đổi đường tọa độ đã hiệu chỉnh đang dùng trên DinoVietnam.
- Hiển thị SteamID64 của tài khoản đã đăng nhập ở tab Bạn bè cùng nút sao chép.
- Bổ sung relay vị trí ngắn hạn giữa những người cùng dùng hub: IslePilot xác
  thực quan hệ đã chấp nhận, trạng thái chia sẻ và cùng server trước khi bạn bè
  được ghép lên bản đồ/minimap. Backend không lưu token, SteamID thô hay tên.

## [2.4.4] — 2026-09-13

- Sửa thao tác cất Dino trên IslePilot theo đúng quy trình server: nhận thời
  gian chờ từ `start`, đếm ngược, gửi `finalize`, rồi theo dõi trạng thái lệnh.
- Hiện số giây còn lại, nhắc đứng yên/không nhận sát thương và cho phép hủy
  quá trình cất ngay trong tab Garage.
- Tự thử lại bước xác nhận và các lần đọc trạng thái bị rớt kết nối tạm thời;
  tạm dừng request IslePilot nền khi Garage đang xử lý để tránh tranh kết nối.
- Nhận `commandId`, `command_id` hoặc `id` ở cả dạng chuỗi và số để tương thích
  các phiên bản backend IslePilot khác nhau.

## [2.4.3] — 2026-09-13

- Xóa tab và backend mở trình cài Việt hóa The Isle khỏi hub.
- Đổi macro phím nhanh sang mở chat và gửi đúng lệnh `!unstuck` của server.
- Bổ sung thư viện skin có tên, cho phép lưu, nạp và xóa tối đa 20 bảng màu
  riêng trên từng provider; vẫn giữ bản nháp hiện tại như trước.
- Nhận cấu trúc `kills` gốc của killfeed DinoVietnam/IslePilot và giữ các mảng
  combat do `/api/overlay/me` trả về để lịch sử hiện tên người giết hoặc tấn
  công khi server cung cấp danh tính đã xác thực.
- Ghi rõ Voice phải dùng hub chính thức của server; islemap-thienvyma chỉ kiểm
  tra trạng thái và mở hub DinoVietnam đã cài trên máy.

## [2.4.2] — 2026-09-13

- Ô thêm bạn IslePilot nhận cả tên trong game và SteamID64. Khi nhập tên, hub
  mở tìm kiếm chính thức của đúng server đang chơi, tự điền truy vấn và để
  người dùng chọn đúng tài khoản nếu có tên trùng.
- Giữ API thêm bạn bằng SteamID64 cho thao tác trực tiếp; không đoán SteamID
  từ tên hoặc gửi lời mời khi người dùng chưa chọn kết quả.

## [2.4.1] — 2026-09-13

- Bổ sung đầy đủ quản lý bạn bè IslePilot trong hub: thêm SteamID64, nhận/từ
  chối lời mời, hủy, xóa và bật/tắt chia sẻ vị trí theo API trung tâm.
- Thêm tab Voice DinoVietnam để tự nhận client chính thức, trạng thái đăng nhập
  Steam và tiến trình đang chạy; một nút mở đúng hệ thống Voice gốc hoặc luồng
  đăng nhập của DinoVietnam.
- Không sao chép token mã hóa hoặc khóa ứng dụng riêng của DinoVietnam. Quyền
  LiveKit, micro và phím nhấn để nói tiếp tục được client chính thức quản lý.

## [2.4.0] — 2026-09-13

- Hoàn tất nghiên cứu khả năng tương thích Voice DinoVietnam và không đóng gói
  tích hợp client nền: API Voice cần quyền ứng dụng do máy chủ cấp ngoài phiên
  Steam/IslePilot của người chơi, nên Hub độc lập chưa thể xác thực hợp lệ.
- Thêm tab Việt hóa để mở trình cài The Isle chính thức đã có trên máy; liên
  kết đúng bản phát hành DinoVietnam khi chưa cài mà không nhúng hoặc giải mã
  lại gói tài sản của bên phát hành.
- Tự phục hồi capture cục bộ khi The Isle respawn hoặc luồng UDP/decoder im
  lặng, giảm lỗi mũi tên minimap đứng cho đến khi Alt-Tab.
- Chuẩn hóa lịch sử giao tranh từ killfeed server: chỉ lưu trận đánh có danh
  tính khớp người chơi đang đăng nhập và chỉ hiện tên hiển thị cùng loài của
  người đã tấn công.
- Bổ sung tài liệu kỹ thuật và ranh giới xác thực cho LiveKit Voice, gồm bằng
  chứng từ binary/WebView2 cache và kết quả đánh giá các phương án tích hợp.

## [2.2.0] — 2026-09-12

- Đổi toàn bộ tên sản phẩm, executable, sidecar và gói phát hành thành
  `islemap-thienvyma`; vẫn nhận deep link `theisle-overlay://` cũ để các luồng
  đăng nhập hiện có tiếp tục hoạt động.
- Tự chuyển một lần dữ liệu roaming từ `TheIsleOverlay` sang
  `islemap-thienvyma` và dữ liệu local sang `islemap-thienvyma-data`, giữ
  nguyên cài đặt, waypoint, lịch sử, bản đồ đã tải và thông tin đăng nhập đã mã hóa.
- Thêm tab Bạn bè cho danh sách được Era, Titan hoặc IslePilot trả về, gồm trạng
  thái online, loài và việc có vị trí trực tiếp hay không.
- Thêm tab Thoại để dò, mở và tùy chọn tự khởi động launcher IsleVOIP chính
  thức; liên kết trang tải chính thức khi chưa cài và giải thích rõ gói tính
  năng do chủ server quản lý.
- Bộ cài dừng và dọn cả executable cũ/mới, đồng thời xóa thư mục chương trình
  `Isle Pulse Overlay` cũ mà không xóa dữ liệu người dùng.
- Chuẩn hóa tài liệu phát hành 2.2.0 thành bản chỉ có văn bản và xóa ảnh hướng
  dẫn, ảnh chụp không còn được tham chiếu.

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

Mọi thay đổi đáng chú ý của islemap-thienvyma được ghi tại đây, theo định dạng
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

- Chuyển toàn bộ metadata, liên kết GitHub và thông tin phát hành sang thienvyma.
- Cờ đích được ưu tiên cho bộ tính hướng; waypoint thường vẫn được lưu và hiện
  trên cả bản đồ lớn lẫn minimap.
- Sửa thứ tự trục tọa độ Titan theo đúng phép chiếu của live map chính thức,
  tránh đánh dấu sai vị trí trên cả hai bản đồ.
- Mũi tên camera vẽ ngay theo từng gói realtime và cửa sổ minimap tắt cơ chế
  WebView2 occlusion/throttling, không còn chờ Alt-Tab mới cập nhật hình.
- Kiểm tra chặt slot Garage, state hash xóa của Era và đúng 7 mã màu trước khi
  gửi; mọi request chỉ đi đến endpoint cố định trên domain provider đã xác minh.
