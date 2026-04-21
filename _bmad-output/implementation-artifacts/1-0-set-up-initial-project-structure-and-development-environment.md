# Story 1.0: Set Up Initial Project Structure and Development Environment

Status: ready-for-dev

## Story

As a developer,
I want thiết lập project structure ban đầu từ template,
so that toàn bộ team có baseline chung để bắt đầu implement.

## Acceptance Criteria

1. Project chạy được với Tauri + React + TypeScript đã cấu hình; FFmpeg binary accessible từ application layer.
2. Dev server mở được app desktop với shell cơ bản (empty state) không có lỗi compile.
3. Build thành công tạo ra Windows executable; thư mục output structure cho job data đã được defined trong constants.
4. Các frontend và backend module skeleton folders đã tồn tại theo đúng cấu trúc trong architecture.

## Tasks / Subtasks

- [ ] Init Tauri project với React + TypeScript template, target Windows x64 (AC: 1)
  - [ ] `cargo tauri init`, chọn Windows target, cấu hình `tauri.conf.json`
  - [ ] Cài dependencies: React 18, TypeScript 5, Vite
  - [ ] Cài Zustand cho state management
- [ ] Bundle FFmpeg Windows binaries (AC: 1)
  - [ ] Download ffmpeg.exe + ffprobe.exe (Windows build, static)
  - [ ] Đặt vào `src-tauri/resources/ffmpeg/`
  - [ ] Cấu hình `tauri.conf.json` resources để bundle cùng app
  - [ ] Expose FFmpeg path từ Rust side qua Tauri Command `get_ffmpeg_path`
- [ ] Tạo frontend module skeleton (AC: 4)
  - [ ] `src/modules/app-shell/`
  - [ ] `src/modules/start-flow/`
  - [ ] `src/modules/preset-management/`
  - [ ] `src/modules/job-review/`
  - [ ] `src/modules/processing-queue/`
  - [ ] `src/modules/segment-review/`
  - [ ] `src/modules/export-reporting/`
  - [ ] Mỗi module có `index.ts` export placeholder
- [ ] Tạo backend service skeleton trong Rust (AC: 4)
  - [ ] `src-tauri/src/services/job_orchestrator.rs`
  - [ ] `src-tauri/src/services/preset_service.rs`
  - [ ] `src-tauri/src/services/mapping_service.rs`
  - [ ] `src-tauri/src/services/audio_policy_service.rs`
  - [ ] `src-tauri/src/services/analysis_service.rs`
  - [ ] `src-tauri/src/services/risk_service.rs`
  - [ ] `src-tauri/src/services/review_service.rs`
  - [ ] `src-tauri/src/services/render_service.rs`
  - [ ] `src-tauri/src/services/persistence_service.rs`
  - [ ] `src-tauri/src/services/logging_service.rs`
- [ ] Define job output directory constants (AC: 3)
  - [ ] `src-tauri/src/constants.rs`: `JOB_MANIFEST`, `VIDEO_STATE_DIR`, `SEGMENT_STATE_DIR`, `REPORTS_DIR`, `LOGS_DIR`, `CACHE_DIR`, `PREVIEW_CACHE_DIR`
  - [ ] Structure: `{app_data}/jobs/{jobId}/job.json`, `videos/`, `segments/`, `reports/`, `logs/`, `cache/previews/`
- [ ] Smoke test build (AC: 2, 3)
  - [ ] `cargo tauri dev` → app mở không lỗi
  - [ ] `cargo tauri build` → Windows .exe tạo ra thành công
  - [ ] FFmpeg path accessible: Tauri Command `get_ffmpeg_path` trả về valid path

## Dev Notes

- **Platform: Windows 10/11 x64 ONLY** — không cần cross-platform abstraction ở V1.
- **FFmpeg**: dùng static Windows build (không cần user install thêm). Không gọi `ffmpeg` từ PATH — luôn dùng bundled binary path từ `get_ffmpeg_path` command.
- **Zustand** là state management library cho toàn bộ frontend — không dùng Redux.
- **Tauri Command pattern**: tất cả frontend → backend calls đều qua `invoke('command_name', args)`. Không dùng REST API internal.
- **Separation of concerns**: UI state tạm thời (Zustand) ≠ domain review state (persisted JSON) ≠ FFmpeg execution details. Ba lớp này phải tách rõ.
- Story này là **developer story**, không phải user story — không có UI yêu cầu đặc biệt, chỉ cần shell chạy được.

### Project Structure Notes

```
desktop-video-rebranding-app/
├── src/                          # React + TypeScript frontend
│   ├── modules/
│   │   ├── app-shell/
│   │   ├── start-flow/
│   │   ├── preset-management/
│   │   ├── job-review/
│   │   ├── processing-queue/
│   │   ├── segment-review/
│   │   └── export-reporting/
│   ├── store/                    # Zustand stores
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── commands/             # Tauri #[tauri::command] handlers
│   │   ├── services/             # Business logic services
│   │   ├── domain/               # Domain models (Job, VideoItem, Segment, etc.)
│   │   ├── constants.rs
│   │   └── main.rs
│   ├── resources/
│   │   └── ffmpeg/
│   │       ├── ffmpeg.exe
│   │       └── ffprobe.exe
│   └── tauri.conf.json
```

### References

- [Source: architecture.md#3.0] Platform target: Windows only
- [Source: architecture.md#3.1] Tauri + React + TypeScript + Zustand
- [Source: architecture.md#3.2] FFmpeg bundled Windows build
- [Source: architecture.md#12] Module structure frontend + backend
- [Source: architecture.md#9.2] Persistence folder layout

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
