# Story 3.3: Preview Before and After for a Selected Segment

Status: ready-for-dev

## Story

As a user,
I want xem before/after cho đúng segment tôi đang review,
so that tôi có thể quyết định nhanh xem có cần sửa hay có thể chấp nhận kết quả hiện tại.

## Acceptance Criteria

1. Chọn segment → preview workspace jump đến đúng time range của segment → giữ trong ngữ cảnh segment đó.
2. Bật before/after hoặc split preview → hiển thị hai trạng thái trước/sau cùng segment và time range.
3. Ở trạng thái paused trên segment → hiển thị overlay handles (logo mới, blur/mask box subtitle cũ, subtitle mới) → user có thể kéo thả handles để điều chỉnh vị trí/kích thước trực tiếp.

## Tasks / Subtasks

- [ ] Implement `PreviewWorkspace.tsx` với video player (AC: 1)
  - [ ] HTML5 `<video>` element cho playback
  - [ ] Khi segment được chọn: set `currentTime` = segment.startTime, auto-pause
  - [ ] Loop trong time range của segment: khi `currentTime > segment.endTime` → reset về `segment.startTime`
  - [ ] Keyboard shortcut: Space = play/pause, ← → = -/+1s
- [ ] Implement before/after toggle (AC: 2)
  - [ ] Layer 1 (primary): HTML5 video + Canvas overlay (real-time interaction)
  - [ ] Layer 2 (validated preview): FFmpeg frame grab cached tại `{job}/cache/previews/{segmentId}-{hash}.mp4`
  - [ ] **Before**: video gốc (source file) tại đúng time range
  - [ ] **After**: processed intermediate file với overlays applied
  - [ ] Split mode: hai `<video>` side-by-side sync thời gian
  - [ ] Toggle button: `Trước / Sau / So sánh` (3 modes)
- [ ] Implement Canvas overlay handles (AC: 3)
  - [ ] Overlay `<canvas>` trên `<video>` element, absolute positioned, same dimensions
  - [ ] Logo handle: draggable rectangle với resize corners
  - [ ] Subtitle old region handle: draggable blur/mask box
  - [ ] Subtitle new position handle: draggable position indicator
  - [ ] Snap to grid: 8px grid (optional, V1 nicety)
  - [ ] Lưu handle position → `reviewStore.pendingFix` (chưa persist, chờ user bấm Apply)
- [ ] Implement `get_frame_preview` Tauri command (AC: 2)
  - [ ] Nhận: `{ videoId, segmentId, timeSeconds }`
  - [ ] Check cache: nếu `{job}/cache/previews/{segmentId}-{hash}.mp4` tồn tại → return path
  - [ ] Nếu không có cache: spawn FFmpeg để extract clip ngắn (segment range), cache kết quả
  - [ ] Hash: `md5(segmentId + quickFixStateJson)` để invalidate khi fix thay đổi

## Dev Notes

- **Hybrid preview architecture** (ADR-08):
  - Layer 1 (Canvas overlay): dùng cho real-time drag/resize interaction, không cần FFmpeg
  - Layer 2 (FFmpeg frame grab): chỉ gọi khi user muốn xem validated "after" với effect thật từ FFmpeg
  - Đây là design quan trọng — không làm ngược lại (không gọi FFmpeg cho mỗi drag event)
- **Frame grab cache**: path `{job}/cache/previews/{segmentId}-{hash}.mp4`. Hash bao gồm quickFixState để invalidate cache khi user thay đổi fix.
- **Canvas coordinate mapping**: video element có thể bị scale trong DOM. Phải convert mouse coordinates từ DOM space sang video pixel space. Dùng `video.videoWidth / video.clientWidth` ratio.
- **Không persist handles tự động**: drag một handle chỉ update `pendingFix` trong memory. Chỉ persist khi user bấm "Áp dụng" (Story 3.4, 3.5).
- **Before source**: dùng original source video (`videoItem.sourcePath`), không phải working file. Đây là quan trọng để user thấy đúng trước/sau.
- **Split sync**: hai video phải play/pause đồng thời. Sync bằng shared `currentTime` state và `requestAnimationFrame`.

### Project Structure Notes

- Frontend: `src/modules/segment-review/PreviewWorkspace.tsx`, `src/modules/segment-review/CanvasOverlay.tsx`, `src/modules/segment-review/PreviewControls.tsx`
- Backend: `src-tauri/src/commands/review_commands.rs` (`get_frame_preview`), `src-tauri/src/services/render_service.rs` (FFmpeg frame grab)
- Cache: `{job}/cache/previews/{segmentId}-{hash}.mp4`
- Store: `src/store/reviewStore.ts` (`pendingFix`, `previewMode`)

### References

- [Source: epics.md#Story 3.3] Acceptance criteria
- [Source: prd.md#7.1] FR16: before/after preview, overlay handles
- [Source: ux-design-specification.md#6.5] Preview workspace (UX-DR10)
- [Source: architecture.md#10] Hybrid preview architecture: Layer 1 Canvas + Layer 2 FFmpeg grab
- [Source: architecture.md#ADR-08] Hybrid preview ADR

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
