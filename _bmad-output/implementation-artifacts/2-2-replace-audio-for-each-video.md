# Story 2.2: Replace Audio for Each Video

Status: ready-for-dev

## Story

As a user,
I want hệ thống thay toàn bộ audio gốc bằng audio mới đã map,
so that video đầu ra dùng đúng audio theo thị trường hoặc brand mới.

## Acceptance Criteria

1. Video có audio mapping hợp lệ + task yêu cầu thay audio → audio gốc được thay hoàn toàn → không mix audio cũ + mới.
2. V1: chỉ full-length audio replacement, không có offset/resync/segment audio.
3. Một video thất bại → đánh dấu lỗi per-video → không dừng toàn bộ job.

## Tasks / Subtasks

- [ ] Implement `audio_replacement_service` (AC: 1, 2)
  - [ ] Nhận `videoPath` và `audioPath` đã mapped
  - [ ] Build FFmpeg command: `-i video.mp4 -i audio.mp3 -map 0:v -map 1:a -c:v copy -c:a aac -shortest output.mp4`
  - [ ] `-map 0:v`: lấy video stream từ input 0 (video gốc)
  - [ ] `-map 1:a`: lấy audio stream từ input 1 (audio mới)
  - [ ] `-c:v copy`: không re-encode video stream (tốc độ, chất lượng giữ nguyên)
  - [ ] `-c:a aac`: encode audio mới sang AAC
  - [ ] Không dùng `-filter_complex`, không mix audio, không amix
- [ ] Implement per-video error isolation (AC: 3)
  - [ ] Wrap mỗi video trong try-catch ở job_orchestrator
  - [ ] Nếu FFmpeg exit code ≠ 0: đánh dấu VideoItem status = `Failed`, ghi log lỗi
  - [ ] Continue queue với video tiếp theo
- [ ] Log audio replacement decision (NFR6)
  - [ ] Log: videoId, audioSourcePath, FFmpeg command summary
  - [ ] Log outcome: success hoặc error với exit code + stderr snippet
- [ ] Emit audio processing events
  - [ ] `audioReplacementStarted { videoId, audioSourcePath }`
  - [ ] `audioReplacementCompleted { videoId, success, errorMessage? }`

## Dev Notes

- **FFmpeg command pattern**: dùng `-map` streams thay vì `-an/-acodec` để tránh side effects. Ví dụ:
  ```
  ffmpeg -i input.mp4 -i new_audio.mp3 -map 0:v:0 -map 1:a:0 -c:v copy -c:a aac -y output.mp4
  ```
- **Audio policy check**: trước khi chạy, kiểm tra `preset.audioReplacementPolicy`. Nếu policy = `NoReplacement` → skip bước này, không gọi FFmpeg.
- **AudioReplacementPolicy enum**: `ReplaceAll` | `NoReplacement`. V1 chỉ support 2 giá trị này.
- **Output file**: ghi vào `{job}/working/{videoId}_audio_replaced.mp4` (intermediate file), không overwrite source.
- **`-shortest` flag**: nếu audio file ngắn hơn video → video bị trim. Nếu audio dài hơn → video kết thúc ở đúng độ dài video. Document behavior này rõ cho user trong report.
- **Không có audio editing V1**: không implement volume normalization, fade in/out, delay/offset. Strictly out of scope.

### Project Structure Notes

- Backend: `src-tauri/src/services/audio_replacement_service.rs`, `src-tauri/src/services/job_orchestrator.rs` (gọi audio service trong pipeline)
- Logs: `{job}/logs/{videoId}.log`
- Working output: `{job}/working/{videoId}_audio_replaced.mp4`
- Events: `audioReplacementStarted`, `audioReplacementCompleted`

### References

- [Source: epics.md#Story 2.2] Acceptance criteria
- [Source: prd.md#7.1] FR14: thay toàn bộ audio, không segment audio, không mix
- [Source: architecture.md#3.2] Audio formats: MP3, AAC, WAV, M4A
- [Source: architecture.md#6] Audio Policy Service
- [Source: architecture.md#13.1] FFmpeg IPC pattern

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
