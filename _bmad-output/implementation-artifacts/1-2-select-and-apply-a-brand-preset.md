# Story 1.2: Select and Apply a Brand Preset

Status: done

## Story

As a user,
I want chon mot preset theo brand/channel cho job,
so that he thong co the ap dung default brand rules truoc khi toi xac nhan mapping va readiness.

## Acceptance Criteria

1. Man Preset Selection hien thi it nhat: brand name, logo mac dinh, audio policy, subtitle style preset, export preset cho preset dang chon.
2. Khi preset duoc ap dung: luu vao job state va cap nhat cac default brand settings.
3. Khi doi preset sau khi job da co mapping hoac review data: hien thi canh bao rang mapping hoac ket qua detect truoc do co the mat hieu luc.

## Tasks / Subtasks

- [x] Tao Preset Selection screen trong `preset-management` module (AC: 1)
  - [x] Danh sach preset ben trai (list voi brand name)
  - [x] Preset preview card ben phai: brand name, logo thumbnail, audio policy label, subtitle style name, export preset name, notes
  - [x] Actions: "Chon preset nay", "Sua preset", "Tao preset moi"
- [x] Implement `select_preset` Tauri command (AC: 2)
  - [x] Gan `presetId` vao current job
  - [x] Resolve preset assets: `defaultLogoPath`, `audioReplacementPolicy`, `subtitleStylePreset`, `layoutRules`, `exportPreset`
  - [x] Persist updated job state (`presetId`) vao `job.json`
  - [x] Tra ve full preset data cho frontend
- [x] Detect preset change warning (AC: 3)
  - [x] Check neu job da co segment flags hoac review data
  - [x] Neu co: hien thi warning inline truoc khi apply
  - [x] User phai confirm de tiep tuc
- [x] Implement preset data persistence (AC: 2)
  - [x] Preset domain model: `src-tauri/src/domain/preset.rs`
  - [x] Presets luu tach biet tai `{app_data}/presets/{presetId}.json`
- [x] Cap nhat Zustand store (AC: 2)
  - [x] `jobStore.setPreset(preset)` luu full preset object vao state
  - [x] Downstream components co the doc preset tu store

## Dev Notes

- Preset duoc the hien nhu goi thuong hieu, khong phai config ky thuat thuan tuy.
- Preset storage tach rieng khoi job o `app_data/presets/` de co the tai su dung cho nhieu jobs.
- Warning chi hien khi preset change co nguy co lam stale data downstream trong `videos/` hoac `segments/`.
- UX dung inline warning va explicit confirm button, khong mo popup/dialog.

### Project Structure Notes

- Frontend: `src/modules/preset-management/PresetSelectionScreen.tsx`, `src/modules/preset-management/PresetCard.tsx`
- Backend: `src-tauri/src/commands/preset_commands.rs`, `src-tauri/src/services/preset_service.rs`, `src-tauri/src/domain/preset.rs`
- Store: `src/store/jobStore.ts`

### References

- [Source: epics.md#Story 1.2] Acceptance criteria
- [Source: prd.md#7.2] Preset & Profiles requirements
- [Source: ux-design-specification.md#6.2] Preset Selection Screen UX
- [Source: architecture.md#4] Preset Service component
- [Source: architecture.md#5.5] Preset domain model fields
- [Source: architecture.md#13.1] selectPreset command

## Dev Agent Record

### Agent Model Used

GPT-5 Codex

### Debug Log References

- `npm test`
- `npm run build`
- `cargo test`
- `cargo check --message-format short`

### Completion Notes List

- Da them preset domain/service/commands o Rust, seed 3 preset mac dinh vao `{app_data}/presets`, va persist `presetId` vao `job.json` khi apply.
- Da them preset warning flow: backend kiem tra `videos/` hoac `segments/` co file hay khong; frontend hien thi warning inline va yeu cau confirm truoc khi doi preset.
- Da render `PresetSelectionScreen` va `PresetCard` trong app shell sau khi draft job ton tai, gom preset list, preview details, va action controls.
- Da mo rong `jobStore` de luu full preset object cho downstream components.

### File List

- `src/modules/preset-management/PresetSelectionScreen.tsx`
- `src/modules/preset-management/PresetCard.tsx`
- `src/modules/preset-management/index.ts`
- `src/modules/start-flow/types.ts`
- `src/modules/app-shell/AppShell.tsx`
- `src/store/jobStore.ts`
- `src/styles.css`
- `src-tauri/src/domain/preset.rs`
- `src-tauri/src/domain/mod.rs`
- `src-tauri/src/commands/preset_commands.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/services/preset_service.rs`
- `src-tauri/src/constants.rs`
- `src-tauri/src/lib.rs`
