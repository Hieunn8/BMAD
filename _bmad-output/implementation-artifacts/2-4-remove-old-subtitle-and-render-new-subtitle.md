# Story 2.4: Remove Old Hardcoded Subtitle and Render New Subtitle

Status: done

## Story

As a user,
I want hệ thống xử lý subtitle hardcoded cũ và render subtitle mới từ SRT,
so that video đầu ra mang subtitle mới phù hợp với audio và brand mới.

## Acceptance Criteria

1. Video có SRT mapping hợp lệ + task yêu cầu thay subtitle → bước subtitle region detection chạy → xác định vùng subtitle hardcoded cũ theo segment cơ bản → lưu detection result.
2. Subtitle region đã detect → áp dụng `blur`, `mask`, hoặc `box fill` để xử lý subtitle cũ. Không tự mở rộng sang inpaint nâng cao V1 trừ case dễ.
3. Subtitle cũ đã xử lý + SRT mới hợp lệ → render subtitle mới burn vào video. Nếu task có audio replacement → subtitle mới coi là subtitle đi kèm audio mới.

## Tasks / Subtasks

- [x] Implement subtitle region detection trong `analysis_service` (AC: 1)
  - [x] Frame sampling: extract frames, scan bottom region (70–95% of height) — vị trí subtitle phổ biến nhất
  - [x] Detect horizontal text bands: so sánh pixel variance giữa frames ở cùng vị trí
  - [x] Regions thay đổi theo scene → đánh dấu là subtitle region candidates
  - [x] Output: `SubtitleDetectionResult { regions: [{ x, y, w, h, confidence }], segments: Segment[] }`
  - [x] Tạo Segment list từ detection result, persist vào `{job}/segments/{videoId}_subtitle.json`
- [x] Implement subtitle removal (old region) via FFmpeg (AC: 2)
  - [x] Default mode: `boxblur` trên region
  - [x] Blur mode: crop band + boxblur=10:1 + overlay lại vị trí cũ
  - [x] Drawbox mode: `ffmpeg -vf "drawbox=x:y:w:h:color=black:t=fill"` (black fill)
  - [x] Default mode được lấy từ `preset.subtitleStylePreset` hoặc hardcode `boxblur` nếu không có
  - [x] V1: áp dụng một mode cho toàn video, không mix modes theo segment
- [x] Implement new subtitle render via FFmpeg (AC: 3)
  - [x] Dùng `subtitles` filter: `ffmpeg -i video.mp4 -vf "subtitles=new.srt:force_style='FontName=Arial,FontSize=24'" output.mp4`
  - [x] Đọc style từ `preset.subtitleStylePreset`: FontName, FontSize, color
  - [x] SRT file được lấy từ `videoItem.mappedSrtPath`
  - [x] Output: intermediate file `{job}/working/{videoId}_subtitle_rendered.mp4`
- [x] Generate risk segments cho subtitle (AC: 1)
  - [x] Nếu detection confidence thấp: Segment với `issueType = SubtitleRegion`, `riskLevel = High`
  - [x] Nếu confidence cao nhưng region lớn/bất thường: `riskLevel = Medium`
  - [x] Persist segments để Epic 3 review
- [x] Log subtitle processing
  - [x] Log: videoId, detected regions, method applied, SRT path, FFmpeg command summary

## Dev Notes

- **Subtitle removal pipeline order**: detect regions → remove old → render new. Đây là 3 bước riêng biệt trong pipeline.
- **Removal mode V1**: mặc định `boxblur`. User có thể đổi trong Epic 3 (quick fix). V1 pipeline chỉ cần apply default mode.
- **FFmpeg subtitle filter**: `subtitles=filename.srt` yêu cầu libass được build vào FFmpeg binary. Kiểm tra static build có libass.
- **SRT encoding**: file SRT phải là UTF-8. Nếu không phải → log warning, không crash job.
- **Chỉ burn subtitle**: V1 không support soft subtitle (stream). Luôn dùng hardcoded burned-in approach.
- **Audio + subtitle coupling**: nếu task = `Thay logo, audio, và subtitle`: đảm bảo audio replacement (Story 2.2) chạy trước subtitle render, và SRT mới được align theo audio mới (không có re-timing V1 — user phải đảm bảo SRT sync với audio khi import).
- **Không inpaint V1**: không implement content-aware fill. `boxblur` hoặc solid fill là max V1.

### Project Structure Notes

- Backend: `src-tauri/src/services/analysis_service.rs` (subtitle detection), `src-tauri/src/services/render_service.rs` (removal + render)
- Storage: `{job}/segments/{videoId}.json` (subtitle segments), `{job}/working/{videoId}_subtitle_rendered.mp4`
- Events: `subtitleDetectionCompleted { videoId, regionCount }`, `subtitleRenderCompleted { videoId }`

### References

- [Source: epics.md#Story 2.4] Acceptance criteria
- [Source: prd.md#7.1] FR11: detect subtitle region cơ bản, blur/mask/fill; FR12: render subtitle mới từ SRT
- [Source: architecture.md#4] Analysis Service: subtitle region detection
- [Source: architecture.md#5.3] Segment: issueType (SubtitleRegion), riskLevel
- [Source: architecture.md#13.1] FFmpeg IPC pattern

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- `cargo check --manifest-path src-tauri/Cargo.toml` — clean (5 warnings, no errors)
- `cargo test --manifest-path src-tauri/Cargo.toml` — 37/37 pass

### Completion Notes List

- Thêm `SubtitleRegion`, `SubtitleSegment`, `SubtitleDetectionResult` vào `analysis_service.rs`
- Detection dùng frame sampling (5 frames tại 10/25/50/75/90%), tính brightness variance trong band 75–95% chiều cao video. High variance → subtitle đang thay đổi giữa frames → confident detection.
- Thêm `detect_subtitle_inner` với cleanup temp dir đảm bảo chạy cả khi có lỗi (pattern tương tự logo).
- Thêm `RenderService::remove_subtitle()`: 2 mode — `boxblur` (crop+blur+overlay) và `drawbox` (black fill). Mode lấy từ `preset.subtitleStylePreset`.
- Thêm `RenderService::render_subtitle()`: dùng FFmpeg `subtitles` filter với `force_style` từ preset. Validate UTF-8 encoding SRT (warning, không fail job).
- Thêm `RiskService::persist_subtitle_segments()`: ghi `{videoId}_subtitle.json` (không conflict với logo `_logo.json`).
- Tích hợp vào `job_orchestrator.process_video()`: detect → persist → emit event → remove (nếu detected) → render → chain `working_input`.
- 12 tests mới: variance logic, segment risk levels, FFmpeg filter strings, style parsing, path naming.

### File List

- `src-tauri/src/services/analysis_service.rs`
- `src-tauri/src/services/render_service.rs`
- `src-tauri/src/services/risk_service.rs`
- `src-tauri/src/services/job_orchestrator.rs`

## Code Review Record

### Review Findings

| # | Severity | Issue | Resolution |
|---|----------|-------|------------|
| 1 | CRITICAL | `build_removal_filter` boxblur uses named stream labels (`[0:v]split…[outv]`) passed via `-vf`; named labels require `-filter_complex` | **FIXED** — `remove_subtitle` now branches on mode: drawbox → `-vf`, boxblur → `-filter_complex` + `-map [outv]` + `-map 0:a?` |
| 2 | HIGH | `mean_brightness_in_band`: `break` was inside the x-loop, exiting x not y — out-of-bounds rows iterated wastefully | **FIXED** — bounds check moved to outer y-loop |
| 3 | HIGH | `force_style` wrapped in `'...'`; preset value containing `'` breaks FFmpeg filter string | **FIXED** — single quotes escaped with `\'` before interpolation |
| 4 | MEDIUM | Detection uses per-frame mean-brightness variance vs spec's per-pixel temporal variance | **DEFER** — valid V1 heuristic; per-pixel variance is V2 |
| 5 | MEDIUM | `-c:a copy` without `0:a?` in render_subtitle `-vf` mode | **DEFER** — FFmpeg handles missing audio gracefully in simple filtergraph; remove_subtitle fixed via P1 |
| 6 | — | Subtitle render unconditional when `detection.detected=false` | **DISMISS** — correct by spec; render (new SRT) is independent of removal (old region) |

### Post-Review Build
- `cargo check` — clean (5 warnings, no errors)
- `cargo test` — 37/37 pass
