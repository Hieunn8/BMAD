# Story 3.6: Resolve Review Gating and Mark Video Ready

Status: done

## Story

As a user,
I want đánh dấu video đã vượt qua review gating,
so that video chỉ được chuyển sang `Ready to Export` khi mọi blocker bắt buộc đã được xử lý hoặc chấp nhận đúng cách.

## Acceptance Criteria

1. Video còn ≥1 segment `High Risk` chưa xử lý/xác nhận → block hành động "đánh dấu ready" → hiển thị rõ blockers còn lại.
2. Segment `Medium Risk` được user xem và chấp nhận → persist review decision → không bắt buộc user phải sửa.
3. Tất cả segment `High Risk` đã được sửa hoặc xác nhận đúng rule → video chuyển sang `Ready to Export` → persist quyết định cho bước export.

## Tasks / Subtasks

- [ ] Implement review gating logic trong `review_service` (AC: 1, 3)
  - [ ] `checkVideoReviewGating(videoId)`:
    - [ ] Load segments từ `segments/{videoId}.json`
    - [ ] Count `High Risk` segments với `reviewStatus = Unreviewed`
    - [ ] Nếu count > 0 → return `{ canProceed: false, blockers: [{ segmentId, timeRange, issueType }] }`
    - [ ] Nếu count = 0 → return `{ canProceed: true }`
  - [ ] High Risk resolution: `reviewStatus = Modified` (fix applied) | `Accepted` (accepted as-is) → cả hai đều hết blocked
- [ ] Implement `mark_video_ready` Tauri command (AC: 1, 3)
  - [ ] Re-run `checkVideoReviewGating` ở backend (server-side validation)
  - [ ] Nếu còn blockers: trả về error với danh sách blockers
  - [ ] Nếu pass: set `VideoItem.status = ReadyToExport`
  - [ ] Persist vào `videos/{videoId}.json`
  - [ ] Emit `videoReadyToExport { videoId }`
- [ ] Implement "Video Ready" button + blocker display (AC: 1, 3)
  - [ ] Button "Đánh dấu xong review" trong review workspace
  - [ ] Khi click: call `mark_video_ready`
  - [ ] Nếu có blockers: hiển thị danh sách inline `"Còn [N] đoạn High Risk chưa xử lý: [list]"`
  - [ ] Nếu pass: navigate hoặc update status badge
- [ ] Implement Medium Risk acceptance flow (AC: 2)
  - [ ] `mark_segment_accepted` command (đã implement trong Story 3.4) dùng cho cả Medium và High Risk
  - [ ] Sau khi accept Medium Risk segment: không tự mark video ready — user phải manually click "Đánh dấu xong review"
  - [ ] Accepted Medium Risk: không còn hiện là unreviewed nhưng vẫn track trong report
- [ ] Cập nhật video status badge trong review screen và processing queue (AC: 3)
  - [ ] Listen `videoReadyToExport` event
  - [ ] Update badge `Ready to Export` trong danh sách video

## Dev Notes

- **High Risk = blocker** (FR16): video chỉ được `ReadyToExport` khi không còn High Risk unreviewed. Medium/Low không block.
- **Double validation**: frontend disable nút khi còn blockers (UX convenience), nhưng `mark_video_ready` command ở Rust phải re-validate từ disk — không tin frontend state.
- **Segment reviewStatus hợp lệ cho export**: `Modified` (fix đã apply) hoặc `Accepted` (accepted as-is). `Unreviewed` với High Risk = vẫn blocked.
- **Gating tiêu thụ data từ Epic 2 + 3**: không tạo lại rule gating mới. Đọc từ `segments/{videoId}.json` là đủ.
- **Persist quyết định**: `VideoItem.status = ReadyToExport` phải được persist vào `videos/{videoId}.json` để Story 4.2 có thể đọc.
- **Batch review close**: nếu tất cả videos trong job đã `ReadyToExport`, job status → `ReadyToExport` và navigate về export screen.

### Project Structure Notes

- Frontend: `src/modules/segment-review/ReviewGatingPanel.tsx` (blockers display, "Đánh dấu xong review" button)
- Backend: `src-tauri/src/commands/review_commands.rs` (`mark_video_ready`), `src-tauri/src/services/review_service.rs` (`checkVideoReviewGating`)
- Storage: `{job}/videos/{videoId}.json` (status = ReadyToExport), `{job}/segments/{videoId}.json` (reviewStatus)
- Events: `videoReadyToExport { videoId }`

### References

- [Source: epics.md#Story 3.6] Acceptance criteria
- [Source: prd.md#7.1] FR16: risk levels, export gating (High Risk phải xử lý); FR20: review bắt buộc cho High Risk
- [Source: architecture.md#5.2] VideoItem status machine: ReviewNeeded → ReadyToExport
- [Source: architecture.md#5.3] Segment reviewStatus: Unreviewed → Modified | Accepted
- [Source: architecture.md#7.2] Export gating validation

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
