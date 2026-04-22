# Story 3.1: Open a Review Workspace for Flagged Videos

Status: done

## Story

As a user,
I want mở một review workspace cho các video có exception,
so that tôi đi thẳng vào các video và segment thực sự cần xác nhận hoặc sửa.

## Acceptance Criteria

1. Job có ít nhất một video chứa flags/exception artifacts → review hiển thị chỉ các video có exception + số segment liên quan → nạp segment list, risk data, preview context từ job snapshot đã persist.
2. Người dùng muốn spot-check video không có flag → vẫn mở được review workspace → hiển thị rõ đây là review tùy chọn, không phải blocker.

## Tasks / Subtasks

- [ ] Tạo `SegmentReviewScreen.tsx` với 3-column layout (AC: 1, 2)
  - [ ] Column 1 (left): Video list + Segment list cho video đang chọn
  - [ ] Column 2 (center): Preview Workspace
  - [ ] Column 3 (right): Quick Fix Panel (placeholder — implement trong Story 3.4, 3.5)
  - [ ] Timeline bar ở bottom (định vị segment, không phải editing tool)
- [ ] Implement `get_review_context` Tauri command (AC: 1)
  - [ ] Load `segments/{videoId}.json` cho video được chọn
  - [ ] Load `videos/{videoId}.json` cho per-video status
  - [ ] Trả về: `{ segments, videoStatus, previewBaseUrl }`
  - [ ] Cache review context trong Zustand `reviewStore`
- [ ] Implement Video list component trong review screen (AC: 1, 2)
  - [ ] Chỉ hiển thị videos có `status = ReviewNeeded` mặc định
  - [ ] Option toggle: "Hiển thị tất cả video" để spot-check video không có flag
  - [ ] Badge số segment cần review trên mỗi video
  - [ ] Khi video không có flag và mở bằng toggle: hiển thị `"Không có đoạn bắt buộc phải sửa"` banner (UX-DR17)
- [ ] Implement Segment list component (AC: 1)
  - [ ] Mỗi segment row: time range, issue type (Logo / Subtitle), risk badge, review status
  - [ ] Click → select segment, scroll preview đến đúng time range
  - [ ] Risk badge colors: High Risk (red), Medium Risk (amber), Low Risk (green) + icon + label (UX-DR16)
- [ ] Implement `get_video_preview` Tauri command (AC: 1)
  - [ ] Trả về path đến intermediate file của video (sau logo/subtitle processing)
  - [ ] Frontend load video qua Tauri asset protocol

## Dev Notes

- **3-column layout** là bắt buộc theo UX-DR8. Timeline ở bottom là navigation aid, không phải editable timeline.
- **Exception-based review** (NFR9): mặc định chỉ hiển thị videos cần review. Người dùng không cần xem lại toàn bộ video.
- **Preview video**: dùng intermediate working file (`{job}/working/{videoId}_*.mp4`), không phải source video gốc.
- **Tauri asset protocol**: để load local video files trong frontend, cần whitelist path qua Tauri's `asset://` protocol. Xem tài liệu Tauri về `convertFileSrc`.
- **reviewStore** (Zustand): lưu `{ selectedVideoId, selectedSegmentId, segments, videoList }`. Không re-fetch từ disk mỗi lần select segment.
- **Responsive collapse** (UX-DR18): khi window hẹp, Quick Fix Panel có thể collapse. Segment list phải luôn visible.

### Project Structure Notes

- Frontend: `src/modules/segment-review/SegmentReviewScreen.tsx`, `src/modules/segment-review/VideoList.tsx`, `src/modules/segment-review/SegmentList.tsx`, `src/modules/segment-review/PreviewWorkspace.tsx`
- Backend: `src-tauri/src/commands/review_commands.rs` (`get_review_context`, `get_video_preview`), `src-tauri/src/services/review_service.rs`
- Store: `src/store/reviewStore.ts`

### References

- [Source: epics.md#Story 3.1] Acceptance criteria
- [Source: prd.md#7.1] FR15: segment list là review UI chính; FR20: review thủ công khi không có flag
- [Source: ux-design-specification.md#6.5] Review Exceptions screen: 3-column layout (UX-DR8)
- [Source: architecture.md#5] Review domain model, Segment data
- [Source: architecture.md#13.1] Review commands

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- `cargo check --message-format short`
- `cargo test`
- `npm run build`
- `npm test`

### Completion Notes List

- Da them `review_commands.rs` voi `get_review_context` va `get_video_preview`, expose vao Tauri invoke handler.
- Da them `review_service.rs` de load video state, segment artifacts logo/subtitle, va resolve preview working file uu tien cao nhat.
- Da them `reviewStore` de cache review context theo video, selected state, preview path, va co the mo/dong review workspace.
- Da them `SegmentReviewScreen` voi 3 cot, `VideoList`, `PreviewWorkspace`, placeholder Quick Fix Panel, timeline marker, va toggle hien thi tat ca video.
- Da noi AppShell/JobSetup/ProcessingQueue de auto/open review workspace cho `ReviewPending` va spot-check `ReadyToExport`.

### File List

- `src-tauri/src/commands/review_commands.rs`
- `src-tauri/src/services/review_service.rs`
- `src-tauri/src/lib.rs`
- `src/store/reviewStore.ts`
- `src/modules/segment-review/SegmentReviewScreen.tsx`
- `src/modules/segment-review/VideoList.tsx`
- `src/modules/segment-review/PreviewWorkspace.tsx`
- `src/modules/app-shell/AppShell.tsx`
- `src/modules/job-review/JobSetupScreen.tsx`
- `src/modules/processing-queue/ProcessingQueueScreen.tsx`
- `src/modules/start-flow/types.ts`
- `src/styles.css`
