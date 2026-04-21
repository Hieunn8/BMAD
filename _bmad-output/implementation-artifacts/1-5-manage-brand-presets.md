# Story 1.5: Manage Brand Presets (Create, Edit, Duplicate)

Status: ready-for-dev

## Story

As a user,
I want tạo mới, chỉnh sửa, và duplicate preset theo brand/channel,
so that tôi có thể quản lý nhiều bộ cấu hình thương hiệu mà không phải nhập lại từ đầu mỗi lần.

## Acceptance Criteria

1. "Tạo preset mới": mở form với đầy đủ fields → lưu preset mới → xuất hiện ngay trong danh sách.
2. "Sửa preset": load form với data hiện tại → lưu thay đổi → warning nếu preset đang được dùng bởi job có review data.
3. "Duplicate preset": tạo bản sao với tên "[Tên gốc] - Copy" → có thể chỉnh sửa độc lập mà không ảnh hưởng bản gốc.

## Tasks / Subtasks

- [ ] Tạo Preset Form component (dùng cho cả Create và Edit) (AC: 1, 2)
  - [ ] Fields: Brand Name (text), Default Logo (file picker → logo file), Audio Replacement Policy (enum select), Subtitle Style Preset (text/select), Layout Rules (text area basic), Export Preset (select: MP4 H264 CRF18/20/23)
  - [ ] Validate: Brand Name không được trống
  - [ ] Submit action: "Lưu preset"
- [ ] Implement `create_preset` Tauri command (AC: 1)
  - [ ] Generate UUID presetId
  - [ ] Lưu `{app_data}/presets/{presetId}.json`
  - [ ] Trả về preset object
  - [ ] Frontend cập nhật danh sách preset
- [ ] Implement `edit_preset` Tauri command (AC: 2)
  - [ ] Load existing preset by presetId
  - [ ] Overwrite với data mới
  - [ ] Check: nếu presetId trùng với job hiện tại VÀ job có segment data → emit warning trước khi save
  - [ ] Warning message: "Thay đổi preset có thể ảnh hưởng đến kết quả detect và quick-fix đã có"
- [ ] Implement `duplicate_preset` Tauri command (AC: 3)
  - [ ] Deep copy preset object
  - [ ] Tạo presetId mới (UUID)
  - [ ] Tên mới: `"{original_name} - Copy"`
  - [ ] Lưu file mới, không modify bản gốc
  - [ ] Trả về duplicated preset
- [ ] Cập nhật Preset Selection screen (AC: 1, 2, 3)
  - [ ] Buttons: "Tạo preset mới", "Sửa preset" (on selected), "Duplicate preset" (on selected)
  - [ ] Sau CRUD: refresh danh sách preset từ disk

## Dev Notes

- **Preset files tách riêng khỏi job**: lưu ở `{app_data}/presets/` không phải trong job folder. Một preset tái sử dụng được qua nhiều jobs.
- **Audio Replacement Policy** là enum: `ReplaceAll` (thay toàn bộ audio) | `NoReplacement` (giữ nguyên). V1 chỉ support 2 giá trị này.
- **Export Preset** ở V1: chỉ cần lưu CRF value (18/20/23) cho H.264 encode. Không cần phức tạp hơn.
- **Warning trước khi edit**: kiểm tra `segments/{videoId}.json` trong current job folder. Nếu có file → job đã qua analysis → warning cần hiện.
- **Commands**: `createPreset`, `editPreset`, `duplicatePreset` đã được define trong architecture Section 13.1.
- **Không cần delete preset** ở V1 — out of scope.

### Project Structure Notes

- Frontend: `src/modules/preset-management/PresetForm.tsx`, `src/modules/preset-management/PresetSelectionScreen.tsx` (add action buttons)
- Backend: `src-tauri/src/commands/preset_commands.rs` (create_preset, edit_preset, duplicate_preset), `src-tauri/src/services/preset_service.rs`
- Storage: `{app_data}/presets/{presetId}.json`

### References

- [Source: epics.md#Story 1.5] Acceptance criteria
- [Source: prd.md#7.2] Preset & Profiles: save, load, duplicate
- [Source: ux-design-specification.md#6.2] Preset screen actions
- [Source: architecture.md#4] Preset Service: load/save/duplicate
- [Source: architecture.md#5.5] Preset domain model
- [Source: architecture.md#13.1] createPreset, editPreset, duplicatePreset commands

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
