# Story 2.5: Persist Processing Status and Flags

Status: ready-for-dev

## Story

As a user,
I want thấy trạng thái xử lý của từng video được cập nhật rõ ràng,
so that tôi biết pipeline đã chạy đến đâu và video nào có vấn đề cần xử lý tiếp.

## Acceptance Criteria

1. Từng video đi qua các bước processing → trạng thái per-video được cập nhật và persist.
2. Bước detect/transform sinh uncertainty hoặc lỗi → flags/exception artifacts được persist → UI không tự mở review ở Epic 2.
3. Toàn bộ queue xong hoặc partial stop → mỗi video được gán outcome phù hợp (`Review Needed`, `Ready to Export`, `Failed`) → data đủ cho Epic 3 dùng.
4. Processing screen hiển thị: progress tổng job, progress từng video, trạng thái hiện tại, panel log ngắn (detect logo, replace audio, detect subtitle, render subtitle, risk scoring). Video thiếu audio mapping bắt buộc được hiển thị trạng thái thiếu đó rõ ràng.

## Tasks / Subtasks

- [ ] Implement per-video state persistence trong `persistence_service` (AC: 1)
  - [ ] Update `videos/{videoId}.json` sau mỗi bước xử lý
  - [ ] Lưu: `status`, `currentStep`, `completedSteps[]`, `timestamp`
  - [ ] VideoItem status lifecycle: `Imported → Processing → ReviewNeeded | ReadyToExport | Failed`
- [ ] Implement exception artifact persistence (AC: 2)
  - [ ] Sau khi mỗi analysis step hoàn tất: ghi segments vào `segments/{videoId}.json`
  - [ ] Nếu segments có `High Risk`: set VideoItem.status = `ReviewNeeded`
  - [ ] Nếu không có High Risk: set `ReadyToExport` (có thể có Medium/Low risk nhưng không block)
  - [ ] Không trigger review UI từ Epic 2 — chỉ persist data
- [ ] Implement job summary aggregation (AC: 3)
  - [ ] Khi tất cả videos hoàn tất: tính job outcome
  - [ ] `job.status` → `ReviewPending` nếu có video ReviewNeeded, `ReadyToExport` nếu tất cả xong
  - [ ] Emit `jobProcessingCompleted { jobId, summary: { total, reviewNeeded, readyToExport, failed } }`
- [ ] Implement Processing Queue screen UI (AC: 4)
  - [ ] `ProcessingQueueScreen.tsx` với layout:
    - Overall progress bar: `X / N videos processed`
    - Per-video progress list: video name, current step, status badge
    - Log panel: stream recent log entries (detect logo, replace audio, etc.)
  - [ ] Subscribe to Tauri events: `videoProcessingStarted`, `processingStepUpdate`, `videoProcessingCompleted`
  - [ ] Nếu video thiếu audio mapping: hiển thị `"Chưa có audio — bỏ qua bước thay audio"` trong log
- [ ] Emit processing events cho real-time UI updates (AC: 4)
  - [ ] `processingStepUpdate { videoId, step, status, message }`
  - [ ] `videoProcessingCompleted { videoId, outcome, segmentCount, riskDistribution }`

## Dev Notes

- **Event-driven real-time update**: dùng Tauri event system để push từng bước. Frontend không poll — subscribe events.
- **Per-video state file** (`videos/{videoId}.json`) là source of truth cho progress. Phải được ghi ngay trước và sau mỗi bước để đảm bảo resume được (Story 4.1).
- **Segment persistence**: `segments/{videoId}.json` chứa danh sách Segment objects từ analysis (logo + subtitle). File này là input cho Epic 3 review.
- **Review gating logic**: chỉ segment có `riskLevel = High` mới block video (không cho sang `ReadyToExport`). Medium/Low risk → video vẫn `ReadyToExport` nhưng review khuyến khích.
- **Log panel**: hiển thị tối đa 50 entries gần nhất. Không load toàn bộ log file — dùng ring buffer hoặc paginated event.
- **Missing audio mapping**: nếu task yêu cầu audio nhưng không có mapping → log `"Bỏ qua bước thay audio: chưa có audio mapping"` → video tiếp tục pipeline (không fail). Đây là step skip, không phải lỗi.

### Project Structure Notes

- Frontend: `src/modules/processing-queue/ProcessingQueueScreen.tsx`
- Backend: `src-tauri/src/services/persistence_service.rs` (per-video state write), `src-tauri/src/services/job_orchestrator.rs` (emit events, aggregate)
- Storage: `{job}/videos/{videoId}.json`, `{job}/segments/{videoId}.json`
- Events: `processingStepUpdate`, `videoProcessingCompleted`, `jobProcessingCompleted`

### References

- [Source: epics.md#Story 2.5] Acceptance criteria
- [Source: prd.md#7.1] FR8: persist per-video status; FR9: persist job state
- [Source: ux-design-specification.md#6.4] Processing screen: progress total, per-video, log panel (UX-DR7)
- [Source: architecture.md#5] Domain models: VideoItem status, Segment riskLevel
- [Source: architecture.md#8] Persistence Service: per-video state, segments

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
