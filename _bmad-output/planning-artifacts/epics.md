---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
inputDocuments:
  - D:\WORKING\BMAD\_bmad-output\planning-artifacts\prd.md
  - D:\WORKING\BMAD\_bmad-output\planning-artifacts\architecture.md
  - D:\WORKING\BMAD\_bmad-output\planning-artifacts\ux-design-specification.md
---

# Desktop Video Rebranding App - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for Desktop Video Rebranding App, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: Hệ thống phải cho phép người dùng chọn task xử lý theo workflow `Thay logo`, `Thay audio`, `Thay subtitle`, hoặc `Thay logo, audio, và subtitle`.
FR2: Hệ thống phải cho phép import một hoặc nhiều video trong cùng một job, cùng với logo, audio mới, và SRT phục vụ re-branding.
FR3: Hệ thống phải tự nhận diện vai trò file cơ bản từ extension và cho phép lưu cấu hình job trước khi chạy.
FR4: V1 chỉ hỗ trợ mỗi video map với đúng một audio output và đúng một SRT output.
FR5: Hệ thống phải tự map audio và SRT vào video bằng exact base filename match trước, phát hiện thiếu hoặc ambiguous mapping, cho phép user sửa mapping thủ công, và chặn chạy job khi task đã chọn còn thiếu input bắt buộc.
FR6: Hệ thống phải hỗ trợ preset theo brand/channel, cho phép save, load, duplicate preset, và áp dụng một preset cho toàn bộ job.
FR7: Mỗi preset phải lưu được logo mặc định, audio replacement policy, subtitle style preset, rule layout cơ bản, và export preset cơ bản.
FR8: Hệ thống phải hỗ trợ batch processing 5-20 video/job với queue nền, trạng thái từng video, và khả năng resume/retry cơ bản.
FR9: Hệ thống phải persist job manifest, mapping file, trạng thái từng video, segment flags, quick-fix state, và khôi phục job đến bước gần nhất sau crash hoặc mất điện.
FR10: Hệ thống phải detect logo cũ ở mức cơ bản, overlay logo mới để che phủ logo cũ, sinh confidence hoặc risk flag, và cho phép user chỉnh vị trí hoặc kích thước logo ở segment bị flag.
FR11: Hệ thống phải detect vùng subtitle hardcoded cũ theo segment ở mức cơ bản, ưu tiên blur/mask hoặc box fill đơn giản, chỉ dùng inpaint cho case dễ nếu khả thi.
FR12: Hệ thống phải import SRT mới, render subtitle mới lên video, và khi task có audio replacement thì phải coi SRT mới là subtitle đi kèm với audio mới.
FR13: Hệ thống phải cho phép user chỉnh nhanh vị trí subtitle mới, scale cỡ chữ, và style preset mà không cung cấp full subtitle style editor trong V1.
FR14: Hệ thống phải hỗ trợ thay toàn bộ audio của video bằng file audio mới đã map, không theo segment, không có offset/resync/audio editing trong V1, và khi export phải thay hoàn toàn audio gốc bằng audio mới.
FR15: Hệ thống phải dùng segment list làm UI chính cho review, timeline nhẹ chỉ đóng vai trò hỗ trợ.
FR16: Hệ thống phải sinh risk levels `Low Risk`, `Medium Risk`, `High Risk`, hiển thị before/after preview cho segment bị flag, cho phép apply fix cho một hoặc nhiều segment được chọn, và chỉ cho video `Ready to Export` khi không còn `High Risk` chưa được xử lý.
FR17: Hệ thống phải export ra MP4, ưu tiên chất lượng ổn định hơn tốc độ tối đa, cho phép export hàng loạt các video sẵn sàng.
FR18: Hệ thống phải sinh risk report cơ bản cho từng video, bao gồm trạng thái job, encode summary, audio source summary, số segment bị flag, và before/after spot check cho vùng đã chỉnh hoặc nghi ngờ.
FR19: Hệ thống phải cung cấp các screen chính gồm Start, Preset Selection, Job Setup/Pre-run Review, Processing Queue, Exception Review, và Export/Report.
FR20: Hệ thống phải cho phép mở review thủ công cả khi video không có segment bị flag, nhưng phải bắt buộc review trước export nếu còn segment `High Risk`.

### NonFunctional Requirements

NFR1: UI phải phản hồi tốt và không block toàn bộ app khi xử lý nền.
NFR2: Hệ thống phải ổn định cho batch 5-20 video/job.
NFR3: Chất lượng output phải ổn định và không được hy sinh rõ rệt chỉ để tăng tốc export.
NFR4: Người mới phải bắt đầu được trong vòng 5 phút đầu mà không cần học một editor phức tạp.
NFR5: UX phải theo mô hình hybrid với wizard/task flow là mặc định và review panel chỉ xuất hiện khi cần.
NFR6: Hệ thống phải có logging đủ để debug pipeline, audit mapping decisions, analysis results, review actions, audio replacement decisions, và export outcomes.
NFR7: Ứng dụng nên là local-first, không ghi đè source video mặc định, và export vào output folder riêng.
NFR8: Failure của một video không được làm hỏng toàn bộ job.
NFR9: Review phải là exception-based, tức người dùng không cần xem lại toàn bộ video mà chỉ tập trung vào segment nghi ngờ.
NFR10: Risk scoring phải là domain data thật, không chỉ là thông tin trang trí UI.

### Additional Requirements

- Kiến trúc V1 phải tách rõ `Desktop UI`, `Application Orchestrator`, `Media Processing Pipeline`, và `Persistence + Job State Layer`.
- Desktop shell đề xuất là `Tauri` với `React + TypeScript`; media engine chính là `FFmpeg`.
- Detection layer của V1 nên dựa trên heuristic + CV cơ bản + frame sampling + rule-based segmentation, không phụ thuộc AI model lớn.
- Cần có các service riêng cho `Preset`, `Input Mapping`, `Audio Policy`, `Analysis`, `Review`, `Render/Export`, `Persistence`, và `Logging`.
- Domain model cần có các thực thể `Job`, `VideoItem`, `Segment`, `QuickFixState`, và `Preset`, bao gồm `mappedAudioPath` và `audioReplacementPolicy`.
- State machine phải tồn tại ở cả cấp job, video, và segment review; `High Risk` phải chặn export cho tới khi thành `Modified` hoặc `Accepted`.
- Export pipeline phải thay hoàn toàn audio gốc bằng audio mới đã map khi task yêu cầu audio; V1 không cho phép audio offset, mix audio, hay audio timeline editing.
- Persistence phải lưu ít nhất `job.json`, per-video state, segment state, reports, logs, và cache để resume được sau crash.
- FFmpeg chỉ là execution engine; logic domain như mapping, audio policy, segment model, risk scoring, quick-fix persistence, và workflow orchestration phải nằm ở application layer.
- Error handling phải fail sớm với invalid mapping, fail one video not whole job, và không được làm mất quick fixes trước đó.
- Logging bắt buộc phải bao quát job lifecycle, file mapping decisions, analysis summaries, segment/risk generation, audio replacement decisions, FFmpeg command summaries, và export results.
- Hệ thống không được drift sang full timeline editor; segment-based review là boundary kiến trúc cần giữ.

### UX Design Requirements

UX-DR1: Màn Start phải dùng wording theo task, không theo thuật ngữ editor, với câu hỏi trung tâm `Bạn muốn làm gì?`.
UX-DR2: Start screen phải có 4 task cards `Thay logo`, `Thay audio`, `Thay subtitle`, `Thay logo, audio, và subtitle`.
UX-DR3: Start screen phải có một vùng drag-and-drop chung cho video, logo, audio, và SRT, và tự gợi ý task nếu người dùng chưa chọn task.
UX-DR4: Preset screen phải thể hiện preset như một “gói thương hiệu”, hiển thị brand name, logo mặc định, audio policy, subtitle style preset, export preset, và notes ngắn.
UX-DR5: Pre-run screen phải có bảng mapping với các cột `video`, `task`, `preset`, `logo`, `audio`, `SRT`, `trạng thái mapping`, và cho phép inline correction thay vì modal overload.
UX-DR6: Copy ở pre-run screen phải dùng microcopy dễ hiểu như `Chưa tìm thấy SRT khớp`, `Chưa tìm thấy audio khớp`, `Có nhiều file có thể phù hợp`, tránh thuật ngữ kỹ thuật khó hiểu.
UX-DR7: Processing screen phải hiển thị progress tổng, progress theo video, trạng thái từng video, và panel log ngắn cho các bước detect logo, replace audio, detect subtitle region, render subtitle, risk scoring.
UX-DR8: Review Exceptions screen phải có layout 3 cột với `Segment List`, `Preview Workspace`, và `Quick Fix Panel`; timeline nhẹ chỉ để định vị ở phía dưới.
UX-DR9: Segment list phải sắp xếp mặc định `High Risk` trước rồi theo thời gian, hỗ trợ filter theo risk hoặc issue type, và hỗ trợ multi-select để apply fix hàng loạt.
UX-DR10: Preview workspace phải có before/after toggle, split preview mode, và overlay handles cho logo mới, blur/mask box của subtitle cũ, và subtitle mới.
UX-DR11: Quick Fix Panel của V1 chỉ gồm controls cho logo, subtitle old region, subtitle new, cùng các action `Áp dụng cho segment này`, `Áp dụng cho các segment đã chọn`, `Đánh dấu đã review`, `Khôi phục mặc định`.
UX-DR12: Audio mapping phải được xử lý hoàn toàn trong Setup Mode; audio không được xuất hiện như quick-fix trong review screen.
UX-DR13: Export screen phải hiển thị danh sách export-ready, các video bị chặn vì chưa review xong, output folder, export preset summary, và audio source summary.
UX-DR14: Report screen phải hiển thị final status, encode summary, audio source used, số segment bị flag, số segment đã sửa, và before/after spot check thumbnails cho từng video.
UX-DR15: Visual direction phải sáng, thoáng, dùng panel trắng/xám nhạt với accent xanh teal hoặc xanh dương sạch, và density thấp ở Start/Preset screens.
UX-DR16: Risk color system phải dùng màu riêng cho `Low Risk`, `Medium Risk`, `High Risk`, `Done/Reviewed`, nhưng không chỉ dựa vào màu mà còn cần icon + label.
UX-DR17: UX phải tối ưu cho exception-based review; nếu không có exception thì phải hiển thị trạng thái rõ kiểu `Không có đoạn bắt buộc phải sửa`.
UX-DR18: App phải desktop-first nhưng vẫn thích ứng khi cửa sổ hẹp hơn bằng cách cho phép collapse quick fix panel và giữ segment list dễ filter.

### FR Coverage Map

FR1: Epic 1 - Chọn task xử lý theo workflow
FR2: Epic 1 - Import batch video và asset
FR3: Epic 1 - Nhận diện role file và lưu job config (Story 1.0, 1.1)
FR4: Epic 1 - Quy tắc 1 video = 1 audio + 1 SRT
FR5: Epic 1 - Auto mapping, ambiguity handling, input gating
FR6: Epic 1 - Preset theo brand/channel (Story 1.2, 1.5)
FR7: Epic 1 - Nội dung preset (Story 1.2, 1.5)
FR8: Epic 2 - Queue nền và batch processing
FR9: Epic 4 - Persistence, resume, recovery
FR10: Epic 2 - Detect/replace logo cơ bản
FR11: Epic 2 - Detect/remove subtitle cũ cơ bản
FR12: Epic 2 - Import/render subtitle mới theo audio mới
FR13: Epic 3 - Quick fix subtitle tối giản
FR14: Epic 2 - Audio replacement toàn-video
FR15: Epic 3 - Segment list là review UI chính
FR16: Epic 3 - Risk levels, before/after, apply fix, export gating
FR17: Epic 4 - Export MP4 hàng loạt
FR18: Epic 4 - Risk report, encode summary, audio source summary
FR19: Epic 1 - Các screen nền tảng của setup flow (Story 1.0)
FR20: Epic 3 - Review thủ công và review bắt buộc cho High Risk

## Epic List

### Epic 1: Prepare Job and Map Assets
Người dùng có thể tạo một job re-branding hợp lệ, nhập batch video cùng logo/audio/SRT, chọn preset brand, xác nhận mapping và chỉ chạy khi đầu vào đã sẵn sàng.
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR19
**Stories:** 1.0, 1.1, 1.2, 1.3, 1.4, 1.5

### Epic 2: Run Automated Rebranding
Người dùng có thể chạy pipeline re-branding tự động cho nhiều video để thay logo, thay audio, xử lý subtitle cũ, render subtitle mới, và tạo ra kết quả sơ bộ cùng trạng thái xử lý cho từng video.
**FRs covered:** FR8, FR10, FR11, FR12, FR14

### Epic 3: Review Exceptions and Apply Targeted Fixes
Người dùng có thể tập trung vào đúng các segment bị flag, xem before/after, sửa nhanh logo hoặc subtitle, và đưa video vượt qua gating chất lượng trước export.
**FRs covered:** FR13, FR15, FR16, FR20

### Epic 4: Export Deliverables and Recover Jobs
Người dùng có thể resume job sau gián đoạn, export hàng loạt các video đủ điều kiện, và xem báo cáo đủ ngắn gọn để vận hành và QC.
**FRs covered:** FR9, FR17, FR18

## Epic 1: Prepare Job and Map Assets

Người dùng có thể tạo một job re-branding hợp lệ, nhập batch video cùng logo/audio/SRT, chọn preset brand, xác nhận mapping và chỉ chạy khi đầu vào đã sẵn sàng.

### Story 1.0: Set Up Initial Project Structure and Development Environment

As a developer,
I want thiết lập project structure ban đầu từ template,
So that toàn bộ team có baseline chung để bắt đầu implement.

**Acceptance Criteria:**

**Given** developer clone hoặc khởi tạo project mới
**When** setup hoàn tất
**Then** project chạy được với Tauri + React + TypeScript đã cấu hình
**And** FFmpeg binary đã được tích hợp và accessible từ application layer

**Given** project đã được setup
**When** developer chạy dev server
**Then** app desktop mở được với shell cơ bản (empty state) mà không có lỗi compile

**Given** project đã được setup
**When** developer chạy build
**Then** build thành công và tạo ra executable cho target platform
**And** thư mục output structure cho job data (`job.json`, `videos/`, `segments/`, `reports/`, `logs/`) đã được defined trong constants

**Given** project đã được setup
**When** developer xem module structure
**Then** các frontend modules (`app-shell`, `start-flow`, `preset-management`, `job-review`, `processing-queue`, `segment-review`, `export-reporting`) và backend modules (`job-orchestrator`, `preset-service`, `mapping-service`, `analysis-service`, `review-service`, `render-service`, `persistence-service`, `logging-service`) đã có skeleton folder structure

### Story 1.1: Import Videos and Assets Into a New Job

As a user,
I want nhập video và asset vào một job mới,
So that tôi có thể bắt đầu một workflow re-branding với toàn bộ đầu vào cần thiết ở cùng một nơi.

**Acceptance Criteria:**

**Given** người dùng mở app và chưa có job đang hoạt động
**When** họ kéo thả một hoặc nhiều file vào vùng nhập liệu
**Then** hệ thống tạo một draft job mới
**And** hiển thị workspace của job với danh sách file đã nhập

**Given** người dùng nhập file hợp lệ
**When** import hoàn tất
**Then** hệ thống phân loại đúng file vào một trong các nhóm `video`, `logo`, `audio`, hoặc `SRT`
**And** hiển thị file đó trong job state tương ứng

**Given** người dùng nhập file không được hỗ trợ hoặc không hợp lệ cho task hiện tại
**When** import hoàn tất
**Then** hệ thống không silently bỏ qua file đó
**And** hiển thị thông báo lỗi ngắn gọn, dễ hiểu cho file bị từ chối

**Given** người dùng đã import ít nhất một video và chọn task
**When** draft job được tạo
**Then** hệ thống persist cấu hình job đó vào local storage ngay lập tức
**And** cấu hình bao gồm danh sách file đã import, task đã chọn, và thời điểm tạo

### Story 1.2: Select and Apply a Brand Preset

As a user,
I want chọn một preset theo brand/channel cho job,
So that hệ thống có thể áp dụng default brand rules trước khi tôi xác nhận mapping và readiness.

**Acceptance Criteria:**

**Given** người dùng đang ở flow setup job
**When** màn preset hiển thị
**Then** mỗi preset hiển thị ít nhất brand name, logo mặc định, audio policy, subtitle style preset, và export preset

**Given** người dùng chọn một preset
**When** preset được áp dụng cho job
**Then** hệ thống lưu preset đó vào job state
**And** cập nhật các default brand settings liên quan trong workspace hiện tại

**Given** người dùng đổi preset sau khi job đã có mapping hoặc review data trước đó
**When** họ xác nhận đổi preset
**Then** hệ thống hiển thị cảnh báo rằng các mapping hoặc kết quả detect trước đó có thể mất hiệu lực

### Story 1.3: Review and Resolve Video-to-Asset Mapping

As a user,
I want xem mapping hệ thống tự đề xuất và sửa các mapping sai,
So that mỗi video có đúng logo, audio, và SRT cần thiết trước khi chạy.

**Acceptance Criteria:**

**Given** job đã có video và asset đã import
**When** hệ thống chạy auto-mapping
**Then** hệ thống dùng exact base filename match để map `audio` và `SRT` vào từng video trước
**And** hiển thị mapping được đề xuất trong bảng job setup

**Given** một video có đúng một audio hoặc SRT khớp
**When** mapping hoàn tất
**Then** hệ thống tự gán mapping đó cho video
**And** đánh dấu mapping là hợp lệ

**Given** một video thiếu audio hoặc SRT bắt buộc theo task đã chọn
**When** mapping hoàn tất
**Then** video đó được gắn trạng thái `Input Needs Review`
**And** hệ thống hiển thị rõ input nào còn thiếu

**Given** nhiều file có thể khớp với một video
**When** hệ thống không xác định được mapping duy nhất
**Then** hệ thống không tự chọn ngẫu nhiên
**And** yêu cầu người dùng xác nhận hoặc sửa tay mapping đó

### Story 1.4: Validate Job Readiness Before Processing

As a user,
I want hệ thống kiểm tra readiness của toàn bộ job trước khi chạy,
So that tôi chỉ có thể bấm xử lý khi mọi blocker bắt buộc đã được giải quyết.

**Acceptance Criteria:**

**Given** người dùng vào màn Job Setup / Pre-run Review
**When** bảng readiness hiển thị
**Then** mỗi video hiển thị ít nhất `task`, `preset`, `logo`, `audio`, `SRT`, và `trạng thái mapping`

**Given** một video còn thiếu input bắt buộc hoặc mapping chưa hợp lệ
**When** readiness check chạy
**Then** video đó được đánh dấu blocked
**And** hệ thống hiển thị lý do chặn cụ thể, dễ hiểu

**Given** job còn ít nhất một video bị blocked do thiếu audio hoặc SRT bắt buộc
**When** người dùng bấm `Chạy tự động`
**Then** nút chạy bị disabled hoặc job không được phép bắt đầu
**And** hệ thống hiển thị rõ vì sao job chưa đủ điều kiện chạy

**Given** tất cả các video đã có input hợp lệ và mapping hợp lệ theo task
**When** readiness check hoàn tất
**Then** job được đánh dấu sẵn sàng xử lý
**And** hành động `Chạy tự động` được enable

### Story 1.5: Manage Brand Presets (Create, Edit, Duplicate)

As a user,
I want tạo mới, chỉnh sửa, và duplicate preset theo brand/channel,
So that tôi có thể quản lý nhiều bộ cấu hình thương hiệu mà không phải nhập lại từ đầu mỗi lần.

**Acceptance Criteria:**

**Given** người dùng vào màn Preset Selection
**When** họ chọn `Tạo preset mới`
**Then** hệ thống mở form tạo preset với các trường: brand name, logo mặc định, audio replacement policy, subtitle style preset, layout rules cơ bản, và export preset
**And** lưu preset mới sau khi người dùng xác nhận
**And** preset mới xuất hiện ngay trong danh sách preset của job hiện tại

**Given** người dùng chọn một preset đã có và bấm `Sửa preset`
**When** họ thay đổi một hoặc nhiều trường
**Then** hệ thống lưu thay đổi vào preset đó
**And** nếu job hiện tại đang dùng preset này, hệ thống hiển thị cảnh báo rằng thay đổi có thể ảnh hưởng đến kết quả detect và quick-fix đã có

**Given** người dùng chọn `Duplicate preset` trên một preset đã có
**When** duplicate hoàn tất
**Then** hệ thống tạo bản sao với tên mặc định dạng `[Tên preset gốc] - Copy`
**And** bản sao có thể được chỉnh sửa độc lập mà không ảnh hưởng preset gốc

## Epic 2: Run Automated Rebranding

Người dùng có thể chạy pipeline re-branding tự động cho nhiều video để thay logo, thay audio, xử lý subtitle cũ, render subtitle mới, và tạo ra kết quả sơ bộ cùng trạng thái xử lý cho từng video.

### Story 2.1: Start Batch Processing for a Valid Job

As a user,
I want chạy auto process cho một job đã hợp lệ,
So that hệ thống có thể bắt đầu xử lý hàng loạt các video mà không cần tôi thao tác từng file.

**Acceptance Criteria:**

**Given** job đã qua readiness check và chưa có processing run đang hoạt động
**When** người dùng bấm `Chạy tự động`
**Then** hệ thống chuyển job sang trạng thái `Processing`
**And** tạo queue xử lý cho từng video trong job

**Given** job không hợp lệ hoặc còn blocker bắt buộc
**When** người dùng cố bắt đầu processing
**Then** hệ thống không cho phép start job
**And** hiển thị rõ lý do job chưa thể chạy

**Given** job đã được start thành công
**When** người dùng hoặc hệ thống gửi thêm yêu cầu start lần nữa trong cùng một run
**Then** hệ thống không tạo processing run trùng lặp
**And** giữ trạng thái job nhất quán

**Given** queue đang xử lý video trong nền
**When** người dùng tương tác với bất kỳ phần nào của UI
**Then** UI vẫn phản hồi bình thường và không bị block
**And** processing queue chạy hoàn toàn trên background thread tách biệt

### Story 2.2: Replace Audio for Each Video

As a user,
I want hệ thống thay toàn bộ audio gốc bằng audio mới đã map,
So that video đầu ra dùng đúng audio theo thị trường hoặc brand mới.

**Acceptance Criteria:**

**Given** video có audio mapping hợp lệ và task yêu cầu thay audio
**When** pipeline xử lý chạy đến bước audio
**Then** audio gốc của video được thay hoàn toàn bằng audio mới đã map
**And** hệ thống không mix audio cũ với audio mới

**Given** V1 không hỗ trợ offset, resync, hoặc audio editing theo segment
**When** bước audio processing chạy
**Then** hệ thống chỉ áp dụng full-length audio replacement
**And** không cung cấp hay áp dụng audio transform ngoài scope V1

**Given** một video xử lý audio thất bại
**When** queue tiếp tục chạy các video khác
**Then** video đó được đánh dấu lỗi ở mức per-video
**And** failure không làm dừng toàn bộ job

### Story 2.3: Detect and Replace Logo

As a user,
I want hệ thống tự detect logo cũ và đặt logo mới đè lên,
So that phần lớn video được re-brand tự động trước khi cần review tay.

**Acceptance Criteria:**

**Given** video có logo input hoặc logo từ preset
**When** bước logo detection chạy
**Then** hệ thống xác định vùng logo cũ ở mức cơ bản
**And** lưu detection result để dùng cho replacement và risk evaluation

**Given** detection result hợp lệ
**When** bước logo replacement chạy
**Then** hệ thống overlay logo mới để che phủ vùng logo cũ
**And** áp dụng kết quả theo rule của preset hiện tại

**Given** detect logo không đủ chắc chắn hoặc không ổn định
**When** bước logo processing hoàn tất
**Then** hệ thống sinh risk signal hoặc flag cho phần video liên quan
**And** persist dữ liệu đó cho các bước sau

### Story 2.4: Remove Old Hardcoded Subtitle and Render New Subtitle

As a user,
I want hệ thống xử lý subtitle hardcoded cũ và render subtitle mới từ SRT,
So that video đầu ra mang subtitle mới phù hợp với audio và brand mới.

**Acceptance Criteria:**

**Given** video có SRT mapping hợp lệ và task yêu cầu thay subtitle
**When** bước subtitle region detection chạy
**Then** hệ thống xác định vùng subtitle hardcoded cũ theo segment ở mức cơ bản
**And** lưu detection result để dùng cho xử lý subtitle cũ

**Given** subtitle region đã được detect
**When** bước subtitle removal chạy
**Then** hệ thống áp dụng `blur`, `mask`, hoặc `box fill` để xử lý subtitle cũ
**And** không tự mở rộng sang inpaint nâng cao ngoài scope V1 trừ case dễ đã định nghĩa

**Given** subtitle cũ đã được xử lý và SRT mới hợp lệ
**When** bước subtitle render chạy
**Then** subtitle mới được burn vào video từ SRT đã map
**And** khi task có audio replacement thì subtitle mới được coi là subtitle đi kèm audio mới

### Story 2.5: Persist Processing Status and Flags

As a user,
I want thấy trạng thái xử lý của từng video được cập nhật rõ ràng,
So that tôi biết pipeline đã chạy đến đâu và video nào có vấn đề cần xử lý tiếp.

**Acceptance Criteria:**

**Given** processing đang chạy cho một job batch
**When** từng video đi qua các bước xử lý
**Then** hệ thống cập nhật trạng thái per-video theo lifecycle xử lý
**And** persist progress để job có thể truy vết hoặc resume

**Given** một bước detect hoặc transform sinh ra uncertainty hay lỗi
**When** video hoàn tất processing phase
**Then** hệ thống persist các flags hoặc exception artifacts liên quan
**And** không tự mở UI review trong Epic 2

**Given** toàn bộ queue đã chạy xong hoặc dừng ở mức partial
**When** trạng thái job được tổng hợp
**Then** mỗi video được gán outcome phù hợp như `Review Needed`, `Ready to Export`, hoặc `Failed`
**And** dữ liệu đầu ra đủ để Epic 3 sử dụng cho review exception

**Given** processing queue đang hiển thị
**When** người dùng xem màn hình Processing
**Then** UI hiển thị progress tổng của job, progress theo từng video, và trạng thái hiện tại của từng video
**And** panel log ngắn hiển thị các bước: detect logo, replace audio, detect subtitle region, render subtitle, risk scoring
**And** nếu một video thiếu audio mapping bắt buộc theo task, hệ thống hiển thị trạng thái thiếu đó rõ ràng trong queue

## Epic 3: Review Exceptions and Apply Targeted Fixes

Người dùng có thể tập trung vào đúng các segment bị flag, xem before/after, sửa nhanh logo hoặc subtitle, và đưa video vượt qua gating chất lượng trước export.

### Story 3.1: Open a Review Workspace for Flagged Videos

As a user,
I want mở một review workspace cho các video có exception,
So that tôi đi thẳng vào các video và segment thực sự cần xác nhận hoặc sửa.

**Acceptance Criteria:**

**Given** job có ít nhất một video chứa flags hoặc exception artifacts
**When** người dùng mở review
**Then** hệ thống chỉ hiển thị các video có exception cùng số lượng segment liên quan
**And** nạp segment list, risk data, và preview context từ job snapshot đã persist

**Given** người dùng muốn spot-check một video không có flag
**When** họ mở review thủ công cho video đó
**Then** hệ thống vẫn mở được review workspace
**And** thể hiện rõ đây là review tùy chọn chứ không phải blocker bắt buộc

### Story 3.2: Prioritize and Filter Exception Segments

As a user,
I want ưu tiên và lọc các segment theo mức độ rủi ro và loại lỗi,
So that tôi có thể xử lý đúng các vấn đề nghiêm trọng nhất trước.

**Acceptance Criteria:**

**Given** review workspace đã mở cho một video
**When** segment list hiển thị
**Then** mỗi segment hiển thị ít nhất time range, issue type, risk level, và review status

**Given** người dùng chưa thay đổi thứ tự hiển thị
**When** segment list được nạp
**Then** hệ thống sắp xếp mặc định `High Risk` trước rồi theo thời gian

**Given** người dùng lọc theo risk level, issue type, hoặc trạng thái resolved/unresolved
**When** filter được áp dụng
**Then** segment list chỉ hiển thị các segment phù hợp với điều kiện đã chọn

### Story 3.3: Preview Before and After for a Selected Segment

As a user,
I want xem before/after cho đúng segment tôi đang review,
So that tôi có thể quyết định nhanh xem có cần sửa hay có thể chấp nhận kết quả hiện tại.

**Acceptance Criteria:**

**Given** người dùng chọn một segment trong segment list
**When** preview workspace cập nhật
**Then** hệ thống jump đến đúng time range của segment đó
**And** giữ preview ở ngữ cảnh của đúng segment đã chọn

**Given** người dùng bật before/after hoặc split preview
**When** preview mode thay đổi
**Then** hệ thống hiển thị được hai trạng thái trước và sau xử lý cho cùng segment và cùng time range

**Given** người dùng đang ở trạng thái paused trên một segment
**When** preview workspace hiển thị segment đó
**Then** hệ thống hiển thị overlay handles cho: logo mới, blur/mask box của subtitle cũ, và subtitle mới
**And** người dùng có thể kéo thả các handles để điều chỉnh vị trí và kích thước trực tiếp trên preview

### Story 3.4: Apply Quick Fixes to Logo Issues

As a user,
I want sửa nhanh các lỗi liên quan đến logo trên segment bị flag,
So that tôi có thể chỉnh đúng vị trí hoặc kích thước logo mà không cần mở full editor.

**Acceptance Criteria:**

**Given** một segment có issue type liên quan đến logo
**When** quick fix panel mở cho segment đó
**Then** hệ thống cho phép dùng các control trong scope V1 gồm `move/resize logo` và `reset to preset default`

**Given** người dùng áp dụng logo fix cho segment hiện tại
**When** họ bấm `Áp dụng cho segment này`
**Then** hệ thống persist quick fix state cho logo của segment đó
**And** cập nhật review status tương ứng

**Given** người dùng chọn nhiều segment compatible
**When** họ bấm `Áp dụng cho các segment đã chọn`
**Then** hệ thống copy logo fix sang các segment compatible đã chọn
**And** không áp dụng im lặng cho segment không compatible

**Given** người dùng muốn tắt logo overlay tạm thời để kiểm tra vùng bên dưới
**When** họ toggle `bật/tắt logo overlay`
**Then** hệ thống ẩn hoặc hiện logo mới trong preview mà không thay đổi quick fix state đã lưu

**Given** người dùng muốn đánh dấu segment logo là đã xem xong mà không sửa gì
**When** họ bấm `Đánh dấu đã review`
**Then** hệ thống cập nhật review status của segment đó thành `Accepted`
**And** segment không còn hiện là unreviewed trong segment list

**Given** người dùng muốn hoàn tác toàn bộ logo fix trên segment hiện tại
**When** họ bấm `Khôi phục mặc định`
**Then** hệ thống reset logo overlay về giá trị mặc định từ preset
**And** xóa custom quick fix state của logo trên segment đó

### Story 3.5: Apply Quick Fixes to Subtitle Issues

As a user,
I want sửa nhanh vùng subtitle cũ và subtitle mới trên segment bị flag,
So that tôi có thể xử lý ngoại lệ subtitle mà không cần timeline editor đầy đủ.

**Acceptance Criteria:**

**Given** một segment có issue type liên quan đến subtitle
**When** quick fix panel mở cho segment đó
**Then** hệ thống cho phép dùng các control trong scope V1 gồm `move/resize blur or mask box`, `đổi vị trí subtitle mới`, `đổi scale subtitle`, và `đổi subtitle style preset`

**Given** người dùng áp dụng subtitle fix cho segment hiện tại
**When** họ bấm `Áp dụng cho segment này`
**Then** hệ thống persist quick fix state cho subtitle của segment đó
**And** cập nhật review status tương ứng

**Given** người dùng chọn nhiều segment compatible
**When** họ bấm `Áp dụng cho các segment đã chọn`
**Then** hệ thống copy subtitle fix sang các segment compatible đã chọn
**And** không áp dụng im lặng cho segment không compatible

**Given** quick fix panel mở cho một segment có subtitle old region
**When** người dùng chọn mode xử lý subtitle cũ
**Then** hệ thống cho phép chọn một trong ba mode: `blur`, `mask`, hoặc `fill`
**And** preview cập nhật ngay để thể hiện mode đang được chọn

**Given** người dùng muốn đánh dấu segment subtitle là đã xem xong mà không sửa gì
**When** họ bấm `Đánh dấu đã review`
**Then** hệ thống cập nhật review status của segment đó thành `Accepted`
**And** segment không còn hiện là unreviewed trong segment list

**Given** người dùng muốn hoàn tác toàn bộ subtitle fix trên segment hiện tại
**When** họ bấm `Khôi phục mặc định`
**Then** hệ thống reset tất cả subtitle controls (vị trí, scale, style, mode) về giá trị mặc định từ preset
**And** xóa custom quick fix state của subtitle trên segment đó

### Story 3.6: Resolve Review Gating and Mark Video Ready

As a user,
I want đánh dấu video đã vượt qua review gating,
So that video chỉ được chuyển sang `Ready to Export` khi mọi blocker bắt buộc đã được xử lý hoặc chấp nhận đúng cách.

**Acceptance Criteria:**

**Given** video còn ít nhất một segment `High Risk` chưa được xử lý hoặc xác nhận
**When** người dùng cố đánh dấu video là ready
**Then** hệ thống chặn hành động đó
**And** hiển thị rõ các blocker còn lại

**Given** một segment `Medium Risk` đã được người dùng xem và chấp nhận
**When** người dùng đánh dấu chấp nhận segment đó
**Then** hệ thống persist review decision tương ứng
**And** không bắt buộc người dùng phải sửa segment này

**Given** tất cả segment `High Risk` của video đã được sửa hoặc xác nhận đúng rule
**When** review gating được tổng hợp lại
**Then** video được phép chuyển sang `Ready to Export`
**And** hệ thống persist quyết định đó cho bước export sau này

## Epic 4: Export Deliverables and Recover Jobs

Người dùng có thể resume job sau gián đoạn, export hàng loạt các video đủ điều kiện, và xem báo cáo đủ ngắn gọn để vận hành và QC.

### Story 4.1: Resume an Interrupted Job from the Last Stable Checkpoint

As a user,
I want mở lại một job đã bị gián đoạn từ checkpoint an toàn gần nhất,
So that tôi không phải làm lại từ đầu sau crash hoặc mất điện.

**Acceptance Criteria:**

**Given** một job đã được persist trước đó với trạng thái hợp lệ
**When** ứng dụng mở lại hoặc người dùng chọn resume job
**Then** hệ thống khôi phục job manifest, per-video status, segment flags, và quick-fix state đã lưu
**And** đưa job trở lại checkpoint ổn định gần nhất thay vì khởi động lại toàn bộ workflow

**Given** một video đang export dở khi app bị gián đoạn
**When** job được resume
**Then** hệ thống không bắt buộc resume đúng giữa tiến trình encode
**And** cho phép export lại video đó từ đầu như một bước an toàn

### Story 4.2: Validate Export Readiness for Batch Output

As a user,
I want hệ thống xác thực rõ video nào đủ điều kiện export và video nào còn bị chặn,
So that tôi không vô tình xuất các video chưa vượt qua đúng các rule review.

**Acceptance Criteria:**

**Given** người dùng vào màn Export với một batch có nhiều trạng thái video khác nhau
**When** export readiness được tính toán
**Then** hệ thống phân biệt rõ video `Ready to Export` với video còn bị blocked
**And** hiển thị lý do blocked ở mức per-video

**Given** một video còn blocker review hoặc trạng thái lỗi
**When** người dùng chuẩn bị export batch
**Then** video đó không được đưa vào tập export-ready
**And** hệ thống tiêu thụ trạng thái readiness từ Epic 3 thay vì tạo lại rule gating mới

### Story 4.3: Export All Ready Videos to MP4

As a user,
I want export hàng loạt các video đã sẵn sàng sang MP4,
So that tôi có thể lấy output cuối cùng cho vận hành mà không phải export từng video một.

**Acceptance Criteria:**

**Given** có ít nhất một video `Ready to Export`
**When** người dùng bấm `Export All Ready Videos`
**Then** hệ thống chỉ export các video đủ điều kiện
**And** xuất output theo preset MP4 của V1 vào output location đã xác định
**And** ưu tiên chất lượng encode ổn định hơn tốc độ tối đa

**Given** batch export đang chạy và một video export thất bại
**When** batch export tiếp tục
**Then** video đó được đánh dấu `Failed` ở mức per-video
**And** các video khác vẫn tiếp tục export nếu không có lỗi riêng

### Story 4.4: Persist Per-Video Export Outcome and Summary Report

As a user,
I want xem và lưu lại outcome cuối cùng của từng video sau export,
So that tôi và QC có đủ thông tin để tin output mà không phải xem lại toàn bộ video.

**Acceptance Criteria:**

**Given** một video đã được export hoặc đã hoàn tất trạng thái cuối cùng của batch
**When** report được tạo
**Then** hệ thống persist outcome của video đó với ít nhất các trường:
`trạng thái cuối`, `encode summary`, `audio source used`, `số segment bị flag`, `số segment đã sửa`, `before/after spot check`, `output path`

**Given** người dùng mở report của một video
**When** họ muốn kiểm tra thêm video đó
**Then** hệ thống cho phép quay lại review từ report context của video
**And** vẫn giữ nguyên dữ liệu outcome đã persist
