# Story 2.4: Remove Old Hardcoded Subtitle and Render New Subtitle

Status: ready-for-dev

## Story

As a user,
I want hệ thống xử lý subtitle hardcoded cũ và render subtitle mới từ SRT,
so that video đầu ra mang subtitle mới phù hợp với audio và brand mới.

## Acceptance Criteria

1. Video có SRT mapping hợp lệ + task yêu cầu thay subtitle → bước subtitle region detection chạy → xác định vùng subtitle hardcoded cũ theo segment cơ bản → lưu detection result.
2. Subtitle region đã detect → áp dụng `blur`, `mask`, hoặc `box fill` để xử lý subtitle cũ. Không tự mở rộng sang inpaint nâng cao V1 trừ case dễ.
3. Subtitle cũ đã xử lý + SRT mới hợp lệ → render subtitle mới burn vào video. Nếu task có audio replacement → subtitle mới coi là subtitle đi kèm audio mới.

## Tasks / Subtasks

- [ ] Implement subtitle region detection trong `analysis_service` (AC: 1)
  - [ ] Frame sampling: extract frames, scan bottom region (70–95% of height) — vị trí subtitle phổ biến nhất
  - [ ] Detect horizontal text bands: so sánh pixel variance giữa frames ở cùng vị trí
  - [ ] Regions thay đổi theo scene → đánh dấu là subtitle region candidates
  - [ ] Output: `SubtitleDetectionResult { regions: [{ x, y, w, h, confidence }], segments: Segment[] }`
  - [ ] Tạo Segment list từ detection result, persist vào `{job}/segments/{videoId}.json`
- [ ] Implement subtitle removal (old region) via FFmpeg (AC: 2)
  - [ ] Default mode: `boxblur` trên region
  - [ ] Blur mode: `ffmpeg -vf "boxblur=10:1:cr=0:cb=0[blur];[blur]crop=w:h:x:y[out]"` — simplified
  - [ ] Drawbox mode: `ffmpeg -vf "drawbox=x:y:w:h:color=black:t=fill"` (black fill)
  - [ ] Mask/overlay mode: overlay solid color rectangle
  - [ ] Default mode được lấy từ `preset.subtitleStylePreset` hoặc hardcode `boxblur` nếu không có
  - [ ] V1: áp dụng một mode cho toàn video, không mix modes theo segment
- [ ] Implement new subtitle render via FFmpeg (AC: 3)
  - [ ] Dùng `subtitles` filter: `ffmpeg -i video.mp4 -vf "subtitles=new.srt:force_style='FontName=Arial,FontSize=24'" output.mp4`
  - [ ] Đọc style từ `preset.subtitleStylePreset`: FontName, FontSize, color
  - [ ] SRT file được lấy từ `videoItem.mappedSrtPath`
  - [ ] Output: intermediate file `{job}/working/{videoId}_subtitle_rendered.mp4`
- [ ] Generate risk segments cho subtitle (AC: 1)
  - [ ] Nếu detection confidence thấp: Segment với `issueType = SubtitleRegion`, `riskLevel = High`
  - [ ] Nếu confidence cao nhưng region lớn/bất thường: `riskLevel = Medium`
  - [ ] Persist segments để Epic 3 review
- [ ] Log subtitle processing
  - [ ] Log: videoId, detected regions, method applied, SRT path, FFmpeg command summary

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

### Debug Log References

### Completion Notes List

### File List
