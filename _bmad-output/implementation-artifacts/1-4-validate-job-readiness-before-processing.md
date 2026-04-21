# Story 1.4: Validate Job Readiness Before Processing

Status: ready-for-dev

## Story

As a user,
I want hệ thống kiểm tra readiness của toàn bộ job trước khi chạy,
so that tôi chỉ có thể bấm xử lý khi mọi blocker bắt buộc đã được giải quyết.

## Acceptance Criteria

1. Bảng readiness hiển thị mỗi video với: task, preset, logo, audio, SRT, trạng thái mapping.
2. Video còn thiếu input bắt buộc hoặc mapping chưa hợp lệ: đánh dấu blocked với lý do cụ thể, dễ hiểu.
3. Job còn ít nhất một video bị blocked bắt buộc: nút "Chạy tự động" disabled, hiển thị rõ lý do job chưa đủ điều kiện.
4. Tất cả video có input hợp lệ và mapping hợp lệ theo task: job sẵn sàng, "Chạy tự động" enabled.

## Tasks / Subtasks

- [ ] Implement readiness validation logic trong `job_orchestrator` (AC: 2, 3, 4)
  - [ ] Với mỗi video: check mapping completeness theo `selectedTask`
  - [ ] Task "Thay audio": yêu cầu audio mapping hợp lệ
  - [ ] Task "Thay subtitle": yêu cầu SRT mapping hợp lệ
  - [ ] Task "Thay logo": yêu cầu logo hợp lệ (từ preset hoặc override)
  - [ ] Task "Thay logo, audio, và subtitle": yêu cầu tất cả
  - [ ] Trả về per-video readiness: `{ videoId, isReady, blockers: string[] }`
- [ ] Hiển thị readiness table trong Job Setup screen (AC: 1, 2)
  - [ ] Reuse/extend `MappingTable` từ Story 1.3
  - [ ] Thêm column "Trạng thái" với Blocked / Ready badge
  - [ ] Expand row để show blockers list khi Blocked
  - [ ] Vietnamese blocker messages: "Chưa có audio" / "Chưa có SRT" / "Mapping chưa được xác nhận"
- [ ] "Chạy tự động" button gating (AC: 3, 4)
  - [ ] Button disabled nếu `anyVideosBlocked === true`
  - [ ] Tooltip hoặc inline message giải thích tại sao disabled
  - [ ] Khi tất cả resolved: button enabled ngay lập tức (reactive)
- [ ] Expose readiness state qua Zustand (AC: 4)
  - [ ] `jobStore.readinessState`: per-video readiness map
  - [ ] Update khi mapping thay đổi (listen `mappingUpdated` event)

## Dev Notes

- **Task-based gating**: readiness phụ thuộc vào `selectedTask`, không phải luôn cần cả logo + audio + SRT. Ví dụ task "Thay logo" không cần SRT.
- **Readiness là reactive**: mỗi khi user sửa mapping (fixMapping), readiness phải tự tính lại và update button state ngay.
- **No processing if blocked**: job_orchestrator phải check readiness trước khi start processing, ngay cả khi frontend bị bypass. Server-side validation là source of truth.
- **Readiness table** là extension của mapping table từ Story 1.3 — không tạo duplicate table component. Thêm readiness column vào existing table.
- Copy guidelines: tránh "invalid", "error", "failed" — dùng "Chưa có audio", "Cần bạn xác nhận file này".

### Project Structure Notes

- Frontend: `src/modules/job-review/JobSetupScreen.tsx` (add readiness column), nút "Chạy tự động"
- Backend: `src-tauri/src/services/job_orchestrator.rs` (validate_readiness), `src-tauri/src/commands/job_commands.rs` (get_job_readiness)
- Store: `src/store/jobStore.ts` (readinessState slice)

### References

- [Source: epics.md#Story 1.4] Acceptance criteria
- [Source: prd.md#7.1] Job gating rules (FR15, FR16)
- [Source: ux-design-specification.md#6.3] Pre-run screen behavior
- [Source: architecture.md#7.1] Pre-run pipeline validation steps
- [Source: architecture.md#14] Error handling: fail early on invalid mappings

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
