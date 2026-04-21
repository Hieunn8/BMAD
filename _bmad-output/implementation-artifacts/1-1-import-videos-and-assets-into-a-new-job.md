# Story 1.1: Import Videos and Assets Into a New Job

Status: ready-for-dev

## Story

As a user,
I want nhập video và asset vào một job mới,
so that tôi có thể bắt đầu một workflow re-branding với toàn bộ đầu vào cần thiết ở cùng một nơi.

## Acceptance Criteria

1. Kéo thả một hoặc nhiều file vào vùng nhập liệu → tạo draft job mới, hiển thị workspace với danh sách file đã nhập.
2. File được phân loại đúng vào nhóm `video`, `logo`, `audio`, hoặc `SRT` dựa trên extension.
3. File không hợp lệ hoặc không được hỗ trợ: không silently bỏ qua, hiển thị thông báo lỗi ngắn gọn.
4. Khi draft job được tạo: hệ thống persist cấu hình job (file list, task đã chọn, timestamp) vào local storage ngay lập tức.

## Tasks / Subtasks

- [ ] Tạo Start Screen component trong `start-flow` module (AC: 1)
  - [ ] Câu hỏi trung tâm: "Bạn muốn làm gì?"
  - [ ] 4 task cards: Thay logo / Thay audio / Thay subtitle / Thay logo, audio, và subtitle
  - [ ] Drag-and-drop zone chung cho tất cả file types
  - [ ] Nếu chưa chọn task: auto-gợi ý task từ loại file đã kéo vào
- [ ] Implement file role classification (AC: 2)
  - [ ] Video: `.mp4`, `.mov`, `.mkv`, `.avi`, `.webm`
  - [ ] Logo: `.png`, `.jpg`, `.jpeg`, `.svg`
  - [ ] Audio: `.mp3`, `.aac`, `.wav`, `.m4a`
  - [ ] SRT: `.srt`
  - [ ] Classify trong Rust side (`mapping_service.rs`) — không classify ở frontend
- [ ] Implement `import_assets` Tauri command (AC: 2, 3)
  - [ ] Nhận danh sách file paths từ frontend
  - [ ] Classify từng file
  - [ ] Trả về kết quả phân loại + list file bị từ chối (với lý do)
- [ ] Tạo Job domain model (AC: 1, 4)
  - [ ] `src-tauri/src/domain/job.rs`: struct `Job` với `jobId`, `createdAt`, `selectedTask`, `presetId`, `outputFolder`, `status`, `videoItems[]`
  - [ ] `src-tauri/src/domain/video_item.rs`: struct `VideoItem`
- [ ] Implement `create_job` Tauri command (AC: 4)
  - [ ] Tạo job với UUID jobId
  - [ ] Persist `job.json` ngay lập tức vào `{app_data}/jobs/{jobId}/`
  - [ ] Trả về jobId cho frontend
- [ ] Cập nhật Zustand job store (AC: 1)
  - [ ] `src/store/jobStore.ts`: slice cho current job state
  - [ ] Actions: `setJob`, `addFiles`, `setSelectedTask`
- [ ] Hiển thị imported files grouped by role (AC: 1, 2)
  - [ ] Section "Videos", "Logo", "Audio", "SRT" trong workspace
  - [ ] File count per section
- [ ] Error display cho file bị từ chối (AC: 3)
  - [ ] Inline error message, không dùng modal
  - [ ] Copy ví dụ: "File .xyz không được hỗ trợ"

## Dev Notes

- **File classification phải ở Rust side** (mapping_service) không phải frontend — để đảm bảo nhất quán với logic mapping sau này.
- **Supported input formats** (xem architecture ADR-07):
  - Video: MP4, MOV, MKV, AVI, WebM với H.264/H.265/VP8/VP9
  - Audio: MP3, AAC, WAV, M4A
- **Drag-and-drop**: Tauri hỗ trợ file drop events qua `listen('tauri://file-drop', ...)` — dùng event này thay vì HTML5 native drag-drop để tránh security restrictions với file paths.
- **Job persistence**: Luôn persist ngay sau khi tạo, không delay. Job folder tạo trước khi ghi file.
- **UX**: Start screen phải có density thấp (nhiều khoảng trắng, icon lớn, copy ngắn). Không hiển thị technical controls ở màn đầu.
- Câu hỏi "Bạn muốn làm gì?" phải là element lớn, trung tâm màn hình.
- **Task selection** ảnh hưởng đến validation sau này (FR15, FR16) — lưu `selectedTask` vào job state và persist.

### Project Structure Notes

- Frontend: `src/modules/start-flow/StartScreen.tsx`, `src/modules/start-flow/DragDropZone.tsx`
- Backend: `src-tauri/src/commands/job_commands.rs` (create_job, import_assets), `src-tauri/src/services/mapping_service.rs` (classify_files), `src-tauri/src/domain/job.rs`
- Store: `src/store/jobStore.ts`

### References

- [Source: epics.md#Story 1.1] Acceptance criteria
- [Source: prd.md#6.1] Start screen task selection flow
- [Source: prd.md#7.1] Ingest & Input Management rules
- [Source: ux-design-specification.md#6.1] Start Screen UX spec
- [Source: architecture.md#3.2] Input formats table
- [Source: architecture.md#5.1] Job domain model
- [Source: architecture.md#13.1] createJob, importAssets commands

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
