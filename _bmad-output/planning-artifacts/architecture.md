---
stepsCompleted:
  - step-01-init
inputDocuments:
  - D:\WORKING\BMAD\_bmad-output\planning-artifacts\prd.md
  - D:\WORKING\BMAD\_bmad-output\planning-artifacts\ux-design-specification.md
  - D:\WORKING\BMAD\_bmad-output\brainstorming\brainstorming-session-2026-04-20-1600.md
workflowType: architecture
projectName: Desktop Video Rebranding App
version: v1
status: draft
---

# Architecture Decision Document - Desktop Video Rebranding App

**Author:** BOSS  
**Date:** 2026-04-20

## 1. Architecture Summary

Desktop Video Rebranding App V1 nên được xây dựng như một desktop application với kiến trúc phân lớp rõ ràng giữa:

- `Desktop Shell + UX Layer`
- `Application Workflow Layer`
- `Media Processing Pipeline`
- `Persistence + Job State Layer`

Mục tiêu của kiến trúc là đảm bảo:

- xử lý batch 5-20 video/job ổn định
- review theo segment, không phải full timeline editing
- persist đầy đủ mapping, segment flags, quick fixes, và audio source state
- export pipeline tách biệt khỏi UI để không block ứng dụng

V1 không cần kiến trúc cho full editor thời gian thực phức tạp. Kiến trúc phải tối ưu cho task-driven workflow `setup -> auto process -> review exception -> export`.

## 2. Architectural Drivers

### 2.1 Drivers từ sản phẩm

- batch processing là năng lực lõi
- preset theo brand/channel là điểm điều hướng chính
- risk-based review là trung tâm của flow
- output quality ổn định quan trọng hơn tốc độ tối đa
- audio replacement toàn-video là một phần của use case cốt lõi
- người mới phải dùng được mà không bị lộ toàn bộ complexity

### 2.2 Drivers từ UX

- segment list là primary review UI
- timeline chỉ hỗ trợ context
- quick fix chỉ cho logo, subtitle old region, subtitle new
- audio replacement là toàn-video, không tham gia quick-fix theo segment
- review mode chỉ bật khi cần

### 2.3 Drivers từ vận hành

- queue phải chạy nền
- job state phải resume được
- failure của một video không được làm hỏng toàn job
- report phải truy ngược được về risk segments, review actions, và audio source đầu ra

## 3. Recommended Technology Direction

### 3.0 Platform Target (Confirmed Decision)

**V1 target: Windows only**

- Build target: Windows 10/11 x64
- Tauri build pipeline chỉ cần configure cho Windows MSVC toolchain
- FFmpeg binary: Windows build (ffmpeg.exe, ffprobe.exe), bundled trong app package
- Không cần cross-platform abstraction layer ở V1
- Phase 2 có thể mở rộng sang macOS nếu cần

### 3.1 Desktop Shell

V1 dùng desktop shell web-based:

- `Tauri` cho desktop shell (Windows target)
- `React + TypeScript` cho UI layer
- `Zustand` cho UI state management (đủ nhẹ cho app workflow-based, không cần Redux overhead)

Lý do chọn Zustand: app có nhiều independent state slices (job state, video list, segment review state, quick-fix state) mà không cần global event bus phức tạp. Zustand phù hợp hơn Redux Toolkit cho use case này.

### 3.2 Media Processing Engine

V1 dùng `FFmpeg` (Windows build, bundled) làm media engine chính cho:

- decode metadata và probe video info
- overlay logo (filter `overlay`)
- replace audio track (`-map` audio stream)
- blur/mask/fill subtitle cũ (`boxblur`, `drawbox`, hay `delogo` filter)
- render subtitle mới từ SRT (`subtitles` filter hoặc `ass` filter)
- export MP4

**Input formats được hỗ trợ (V1):**

| Container | Video codec | Audio codec | Nguồn phổ biến |
|-----------|-------------|-------------|----------------|
| MP4 (.mp4) | H.264, H.265 | AAC, MP3 | YouTube, Facebook, TikTok |
| MOV (.mov) | H.264, ProRes | AAC | Xuất từ Premiere/Final Cut |
| MKV (.mkv) | H.264, H.265 | AAC, AC3 | Download/archive |
| AVI (.avi) | H.264, MPEG4 | MP3, AAC | Legacy sources |
| WebM (.webm) | VP8, VP9 | Opus | YouTube download |

**Output format duy nhất của V1:**
- Container: MP4
- Video codec: H.264 (libx264), CRF 18–23 tùy preset
- Audio codec: AAC 192kbps
- Lý do: tương thích tốt nhất với YouTube, Facebook, TikTok upload requirements

**FFmpeg subprocess communication (Tauri):**
- Tauri Command (Rust side) spawn FFmpeg process
- Progress theo dõi qua parse stdout `-progress pipe:1`
- Cancel/abort: kill subprocess PID từ Rust side
- FFmpeg crash không làm chết Tauri process — handled bằng `Child::wait()` với error check

### 3.3 Detection / Analysis Layer

V1 không phụ thuộc AI model lớn. Detection layer là tổ hợp:

- heuristic + CV cơ bản
- metadata + frame sampling
- rule-based segmentation

Với khả năng mở rộng về sau sang:

- logo tracking nâng cao
- subtitle region detection nâng cao
- object-aware subtitle placement

## 4. High-level System Architecture

### 4.1 Primary components

1. `Desktop UI`
- start flow
- preset selection
- mapping review
- processing queue
- segment review
- export/report

2. `Application Orchestrator`
- điều phối workflow theo job
- quản lý state machine của video/job
- gọi detection, render, export
- sync UI state với persisted state

3. `Preset Service`
- load/save/duplicate preset
- resolve preset assets và rule layout

4. `Input Mapping Service`
- detect file role
- map video -> SRT -> audio -> logo -> preset
- validate pre-run requirements

5. `Audio Policy Service`
- xác định audio replacement policy cho job
- validate audio task requirements
- resolve audio source used trong export

6. `Analysis Service`
- detect logo vùng cũ
- detect subtitle region cũ
- generate segment candidates
- generate risk scoring inputs

7. `Review Service`
- quản lý segment flags
- apply quick fixes
- persist review decisions
- validate export readiness

8. `Render/Export Service`
- build FFmpeg pipeline
- render preview artifacts khi cần
- export final MP4
- write encode summary

9. `Persistence Layer`
- job manifest
- per-video state
- segment state
- review actions
- logs and reports

## 5. Core Domain Model

### 5.1 Job

`Job` là aggregate root của workflow.

Thuộc tính chính:

- jobId
- createdAt
- selectedTask
- presetId
- outputFolder
- status
- videoItems[]

### 5.2 VideoItem

Mỗi `VideoItem` là một video trong job.

Thuộc tính chính:

- videoId
- sourcePath
- sourceMetadata
- mappedLogoPath
- mappedAudioPath
- mappedSrtPath
- status
- segmentList[]
- reviewSummary
- exportSummary

### 5.3 Segment

`Segment` là đơn vị review cốt lõi của V1.

Thuộc tính chính:

- segmentId
- videoId
- startTimeMs
- endTimeMs
- issueTypes[]
- riskLevel
- detectionSnapshot
- reviewStatus
- quickFixState

### 5.4 QuickFixState

Bao gồm:

- logoOverlayRect
- subtitleOldMaskRect
- subtitleOldMode
- subtitleNewPosition
- subtitleNewScale
- subtitleStylePresetId

### 5.5 Preset

Bao gồm:

- presetId
- brandName
- defaultLogoPath
- audioReplacementPolicy
- subtitleStylePreset
- layoutRules
- exportPreset

## 6. State Machines

### 6.1 Job state machine

- Draft
- ReadyToRun
- Processing
- ReviewPending
- ReadyToExport
- Exporting
- Completed
- PartialFailure
- Failed

### 6.2 Video state machine

- Imported
- InputNeedsReview
- Processing
- ReviewNeeded
- ReadyToExport
- Exporting
- Done
- Failed

### 6.3 Segment review state

- Unreviewed
- Modified
- Accepted
- Blocked

`High Risk` segment không được rời trạng thái chặn export cho tới khi thành `Modified` hoặc `Accepted`.

## 7. Processing Pipeline

### 7.1 Pre-run pipeline

1. ingest file list
2. classify file roles
3. map assets to videos
4. validate required inputs
5. load preset
6. create persisted job manifest

### 7.2 Analysis pipeline

Cho từng video:

1. read metadata
2. sample frames
3. detect logo candidate regions
4. detect hardcoded subtitle candidate regions
5. build raw segments from layout/detection changes
6. calculate risk level
7. persist segment list

### 7.3 Review pipeline

1. load flagged segments
2. show preview for selected segment
3. apply quick fixes
4. persist fix state
5. re-evaluate segment readiness
6. re-evaluate video readiness

### 7.4 Export pipeline

1. resolve final segment instructions
2. build render recipe for each video
3. invoke FFmpeg job
4. write output file
5. capture encode summary
6. generate report artifact

Audio handling trong export pipeline:

- nếu task có audio replacement, render recipe phải thay hoàn toàn audio gốc bằng audio mới đã map
- V1 không cho phép audio offset hoặc mix audio
- subtitle mới được coi là subtitle đi cùng audio mới nếu task có cả audio và subtitle

## 8. Segment and Risk Strategy

### 8.1 Why segment-based architecture

V1 không cần keyframe editor, nhưng vẫn cần một đơn vị nhỏ hơn `video` để:

- định vị đúng chỗ lỗi
- review nhanh
- copy fixes giữa các đoạn
- gắn risk có ngữ cảnh

Segment là abstraction phù hợp nhất cho UX và architecture của V1.

### 8.2 Segment generation strategy

V1 nên tạo segment từ:

- mốc thay đổi detect logo
- mốc thay đổi subtitle region
- scene/layout shifts ở mức đơn giản
- khoảng thời gian có confidence thấp liên tục

Không cần scene graph phức tạp ở V1.

### 8.3 Risk scoring strategy

Input cho risk scoring:

- logo detect confidence
- subtitle region detect confidence
- frame-to-frame stability
- mapping ambiguity
- preset/layout mismatch indicators

Output:

- Low Risk
- Medium Risk
- High Risk

Risk scoring nên được persist như dữ liệu domain, không chỉ là UI decoration.

## 9. Persistence Design

### 9.1 Persistence goals

- resume được sau crash
- không mất quick fixes
- không phải phân tích lại từ đầu nếu job đã qua phase analysis

### 9.2 Recommended local storage layout

Nên có thư mục local per job:

- `job.json`
- `videos/{videoId}.json`
- `segments/{videoId}.json`
- `reports/{videoId}-report.json`
- `logs/{videoId}.log`
- `cache/`

### 9.3 What must be persisted

- job manifest
- selected task
- preset used
- file mappings
- per-video status
- segment flags
- quick-fix state
- export results

### 9.4 What can be recomputed

- non-critical previews
- temporary thumbnails
- some intermediate sampled frames if cache is missing

## 10. Rendering and Preview Architecture

### 10.1 Design principle (Confirmed: Hybrid approach)

V1 dùng hybrid preview gồm 2 lớp tách biệt — lớp nhanh cho interaction, lớp chính xác cho validation. Không dùng WebCodecs (quá phức tạp) hay full real-time render.

### 10.2 Preview layers

**Layer 1 — UI Overlay Preview (interaction mode)**
- HTML5 `<video>` element load file gốc qua `asset://` protocol của Tauri
- Canvas/CSS overlay hiển thị logo position, blur box, subtitle position
- Dùng khi user kéo thả handles để adjust
- Phản hồi tức thì (0ms latency), không cần FFmpeg call
- Giới hạn: không pixel-accurate với FFmpeg output thật

**Layer 2 — Rendered Validation Preview (confirm mode)**
- Khi user bấm "xem before/after" hoặc khi segment được chọn để validate
- Tauri Command → Rust → spawn FFmpeg để render 1 frame hoặc clip ngắn 2–3 giây
- Command mẫu: `ffmpeg -ss {timestamp} -t 3 -i {input} -vf "{overlay_filter},{subtitle_filter}" -frames:v 30 {temp_preview.mp4}`
- Output temp file → load lại vào `<video>` element cho "after" view
- "Before" = frame grab từ original không có filter
- Latency chấp nhận được: 1–3 giây cho một preview clip ngắn

### 10.3 Khi nào trigger Layer 2

- User chọn segment trong segment list → auto trigger grab 1 before frame + 1 after frame
- User bật split preview mode → trigger clip ngắn cả 2 phía
- User apply fix và muốn xác nhận → trigger lại after clip với fix mới

### 10.4 Temp file management

- Preview clips lưu trong `{job_folder}/cache/previews/{segmentId}-{hash}.mp4`
- Cache theo segmentId + quick-fix state hash để tránh render lại khi không cần
- Xóa cache khi job bị xóa hoặc reset

## 11. FFmpeg Integration Strategy

### 11.1 Responsibilities of FFmpeg layer

- metadata extraction
- logo overlay
- audio track replacement
- subtitle blur/mask
- subtitle burn-in
- encode/export

### 11.2 Application-owned responsibilities

- mapping
- audio policy
- segment model
- risk scoring
- quick-fix persistence
- workflow orchestration
- report generation

### 11.3 Why this boundary matters

Nếu logic domain bị nhét hết vào FFmpeg command assembly, codebase sẽ khó maintain. FFmpeg chỉ nên là execution engine, còn orchestration và domain state phải ở application layer.

## 12. Module Structure

### 12.1 Frontend modules

- `app-shell`
- `start-flow`
- `preset-management`
- `job-review`
- `processing-queue`
- `segment-review`
- `export-reporting`

### 12.2 Backend/application modules

- `job-orchestrator`
- `preset-service`
- `mapping-service`
- `audio-policy-service`
- `analysis-service`
- `risk-service`
- `review-service`
- `render-service`
- `persistence-service`
- `logging-service`

## 13. APIs Between Layers

### 13.1 UI -> Application commands

- createJob
- importAssets
- selectPreset
- createPreset
- editPreset
- duplicatePreset
- fixMapping
- startAnalysis
- openSegmentReview
- applyQuickFix
- markSegmentAccepted
- exportReadyVideos
- resumeJob

### 13.2 Application -> UI events

- jobCreated
- mappingUpdated
- inputFileReplacedAfterReview
- analysisStarted
- videoStatusChanged
- segmentFlagsGenerated
- reviewStateChanged
- exportStarted
- exportFinished
- jobRecovered

## 14. Error Handling Strategy

### 14.1 Principles

- fail one video, not the whole job
- fail early on invalid mappings
- preserve user work before showing failure

### 14.2 Error categories

- input validation errors
- media analysis errors
- rendering/export errors
- persistence/recovery errors

### 14.3 UX-aligned handling

- mapping errors appear in pre-run screen
- per-video process errors appear in queue
- review errors should not erase previous quick fixes
- export errors should mark only affected videos as failed

## 15. Logging and Observability

### 15.1 Required logs

- job lifecycle events
- file mapping decisions
- analysis results summary
- segment/risk generation summary
- audio replacement decisions
- quick-fix changes
- FFmpeg command execution summary
- export result summary

### 15.2 Why this matters

Logging là bắt buộc để:

- debug batch failures
- support resume
- explain QC/report results
- audit why a video was marked `Review Needed`

## 16. Security and Safety Guardrails

### 16.1 Local-first assumptions

V1 nên là local-first desktop app.

Điều này giúp:

- tránh gửi media nhạy cảm ra ngoài
- giảm phụ thuộc mạng
- kiểm soát tốt hơn throughput của media pipeline

### 16.2 File safety

- không ghi đè source video mặc định
- export vào output folder riêng
- job manifest phải giữ reference rõ tới source files

## 17. Scalability Boundaries

### 17.1 What V1 is designed for

- 5-20 video/job
- review theo segment
- local execution
- MP4 export

### 17.2 What V1 is not optimized for

- hundreds of videos/job
- distributed rendering
- cloud processing
- multi-user collaboration
- advanced tracking/inpainting AI workloads

## 18. ADR-style Key Decisions

### ADR-01: V1 uses segment-based review instead of timeline-native editing

**Decision:** segment là đơn vị review chính  
**Reason:** khớp với UX exception-first, giảm complexity editor  
**Tradeoff:** mất một số flexibility của full timeline workflows

### ADR-02: V1 uses FFmpeg as media execution engine

**Decision:** dùng FFmpeg cho render/export/media transforms  
**Reason:** ổn định, mature, phù hợp batch processing  
**Tradeoff:** preview tương tác sâu sẽ cần lớp bổ sung, không nên đẩy hết vào FFmpeg

### ADR-03: V1 uses local persisted job state

**Decision:** persist job/segment/review state trên local storage  
**Reason:** resume và crash safety là promise của sản phẩm  
**Tradeoff:** cần quản lý tốt schema versioning và cache invalidation

### ADR-04: Risk scoring is a domain concern, not only a UI concern

**Decision:** persist risk levels như dữ liệu nghiệp vụ  
**Reason:** queue state, review flow, và export gating đều phụ thuộc vào risk  
**Tradeoff:** cần mô hình scoring rõ ràng từ đầu

### ADR-06: V1 targets Windows only

**Decision:** Build và ship cho Windows 10/11 x64 duy nhất  
**Reason:** Đơn giản hóa build pipeline, FFmpeg binary packaging, và testing; cross-platform không phải priority V1  
**Tradeoff:** macOS/Linux users không dùng được V1; cần revisit nếu user base mở rộng

### ADR-07: Input supports common containers/codecs for YouTube/Facebook/TikTok; output is H.264/AAC MP4 only

**Decision:** Accept MP4, MOV, MKV, AVI, WebM với H.264/H.265/VP8/VP9 video và AAC/MP3/Opus/AC3 audio; output duy nhất là H.264 + AAC MP4  
**Reason:** Bao phủ 95%+ use case thực tế của re-branding video từ các platform phổ biến; output MP4 H.264 tương thích upload mọi platform  
**Tradeoff:** ProRes, DNxHD hay raw formats không được hỗ trợ ở V1

### ADR-08: Preview uses hybrid approach — Canvas overlay for interaction, FFmpeg frame grab for validation

**Decision:** HTML5 video + Canvas overlay cho real-time drag/adjust; FFmpeg subprocess render clip 2–3s cho before/after validation  
**Reason:** Canvas overlay cho latency 0ms khi kéo thả; FFmpeg grab cho accuracy 100% khi validate  
**Tradeoff:** Có 1–3s wait khi trigger validated preview; cần cache management để tránh render lại không cần thiết

### ADR-05: V1 includes full-length audio replacement but defers audio sync/editing and fingerprint variation

**Decision:** đưa audio replacement toàn-video vào V1, nhưng không đưa audio sync/editing nâng cao hay fingerprint variation vào kiến trúc lõi  
**Reason:** audio replacement là phần của use case cốt lõi, nhưng sync/editing nâng cao sẽ làm nổ scope  
**Tradeoff:** phase 2 vẫn cần mở rộng domain model cho sync, mix, và audio timeline behaviors

## 19. Implementation Guidance

### 19.1 Build order recommendation

1. Job + preset + mapping foundation
2. Persistence and queue state
3. Basic analysis service
4. Segment review model
5. FFmpeg render/export integration
6. Report generation

### 19.2 Hard architectural boundary to preserve

Không được trộn:

- UI state tạm thời
- domain review state
- FFmpeg execution details

Ba lớp này phải tách rõ để tránh app trở thành một mớ callback khó debug.

## 20. Conclusion

Kiến trúc V1 của Desktop Video Rebranding App nên tối ưu cho workflow reliability hơn là editor flexibility. Trọng tâm không nằm ở việc dựng video thời gian thực, mà nằm ở khả năng điều phối batch job, thay audio toàn-video, sinh segment rủi ro, cho phép quick-fix có cấu trúc, và export ổn định với local job persistence rõ ràng.
