// Toàn bộ chuỗi hiển thị tiếng Việt. Port từ strings_vi.py của bản gốc,
// thêm các khóa mới cho tab, danh sách waypoint, cài đặt và hướng dẫn.
// Không file UI nào được viết thẳng chuỗi hiển thị.

export const vi = {
  // --- chung ---
  "app.title": "islemap-thienvyma",
  "app.minimap_title": "Bản đồ nhỏ",
  "app.fullmap_title": "Bản đồ Gateway",

  // --- tab ---
  "tab.map": "Bản đồ",
  "tab.dino": "Khủng long",
  "tab.settings": "Cài đặt",
  "tab.garage": "Garage",
  "tab.skin": "Đổi skin",
  "tab.history": "Lịch sử",
  "tab.friends": "Bạn bè",
  "tab.voice": "Thoại",

  // --- lịch sử giao tranh Era ---
  "history.title": "Lịch sử giao tranh",
  "history.subtitle":
    "Theo dõi các lần mất máu trên mọi provider đang kết nối. Sự kiện xác thực do server cấp cũng được lưu; ba sự kiện mới nhất xuất hiện tạm thời trên minimap.",
  "history.live": "THEO DÕI TRỰC TIẾP",
  "history.identity_title": "Dữ liệu danh tính",
  "history.identity_hint":
    "Hub chỉ điền tên và loài đối phương khi server gửi dữ liệu xác thực. Sự kiện suy ra từ giảm máu trên Era, Titan hoặc IslePilot luôn ghi Không xác định.",
  "history.filter": "Lọc lịch sử",
  "history.filter_all": "Tất cả",
  "history.filter_incoming": "Nhận sát thương",
  "history.filter_outgoing": "Đã tấn công",
  "history.filter_death": "Tử vong",
  "history.events": "sự kiện",
  "history.loading": "Đang đọc lịch sử…",
  "history.error": "Không thể đọc lịch sử giao tranh.",
  "history.empty": "Chưa có sự kiện giao tranh",
  "history.empty_hint": "Giữ hub chạy và kết nối provider trong lúc chơi để ghi nhận các thay đổi máu.",
  "history.health_drop": "Phát hiện giảm máu",
  "history.incoming": "Bị người chơi tấn công",
  "history.outgoing": "Bạn đã tấn công",
  "history.death": "Khủng long đã chết",
  "history.player": "Người chơi",
  "history.species": "Loài đối phương",
  "history.your_species": "Loài của bạn",
  "history.damage": "Sát thương",
  "history.unknown": "Không xác định",
  "history.estimated": "Suy ra từ máu",
  "history.verified": "Era xác nhận",

  "friends.provider_title": "Bạn bè do server cung cấp",
  "friends.provider_empty":
    "Provider hiện trả về 0 bạn bè đã chấp nhận. Hub sẽ hiển thị ngay khi server gửi danh sách.",
  "friends.provider_no_position":
    "Đã nhận danh sách bạn bè nhưng chưa ai online và chia sẻ vị trí.",
  "friends.provider_visible": "Đang hiển thị {count} bạn bè có vị trí trên bản đồ.",
  "friends.title": "Bạn bè",
  "friends.subtitle": "Danh sách bạn bè được server hiện tại xác nhận.",
  "friends.connect": "Kết nối một server để xem danh sách bạn bè mà server cung cấp.",
  "friends.loading": "Đang đọc danh sách bạn bè…",
  "friends.waiting": "Server chưa gửi ảnh chụp danh sách bạn bè.",
  "friends.error": "Không thể đọc danh sách bạn bè lúc này.",
  "friends.dino_unknown": "Chưa rõ loài",
  "friends.online": "Online",
  "friends.offline": "Offline",
  "friends.position_available": "Có vị trí trực tiếp",
  "friends.position_unavailable": "Không có vị trí",
  "friends.positioned": "{count}/{total} có vị trí",
  "friends.updated": "Cập nhật lúc {time}",

  "voice.title": "IsleVOIP",
  "voice.subtitle": "Trình khởi chạy thoại chính thức cho các server được hỗ trợ.",
  "voice.checking": "Đang kiểm tra IsleVOIP…",
  "voice.installed": "Đã cài IsleVOIP",
  "voice.running": "Trình khởi chạy đang chạy.",
  "voice.not_running": "Trình khởi chạy chưa chạy.",
  "voice.start": "Mở IsleVOIP",
  "voice.starting": "Đang mở…",
  "voice.auto_start": "Tự mở IsleVOIP khi mở islemap-thienvyma",
  "voice.auto_start_hint": "Tùy chọn này được lưu trên máy này.",
  "voice.not_installed": "Chưa cài IsleVOIP",
  "voice.install_hint": "Cài trình khởi chạy chính thức để đăng nhập Steam và tham gia thoại trên server hỗ trợ.",
  "voice.install": "Mở trang cài đặt",
  "voice.refresh": "Kiểm tra lại",
  "voice.help": "Trang chính thức",
  "voice.status_error": "Không thể kiểm tra trạng thái IsleVOIP.",
  "voice.launch_error": "Không thể mở IsleVOIP. Hãy kiểm tra trình khởi chạy rồi thử lại.",
  "voice.setting_error": "Không thể lưu tùy chọn tự mở.",
  "voice.link_error": "Không thể mở trình duyệt. Hãy vào isle-voip.com.",
  "voice.pro_title": "Về IsleVOIP Pro",
  "voice.pro_body": "Người chơi không cần mua Pro. Chủ server chọn gói IsleVOIP; thoại cơ bản có trên server miễn phí, còn thoại theo khoảng cách/3D chỉ có khi server bật gói tương ứng.",

  // --- trạng thái vị trí ---
  "pos.none": "Chưa có vị trí",
  "pos.hint":
    "Đang chờ vị trí realtime từ server. Chế độ thủ công vẫn có thể dùng Asset Location trong bảng Tab.",
  "pos.off_map": "Ngoài bản đồ",

  // --- hướng ---
  "dir.N": "Bắc",
  "dir.NE": "Đông Bắc",
  "dir.E": "Đông",
  "dir.SE": "Đông Nam",
  "dir.S": "Nam",
  "dir.SW": "Tây Nam",
  "dir.W": "Tây",
  "dir.NW": "Tây Bắc",
  "heading.unknown": "Chưa rõ hướng",
  "heading.hint": "Chép tọa độ lần nữa sau khi di chuyển để biết hướng đi.",

  // --- layer POI ---
  "layer.freshwater": "Nước ngọt",
  "layer.water": "Nguồn nước",
  "layer.sanctuary": "Khu bảo tồn",
  "layer.migration": "Vùng di cư",
  "layer.saltlick": "Mỏ muối",
  "layer.mudwallow": "Vũng bùn",
  "layer.food": "Khu vực thức ăn",
  "layer.patrol": "Vùng tuần tra AI",
  "layer.region": "Tên vùng",
  "layer.landmark": "Địa điểm",
  "layer.animal": "Động vật",
  "layers.title": "Lớp bản đồ",
  "layers.zone_labels": "Tên vùng khoanh",
  "layers.collapse": "Thu gọn",
  "layers.expand": "Mở rộng",

  // --- waypoint ---
  "wp.title": "Điểm đánh dấu",
  "wp.new": "Điểm đánh dấu mới",
  "wp.add": "Thêm điểm",
  "wp.remove": "Xóa điểm",
  "wp.rename": "Đổi tên",
  "wp.name_prompt": "Tên điểm đánh dấu:",
  "wp.destination": "Điểm đến",
  "wp.destination_hint":
    "Chuột trái: đặt hoặc thay cờ đích. Chuột phải: thêm điểm đánh dấu thường.",
  "wp.empty": "Chưa có điểm nào. Bấm chuột trái để đặt cờ đích.",
  "wp.distance": "{dir} · {dist}",
  "wp.here": "Vị trí của tôi",
  "wp.confirm_delete": "Xóa điểm “{name}”?",
  "wp.color": "Đổi màu",

  // --- tìm kiếm & điều hướng ---
  "search.placeholder": "Tìm địa danh hoặc dán tọa độ…",
  "search.goto_coords": "Tới tọa độ đã nhập",
  "search.no_results": "Không thấy địa danh nào",
  "search.coords_failed": "Không đọc được tọa độ — kiểm tra lại chuỗi đã dán",
  "map.recenter": "Về vị trí của tôi",

  // --- vết đường ---
  "trail.title": "Đường đã đi",
  "trail.previous": "Đường đi phiên trước",
  "trail.clear": "Xóa đường đi",
  "trail.clear_hint":
    "Xóa vết trên cả hai bản đồ cho đỡ rối; file lịch sử trên máy vẫn giữ nguyên.",

  // --- nút chung ---
  "btn.close": "Đóng",
  "btn.ok": "Đồng ý",
  "btn.cancel": "Hủy",
  "btn.save": "Lưu",

  // --- cảnh báo ---
  "warn.exclusive_fullscreen":
    "Game đang chạy chế độ Toàn màn hình. Bản đồ nhỏ sẽ không hiện đè lên được. " +
    "Hãy vào Cài đặt › Hình ảnh trong game và đổi sang “Cửa sổ” hoặc “Toàn màn hình không viền”.",
  "warn.hotkey_failed":
    "Không đăng ký được các phím tắt sau, vì ứng dụng khác đang giữ chúng:",
  "warn.no_data":
    "Chưa có dữ liệu bản đồ trên máy. Cần tải về một lần trước khi dùng.",

  // --- phím tắt (tên hành động) ---
  "hotkey.toggle_minimap": "Hiện/ẩn bản đồ nhỏ",
  "hotkey.toggle_fullmap": "Mở/đóng bản đồ lớn",
  "hotkey.toggle_click_through": "Bật/tắt chế độ bấm được",
  "hotkey.mark_here": "Đánh dấu vị trí hiện tại",
  "hotkey.opacity_up": "Bản đồ nhỏ đậm hơn",
  "hotkey.opacity_down": "Bản đồ nhỏ nhạt hơn",
  "hotkey.zoom_in": "Thu gần vùng nhìn",
  "hotkey.zoom_out": "Nhìn xa hơn",
  "hotkey.toggle_quests": "Hiện/ẩn bảng nhiệm vụ Prime",
  "hotkey.unstuck": "Mở chat và gửi /unstuck",
  "hotkey.reload_ui": "Tải lại giao diện (khi bị đơ)",

  // --- cài đặt ---
  "settings.language": "Ngôn ngữ · Language",
  "settings.updates": "Cập nhật ứng dụng",
  "settings.minimap": "Bản đồ nhỏ",
  "settings.visible": "Hiện bản đồ nhỏ",
  "settings.require_game": "Chỉ hiện khi đang trong game (Alt-Tab ra là tự ẩn)",
  "settings.click_through": "Chuột bấm xuyên qua (không cản trở lúc chơi)",
  "settings.show_trail": "Hiện đường đi trên bản đồ nhỏ",
  "settings.show_waypoints": "Hiện waypoint trên bản đồ nhỏ",
  "settings.corner": "Góc neo theo cửa sổ game",
  "corner.top-left": "Trên trái",
  "corner.top-right": "Trên phải",
  "corner.bottom-left": "Dưới trái",
  "corner.bottom-right": "Dưới phải",
  "settings.size": "Kích thước",
  "settings.margin": "Cách mép",
  "settings.opacity": "Độ đậm",
  "settings.radius": "Bán kính vùng nhìn",
  "settings.hotkeys": "Phím tắt",
  "settings.hotkeys_hint":
    "Bấm vào ô phím rồi nhấn tổ hợp mới. Phím thường cần Ctrl/Alt/Shift/Win; /unstuck được phép dùng riêng phím `.",
  "settings.press_keys": "Nhấn tổ hợp phím… (Esc để hủy)",
  "settings.hotkey_in_use": "Tổ hợp này đang bị ứng dụng khác giữ",
  "settings.hotkey_duplicate": "Trùng với một phím tắt khác trong ứng dụng",
  "settings.hotkey_invalid": "Tổ hợp không hợp lệ — cần ít nhất một phím bổ trợ",
  "settings.number_format": "Định dạng số tọa độ",
  "format.auto": "Tự động nhận biết",
  "format.us": "Kiểu Mỹ — 1,234.5",
  "format.eu": "Kiểu Châu Âu — 1.234,5",
  "settings.data": "Dữ liệu",
  "settings.open_trails": "Mở thư mục đường đi",
  "settings.redownload": "Tải lại dữ liệu bản đồ",
  "settings.basemap": "Nền bản đồ",
  "basemap.vulnona": "Vulnona (mặc định)",
  "basemap.islemaps_light": "IsleMaps — sáng",
  "basemap.islemaps_dark": "IsleMaps — tối",
  "basemap.hint":
    "Áp dụng cho cả bản đồ lớn lẫn bản đồ nhỏ. Lần đầu chọn sẽ tải ảnh nền " +
    "(~5–7 MB) về máy — sau đó dùng offline. Bản IsleMaps vẽ theo phiên bản game " +
    "mới hơn, thấy cả quần đảo đông nam (Hell's Mouth).",
  "basemap.downloading": "Đang tải ảnh nền…",
  "basemap.failed":
    "Tải ảnh nền thất bại — kiểm tra mạng rồi thử lại. Vẫn dùng nền hiện tại.",

  // --- chạy lần đầu ---
  "firstrun.title": "Tải dữ liệu bản đồ",
  "firstrun.explain":
    "Ứng dụng cần tải ảnh bản đồ (~3 MB) và dữ liệu điểm về máy bạn một lần. " +
    "Dữ liệu được tải trực tiếp từ nguồn thay vì đóng gói sẵn — đây là bản sao cá nhân " +
    "trên máy bạn, không phải bản phát hành lại.",
  "firstrun.start": "Bắt đầu tải",
  "firstrun.downloading": "Đang tải…",
  "firstrun.done": "Xong! Đang mở bản đồ…",
  "firstrun.partial":
    "Đã tải được ảnh bản đồ nhưng dữ liệu điểm bị lỗi. Bạn vẫn dùng được bản đồ; " +
    "thử tải lại dữ liệu trong phần Cài đặt sau.",
  "firstrun.failed": "Tải thất bại. Kiểm tra kết nối mạng rồi thử lại.",
  "firstrun.retry": "Thử lại",
  "firstrun.continue": "Tiếp tục với bản đồ",

  // --- kết nối dữ liệu server ---
  "provider.title": "Kết nối server The Isle",
  "provider.subtitle": "Đăng nhập một lần để bản đồ tự cập nhật theo nhân vật.",
  "provider.website": "Website quản lý hoặc live map của server",
  "provider.detect": "Nhận diện",
  "provider.supported":
    "Hiện hỗ trợ Era Gaming VN, The Real Server VN (Titan) và các server IslePilot.",
  "provider.adapter_required":
    "Website khác cần có adapter tương thích; ứng dụng không đoán dữ liệu hoặc gửi cookie thử sang domain lạ.",
  "provider.detected": "Đã nhận diện",
  "provider.login": "Mở đăng nhập",
  "provider.login_wait": "Hãy hoàn tất đăng nhập trong cửa sổ vừa mở…",
  "provider.manual": "Dùng tọa độ thủ công từ Asset Location",
  "provider.active": "Nguồn dữ liệu",
  "provider.change": "Đổi kết nối",
  "provider.temporary": "Mất kết nối tạm thời; ứng dụng sẽ tự thử lại.",
  "provider.retrying": "Đang kết nối lại",
  "provider.stale": "Đang hiển thị dữ liệu gần nhất; vị trí và chỉ số có thể đã thay đổi.",
  "dino.era_prime_unavailable": "Era chưa gửi trạng thái Prime. Hub sẽ hiển thị lại khi server cung cấp dữ liệu; đây không phải tiến độ 0/10.",
  "dino.era_heading_cadence": "Hướng nhìn Era cập nhật theo dữ liệu server, không liên tục theo từng khung hình. Khi thiếu góc nhìn, ký hiệu ≈ là hướng suy từ di chuyển.",
  "provider.offline_hint": "Đã đăng nhập nhưng hiện chưa thấy nhân vật online trong game.",
  "provider.capability_missing": "Server này chưa cung cấp các chỉ số chi tiết còn thiếu.",
  "provider.mutations": "Đột biến",
  "poi.islepilot_provider_only": "POI trực tiếp của server chỉ có khi đang dùng IslePilot.",

  // --- khủng long của bạn ---
  "dino.title": "Khủng long của bạn",
  "dino.explain":
    "Đọc thông tin khủng long của chính bạn từ trang quản lý IslePilot của server " +
    "(growth, máu, đói, khát, Prime progress). Chỉ là kết nối HTTPS tới website của server " +
    "— không đụng gì tới game, an toàn với anti-cheat.",
  "dino.server": "Server",
  "dino.login": "Đăng nhập Steam",
  "dino.login_wait": "Đang chờ bạn đăng nhập trong cửa sổ vừa mở…",
  "dino.login_failed": "Đăng nhập không thành công. Thử lại.",
  "dino.logged_in": "Đã đăng nhập",
  "dino.logout": "Đăng xuất",
  "dino.auth_expired": "Phiên đăng nhập đã hết hạn — hãy đăng nhập lại.",
  "dino.supported_servers":
    "Hỗ trợ mọi server chạy IslePilot — dạng xxx.islepilot.eu hoặc islepilot.eu/p/tên-server.",
  "dino.manual_cookie": "Dán cookie đăng nhập",
  "dino.manual_cookie_hint":
    "Mở trang server trong trình duyệt và đăng nhập Steam. Bấm F12 → tab Application " +
    "(Chrome) hoặc Storage (Firefox) → Cookies → chọn domain server → tìm cookie tên " +
    "islepilot_player rồi copy phần Value dán vào đây.",
  "dino.cancel_login": "Hủy đăng nhập",
  "dino.manual_cookie_save": "Kiểm tra & lưu cookie",
  "dino.manual_cookie_checking": "Đang kiểm tra cookie…",
  "dino.manual_cookie_bad":
    "Cookie không hợp lệ hoặc phiên chưa đăng nhập — kiểm tra lại chuỗi đã dán.",
  "dino.server_settings": "Cài đặt server",
  "dino.token_login": "Đăng nhập Steam (1 lần, dùng cho mọi server)",
  "dino.token_login_hint":
    "Đăng nhập qua islepilot.eu một lần duy nhất — token dùng chung cho MỌI server IslePilot " +
    "(mixi, hoho, sdvn…), không cần nhập server hay copy cookie nữa. Đổi server trong game " +
    "là dữ liệu tự đổi theo.",
  "dino.token_paste": "Hoặc dán token thủ công",
  "dino.token_paste_hint":
    "Nếu cửa sổ đăng nhập không tự bắt được token: dán token overlay (hoặc nguyên link " +
    "islemap-thienvyma://…; link cũ theisle-overlay://… / isle-overlay://… vẫn dùng được) vào đây.",
  "dino.token_save": "Kiểm tra & lưu token",
  "dino.token_checking": "Đang kiểm tra token…",
  "dino.token_bad": "Token không hợp lệ — kiểm tra lại chuỗi đã dán.",
  "dino.legacy_section": "Cách cũ: nhập server + cookie (dự phòng)",
  "dino.legacy_hint":
    "Chỉ cần khi cách đăng nhập mới không hoạt động với server của bạn. Cookie lưu riêng " +
    "cho từng server.",
  "dino.live_map_yes": "Server có live map — vị trí sẽ tự cập nhật",
  "dino.live_map_checking": "Đang kiểm tra live map của server…",
  "dino.enabled": "Theo dõi thông tin khủng long",
  "dino.interval": "Tần suất cập nhật",
  "dino.overlay_panel": "Hiện thanh chỉ số dưới bản đồ nhỏ",
  "dino.quests_panel": "Hiện nhiệm vụ Prime dưới bản đồ nhỏ",
  "dino.use_map_position":
    "Lấy vị trí tự động từ live map của server (thay cho copy tọa độ thủ công)",
  "dino.rules_note":
    "⚠ Nên hỏi admin server trước khi dùng thường xuyên — một số server có luật riêng về " +
    "công cụ bên thứ ba. Dữ liệu hiển thị chỉ là của chính bạn, do panel của server cung cấp.",
  "dino.growth": "Trưởng thành",
  "dino.health": "Máu",
  "dino.hunger": "Đói",
  "dino.thirst": "Khát",
  "dino.stamina": "Thể lực",
  "dino.nutrition": "Dinh dưỡng",
  "dino.nutrition_carb": "Carb",
  "dino.nutrition_protein": "Đạm",
  "dino.nutrition_lipid": "Béo",
  "dino.server_playing": "Server",
  "dino.sex_female": "Cái",
  "dino.sex_male": "Đực",
  "dino.prime": "Prime progress",
  "dino.prime_done": "Đã hoàn thành",
  "dino.prime_pending": "Chưa hoàn thành",
  "dino.online": "Online",
  "dino.offline": "Offline",
  "dino.updated": "Cập nhật lúc {time}",
  "dino.no_data": "Chưa có dữ liệu — bật theo dõi và chờ lần cập nhật đầu.",
  "dino.fetch_error": "Lỗi kết nối tới panel:",
  "dino.layout_changed":
    "IslePilot vừa cập nhật phiên bản mới — nếu số liệu trông sai, giao diện của họ có thể " +
    "đã đổi và app cần cập nhật theo.",
  "dino.map_disabled": "Server này tắt live map.",
  "dino.crashed":
    "Phần Khủng long gặp lỗi và đã được cách ly — bản đồ và các tính năng khác không bị ảnh hưởng.",

  // --- garage (gacha) — chỉ có ở chế độ đăng nhập token ---
  "garage.title": "Garage (Gacha)",
  "garage.hint":
    "Danh sách khủng long đã gửi vào garage của server. Park/Restore mất tới ~60 giây " +
    "vì server xử lý bất đồng bộ.",
  "garage.refresh": "Làm mới",
  "garage.park": "Park dino hiện tại",
  "garage.restore": "Restore",
  "garage.sell": "Bán",
  "garage.rename": "Đổi tên",
  "garage.rename_prompt": "Tên mới cho dino:",
  "garage.confirm_restore": "Restore dino “{name}”? Dino đang chơi có thể bị thay thế.",
  "garage.confirm_sell": "Bán dino “{name}”? Không thể hoàn tác.",
  "garage.empty": "Garage trống.",
  "garage.busy": "Đang gửi lệnh tới server… (tối đa ~60 giây)",
  "garage.error": "Lệnh thất bại:",
  "garage.sold": "Đã bán — nhận {amount} {currency}",
  "garage.done": "Xong!",
  "garage.need_token":
    "Garage cần đăng nhập Steam qua IslePilot (1 lần, dùng cho mọi server) — vào tab " +
    "Khủng long để đăng nhập. Cách cũ nhập server + cookie không dùng được Garage.",
  "garage.unsupported":
    "Không lấy được Garage — server bạn đang chơi có thể không hỗ trợ tính năng này.",
  "garage.updated":
    "Cập nhật lúc {time} · tự làm mới sau mỗi 10 phút — bấm Làm mới nếu cần ngay.",
  "garage.offline_hint":
    "IslePilot đang báo bạn offline hoặc chưa có Dino hoạt động. Hãy vào hẳn server rồi làm mới; nút cất/lấy Dino sẽ tự bật.",

  // --- Garage đa server ---
  "server_garage.title": "Dino Garage",
  "server_garage.subtitle": "Kho khủng long trực tiếp của {provider}.",
  "server_garage.standard": "THƯỜNG",
  "server_garage.uncertain": "CHƯA XÁC ĐỊNH TRẠNG THÁI",
  "server_garage.pending": "Server đang xử lý một thao tác Garage.",
  "server_garage.countdown": "Còn {seconds} giây trước khi cất Dino.",
  "server_garage.locked": "ĐÃ KHÓA",
  "server_garage.restoring": "ĐANG LẤY",
  "server_garage.stored": "ĐÃ CẤT",
  "server_garage.empty": "TRỐNG",
  "server_garage.empty_hint": "Slot trống · có thể cất Dino đang chơi",
  "server_garage.park": "Cất Dino",
  "server_garage.restore": "Lấy Dino",
  "server_garage.delete": "Xóa vĩnh viễn",
  "server_garage.failed": "Server từ chối thao tác Garage.",
  "server_garage.confirm_park_titan":
    "Cất Dino vào slot {slot}? Titan sẽ đếm ngược rồi kết thúc Dino đang chơi sau khi lưu.",
  "server_garage.confirm_park_era":
    "Cất Dino vào slot {slot}? Era sẽ lưu an toàn rồi kết thúc Dino đang chơi.",
  "server_garage.confirm_restore":
    "Lấy {name}? Trước tiên hãy spawn Dino non đúng loài và giới tính, đồng thời tránh đứng gần người chơi khác.",
  "server_garage.confirm_delete": "Xóa vĩnh viễn {name} khỏi Garage? Không thể hoàn tác.",
  "server_garage.titan_note":
    "Titan: thao tác cần Dino online và cầu nối server hoạt động. Khi cất, bạn có thể hủy trong thời gian đếm ngược.",
  "server_garage.era_note":
    "Era: lấy Dino yêu cầu đang online, spawn đúng loài/giới tính và không có người chơi trong bán kính an toàn.",

  // --- đổi skin đa server ---
  "skin.title": "Đổi skin trực tiếp",
  "skin.subtitle": "Bảng phối màu theo quyền tài khoản trên {provider}.",
  "skin.not_supported": "Server hiện tại chưa bật API đổi skin cho overlay",
  "skin.not_supported_hint":
    "Garage IslePilot vẫn hoạt động. Mục đổi skin chỉ mở khi server bật Live Skin cho tài khoản này.",
  "skin.server_disabled":
    "Server đang chơi đã tắt Live Skin hoặc tài khoản chưa được cấp quyền. Hub sẽ tự mở bảng màu khi IslePilot trả về quyền sử dụng.",
  "skin.failed": "Không thể đọc hoặc áp dụng skin lúc này.",
  "skin.done": "Đã áp dụng bảng màu cho Dino đang chơi.",
  "skin.cooldown": "Đổi lại sau {time}",
  "skin.preview": "XEM TRƯỚC BẢNG MÀU",
  "skin.presets": "PHỐI MÀU NHANH",
  "skin.preset_forest": "Rừng",
  "skin.preset_desert": "Sa mạc",
  "skin.preset_shadow": "Bóng tối",
  "skin.preset_snow": "Tuyết",
  "skin.choose_for": "Chọn một trong 16 màu cho vùng: {zone}",
  "skin.variation": "Biến thể hoa văn",
  "skin.online_only":
    "Dino phải đang online trên đúng server. Mỗi server tự kiểm tra quyền màu và thời gian chờ.",
  "skin.applying": "Đang áp dụng…",
  "skin.apply": "Áp dụng vào game",

  // --- xem 3D ---
  "dino3d.loading": "Đang tải model 3D…",
  "dino3d.no_model": "Loài này chưa có model 3D.",
  "dino3d.error": "Không tải được model 3D — kiểm tra mạng rồi thử lại.",

  // --- POI IslePilot trên bản đồ ---
  "layer.islepilot": "POI server (IslePilot)",
  "poi.islepilot_discord":
    "Cần liên kết Discord với IslePilot để mở khóa bản đồ server.",
  "poi.islepilot_disabled": "Server này tắt live map.",
  "poi.islepilot_login": "Đăng nhập token (tab Khủng long) để hiện POI của server.",
  "poi.islepilot_empty": "Server chưa có POI nào.",
  "map.crashed":
    "Bản đồ gặp lỗi hiển thị. Bấm Thử lại, hoặc nhấn F5 để tải lại toàn bộ ứng dụng.",
  "btn.retry": "Thử lại",

  // --- footer ---
  "footer.developed_by": "Được phát triển bởi",
  "footer.custom_build": "Bản đa server thử nghiệm",
  "footer.reload_hint": "Nếu ứng dụng bị lỗi, nhấn F5 hoặc Ctrl+Alt+R để tải lại",

  // --- cập nhật ứng dụng ---
  "update.available": "Có bản cập nhật {version}",
  "update.body": "Kênh cập nhật chính thức: thienvyma/hubisle.",
  "update.title": "Cập nhật islemap-thienvyma",
  "update.check": "Kiểm tra cập nhật",
  "update.manual_hint": "Luôn kiểm tra trực tiếp kênh phát hành thienvyma/hubisle.",
  "update.checking": "Đang kiểm tra cập nhật…",
  "update.current": "Bạn đang dùng phiên bản mới nhất của kênh cập nhật.",
  "update.check_failed": "Chưa thể kiểm tra cập nhật",
  "update.check_failed_body": "Kiểm tra kết nối mạng hoặc thử lại sau. Kênh phát hành có thể chưa khả dụng.",
  "update.notes": "Nội dung bản cập nhật",
  "update.releases": "Xem trên GitHub",
  "update.downloading_unknown": "Đang tải bản cập nhật…",
  "update.link_failed": "Không mở được trình duyệt. Truy cập github.com/thienvyma/hubisle/releases.",
  "update.install": "Tải và cài đặt",
  "update.downloading": "Đang tải {percent}%",
  "update.installing": "Đang mở bộ cài…",
  "update.later": "Để sau",
  "update.failed": "Không thể cài bản cập nhật. Hãy thử lại sau.",


  // --- số liệu sử dụng & phản hồi ---
  "telemetry.title": "Số liệu sử dụng & phản hồi",
  "telemetry.enabled": "Gửi số liệu sử dụng ẩn danh",
  "telemetry.hint":
    "Chỉ gồm: một mã cài đặt ngẫu nhiên, phiên bản app, số hiệu bản Windows, " +
    "ngôn ngữ giao diện và số lần dùng từng tính năng. Không gửi địa chỉ IP, " +
    "không gửi vị trí trong game, không gửi tên tài khoản Windows.",
  "feedback.title": "Gửi phản hồi",
  "feedback.cat_bug": "Lỗi",
  "feedback.cat_idea": "Góp ý",
  "feedback.cat_other": "Khác",
  "feedback.body": "Mô tả (tối đa 2000 ký tự)",
  "feedback.contact": "Cách liên hệ lại (không bắt buộc)",
  "feedback.send": "Gửi",
  "feedback.sending": "Đang gửi…",
  "feedback.sent": "Đã gửi. Cảm ơn bạn!",
  "feedback.failed": "Không gửi được. Kiểm tra mạng rồi thử lại.",
  // --- ghi công ---
  "credits.title": "Nguồn dữ liệu",
  "credits.body":
    "Ảnh nền: VulnonaMAP (Coco.N) — ghép từ ảnh chụp trong game. " +
    "Nền IsleMaps & điểm động vật: IsleMaps.com (Pont & Emeara). " +
    "Hình ảnh thuộc bản quyền Afterthought LLC (The Isle). " +
    "Dữ liệu điểm: VulnonaMAP, myislemap.com, hướng dẫn Steam của wiredredman. " +
    "Ứng dụng này không liên kết với Afterthought LLC.",
} as const;

export type MsgKey = keyof typeof vi;
