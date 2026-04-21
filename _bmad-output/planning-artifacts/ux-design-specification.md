---
stepsCompleted:
  - step-01-init
inputDocuments:
  - D:\WORKING\BMAD\_bmad-output\planning-artifacts\prd.md
  - D:\WORKING\BMAD\_bmad-output\brainstorming\brainstorming-session-2026-04-20-1600.md
projectName: Desktop Video Rebranding App
documentType: ux-design-specification
status: draft
---

# UX Design Specification - Desktop Video Rebranding App

**Author:** BOSS  
**Date:** 2026-04-20

---

## 1. UX Vision

Desktop Video Rebranding App V1 phải cho người dùng cảm giác đang chạy một workflow rõ ràng, không phải đang học một video editor mới. Trải nghiệm cốt lõi là:

- nhập video và asset nhanh
- chọn preset theo brand
- để hệ thống xử lý phần lớn công việc
- thay audio toàn-video bằng file mới đã map
- chỉ sửa đúng các đoạn bị nghi ngờ
- export hàng loạt với cảm giác kiểm soát được chất lượng

V1 theo hướng `wizard-first, review-exception-centered`. Review là bước quan trọng, nhưng không bao giờ được làm người mới thấy đây là một timeline editor phức tạp.

## 2. UX Principles

### 2.1 Task-driven, not tool-driven

Người dùng không nên phải nghĩ theo khái niệm như layer, track, keyframe, hay compositing. Hệ thống phải dẫn bằng task:

- thay logo
- thay audio
- thay subtitle
- review đoạn lỗi
- export

### 2.2 Auto first, human correction second

Hệ thống luôn tự làm trước. Người dùng chỉ xuất hiện ở chỗ có rủi ro hoặc ambiguity.

### 2.3 Progressive disclosure

- Người mới chỉ thấy các bước cơ bản.
- Người vận hành có thêm panel review và mapping.
- Không hiển thị controls nâng cao trước khi có ngữ cảnh cụ thể.

### 2.4 Segment, not timeline

Review chính bám theo `segment list`. Timeline chỉ để hỗ trợ định vị, không phải trung tâm của thao tác.

### 2.5 Speed with confidence

Mục tiêu UX không phải nhanh bằng mọi giá. Mục tiêu là:

- nhanh ở phần lặp
- rõ ràng ở phần rủi ro
- tin cậy khi chuẩn bị export

## 3. Primary Users and UX Implications

### 3.1 Chị Mai - Core operator

UX implication:

- cần import batch nhanh
- cần preset rõ ràng
- cần review rất có trọng tâm
- cần copy fix giữa các segment

### 3.2 Em Hà - New or low-skill user

UX implication:

- cần màn hình đầu cực rõ
- cần ít thuật ngữ kỹ thuật
- cần mapping dễ kiểm tra
- cần hệ thống chặn lỗi sớm thay vì để fail muộn

### 3.3 Anh Nam - Quality checker

UX implication:

- cần thấy risk segments và before/after
- cần trạng thái `đã review` rõ
- cần report đủ ngắn để ra quyết định nhanh
- cần nhìn rõ audio source cuối cùng của từng video

## 4. UX Model

### 4.1 Mode model

V1 có 3 mode chính:

1. `Setup Mode`
- nhập file
- chọn task
- chọn preset
- kiểm tra mapping

2. `Review Mode`
- chỉ mở khi có nhu cầu review hoặc người dùng chủ động mở
- segment list là điều hướng chính
- preview và quick-fix panel chỉ phục vụ segment đang chọn

3. `Export Mode`
- xác nhận danh sách video sẵn sàng export
- theo dõi trạng thái xuất
- xem report sau xử lý

### 4.2 Transition rules

- từ `Setup Mode` sang `Auto Process` khi job hợp lệ
- từ `Auto Process` sang `Review Mode` nếu có `High Risk` hoặc user chọn review thủ công
- từ `Auto Process` sang `Export Mode` nếu không có `High Risk` và user bỏ qua spot-check
- từ `Review Mode` sang `Export Mode` khi tất cả segment bắt buộc đã được xử lý hoặc xác nhận

Audio mapping được xử lý hoàn toàn ở `Setup Mode`. Nếu thiếu hoặc sai audio, job không nên đi vào review mode.

## 5. Information Architecture

### 5.1 Top-level navigation

V1 không nên có thanh navigation kiểu app nhiều module. Thay vào đó dùng flow-based layout:

- `Start`
- `Preset`
- `Review Job`
- `Processing`
- `Review Exceptions`
- `Export`
- `Report`

### 5.2 Persistent objects

Các object người dùng cần hiểu xuyên suốt:

- `Job`
- `Video`
- `Preset`
- `Mapping`
- `Segment`
- `Risk level`
- `Audio source`
- `Export result`

### 5.3 Mental model

Người dùng nên hiểu hệ thống theo câu đơn giản:

`Một job gồm nhiều video. Mỗi video dùng một preset. Hệ thống tự map logo, audio, và SRT, tự xử lý, đánh dấu segment rủi ro, rồi mình sửa nhanh trước khi export.`

## 6. Screen-by-Screen UX Specification

## 6.1 Start Screen

### Mục tiêu

Giúp user bắt đầu trong vòng vài giây mà không bị áp lực về thuật ngữ hoặc cấu hình.

### Nội dung chính

- câu hỏi lớn: `Bạn muốn làm gì?`
- 4 task cards:
  - `Thay logo`
  - `Thay audio`
  - `Thay subtitle`
  - `Thay logo, audio, và subtitle`
- vùng drag-and-drop trung tâm:
  - video
  - logo
  - audio
  - SRT
- nút phụ:
  - `Dùng preset có sẵn`
  - `Mở job gần đây`

### Hành vi

- khi file được kéo vào, hệ thống tự nhận diện loại file
- nếu chưa chọn task, hệ thống gợi ý task dựa trên loại file đã nhập
- nếu thiếu file bắt buộc cho task đã chọn, hiển thị guidance ngắn ngay tại chỗ

### Tone UI

- sáng, thoáng, hiện đại
- nhiều khoảng trắng
- icon lớn, copy ngắn
- tránh panel chồng lớp ở màn đầu

## 6.2 Preset Selection Screen

### Mục tiêu

Giúp user hiểu preset là “gói thương hiệu” chứ không phải cấu hình kỹ thuật rời rạc.

### Nội dung chính

- danh sách preset theo brand/channel
- preview card cho preset đang chọn:
  - tên brand
  - logo mặc định
  - audio policy
  - subtitle style preset
  - export preset
  - notes ngắn
- action:
  - `Chọn preset này`
  - `Sửa preset`
  - `Tạo preset mới`

### Hành vi

- preset được apply cho toàn bộ job theo mặc định
- nếu user đổi preset sau khi đã review segment, phải hiện cảnh báo rằng review cũ có thể mất hiệu lực

## 6.3 Job Review / Pre-run Screen

### Mục tiêu

Giải quyết ambiguity trước khi auto process bắt đầu.

### Nội dung chính

- bảng danh sách video
- cột:
  - tên video
  - task
  - preset
  - logo
  - audio
  - SRT
  - trạng thái mapping
- trạng thái mapping có thể là:
  - `Matched`
  - `Missing`
  - `Needs Review`

### Hành vi

- exact base filename match được gán tự động
- user có thể sửa mapping bằng inline picker
- nút `Chạy tự động` bị disable nếu task yêu cầu subtitle nhưng còn video chưa có SRT hợp lệ
- nút `Chạy tự động` bị disable nếu task yêu cầu audio nhưng còn video chưa có audio hợp lệ

### Copy guideline

Không dùng câu như `invalid binding`. Dùng câu dễ hiểu:

- `Chưa tìm thấy SRT khớp`
- `Chưa tìm thấy audio khớp`
- `Có nhiều file có thể phù hợp`
- `Cần bạn chọn đúng file`

## 6.4 Processing Screen

### Mục tiêu

Cho user cảm giác hệ thống đang làm việc có trật tự và có thể tin cậy.

### Nội dung chính

- progress tổng của job
- progress theo từng video
- trạng thái:
  - Imported
  - InputNeedsReview
  - Processing
  - Review Needed
  - Ready to Export
  - Failed
- panel log ngắn:
  - detect logo
  - replace audio
  - detect subtitle region
  - render subtitle
  - risk scoring

### Hành vi

- nếu một video fail, không chặn toàn job nếu các video khác vẫn xử lý được
- nếu video vào `Review Needed`, user có thể click để mở review ngay hoặc tiếp tục chờ toàn job hoàn tất

## 6.5 Review Exceptions Screen

### Mục tiêu

Đây là màn hình quan trọng nhất của V1. Mục tiêu là sửa nhanh, không “dựng lại” video.

Audio replacement không phải là quick-fix trong review screen. Nếu audio mapping sai hoặc thiếu, vấn đề phải được chặn ở setup mode thay vì đưa sang review mode.

### Layout

- trái: `Segment List`
- giữa: `Preview Workspace`
- phải: `Quick Fix Panel`
- dưới cùng: timeline nhẹ, chỉ để định vị

### 6.5.1 Segment List

#### Nội dung mỗi row

- time range
- loại vấn đề:
  - logo
  - subtitle old region
  - subtitle new placement
  - mixed
- risk level:
  - Low
  - Medium
  - High
- trạng thái review:
  - chưa xem
  - đã sửa
  - đã xác nhận

#### Hành vi

- sort mặc định: `High Risk` trước, rồi theo thời gian
- cho phép filter theo risk hoặc issue type
- cho phép chọn nhiều segment để apply cùng một fix khi phù hợp

### 6.5.2 Preview Workspace

#### Thành phần

- video player
- before/after toggle
- split preview mode
- overlay handles cho:
  - logo mới
  - blur/mask box cho subtitle cũ
  - subtitle mới

#### Hành vi

- khi chọn segment, preview tự jump đến time range đó
- when paused, handles hiển thị rõ để kéo thả
- khi đang play, controls tối giản hơn để tránh rối

### 6.5.3 Quick Fix Panel

#### Nhóm `Logo`

- bật/tắt logo overlay
- move
- resize
- reset to preset default

#### Nhóm `Subtitle Old Region`

- move blur/mask box
- resize blur/mask box
- chọn mode:
  - blur
  - mask
  - fill

#### Nhóm `Subtitle New`

- position preset
- scale
- style preset

#### Action nhóm cuối

- `Áp dụng cho segment này`
- `Áp dụng cho các segment đã chọn`
- `Đánh dấu đã review`
- `Khôi phục mặc định`

### 6.5.4 Review rules in UI

- `High Risk` phải có action rõ ràng trước khi video được export
- `Medium Risk` cho phép `Đánh dấu chấp nhận`
- `Low Risk` không bắt người dùng vào review, nhưng vẫn xuất hiện trong spot-check nếu user muốn

## 6.6 Export Screen

### Mục tiêu

Tạo cảm giác “ready for delivery”, không phải “configure encoder”.

### Nội dung chính

- danh sách video sẵn sàng export
- các video bị chặn vì chưa review xong
- output folder
- export preset summary
- audio source summary
- nút chính:
  - `Export All Ready Videos`

### Hành vi

- video còn `High Risk` không được nằm trong danh sách export-ready
- user thấy rõ vì sao một video chưa thể export

## 6.7 Report Screen

### Mục tiêu

Cho người vận hành và QC đủ thông tin để tin tưởng output mà không phải xem lại toàn bộ video.

### Nội dung chính cho mỗi video

- trạng thái cuối:
  - Done
  - Review Needed
  - Failed
- encode summary
- audio source used
- số segment bị flag
- số segment đã sửa
- before/after spot check thumbnails

### Hành vi

- cho phép mở lại review từ report nếu user muốn kiểm tra thêm

## 7. Key Interaction Patterns

### 7.1 Drag-and-drop ingestion

- tất cả input chính vào qua một vùng chung
- hệ thống gán role cho file tự động
- nếu ambiguity, hỏi đúng một câu ngắn tại chỗ

### 7.2 Inline correction, not modal overload

- mapping sửa trực tiếp trong bảng
- quick fix diễn ra trong side panel
- tránh popup nối popup

Audio mapping được sửa ở đây giống với SRT mapping, không kéo vào review screen.

### 7.3 Batch confidence workflow

- auto process tất cả video
- gom các vấn đề vào review queue
- user xử lý từ vấn đề nghiêm trọng nhất trở xuống

### 7.4 Copy-forward fixes

Đây là pattern quan trọng cho persona Chị Mai.

- nếu nhiều segment gần nhau cùng lỗi, user có thể apply một fix sang nhiều segment
- hệ thống phải giúp giảm thao tác lặp ở đây

## 8. Visual Direction

### 8.1 Overall look

- nền sáng
- panel trắng và xám nhạt
- accent xanh teal hoặc xanh dương sạch
- typography hiện đại, rõ ràng, không quá “enterprise”

### 8.2 Density

- Start/Preset screens: mật độ thấp
- Review screen: mật độ trung bình
- không dùng density cao kiểu workstation kỹ thuật ở V1

### 8.3 Risk color system

- `Low Risk`: xám xanh nhạt
- `Medium Risk`: vàng/amber
- `High Risk`: đỏ cam
- `Done/Reviewed`: xanh lá nhẹ

Không dùng màu quá bão hòa trên nền sáng vì dễ làm màn review trông nặng nề.

## 9. Content and Microcopy Guidelines

### 9.1 Tone

- rõ
- ngắn
- không kỹ thuật quá mức
- không đổ lỗi cho user

### 9.2 Good microcopy examples

- `Chưa tìm thấy SRT khớp cho video này`
- `Chưa tìm thấy audio khớp cho video này`
- `Đã phát hiện 3 đoạn cần bạn kiểm tra`
- `Video này đã sẵn sàng để export`
- `Đổi preset có thể làm mất hiệu lực các chỉnh sửa trước đó`

### 9.3 Avoid

- `Fatal error`
- `Invalid configuration state`
- `Non-blocking warning severity level 2`

## 10. Empty, Loading, and Error States

### 10.1 Empty states

- Start: hướng dẫn kéo thả rất ngắn
- Preset: nếu chưa có preset, cho tạo preset đầu tiên
- Review: nếu không có exception, hiển thị thông điệp `Không có đoạn bắt buộc phải sửa`

### 10.2 Loading states

- processing queue phải cho cảm giác tiến triển liên tục
- mỗi step có label ngắn, tránh spinner vô nghĩa

### 10.3 Error states

- lỗi mapping input phải chặn sớm ở pre-run
- lỗi từng video không nên phá toàn bộ job
- nếu resume được, câu chính nên là:
  - `Job trước đó đã được khôi phục`

## 11. Accessibility and Responsiveness

### 11.1 Accessibility

- contrast tốt trên nền sáng
- controls kéo thả phải đủ lớn
- không chỉ dựa vào màu để phân biệt risk
- cần icon + label + màu

### 11.2 Responsiveness

V1 là desktop-first nhưng vẫn nên scale cho cửa sổ nhỏ hơn:

- dưới width hẹp, quick fix panel có thể collapse
- segment list giữ được khả năng filter nhanh
- preview vẫn là trọng tâm

## 12. UX Risks

- nếu segment bị flag quá nhiều, review screen sẽ mất lợi thế “exception-first”
- nếu mapping sai nhưng không bị chặn sớm, người dùng sẽ mất niềm tin vào pipeline
- nếu quick fix panel lộ quá nhiều control cùng lúc, Em Hà sẽ bị overwhelm
- nếu preview không đủ rõ before/after, QC sẽ không tin output
- nếu audio được thay nhưng không hiển thị rõ audio source cuối cùng, QC sẽ khó tin output

## 13. Wireframe Priorities

Nếu chuyển sang wireframe hoặc UI design, cần ưu tiên theo thứ tự:

1. Start screen
2. Pre-run mapping screen
3. Exception review screen
4. Processing queue
5. Export/report

## 14. Handoff Notes for Architecture and UI

- UX mặc định không cần full timeline engine ở lớp đầu
- architecture phải hỗ trợ segment-based state, không chỉ video-level state
- review UI cần persist quick fixes theo segment
- risk scoring phải trả về output đủ dùng cho segment list và mode transition
- export pipeline phải hiểu rõ video nào còn bị chặn vì `High Risk`
- setup flow phải hiểu rõ audio là dữ liệu toàn-video, không phải dữ liệu theo segment

## 15. Summary

UX của Desktop Video Rebranding App V1 phải khiến người dùng cảm thấy họ đang điều phối một workflow re-branding thông minh, chứ không phải học dựng video. Mọi quyết định giao diện đều nên phục vụ một câu hỏi duy nhất: `Làm sao để người dùng xử lý nhanh nhiều video, nhưng chỉ phải tập trung vào đúng những đoạn hệ thống chưa đủ chắc chắn?`
