# Story 4.4: Persist Per-Video Export Outcome and Summary Report

Status: ready-for-dev

## Story

As a user,
I want xem và lưu lại outcome cuối cùng của từng video sau export,
so that tôi và QC có đủ thông tin để tin output mà không phải xem lại toàn bộ video.

## Acceptance Criteria

1. Video đã export hoặc hoàn tất trạng thái cuối → report được tạo với: trạng thái cuối, encode summary, audio source used, số segment bị flag, số segment đã sửa, before/after spot check, output path.
2. User mở report video → có thể quay lại review từ report context → dữ liệu outcome đã persist được giữ nguyên.

## Tasks / Subtasks

- [ ] Implement `generate_video_report` Tauri command (AC: 1)
  - [ ] Nhận: `{ videoId }`
  - [ ] Aggregate data từ:
    - `videos/{videoId}.json`: status, audioReplacementApplied, outputPath, encodeSummary
    - `segments/{videoId}.json`: count total, count flagged, count modified (fix applied), count accepted
  - [ ] Tạo `VideoReport` object:
    ```
    {
      videoId, videoName, finalStatus,
      encodeSummary: { codec, crf, outputSizeMb, durationSeconds },
      audioSource: { policy, audioFilePath? },
      segmentStats: { total, flagged, modified, accepted, highRiskRemaining },
      spotCheckThumbnails: [{ segmentId, beforePath, afterPath }],
      outputPath, reportGeneratedAt
    }
    ```
  - [ ] Persist vào `{job}/reports/{videoId}-report.json`
- [ ] Implement spot check thumbnail generation (AC: 1)
  - [ ] Chọn tối đa 3 segments: ưu tiên High Risk đã fix + 1–2 Medium Risk
  - [ ] Extract 1 frame từ "before" (source video tại segment midpoint)
  - [ ] Extract 1 frame từ "after" (processed video tại segment midpoint)
  - [ ] Lưu thumbnails: `{job}/reports/thumbnails/{segmentId}-before.jpg`, `{job}/reports/thumbnails/{segmentId}-after.jpg`
  - [ ] Dùng FFmpeg: `ffmpeg -i video.mp4 -ss <time> -vframes 1 thumbnail.jpg`
- [ ] Tạo `ReportScreen.tsx` (AC: 1, 2)
  - [ ] Per-video report card với tất cả các fields trên
  - [ ] Before/after thumbnail pairs
  - [ ] Button "Xem lại video này" → navigate về review workspace với video đó selected (AC: 2)
  - [ ] Job summary: total videos, success, failed, total output size
- [ ] Implement `get_report` Tauri command (AC: 2)
  - [ ] Load `{job}/reports/{videoId}-report.json`
  - [ ] Nếu chưa có (video chưa export): generate on-demand
- [ ] Auto-generate report sau mỗi video export (AC: 1)
  - [ ] Trong export pipeline (Story 4.3): sau khi `videoExportCompleted` → call `generate_video_report`
  - [ ] Nếu generate fail: log warning, không fail export

## Dev Notes

- **Report = immutable snapshot**: khi report đã generate, không overwrite nếu user quay lại review và sửa thêm. Chỉ regenerate khi user explicitly request hoặc khi export lại.
- **Spot check selection logic**: ưu tiên `High Risk + Modified` (user đã fix) → `High Risk + Accepted` (user accepted as-is) → `Medium Risk + Modified`. Max 3 thumbnails.
- **FFmpeg thumbnail**: `-ss <time> -vframes 1 -q:v 2` cho JPEG quality. Dùng `segment.startTime + (segment.endTime - segment.startTime) / 2` để lấy midpoint.
- **Encode summary**: parse FFmpeg output sau export để lấy output file size, bitrate, duration. Ghi vào `videos/{videoId}.json` khi export complete.
- **"Quay lại review" từ report**: navigate về `SegmentReviewScreen` với `selectedVideoId` preset. Existing review state (segments, quickfix) vẫn giữ nguyên — report context không làm thay đổi data.
- **Job-level summary**: aggregate từ tất cả per-video reports. Hiển thị sau khi batch export hoàn tất.

### Project Structure Notes

- Frontend: `src/modules/export-reporting/ReportScreen.tsx`, `src/modules/export-reporting/VideoReportCard.tsx`
- Backend: `src-tauri/src/commands/export_commands.rs` (`generate_video_report`, `get_report`), `src-tauri/src/services/export_service.rs` (report generation)
- Storage: `{job}/reports/{videoId}-report.json`, `{job}/reports/thumbnails/{segmentId}-{before|after}.jpg`

### References

- [Source: epics.md#Story 4.4] Acceptance criteria
- [Source: prd.md#7.1] FR18: risk report, encode summary, audio source summary, before/after spot check
- [Source: ux-design-specification.md#6.6] Report screen (UX-DR14)
- [Source: architecture.md#8] Persistence: reports/{videoId}-report.json
- [Source: architecture.md#5] VideoReport: fields, spot check thumbnails

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
