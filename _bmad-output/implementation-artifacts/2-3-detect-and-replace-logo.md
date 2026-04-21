# Story 2.3: Detect and Replace Logo

Status: ready-for-dev

## Story

As a user,
I want hệ thống tự detect logo cũ và đặt logo mới đè lên,
so that phần lớn video được re-brand tự động trước khi cần review tay.

## Acceptance Criteria

1. Video có logo input (từ preset hoặc override) → bước logo detection chạy → xác định vùng logo cũ ở mức cơ bản → lưu detection result.
2. Detection result hợp lệ → logo mới được overlay để che phủ vùng logo cũ → áp dụng theo rule của preset.
3. Detection không đủ chắc chắn hoặc không ổn định → sinh risk signal/flag → persist cho các bước sau.

## Tasks / Subtasks

- [ ] Implement `analysis_service` logo detection (AC: 1, 3)
  - [ ] Frame sampling: extract N frames đều nhau từ video (VD: mỗi 2–5 giây)
  - [ ] Tìm vùng logo cũ bằng heuristic: kiểm tra góc ảnh (top-left, top-right, bottom-right, bottom-left), so sánh histogram/color consistency giữa các frame
  - [ ] Nếu vùng ổn định qua nhiều frame: đánh dấu `Matched`, ghi bounding box
  - [ ] Nếu vùng không ổn định hoặc mờ: đánh dấu `LowConfidence`
  - [ ] Output: `LogoDetectionResult { boundingBox, confidence: float, riskLevel }`
- [ ] Implement logo overlay via FFmpeg (AC: 2)
  - [ ] Build FFmpeg filter: `overlay=x=<x>:y=<y>` với logo mới
  - [ ] Logo source: `preset.defaultLogoPath` (hoặc per-video override)
  - [ ] Apply overlay trên toàn video (V1: không theo segment timeline)
  - [ ] Output: intermediate file `{job}/working/{videoId}_logo_replaced.mp4`
- [ ] Generate Segment list với logo risk flags (AC: 3)
  - [ ] Với mỗi frame region `LowConfidence`: tạo Segment với `issueType = LogoPosition`, `riskLevel = High|Medium`
  - [ ] Persist segments vào `{job}/segments/{videoId}.json`
  - [ ] Risk mapping: confidence < 0.5 → High Risk; 0.5–0.8 → Medium Risk; > 0.8 → Low Risk
- [ ] Log logo detection and replacement
  - [ ] Log: videoId, detection bounding box, confidence score, FFmpeg command summary
  - [ ] Log: số segment được tạo, risk distribution

## Dev Notes

- **Heuristic-first approach** (no AI model): V1 dùng frame sampling + basic CV (histogram, region consistency). Detection layer không được phụ thuộc large AI model.
- **Frame sampling strategy**: sample ở 10%, 25%, 50%, 75%, 90% duration. Kiểm tra top-left region (most common logo position) đầu tiên.
- **Confidence threshold**: đây là domain data thật (NFR10), không chỉ để hiển thị. `confidence` ảnh hưởng trực tiếp đến `riskLevel` của Segment.
- **Logo overlay position**: lấy `x, y, width, height` từ detection result. Nếu không detect được, dùng position mặc định từ preset `layoutRules`.
- **FFmpeg overlay filter**: `ffmpeg -i video.mp4 -i logo.png -filter_complex "overlay=x=10:y=10" output.mp4`
- **Không thay đổi V1 boundary**: không detect multiple logos, không track logo movement per-frame. Chỉ find-once + overlay-entire-video.
- **Segment granularity**: trong V1, mỗi "problematic region" được model như 1 segment với time range rộng (VD: 0:00–end). Segment-level timeline editing là Epic 3.

### Project Structure Notes

- Backend: `src-tauri/src/services/analysis_service.rs` (logo detection), `src-tauri/src/services/render_service.rs` (FFmpeg overlay)
- Storage: `{job}/segments/{videoId}.json` (logo segments với risk), `{job}/working/{videoId}_logo_replaced.mp4`
- Events: `logoDetectionCompleted { videoId, confidence, segmentCount }`, `logoReplacementCompleted { videoId }`

### References

- [Source: epics.md#Story 2.3] Acceptance criteria
- [Source: prd.md#7.1] FR10: detect logo cũ cơ bản, overlay logo mới, sinh risk flag
- [Source: architecture.md#4] Analysis Service: heuristic + CV cơ bản + frame sampling
- [Source: architecture.md#5.3] Segment domain model: riskLevel, issueType
- [Source: architecture.md#13.1] FFmpeg IPC pattern

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
