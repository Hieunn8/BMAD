# Story 1.0: Set Up Initial Project Structure and Development Environment

Status: done

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

- [x] Init Tauri project với React + TypeScript template, target Windows x64 (AC: 1)
  - [x] `cargo tauri init`, chọn Windows target, cấu hình `tauri.conf.json`
  - [x] Cài dependencies: React 18, TypeScript 5, Vite
  - [x] Cài Zustand cho state management
- [x] Bundle FFmpeg Windows binaries (AC: 1)
  - [x] Download ffmpeg.exe + ffprobe.exe (Windows build, static)
  - [x] Đặt vào `src-tauri/resources/ffmpeg/`
  - [x] Cấu hình `tauri.conf.json` resources để bundle cùng app
  - [x] Expose FFmpeg path từ Rust side qua Tauri Command `get_ffmpeg_path`
- [x] Tạo frontend module skeleton (AC: 4)
  - [x] `src/modules/app-shell/`
  - [x] `src/modules/start-flow/`
  - [x] `src/modules/preset-management/`
  - [x] `src/modules/job-review/`
  - [x] `src/modules/processing-queue/`
  - [x] `src/modules/segment-review/`
  - [x] `src/modules/export-reporting/`
  - [x] Mỗi module có `index.ts` export placeholder
- [x] Tạo backend service skeleton trong Rust (AC: 4)
  - [x] `src-tauri/src/services/job_orchestrator.rs`
  - [x] `src-tauri/src/services/preset_service.rs`
  - [x] `src-tauri/src/services/mapping_service.rs`
  - [x] `src-tauri/src/services/audio_policy_service.rs`
  - [x] `src-tauri/src/services/analysis_service.rs`
  - [x] `src-tauri/src/services/risk_service.rs`
  - [x] `src-tauri/src/services/review_service.rs`
  - [x] `src-tauri/src/services/render_service.rs`
  - [x] `src-tauri/src/services/persistence_service.rs`
  - [x] `src-tauri/src/services/logging_service.rs`
- [x] Define job output directory constants (AC: 3)
  - [x] `src-tauri/src/constants.rs`: `JOB_MANIFEST`, `VIDEO_STATE_DIR`, `SEGMENT_STATE_DIR`, `REPORTS_DIR`, `LOGS_DIR`, `CACHE_DIR`, `PREVIEW_CACHE_DIR`
  - [x] Structure: `{app_data}/jobs/{jobId}/job.json`, `videos/`, `segments/`, `reports/`, `logs/`, `cache/previews/`
- [x] Smoke test build (AC: 2, 3)
  - [x] `cargo tauri dev` → app mở không lỗi
  - [x] `cargo tauri build` → Windows .exe tạo ra thành công
  - [x] FFmpeg path accessible: Tauri Command `get_ffmpeg_path` trả về valid path

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

GPT-5 Codex

### Debug Log References

- `npm test`
- `npm run build`
- `cargo test`
- `npm run tauri build`
- `npm run tauri dev` (xác nhận process dev giữ ổn định sau khi compile và không báo lỗi startup)

### Completion Notes List

- Đã bootstrap project Tauri 2 + React 18 + TypeScript 5 + Vite trong repo root và thêm `zustand` cho app shell state.
- Đã bundle `ffmpeg.exe` / `ffprobe.exe` vào `src-tauri/resources/ffmpeg/`, khai báo resource trong `tauri.conf.json`, expose command `get_ffmpeg_path` và render path trong app shell.
- Đã tạo skeleton frontend modules, Zustand store, test Vitest cho empty state shell, cùng skeleton Rust cho `commands`, `domain`, `services`, `constants`.
- Đã xác nhận bundle build Windows x64 thành công: `src-tauri/target/release/bundle/nsis/Desktop Video Rebranding App_0.1.0_x64-setup.exe`.

### File List

- `.gitignore`
- `index.html`
- `package.json`
- `package-lock.json`
- `tsconfig.json`
- `tsconfig.node.json`
- `vite.config.ts`
- `public/tauri.svg`
- `public/vite.svg`
- `src/App.tsx`
- `src/main.tsx`
- `src/styles.css`
- `src/modules/app-shell/AppShell.tsx`
- `src/modules/app-shell/AppShell.test.tsx`
- `src/modules/app-shell/index.ts`
- `src/modules/start-flow/index.ts`
- `src/modules/preset-management/index.ts`
- `src/modules/job-review/index.ts`
- `src/modules/processing-queue/index.ts`
- `src/modules/segment-review/index.ts`
- `src/modules/export-reporting/index.ts`
- `src/store/app-shell-store.ts`
- `src/test/setup.ts`
- `src-tauri/Cargo.toml`
- `src-tauri/build.rs`
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`
- `src-tauri/resources/ffmpeg/ffmpeg.exe`
- `src-tauri/resources/ffmpeg/ffprobe.exe`
- `src-tauri/src/lib.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/constants.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/app.rs`
- `src-tauri/src/domain/mod.rs`
- `src-tauri/src/services/mod.rs`
- `src-tauri/src/services/job_orchestrator.rs`
- `src-tauri/src/services/preset_service.rs`
- `src-tauri/src/services/mapping_service.rs`
- `src-tauri/src/services/audio_policy_service.rs`
- `src-tauri/src/services/analysis_service.rs`
- `src-tauri/src/services/risk_service.rs`
- `src-tauri/src/services/review_service.rs`
- `src-tauri/src/services/render_service.rs`
- `src-tauri/src/services/persistence_service.rs`
- `src-tauri/src/services/logging_service.rs`
- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.icns`
- `src-tauri/icons/icon.ico`
- `src-tauri/icons/icon.png`
- `src-tauri/icons/Square30x30Logo.png`
- `src-tauri/icons/Square44x44Logo.png`
- `src-tauri/icons/Square71x71Logo.png`
- `src-tauri/icons/Square89x89Logo.png`
- `src-tauri/icons/Square107x107Logo.png`
- `src-tauri/icons/Square142x142Logo.png`
- `src-tauri/icons/Square150x150Logo.png`
- `src-tauri/icons/Square284x284Logo.png`
- `src-tauri/icons/Square310x310Logo.png`
- `src-tauri/icons/StoreLogo.png`

## Change Log

- 2026-04-21: Bootstrap project Tauri/React/TypeScript, bundle FFmpeg Windows binaries, thêm app shell + Zustand baseline, services/constants skeleton, frontend/Rust tests, và xác nhận Windows bundle build.
