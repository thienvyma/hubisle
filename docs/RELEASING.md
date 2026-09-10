# Phát hành và cập nhật Isle Pulse Overlay

Ứng dụng kiểm tra phiên bản mới tại:

`https://github.com/thienvyma/hubisle/releases/latest/download/latest.json`

Repository cần để **Public** để máy người dùng tải `latest.json` và installer
mà không phải nhúng GitHub token vào ứng dụng.

## Luồng thông báo

- App kiểm tra khi khởi động, mỗi giờ và khi kết nối mạng trở lại.
- Nút **Kiểm tra cập nhật** ở chân cửa sổ hoạt động cả trước khi đăng nhập server.
- Kiểm tra tự động không bật thông báo khi không có bản mới hoặc mất mạng.
  Kiểm tra thủ công hiển thị rõ thành công hoặc lỗi.
- **Để sau** ẩn thông báo cho cùng phiên bản trong phiên chạy hiện tại.
  Kiểm tra thủ công hoặc một phiên bản mới hơn sẽ mở lại thông báo.
- Nội dung phát hành hiển thị dạng văn bản; nút GitHub mở đúng
  `https://github.com/thienvyma/hubisle/releases`.

## Chuyển từ GitHub cũ

Endpoint được đóng gói trong bộ cài. Bản app đã cài mà vẫn trỏ GitHub cũ sẽ
không tự biết GitHub mới: cần cài thủ công một bản chuyển tiếp từ hubisle,
hoặc phát hành bản chuyển tiếp trên kênh cũ nếu vẫn quản lý được kênh đó.
Giữ nguyên khóa ký đang dùng nếu muốn các bản đã cài chấp nhận bản cập nhật.

Kiểm tra trực tiếp bằng trình duyệt ngày 2026-09-10: repository là **Public**
nhưng đang trống (chưa có mã nguồn, nhánh hoặc tag). Đường dẫn latest.json
chưa khả dụng. Cần đẩy mã nguồn, thiết lập secret ký và phát hành bản đầu tiên
trước khi thử cập nhật thật từ một máy khác.

## Thiết lập một lần

Khóa ký riêng được lưu ngoài repository tại:

`C:\Users\thien\.tauri\hubisle.key`

Không commit, gửi hoặc chia sẻ file này. Trong GitHub, vào **Settings → Secrets
and variables → Actions → New repository secret**, tạo secret:

- Tên: `TAURI_SIGNING_PRIVATE_KEY`
- Giá trị: toàn bộ nội dung file `hubisle.key`

Khóa công khai đã nằm trong `src-tauri/tauri.conf.json`; ứng dụng dùng nó để từ
chối mọi bản cập nhật bị sửa hoặc được ký bằng khóa khác.

## Tạo phiên bản mới

1. Sửa cùng một số phiên bản trong `package.json`, `src-tauri/Cargo.toml` và
   `src-tauri/tauri.conf.json`, ví dụ `2.1.1`.
2. Ghi thay đổi vào `CHANGELOG.md`.
3. Chạy kiểm tra phiên bản:

   ```powershell
   ./scripts/check-versions.ps1
   ```

4. Commit, tạo tag trùng số phiên bản rồi push repository và tag:

   ```powershell
   git tag v2.1.1
   git push origin main
   git push origin v2.1.1
   ```

Workflow `Release Isle Pulse Overlay` chỉ chạy phát hành trong
`thienvyma/hubisle`. Workflow kiểm tra phiên bản, cấu hình updater, kiểm thử
logic thông báo và Svelte trước khi build. Bộ cài có chữ ký và `latest.json`
được tải lên một draft release; manifest phải đúng phiên bản và URL bộ cài
Windows của hubisle rồi release mới được công bố. URL API hoặc URL tạm của draft được đổi sang URL tải công khai theo tag trước khi upload lại latest.json. Chữ ký bộ cài được giữ nguyên.

Kiểm tra tại máy trước khi push:

```powershell
npm run check:updater
npm run test:updater
npm run check
npm run build
```

Bản phát hành tiếp theo phải có số phiên bản cao hơn 2.1.0, đồng bộ cả ba
file phiên bản và lockfile. Chuyển mục Unreleased trong CHANGELOG thành số
phiên bản đó trước khi tạo tag.

Nếu làm mất khóa riêng, các bản đang cài sẽ không chấp nhận khóa mới. Vì vậy
cần sao lưu `hubisle.key` ở nơi an toàn.
