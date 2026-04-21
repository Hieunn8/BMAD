# Story 1.3: Review and Resolve Video-to-Asset Mapping

Status: ready-for-dev

## Story

As a user,
I want xem mapping hệ thống tự đề xuất và sửa các mapping sai,
so that mỗi video có đúng logo, audio, và SRT cần thiết trước khi chạy.

## Acceptance Criteria

1. Hệ thống dùng exact base filename match để map audio và SRT vào từng video; hiển thị mapping trong bảng job setup.
2. 1 audio/SRT khớp: tự gán và đánh dấu hợp lệ.
3. Video thiếu audio/SRT bắt buộc theo task: gán trạng thái `Input Needs Review`, hiển thị rõ input nào còn thiếu.
4. Nhiều file có thể khớp: không tự chọn ngẫu nhiên, yêu cầu user xác nhận hoặc sửa tay.

## Tasks / Subtasks

- [ ] Implement auto-mapping algorithm trong `mapping_service` (AC: 1, 2, 3, 4)
  - [ ] So sánh base filename (không có extension) của video với audio và SRT files
  - [ ] 1 match → tự gán, đánh dấu `Matched`
  - [ ] 0 match → đánh dấu `Missing`
  - [ ] 2+ match → đánh dấu `NeedsReview` (không chọn ngẫu nhiên)
  - [ ] Emit `mappingUpdated` event sau khi tính xong
- [ ] Tạo Job Setup / Pre-run Review screen trong `job-review` module (AC: 1)
  - [ ] Bảng với các cột: `tên video`, `task`, `preset`, `logo`, `audio`, `SRT`, `trạng thái mapping`
  - [ ] Mỗi video một row
  - [ ] Status badge: Matched (green) / Missing (red) / Needs Review (amber)
- [ ] Inline mapping correction (AC: 4)
  - [ ] Mỗi cell audio/SRT/logo có inline picker (dropdown hoặc file browser)
  - [ ] User có thể chọn file thủ công từ danh sách imported files
  - [ ] Không dùng modal — sửa trực tiếp trong bảng
- [ ] Implement `fix_mapping` Tauri command (AC: 4)
  - [ ] Nhận videoId, field (audio/srt/logo), filePath
  - [ ] Update mapping trong job state
  - [ ] Persist updated mapping vào job.json
  - [ ] Emit `mappingUpdated` event
- [ ] Vietnamese microcopy cho mapping status (AC: 3, 4)
  - [ ] "Chưa tìm thấy SRT khớp" (không phải "No SRT found")
  - [ ] "Chưa tìm thấy audio khớp"
  - [ ] "Có nhiều file có thể phù hợp — cần bạn chọn đúng file"
  - [ ] "Đã khớp tự động"
- [ ] Cập nhật Zustand store với mapping state

## Dev Notes

- **Exact base filename match**: so sánh `Path::file_stem()` (filename không extension) của video với audio/SRT. Case-insensitive trên Windows.
  - Ví dụ: `video_01.mp4` match với `video_01.srt` và `video_01.mp3`
- **1 video = 1 audio + 1 SRT** (V1 rule) — không cho phép map nhiều audio hoặc nhiều SRT vào cùng 1 video.
- **Logo mapping** ở V1: mặc định từ preset. User có thể override per-video nếu muốn.
- **Inline correction** phải hiển thị danh sách các files đã import theo role tương ứng — không cho browse file system tùy ý.
- Khi đổi file input sau khi đã có review data: emit `inputFileReplacedAfterReview` event để UI hiện warning (FR30). Kiểm tra segments/{videoId}.json tồn tại.
- **mappingUpdated event**: frontend lắng nghe event này để re-render bảng mapping. Không poll, dùng event-driven.

### Project Structure Notes

- Frontend: `src/modules/job-review/JobSetupScreen.tsx`, `src/modules/job-review/MappingTable.tsx`, `src/modules/job-review/InlinePicker.tsx`
- Backend: `src-tauri/src/services/mapping_service.rs` (auto_map, fix_mapping), `src-tauri/src/commands/mapping_commands.rs`
- Events: `mappingUpdated`, `inputFileReplacedAfterReview` (Application → UI)

### References

- [Source: epics.md#Story 1.3] Acceptance criteria
- [Source: prd.md#7.1] Mapping rules (FR6–FR16 in detail)
- [Source: ux-design-specification.md#6.3] Job Review / Pre-run Screen UX
- [Source: architecture.md#4] Input Mapping Service
- [Source: architecture.md#13.1] fixMapping command
- [Source: architecture.md#13.2] mappingUpdated, inputFileReplacedAfterReview events

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
