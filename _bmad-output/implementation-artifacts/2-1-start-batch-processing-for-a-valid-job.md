# Story 2.1: Start Batch Processing for a Valid Job

Status: ready-for-dev

## Story

As a user,
I want chạy auto process cho một job đã hợp lệ,
so that hệ thống có thể bắt đầu xử lý hàng loạt các video mà không cần tôi thao tác từng file.

## Acceptance Criteria

1. Job đã qua readiness check → bấm "Chạy tự động" → job chuyển sang trạng thái `Processing`, tạo queue cho từng video.
2. Job chưa hợp lệ hoặc còn blocker → hệ thống không cho start → hiển thị lý do rõ ràng.
3. Job đã đang chạy → không tạo processing run trùng lặp → giữ trạng thái nhất quán.
4. Queue chạy trên background thread → UI vẫn phản hồi bình thường, không bị block.

## Tasks / Subtasks

- [ ] Implement `start_job` Tauri command (AC: 1, 2, 3)
  - [ ] Validate job đã qua readiness check (preJobValidation)
  - [ ] Nếu job chưa sẵn sàng: trả về lỗi với danh sách blocker
  - [ ] Nếu job đang chạy: return early, không spawn run mới
  - [ ] Transition job state → `Processing`
  - [ ] Enqueue mỗi VideoItem vào processing queue
  - [ ] Persist job state sau khi start
- [ ] Implement `job_orchestrator` processing queue (AC: 4)
  - [ ] Spawn async Tokio task để chạy queue trong nền
  - [ ] Process videos tuần tự hoặc có thể concurrent (V1: tuần tự)
  - [ ] Tách processing thread khỏi Tauri main thread
  - [ ] Trả về ngay lập tức sau khi enqueue (non-blocking)
- [ ] Emit job lifecycle events (AC: 1, 4)
  - [ ] `jobStarted` khi job bắt đầu
  - [ ] `videoProcessingStarted { videoId }` khi bắt đầu từng video
  - [ ] Frontend subscribe và update UI state
- [ ] Frontend: "Chạy tự động" button handler (AC: 1, 2)
  - [ ] Call `start_job` command
  - [ ] Navigate đến Processing Queue screen nếu thành công
  - [ ] Hiển thị error inline nếu thất bại

## Dev Notes

- **Non-blocking là bắt buộc** (NFR1): `start_job` command phải trả về ngay, không block UI trong khi queue chạy. Dùng `tokio::spawn` trong Rust.
- **Idempotency**: nếu job đang ở trạng thái `Processing`, gọi `start_job` lần 2 phải an toàn — return existing state, không tạo duplicate queue.
- **Job state machine**: `Draft/ReadyToRun` → `Processing` (valid transition); `Processing` → `Processing` (no-op); bất kỳ trạng thái khác → error.
- **Per-video queue**: mỗi VideoItem được enqueue như một job unit riêng. Failure của một video không ảnh hưởng các video khác (NFR8).
- **Pre-job validation**: `start_job` phải re-validate ở server side (Rust), không chỉ tin vào frontend state. Frontend gating là UX convenience, không phải security.
- **Events**: dùng Tauri event system (`app_handle.emit_all`) để push trạng thái về frontend.

### Project Structure Notes

- Frontend: `src/modules/processing-queue/ProcessingQueueScreen.tsx` (navigate after start)
- Backend: `src-tauri/src/commands/job_commands.rs` (`start_job`), `src-tauri/src/services/job_orchestrator.rs` (queue runner), `src-tauri/src/domain/job.rs` (state machine)
- Events: `jobStarted`, `videoProcessingStarted` (Application → UI)

### References

- [Source: epics.md#Story 2.1] Acceptance criteria
- [Source: prd.md#7.1] FR8: queue nền và batch processing
- [Source: architecture.md#7.1] Pre-run pipeline validation
- [Source: architecture.md#13.1] startJob command
- [Source: architecture.md#13.2] jobStarted, videoProcessingStarted events

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
