# Story 4.1: Resume an Interrupted Job from the Last Stable Checkpoint

Status: ready-for-dev

## Story

As a user,
I want mở lại một job đã bị gián đoạn từ checkpoint an toàn gần nhất,
so that tôi không phải làm lại từ đầu sau crash hoặc mất điện.

## Acceptance Criteria

1. Job đã persist với trạng thái hợp lệ → mở lại app hoặc chọn resume → khôi phục job manifest, per-video status, segment flags, quick-fix state → đưa về checkpoint ổn định gần nhất.
2. Một video đang export dở khi app bị gián đoạn → resume không bắt buộc tiếp tục giữa chừng encode → cho phép export lại video từ đầu an toàn.

## Tasks / Subtasks

- [ ] Implement `list_jobs` Tauri command
  - [ ] Scan `{app_data}/jobs/` directory
  - [ ] Load mỗi `job.json` file
  - [ ] Trả về danh sách jobs với: jobId, createdAt, status, videoCount, lastModified
  - [ ] Sort by lastModified descending
- [ ] Implement `load_job` Tauri command (AC: 1)
  - [ ] Nhận: `{ jobId }`
  - [ ] Load `job.json` (manifest)
  - [ ] Load tất cả `videos/{videoId}.json` (per-video status)
  - [ ] Load tất cả `segments/{videoId}.json` nếu tồn tại (flags + quickfix)
  - [ ] Trả về full job state object
- [ ] Implement checkpoint detection logic (AC: 1, 2)
  - [ ] Nếu job.status = `Processing` khi load (bị interrupt mid-processing):
    - [ ] Reset status về `ReadyToRun` (safe checkpoint trước khi chạy)
    - [ ] Log warning: `"Job bị gián đoạn khi đang xử lý, reset về trạng thái sẵn sàng chạy"`
  - [ ] Nếu job.status = `Exporting` khi load (bị interrupt mid-export):
    - [ ] Tìm videos có `status = Exporting` → reset về `ReadyToExport`
    - [ ] Các videos đã `status = Exported` (completed) → giữ nguyên
  - [ ] Các trạng thái khác (Draft, ReviewPending, ReadyToExport, Completed) → giữ nguyên
- [ ] Implement Job List screen (Start screen enhancement)
  - [ ] Hiển thị danh sách recent jobs khi mở app
  - [ ] Mỗi job row: job name/date, status badge, video count, last modified
  - [ ] Button "Tiếp tục" → gọi `load_job` + navigate đến đúng screen
  - [ ] Button "Tạo job mới" → go to Start screen
- [ ] Cập nhật Zustand store khi load job (AC: 1)
  - [ ] `jobStore.loadJob(jobData)` → populate toàn bộ store từ loaded data
  - [ ] Navigate đến đúng screen dựa trên job.status sau checkpoint adjustment

## Dev Notes

- **Safe checkpoint strategy**: trạng thái an toàn nhất là `ReadyToRun` (trước processing) và `ReviewPending` (sau processing, trước export). Khi resume, snap về trạng thái safe gần nhất.
- **Segment data preserved**: `segments/{videoId}.json` chứa quick-fix state từ Epic 3. Phải load hoàn toàn để user không mất review work đã làm.
- **In-progress encode = unsafe**: FFmpeg process bị kill giữa chừng có thể tạo ra corrupt output file. Tốt nhất là xóa file dở và export lại. Đây là lý do AC2 không force-resume encode.
- **Working files**: `{job}/working/*.mp4` intermediate files. Khi resume, kiểm tra integrity nếu cần (V1: không cần checksum, chỉ check file exists và kích thước > 0).
- **Job discovery**: `{app_data}/jobs/` scan toàn bộ subdirectory, mỗi job là một folder có `job.json`. Không có global registry file — disk scan là cách duy nhất.
- **App startup flow**: Start screen → check nếu có recent jobs → show Job List. User có thể bypass về New Job.

### Project Structure Notes

- Frontend: `src/modules/start-flow/StartScreen.tsx` (add recent jobs section), `src/modules/start-flow/JobListItem.tsx`
- Backend: `src-tauri/src/commands/job_commands.rs` (`list_jobs`, `load_job`), `src-tauri/src/services/persistence_service.rs` (load all job files), `src-tauri/src/services/job_orchestrator.rs` (checkpoint detection)
- Store: `src/store/jobStore.ts` (`loadJob` action)

### References

- [Source: epics.md#Story 4.1] Acceptance criteria
- [Source: prd.md#7.1] FR9: persist và resume sau crash
- [Source: architecture.md#8] Persistence Service: job manifest, per-video, segments
- [Source: architecture.md#5.1] Job status machine: Processing, Exporting trạng thái có thể interrupt

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
