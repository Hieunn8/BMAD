# Story 4.3: Export All Ready Videos to MP4

Status: ready-for-dev

## Story

As a user,
I want export hàng loạt các video đã sẵn sàng sang MP4,
so that tôi có thể lấy output cuối cùng cho vận hành mà không phải export từng video một.

## Acceptance Criteria

1. Có ít nhất 1 video `Ready to Export` → bấm "Export All Ready Videos" → chỉ export videos đủ điều kiện → output MP4 theo preset V1 vào output location → ưu tiên chất lượng ổn định hơn tốc độ.
2. Batch export đang chạy, 1 video thất bại → video đó marked `Failed` per-video → các video khác tiếp tục.

## Tasks / Subtasks

- [ ] Implement `start_export` Tauri command (AC: 1)
  - [ ] Validate: ít nhất 1 video ReadyToExport, outputFolder đã được set
  - [ ] Transition job.status → `Exporting`
  - [ ] Enqueue từng ReadyToExport video vào export queue
  - [ ] Spawn async Tokio task (non-blocking, same pattern như processing queue)
- [ ] Implement export pipeline cho từng video (AC: 1)
  - [ ] Compose final FFmpeg command từ working files + quick fixes:
    - [ ] Input: intermediate working files (`_audio_replaced`, `_logo_replaced`, `_subtitle_rendered`)
    - [ ] Apply quick fix adjustments (logo position, subtitle position, removal mode) từ `QuickFixState`
    - [ ] Output codec: H.264 video + AAC audio
    - [ ] CRF: lấy từ `preset.exportPreset.crfValue` (18 / 20 / 23)
    - [ ] Preset: `-preset slow` (ưu tiên chất lượng — ADR-07, NFR3)
    - [ ] Output: `{outputFolder}/{videoId}_{videoName}_rebranded.mp4`
  - [ ] Theo dõi FFmpeg progress qua `-progress pipe:1`
  - [ ] Update `VideoItem.status = Exporting` khi bắt đầu
  - [ ] Update `VideoItem.status = Exported` khi xong
  - [ ] Update `VideoItem.status = Failed` nếu FFmpeg exit code ≠ 0
- [ ] Implement per-video error isolation (AC: 2)
  - [ ] Wrap mỗi video export trong try-catch
  - [ ] Failure của một video không stop queue
  - [ ] Log chi tiết: videoId, FFmpeg stderr, exit code
- [ ] Emit export progress events
  - [ ] `videoExportStarted { videoId }`
  - [ ] `exportProgress { videoId, percent }` (từ FFmpeg -progress output)
  - [ ] `videoExportCompleted { videoId, success, outputPath?, errorMessage? }`
  - [ ] `batchExportCompleted { total, success, failed }`
- [ ] Implement export progress UI trong `ExportScreen.tsx`
  - [ ] Per-video progress bar (từ FFmpeg progress events)
  - [ ] Overall batch progress: `X / N videos exported`
  - [ ] Status badge update: Exporting → Exported / Failed

## Dev Notes

- **FFmpeg compose order**: pipeline phải compose đúng thứ tự filter. Ví dụ final compose command:
  ```
  ffmpeg -i audio_replaced.mp4 
    -i new_logo.png 
    -filter_complex "
      [0:v]overlay=x=<x>:y=<y>:enable='between(t,0,99999)'[v1];
      [v1]boxblur=10:1:cr=0:cb=0[v2];
      [v2]subtitles=new.srt[vout]
    "
    -map "[vout]" -map 0:a
    -c:v libx264 -crf 20 -preset slow
    -c:a aac -b:a 192k
    -y output.mp4
  ```
  Note: order của filters là logo overlay → subtitle removal → subtitle render.
- **CRF guideline** (ADR-07): CRF 18 = visually lossless (large file), CRF 20 = default balance, CRF 23 = smaller file (slight quality reduction). Default = CRF 20.
- **`-preset slow`**: trade-off encode time cho file size/quality. Đây là bắt buộc (NFR3 — không hy sinh chất lượng chỉ để tăng tốc).
- **Output file naming**: `{videoId}_{original_filename}_rebranded.mp4`. Không dùng spaces trong filename (escape safely).
- **FFmpeg progress parsing**: parse output của `-progress pipe:1`. Format: `out_time_ms=<microseconds>`, `progress=continue|end`. Tính percent từ `out_time_ms / total_duration_ms * 100`.
- **Working files cleanup**: sau khi export thành công, có thể xóa intermediate working files để tiết kiệm disk. V1: giữ lại working files cho debug (chỉ clean up theo user request hoặc sau report confirm).

### Project Structure Notes

- Frontend: `src/modules/export-reporting/ExportScreen.tsx` (progress UI)
- Backend: `src-tauri/src/commands/export_commands.rs` (`start_export`), `src-tauri/src/services/export_service.rs` (pipeline), `src-tauri/src/services/render_service.rs` (FFmpeg compose)
- Storage: `{outputFolder}/{videoId}_{name}_rebranded.mp4`, `{job}/videos/{videoId}.json` (status update)
- Events: `videoExportStarted`, `exportProgress`, `videoExportCompleted`, `batchExportCompleted`

### References

- [Source: epics.md#Story 4.3] Acceptance criteria
- [Source: prd.md#7.1] FR17: export MP4 hàng loạt; NFR3: chất lượng ổn định hơn tốc độ
- [Source: architecture.md#3.2] Output format: H.264 + AAC MP4, CRF 18–23 (ADR-07)
- [Source: architecture.md#9] Export pipeline, FFmpeg compose
- [Source: architecture.md#13.1] startExport command, FFmpeg IPC progress

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
