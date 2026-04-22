# Story 2.5: Persist Processing Status and Flags

Status: done

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

- [x] Implement per-video state persistence trong `persistence_service` (AC: 1)
  - [x] Update `videos/{videoId}.json` sau mỗi bước xử lý
  - [x] Lưu: `status`, `currentStep`, `completedSteps[]`, `timestamp`
  - [x] VideoItem status lifecycle: `Imported → Processing → ReviewNeeded | ReadyToExport | Failed`
- [x] Implement exception artifact persistence (AC: 2)
  - [x] Sau khi mỗi analysis step hoàn tất: ghi segments vào `segments/{videoId}.json`
  - [x] Nếu segments có `High Risk`: set VideoItem.status = `ReviewNeeded`
  - [x] Nếu không có High Risk: set `ReadyToExport` (có thể có Medium/Low risk nhưng không block)
  - [x] Không trigger review UI từ Epic 2 — chỉ persist data
- [x] Implement job summary aggregation (AC: 3)
  - [x] Khi tất cả videos hoàn tất: tính job outcome
  - [x] `job.status` → `ReviewPending` nếu có video ReviewNeeded, `ReadyToExport` nếu tất cả xong
  - [x] Emit `jobProcessingCompleted { jobId, summary: { total, reviewNeeded, readyToExport, failed } }`
- [x] Implement Processing Queue screen UI (AC: 4)
  - [x] `ProcessingQueueScreen.tsx` với layout:
    - Overall progress bar: `X / N videos processed`
    - Per-video progress list: video name, current step, status badge
    - Log panel: stream recent log entries (detect logo, replace audio, etc.)
  - [x] Subscribe to Tauri events: `videoProcessingStarted`, `processingStepUpdate`, `videoProcessingCompleted`
  - [x] Nếu video thiếu audio mapping: hiển thị `"Chưa có audio — bỏ qua bước thay audio"` trong log
- [x] Emit processing events cho real-time UI updates (AC: 4)
  - [x] `processingStepUpdate { videoId, step, status, message }`
  - [x] `videoProcessingCompleted { videoId, outcome, segmentCount, riskDistribution }`

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

claude-sonnet-4-6

### Debug Log References

- `cargo check --manifest-path src-tauri/Cargo.toml` — clean (5 warnings, no errors)
- `cargo test --manifest-path src-tauri/Cargo.toml` — 46/46 pass (9 new)
- `npx tsc --noEmit` — no TypeScript errors

### Completion Notes List

- Thêm `VideoProcessingState` struct + `make_video_state()` helper vào `persistence_service.rs`; `persist_video_state()` ghi `{job.output_folder}/videos/{videoId}.json` với status, currentStep, completedSteps, timestamp ISO-8601.
- Thêm `determine_video_outcome()`: high_risk_count > 0 → `ReviewNeeded`, else → `ReadyToExport`.
- Thêm `compute_job_summary()` và `determine_job_status()`: aggregates per-video outcomes → job status (`ReviewPending` / `ReadyToExport` / `ProcessedWithFailures`).
- Refactored `process_video()` to return `Result<VideoProcessingOutcome, String>` instead of `Result<(), String>`. Now: persists state before/after each step, emits `processingStepUpdate` before and after each pipeline step, tracks high/medium risk counts from detection results.
- Missing audio mapping trong `process_video()`: emit `processingStepUpdate` với status="skipped" thay vì fail. Video tiếp tục pipeline.
- Thêm 3 new event structs: `ProcessingStepUpdateEvent`, `VideoProcessingCompletedEvent`, `JobProcessingCompletedEvent` (với `RiskDistribution`, `JobSummary`).
- `process_job_queue()` updated: collect outcomes, emit `videoProcessingCompleted` với outcome/segmentCount/riskDistribution sau mỗi video; emit `jobProcessingCompleted` cuối queue.
- Frontend `ProcessingQueueScreen.tsx`: overall progress bar (%), per-video list với current step label, log panel ring buffer max 50 entries (auto-scroll), job summary panel sau khi hoàn tất.
- Types: 5 new event types added vào `types.ts`.
- 9 new tests: `persist_video_state`, path sanitization, `determine_video_outcome`, `compute_job_summary`, `determine_job_status`, risk distribution underflow guard.

### File List

- `src-tauri/src/services/persistence_service.rs`
- `src-tauri/src/services/job_orchestrator.rs`
- `src/modules/start-flow/types.ts`
- `src/modules/processing-queue/ProcessingQueueScreen.tsx`
- Review follow-up: persist fail state vao `videos/{videoId}.json` trong nhanh error cua queue de source-of-truth tren disk khop voi `job.json`.
