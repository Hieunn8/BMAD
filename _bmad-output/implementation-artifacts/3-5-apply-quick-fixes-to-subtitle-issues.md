# Story 3.5: Apply Quick Fixes to Subtitle Issues

Status: done

## Story

As a user,
I want sửa nhanh vùng subtitle cũ và subtitle mới trên segment bị flag,
so that tôi có thể xử lý ngoại lệ subtitle mà không cần timeline editor đầy đủ.

## Acceptance Criteria

1. Segment có issue type liên quan đến subtitle → Quick Fix Panel cho phép: `move/resize blur or mask box`, `đổi vị trí subtitle mới`, `đổi scale subtitle`, `đổi subtitle style preset`.
2. Bấm "Áp dụng cho segment này" → persist quick fix state subtitle → cập nhật review status.
3. Chọn nhiều segment compatible → "Áp dụng cho các segment đã chọn" → copy fix, không áp dụng cho incompatible.
4. Mở quick fix panel cho segment có subtitle old region → cho phép chọn mode: `blur`, `mask`, hoặc `fill` → preview cập nhật ngay.
5. Bấm "Đánh dấu đã review" → review status = `Accepted`.
6. Bấm "Khôi phục mặc định" → reset tất cả subtitle controls (vị trí, scale, style, mode) về preset default → xóa custom quick fix state.

## Tasks / Subtasks

- [ ] Implement `QuickFixPanel.tsx` subtitle section (AC: 1)
  - [ ] Subtitle old region controls: resize/move box (drag trên canvas), mode picker (`blur` / `mask` / `fill`)
  - [ ] Subtitle new controls: position (X, Y), scale (%), style preset selector
  - [ ] Style preset selector: dropdown với danh sách subtitleStylePreset từ presets
  - [ ] Buttons: `Áp dụng cho segment này`, `Áp dụng cho các segment đã chọn`, `Đánh dấu đã review`, `Khôi phục mặc định`
  - [ ] Panel chỉ hiển thị subtitle controls khi segment.issueType = `SubtitleRegion` hoặc `SubtitleStyle`
- [ ] Implement mode picker real-time preview (AC: 4)
  - [ ] Khi user đổi mode → update `reviewStore.pendingSubtitleFix.removalMode`
  - [ ] Canvas overlay cập nhật visualization ngay (blur = translucent box, mask = solid, fill = color fill)
  - [ ] Layer 2 FFmpeg preview chỉ rebuild khi user bấm Apply hoặc request validate preview
- [ ] Implement `apply_subtitle_fix` Tauri command (AC: 2, 3)
  - [ ] Nhận: `{ segmentIds: string[], subtitleFix: { oldRegion?: { x, y, w, h, mode }, newPosition?: { x, y }, newScale?: float, stylePreset?: string } }`
  - [ ] Validate: chỉ apply cho segments có issueType = SubtitleRegion | SubtitleStyle
  - [ ] Update `QuickFixState.subtitleFix` cho từng segment
  - [ ] Set `segment.reviewStatus = Modified`
  - [ ] Persist vào `segments/{videoId}.json`
  - [ ] Invalidate cache: xóa `{job}/cache/previews/{segmentId}-*.mp4`
- [ ] Implement bulk apply compatibility check (AC: 3)
  - [ ] Compatible: cùng video, cùng issueType subtitle
  - [ ] Warn nếu selection có incompatible segments (không block)
- [ ] Implement `reset_subtitle_fix` Tauri command (AC: 6)
  - [ ] Clear `QuickFixState.subtitleFix` (set về null)
  - [ ] Set `segment.reviewStatus = Unreviewed`
  - [ ] Reload subtitle defaults từ `preset.subtitleStylePreset` và `preset.layoutRules`
  - [ ] Persist changes

## Dev Notes

- **Subtitle fix schema** (phần của QuickFixState):
  ```json
  {
    "subtitleFix": {
      "oldRegion": { "x": 0, "y": 850, "w": 1920, "h": 80, "mode": "blur" },
      "newPosition": { "x": 960, "y": 920 },
      "newScale": 1.0,
      "stylePreset": "default"
    }
  }
  ```
- **Removal mode** (`blur` / `mask` / `fill`): đây là per-segment fix. Mode mặc định lấy từ `preset.subtitleStylePreset`. User chỉ override khi segment đó cần xử lý đặc biệt.
- **Mode visualization on canvas**: `blur` = semi-transparent gray box; `mask` = opaque white box; `fill` = opaque black box. Đây chỉ là visual hint — FFmpeg sẽ apply đúng effect khi export.
- **SubtitleStyle fix**: nếu segment có issue type `SubtitleStyle` (new subtitle render lệch hoặc sai font), quick fix chỉ cần adjust position + scale + style preset. Không cần oldRegion controls.
- **Audio UX boundary** (UX-DR12): audio mapping không được xuất hiện trong Quick Fix Panel. Audio issues chỉ xử lý ở Setup Mode (Story 1.3).
- **Immutability**: như Story 3.4, tất cả state changes qua Tauri command, không mutate in-place.

### Project Structure Notes

- Frontend: `src/modules/segment-review/QuickFixPanel.tsx` (subtitle section), `src/modules/segment-review/SubtitleControls.tsx`, `src/modules/segment-review/ModePicker.tsx`
- Backend: `src-tauri/src/commands/review_commands.rs` (`apply_subtitle_fix`, `reset_subtitle_fix`), `src-tauri/src/services/review_service.rs`
- Storage: `{job}/segments/{videoId}.json` (QuickFixState.subtitleFix), `{job}/cache/previews/` (invalidate on fix)

### References

- [Source: epics.md#Story 3.5] Acceptance criteria
- [Source: prd.md#7.1] FR11: blur/mask/fill; FR12: render subtitle mới; FR13: quick fix subtitle tối giản
- [Source: ux-design-specification.md#6.5] Quick Fix Panel V1 controls (UX-DR11), audio UX boundary (UX-DR12)
- [Source: architecture.md#5.4] QuickFixState: subtitleFix fields
- [Source: architecture.md#13.1] Review commands

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
