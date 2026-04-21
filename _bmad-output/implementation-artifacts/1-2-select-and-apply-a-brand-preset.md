# Story 1.2: Select and Apply a Brand Preset

Status: ready-for-dev

## Story

As a user,
I want chọn một preset theo brand/channel cho job,
so that hệ thống có thể áp dụng default brand rules trước khi tôi xác nhận mapping và readiness.

## Acceptance Criteria

1. Màn Preset Selection hiển thị ít nhất: brand name, logo mặc định, audio policy, subtitle style preset, export preset cho preset đang chọn.
2. Khi preset được áp dụng: lưu vào job state và cập nhật các default brand settings.
3. Khi đổi preset sau khi job đã có mapping hoặc review data: hiển thị cảnh báo rằng mapping hoặc kết quả detect trước đó có thể mất hiệu lực.

## Tasks / Subtasks

- [ ] Tạo Preset Selection screen trong `preset-management` module (AC: 1)
  - [ ] Danh sách preset bên trái (list với brand name)
  - [ ] Preset preview card bên phải: brand name, logo thumbnail, audio policy label, subtitle style name, export preset name, notes
  - [ ] Actions: "Chọn preset này", "Sửa preset", "Tạo preset mới"
- [ ] Implement `select_preset` Tauri command (AC: 2)
  - [ ] Gán presetId vào current job
  - [ ] Resolve preset assets: defaultLogoPath, audioReplacementPolicy, subtitleStylePreset, layoutRules, exportPreset
  - [ ] Persist updated job state (presetId) vào job.json
  - [ ] Trả về full preset data cho frontend
- [ ] Detect preset change warning (AC: 3)
  - [ ] Check nếu job đã có segment flags hoặc review data
  - [ ] Nếu có: hiển thị warning inline trước khi apply: "Đổi preset có thể làm mất hiệu lực các chỉnh sửa trước đó"
  - [ ] User phải confirm để tiếp tục
- [ ] Implement preset data persistence (AC: 2)
  - [ ] Preset domain model: `src-tauri/src/domain/preset.rs` với các fields: presetId, brandName, defaultLogoPath, audioReplacementPolicy, subtitleStylePreset, layoutRules, exportPreset
  - [ ] Presets lưu tách biệt: `{app_data}/presets/{presetId}.json` (không lưu trong job folder)
- [ ] Cập nhật Zustand store (AC: 2)
  - [ ] `jobStore.setPreset(preset)` — lưu full preset object vào state
  - [ ] Downstream components (mapping, rendering) đọc preset từ store

## Dev Notes

- **Preset hiểu như "gói thương hiệu"** không phải config kỹ thuật — UX phải thể hiện điều này. Brand name và logo là điều user thấy đầu tiên.
- **Preset storage**: tách riêng khỏi job (`app_data/presets/`), vì một preset có thể được dùng bởi nhiều jobs.
- **Audio replacement policy** trong preset xác định: có thay audio không, file audio mặc định nào (nếu có). Audio policy service đọc từ preset.
- **Warning timing**: cảnh báo chỉ hiện khi job ĐÃ có segment flags hoặc review data — không hiện ở lần chọn preset đầu tiên.
- **No modal overload**: warning là inline message, không phải popup/dialog. User xác nhận qua một action rõ ràng.
- Logic check "đã có review data" phải kiểm tra `segments/{videoId}.json` tồn tại và có content.

### Project Structure Notes

- Frontend: `src/modules/preset-management/PresetSelectionScreen.tsx`, `src/modules/preset-management/PresetCard.tsx`
- Backend: `src-tauri/src/commands/preset_commands.rs` (select_preset), `src-tauri/src/services/preset_service.rs`, `src-tauri/src/domain/preset.rs`
- Store: `src/store/jobStore.ts` (setPreset action)

### References

- [Source: epics.md#Story 1.2] Acceptance criteria
- [Source: prd.md#7.2] Preset & Profiles requirements
- [Source: ux-design-specification.md#6.2] Preset Selection Screen UX
- [Source: architecture.md#4] Preset Service component
- [Source: architecture.md#5.5] Preset domain model fields
- [Source: architecture.md#13.1] selectPreset command

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
