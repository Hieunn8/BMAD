# Story 1.1: Import Videos and Assets Into a New Job

Status: done

## Story

As a user,
I want nhap video va asset vao mot job moi,
so that toi co the bat dau mot workflow re-branding voi toan bo dau vao can thiet o cung mot noi.

## Acceptance Criteria

1. Keo tha mot hoac nhieu file vao vung nhap lieu -> tao draft job moi, hien thi workspace voi danh sach file da nhap.
2. File duoc phan loai dung vao nhom `video`, `logo`, `audio`, hoac `SRT` dua tren extension.
3. File khong hop le hoac khong duoc ho tro: khong silently bo qua, hien thi thong bao loi ngan gon.
4. Khi draft job duoc tao: he thong persist cau hinh job (file list, task da chon, timestamp) vao local storage ngay lap tuc.

## Tasks / Subtasks

- [x] Tao Start Screen component trong `start-flow` module (AC: 1)
  - [x] Cau hoi trung tam: "Ban muon lam gi?"
  - [x] 4 task cards: Thay logo / Thay audio / Thay subtitle / Thay logo, audio, va subtitle
  - [x] Drag-and-drop zone chung cho tat ca file types
  - [x] Neu chua chon task: auto-goi y task tu loai file da keo vao
- [x] Implement file role classification (AC: 2)
  - [x] Video: `.mp4`, `.mov`, `.mkv`, `.avi`, `.webm`
  - [x] Logo: `.png`, `.jpg`, `.jpeg`, `.svg`
  - [x] Audio: `.mp3`, `.aac`, `.wav`, `.m4a`
  - [x] SRT: `.srt`
  - [x] Classify trong Rust side (`mapping_service.rs`) thay vi classify o frontend
- [x] Implement `import_assets` Tauri command (AC: 2, 3)
  - [x] Nhan danh sach file paths tu frontend
  - [x] Classify tung file
  - [x] Tra ve ket qua phan loai + list file bi tu choi (voi ly do)
- [x] Tao Job domain model (AC: 1, 4)
  - [x] `src-tauri/src/domain/job.rs`: struct `Job` voi `jobId`, `createdAt`, `selectedTask`, `presetId`, `outputFolder`, `status`, `videoItems[]`
  - [x] `src-tauri/src/domain/video_item.rs`: struct `VideoItem`
- [x] Implement `create_job` Tauri command (AC: 4)
  - [x] Tao job voi UUID `jobId`
  - [x] Persist `job.json` ngay lap tuc vao `{app_data}/jobs/{jobId}/`
  - [x] Tra ve job cho frontend
- [x] Cap nhat Zustand job store (AC: 1)
  - [x] `src/store/jobStore.ts`: slice cho current job state
  - [x] Actions: `setJob`, `addFiles`, `setSelectedTask`
- [x] Hien thi imported files grouped by role (AC: 1, 2)
  - [x] Section "Videos", "Logo", "Audio", "SRT" trong workspace
  - [x] File count per section
- [x] Error display cho file bi tu choi (AC: 3)
  - [x] Inline error message, khong dung modal
  - [x] Copy theo mau: "File .xyz khong duoc ho tro"

## Dev Notes

- File classification o Rust side (`mapping_service`) thay vi frontend de giu logic nhat quan voi mapping sau nay.
- Supported input formats:
  - Video: MP4, MOV, MKV, AVI, WebM
  - Audio: MP3, AAC, WAV, M4A
- Job persistence phai xay ra ngay sau khi tao hoac cap nhat draft job.
- Start screen uu tien low-density layout, copy ngan, khong mo technical controls o man dau.
- `selectedTask` phai duoc luu vao job state va persist vi anh huong den validation sau nay.

### Project Structure Notes

- Frontend: `src/modules/start-flow/StartScreen.tsx`, `src/modules/start-flow/DragDropZone.tsx`
- Backend: `src-tauri/src/commands/job_commands.rs`, `src-tauri/src/services/mapping_service.rs`, `src-tauri/src/domain/job.rs`
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

GPT-5 Codex

### Debug Log References

- `npm test`
- `npm run build`
- `cargo check --message-format short`
- `npm run dev -- --host 127.0.0.1 --port 4173`
- `Invoke-WebRequest http://127.0.0.1:4174` -> `200 OK`

### Completion Notes List

- Start flow UI da co `StartScreen` va `DragDropZone` voi 4 task cards, auto-suggest task, workspace grouped theo role, va inline rejected-file state.
- `jobStore` da track `currentJob`, `importedFiles`, `rejectedFiles`, `selectedTask`, kem merge/dedupe behavior khi import nhieu luot file.
- Rust side da implement `MappingService::classify_files`, `import_assets`, `create_job`, `Job`, `VideoItem`, va persist draft `job.json` ngay khi tao/cap nhat job.
- Smoke verify da du: Vite launch duoc tren localhost, frontend tests pass, production build pass, va `cargo check` pass voi warning scaffold chua dung.

### File List

- `src/modules/start-flow/StartScreen.tsx`
- `src/modules/start-flow/DragDropZone.tsx`
- `src/modules/start-flow/types.ts`
- `src/modules/start-flow/taskSuggestion.test.ts`
- `src/store/jobStore.ts`
- `src/App.tsx`
- `src-tauri/src/commands/job_commands.rs`
- `src-tauri/src/domain/job.rs`
- `src-tauri/src/domain/video_item.rs`
- `src-tauri/src/services/mapping_service.rs`
