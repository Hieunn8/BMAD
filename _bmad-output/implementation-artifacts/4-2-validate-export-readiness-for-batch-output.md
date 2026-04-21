# Story 4.2: Validate Export Readiness for Batch Output

Status: ready-for-dev

## Story

As a user,
I want hệ thống xác thực rõ video nào đủ điều kiện export và video nào còn bị chặn,
so that tôi không vô tình xuất các video chưa vượt qua đúng các rule review.

## Acceptance Criteria

1. Vào màn Export với batch nhiều trạng thái → hệ thống phân biệt rõ `Ready to Export` với blocked → hiển thị lý do blocked per-video.
2. Video còn blocker review hoặc trạng thái lỗi → không được đưa vào export-ready set → hệ thống dùng readiness state từ Epic 3 thay vì tạo rule gating mới.

## Tasks / Subtasks

- [ ] Tạo `ExportScreen.tsx` (AC: 1, 2)
  - [ ] Danh sách export-ready videos với status badge `Ready to Export`
  - [ ] Danh sách blocked videos với lý do blocked (per-video)
  - [ ] Export settings: output folder picker, export preset summary (CRF value, H.264)
  - [ ] Audio source summary per video (UX-DR13)
  - [ ] Button: `Export All Ready Videos` (enable khi có ít nhất 1 ready video)
- [ ] Implement `get_export_readiness` Tauri command (AC: 1, 2)
  - [ ] Load tất cả VideoItems từ job
  - [ ] Phân loại: `readyVideos[]` (status = ReadyToExport), `blockedVideos[]` (với reason per-video)
  - [ ] Reason mapping:
    - status = `ReviewNeeded` và còn High Risk unreviewed → `"Còn đoạn High Risk chưa xử lý"`
    - status = `Failed` → `"Video bị lỗi trong quá trình xử lý"`
    - status = `Processing` (edge case) → `"Video đang được xử lý"`
  - [ ] Không re-compute review gating rule — chỉ đọc `VideoItem.status` từ disk
- [ ] Implement output folder selection (AC: 1)
  - [ ] Tauri `dialog::pick_folder` để user chọn output folder
  - [ ] Default: `{job}/output/` nếu chưa có selection
  - [ ] Persist selected folder vào job state
- [ ] Cập nhật Zustand exportStore (AC: 1)
  - [ ] `exportStore.readyVideos`, `exportStore.blockedVideos`, `exportStore.outputFolder`
  - [ ] Load khi vào Export screen

## Dev Notes

- **Không tạo rule gating mới**: `get_export_readiness` chỉ đọc `VideoItem.status` từ disk. Rule gating là domain của Epic 3 (`mark_video_ready` command). Giữ separation of concerns.
- **Export screen = checkpoint**: user thấy toàn cảnh trước khi export. Không allow partial export của một video (hoặc export toàn video hoặc không).
- **Audio source summary**: lấy từ `videos/{videoId}.json` field `audioReplacementApplied` và `audioSourcePath`. Hiển thị: "Audio: [filename]" hoặc "Audio: Giữ nguyên" nếu NoReplacement policy.
- **Output folder không được là source folder**: validate rằng output path ≠ source video path để tránh overwrite. (NFR7)
- **Export preset summary**: hiển thị CRF value và codec. V1 luôn là H.264 CRF-18/20/23. Đọc từ `job.preset.exportPreset`.

### Project Structure Notes

- Frontend: `src/modules/export-reporting/ExportScreen.tsx`, `src/modules/export-reporting/ExportReadinessList.tsx`
- Backend: `src-tauri/src/commands/export_commands.rs` (`get_export_readiness`), `src-tauri/src/services/export_service.rs`
- Store: `src/store/exportStore.ts`

### References

- [Source: epics.md#Story 4.2] Acceptance criteria
- [Source: prd.md#7.1] FR17: export batch; FR16: export gating rule (từ Epic 3)
- [Source: ux-design-specification.md#6.6] Export screen (UX-DR13)
- [Source: architecture.md#5.1] VideoItem.status: ReadyToExport
- [Source: architecture.md#7.2] Export gating: consume Epic 3 readiness, không tạo lại

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
