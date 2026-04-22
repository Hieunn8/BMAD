# Story 3.4: Apply Quick Fixes to Logo Issues

Status: done

## Story

As a user,
I want sửa nhanh các lỗi liên quan đến logo trên segment bị flag,
so that tôi có thể chỉnh đúng vị trí hoặc kích thước logo mà không cần mở full editor.

## Acceptance Criteria

1. Segment có issue type Logo → Quick Fix Panel cho phép `move/resize logo` và `reset to preset default`.
2. Bấm "Áp dụng cho segment này" → persist quick fix state cho logo của segment đó → cập nhật review status.
3. Chọn nhiều segment compatible → bấm "Áp dụng cho các segment đã chọn" → copy logo fix, không áp dụng cho segment không compatible.
4. Toggle bật/tắt logo overlay → ẩn/hiện logo mới trong preview → không thay đổi quick fix state đã lưu.
5. Bấm "Đánh dấu đã review" → review status = `Accepted` → segment không còn hiện là unreviewed.
6. Bấm "Khôi phục mặc định" → reset logo overlay về preset default → xóa custom quick fix state của logo.

## Tasks / Subtasks

- [ ] Implement `QuickFixPanel.tsx` cho logo issues (AC: 1)
  - [ ] Hiển thị logo position controls: X, Y offset input + canvas drag (từ Story 3.3)
  - [ ] Logo size controls: Width, Height input hoặc scale percentage
  - [ ] Buttons: `Áp dụng cho segment này`, `Áp dụng cho các segment đã chọn`, `Đánh dấu đã review`, `Khôi phục mặc định`
  - [ ] Panel chỉ hiển thị logo controls khi segment.issueType = `LogoPosition`
- [ ] Implement `apply_logo_fix` Tauri command (AC: 2, 3)
  - [ ] Nhận: `{ segmentIds: string[], logoFix: { x, y, width, height } }`
  - [ ] Với mỗi segmentId:
    - [ ] Validate segment có issueType = Logo
    - [ ] Update `QuickFixState.logoFix` cho segment
    - [ ] Set `segment.reviewStatus = Modified`
    - [ ] Persist vào `segments/{videoId}.json`
  - [ ] Trả về updated segments
- [ ] Implement bulk apply với compatibility check (AC: 3)
  - [ ] Compatible: segments trong cùng video, cùng issueType Logo
  - [ ] Hiển thị warning nếu có segments không compatible trong selection: `"Một số đoạn không thể áp dụng fix này"` (không block, chỉ notify)
- [ ] Implement logo overlay toggle (AC: 4)
  - [ ] Toggle button trong preview toolbar
  - [ ] Chỉ ảnh hưởng `reviewStore.logoOverlayVisible` (UI-only state)
  - [ ] Không gọi command backend khi toggle
- [ ] Implement `mark_segment_accepted` Tauri command (AC: 5)
  - [ ] Nhận: `{ segmentId }`
  - [ ] Set `segment.reviewStatus = Accepted`
  - [ ] Persist vào `segments/{videoId}.json`
- [ ] Implement `reset_logo_fix` Tauri command (AC: 6)
  - [ ] Nhận: `{ segmentId }`
  - [ ] Clear `QuickFixState.logoFix` (set về null)
  - [ ] Set `segment.reviewStatus = Unreviewed`
  - [ ] Reload logo position từ `preset.layoutRules.logoPosition`
  - [ ] Persist changes

## Dev Notes

- **QuickFixState schema**: `{ logoFix?: { x, y, width, height }, subtitleFix?: {...} }`. Stored trong Segment object, persisted vào `segments/{videoId}.json`.
- **Review status transitions**: `Unreviewed` → `Modified` (after fix applied) | `Accepted` (after mark reviewed). `Modified` → `Accepted` chỉ qua mark-reviewed. `Accepted/Modified` → `Unreviewed` chỉ qua reset.
- **Backend authoritative**: tất cả state changes phải qua Tauri command và persist. Frontend không tự mutate segment state.
- **Invalidate preview cache**: khi `apply_logo_fix`, phải xóa cache entry `{job}/cache/previews/{segmentId}-*.mp4` để Layer 2 preview tự rebuild.
- **Compatible check cho bulk**: chỉ cần cùng issueType. Không cần cùng time range hay cùng confidence score.
- **Immutability**: khi update segment, tạo copy mới của Segment object, không mutate in-place. Đảm bảo cả Rust side và TS side đều theo immutable pattern.

### Project Structure Notes

- Frontend: `src/modules/segment-review/QuickFixPanel.tsx` (logo section), `src/modules/segment-review/LogoControls.tsx`
- Backend: `src-tauri/src/commands/review_commands.rs` (`apply_logo_fix`, `mark_segment_accepted`, `reset_logo_fix`), `src-tauri/src/services/review_service.rs`
- Storage: `{job}/segments/{videoId}.json` (QuickFixState), `{job}/cache/previews/` (invalidate on fix)

### References

- [Source: epics.md#Story 3.4] Acceptance criteria
- [Source: prd.md#7.1] FR16: apply fix, export gating; FR10: overlay handles
- [Source: ux-design-specification.md#6.5] Quick Fix Panel (UX-DR11)
- [Source: architecture.md#5.4] QuickFixState domain model
- [Source: architecture.md#13.1] Review commands: applyFix, markAccepted

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
