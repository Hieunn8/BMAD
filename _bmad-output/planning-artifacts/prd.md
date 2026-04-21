---
stepsCompleted:
  - step-01-init
inputDocuments:
  - D:\WORKING\BMAD\_bmad-output\brainstorming\brainstorming-session-2026-04-20-1600.md
workflowType: prd
projectName: Desktop Video Rebranding App
version: v1
status: draft
---

# Product Requirements Document - Desktop Video Rebranding App

**Author:** BOSS  
**Date:** 2026-04-20  
**Document Language:** Vietnamese

## 1. Tóm Tắt Sản Phẩm

Desktop Video Rebranding App là ứng dụng desktop semi-auto giúp xử lý batch video re-branding theo preset thương hiệu. V1 tập trung vào thay logo cũ bằng logo mới, thay toàn bộ audio của video bằng audio mới, che/xử lý subtitle hardcoded cũ ở mức blur hoặc mask, chèn subtitle mới từ SRT gắn với audio mới, và dẫn người dùng đi qua quy trình review theo exception trước khi export.

Sản phẩm không định vị là full video editor. V1 là một task-driven workflow tool cho nhu cầu re-branding video lặp lại, với trọng tâm là xử lý nhanh, nhất quán, và đủ tin cậy ở quy mô vận hành 5-20 video mỗi job.

## 2. Bài Toán Kinh Doanh

Người dùng hiện đang phải dùng nhiều công cụ rời rạc hoặc chỉnh tay lặp đi lặp lại để:

- thay logo thương hiệu trên video nguồn
- thay audio theo thị trường hoặc bản phát hành mới
- xử lý subtitle hardcoded cũ không còn phù hợp
- chèn subtitle mới theo brand hoặc thị trường mới
- rà soát thủ công gần như toàn bộ video sau khi chỉnh sửa

Quy trình này chậm, dễ lỗi, khó tái sử dụng, và không phù hợp với nhu cầu xử lý lô video lặp lại mỗi ngày.

## 3. Mục Tiêu V1

### 3.1 Mục tiêu chính

- Giảm mạnh thao tác lặp khi thay logo, audio, và subtitle trên nhiều video.
- Cho phép xử lý hàng loạt theo preset thương hiệu.
- Tự động xử lý phần lớn trường hợp phổ biến, nhưng không cố giả vờ “one-click hoàn hảo”.
- Chuyển nỗ lực của người dùng từ “chỉnh toàn bộ video” sang “review đúng segment nghi ngờ”.
- Giữ chất lượng output ổn định và đủ tin cậy cho vận hành thực tế.

### 3.2 Nguyên tắc sản phẩm V1

- Auto xử lý trước, user chỉ sửa phần AI hoặc rule engine không chắc chắn.
- Review theo exception, không bắt xem lại toàn bộ video.
- Preset và batch là lõi sản phẩm, không phải tính năng phụ.
- UX mặc định phải dễ dùng cho người mới, nhưng không làm chậm người dùng vận hành.

### 3.3 Non-goals của V1

- Không xây dựng full timeline editor kiểu Premiere.
- Không giải quyết fingerprint variation trong V1.
- Không cam kết inpaint chất lượng cao cho mọi nền cảnh phức tạp.
- Không đưa vào QC metric nặng như VMAF, PSNR, SSIM ở V1.
- Không làm audio offset, audio resync, hoặc audio editing theo segment trong V1.

### 3.4 Ngưỡng thành công vận hành cho V1

Để tránh diễn giải mơ hồ, V1 dùng các ngưỡng vận hành mục tiêu sau:

- ít nhất 70% video trong một job phổ biến đi hết pipeline mà không cần quick-fix thủ công quá 3 segment mỗi video
- trung bình không quá 20% tổng segment của một video bị đẩy vào trạng thái cần review
- thời gian sửa tay trung bình cho một segment bị flag nên dưới 30 giây trong các case phổ biến
- tỷ lệ export thất bại do lỗi hệ thống trong job chuẩn phải dưới 5%
- với job đã review xong, người dùng phải export được toàn bộ các video `Ready to Export` mà không cần cấu hình kỹ thuật bổ sung

## 4. Personas Mục Tiêu

### 4.1 Persona chính: Chị Mai - Content Re-packager

**Bối cảnh**

- Xử lý video có sẵn để re-brand cho thị trường mới.
- Có sẵn logo, audio mới, và SRT mới.
- Cần workflow nhanh, nhất quán, ít chỉnh tay.

**Nhu cầu cốt lõi**

- Import video + logo + audio + SRT.
- Tự detect logo và subtitle cũ ở mức đủ tốt.
- Thay audio toàn-video theo file mới đã map.
- Chỉ review một số edge case.
- Áp dụng preset cho cả lô video.

### 4.2 Persona UX dẫn hướng: Em Hà - Người dùng không rành kỹ thuật

**Bối cảnh**

- Không quen tool dựng video phức tạp.
- Cần dùng được nhanh trong lần đầu.
- Muốn hệ thống dẫn dắt theo task thay vì học nhiều menu.

**Nhu cầu cốt lõi**

- Màn hình khởi đầu đơn giản.
- Drag-and-drop file.
- Workflow rõ ràng: nhập file, chạy, sửa nhanh, export.
- Không bị lộ quá nhiều setting kỹ thuật ngay từ đầu.

### 4.3 Persona phụ: Anh Nam - QC Lead

**Bối cảnh**

- Chịu trách nhiệm chất lượng output.
- Không thể xem lại toàn bộ video thủ công.

**Nhu cầu cốt lõi**

- Có encode summary.
- Có risk segments và before/after spot check.
- Có báo cáo đủ để quyết định pass hay review thêm.

## 5. Định Nghĩa Sản Phẩm V1

V1 là một desktop app semi-auto batch video re-branding với luồng mặc định:

1. Import video, logo, audio, SRT
2. Chọn preset theo brand/channel
3. Chạy auto process
4. Review exception theo segment
5. Export MP4 và sinh báo cáo cơ bản

Preset được gắn theo brand/channel là chính. Bên trong preset có thể chứa các rule layout cơ bản, audio policy cơ bản, và style subtitle mặc định để hỗ trợ detect và render ổn định hơn cho những nguồn video quen thuộc.

## 6. User Flow Cấp Cao

### 6.1 Start

Người dùng vào màn hình đầu và chọn một trong các task:

- Thay logo
- Thay audio
- Thay subtitle
- Thay logo, audio, và subtitle

Sau đó người dùng kéo thả video, logo, audio và SRT vào cùng một khu vực nhập liệu.

### 6.2 Preset

Người dùng chọn preset theo brand/channel.

Preset xác định:

- logo mặc định
- audio replacement policy
- style subtitle mặc định
- một số rule layout cơ bản
- export preset cơ bản

### 6.3 Auto Process

Ứng dụng chạy pipeline tự động cho từng video:

- detect logo cũ ở mức cơ bản
- detect subtitle hardcoded cũ theo segment
- thay toàn bộ audio track của video bằng audio mới đã map
- đặt logo mới đè lên logo cũ
- blur hoặc mask vùng subtitle cũ
- render subtitle mới từ SRT tương ứng với audio mới
- gắn cờ các segment có confidence thấp hoặc có rủi ro

Sau bước này, mỗi video phải rơi vào một trong hai trạng thái:

- `Ready to Export` nếu không có segment nào vượt ngưỡng review bắt buộc
- `Review Needed` nếu có ít nhất một segment bị gắn mức rủi ro cần người dùng xác nhận hoặc sửa

### 6.4 Review Exceptions

Người dùng không bị kéo vào một editor đầy đủ. Họ vào màn hình review với segment list là UI chính.

Người dùng chỉ kiểm tra và sửa những segment được flag:

- chỉnh vị trí và kích thước logo
- chỉnh vùng blur/mask cho subtitle cũ
- chỉnh vị trí, scale và preset style cho subtitle mới
- áp dụng fix cho segment hiện tại hoặc các segment được chọn

Audio replacement trong V1 là thao tác toàn-video, không phải chỉnh theo segment. Audio không xuất hiện như một quick-fix trong review panel; hệ thống chỉ cần đảm bảo audio mapping đúng trước khi chạy.

Review mode của V1 tuân theo các rule chuyển trạng thái sau:

- người dùng có thể mở review thủ công ngay cả khi video không có segment bị flag
- nếu một video có bất kỳ segment mức `High Risk`, video đó không được export trước khi người dùng đánh dấu đã review
- nếu video chỉ có segment mức `Medium Risk`, hệ thống cho phép user review rồi xác nhận bỏ qua
- nếu video không có segment bị flag, review chỉ là spot-check tùy chọn
- khi tất cả segment bắt buộc đã được sửa hoặc xác nhận, video chuyển sang `Ready to Export`

### 6.5 Export

Sau khi review xong, người dùng export hàng loạt ra MP4. Mỗi video có encode summary, audio source summary, trạng thái job, và spot check cho những vùng đã bị chỉnh sửa hoặc bị flag.

## 7. Functional Requirements

### 7.1 Ingest & Input Management

- Hệ thống phải cho phép import một hoặc nhiều video trong cùng một job.
- Hệ thống phải cho phép import logo, audio mới, và SRT phục vụ re-branding.
- Hệ thống phải tự nhận diện loại file cơ bản từ extension và gán đúng vai trò trong pipeline.
- Hệ thống phải cho phép người dùng lưu cấu hình job trước khi chạy.
- V1 chỉ hỗ trợ mỗi video map với đúng một audio output và đúng một SRT output.

#### Quy tắc mapping file cho V1

- hệ thống phải cố gắng map SRT vào video bằng exact base filename match trước
- hệ thống phải cố gắng map audio vào video bằng exact base filename match trước
- nếu có đúng một SRT khớp với một video, hệ thống tự gán mapping đó
- nếu có đúng một audio khớp với một video, hệ thống tự gán mapping đó
- nếu một video không có SRT khớp, video đó được gắn trạng thái `Input Needs Review` trước khi chạy
- nếu một video không có audio khớp cho task yêu cầu audio, video đó được gắn trạng thái `Input Needs Review` trước khi chạy
- nếu nhiều SRT cùng có khả năng khớp với một video, hệ thống không được tự chọn ngẫu nhiên mà phải yêu cầu user xác nhận
- nếu nhiều file audio cùng có khả năng khớp với một video, hệ thống không được tự chọn ngẫu nhiên mà phải yêu cầu user xác nhận
- trước khi bấm `Chạy tự động`, người dùng phải thấy toàn bộ mapping video-logo-audio-SRT-preset và có thể sửa tay các mapping chưa đúng
- job không được phép bắt đầu nếu task đã chọn yêu cầu subtitle nhưng vẫn còn video chưa map SRT hợp lệ
- job không được phép bắt đầu nếu task đã chọn yêu cầu audio nhưng vẫn còn video chưa map audio hợp lệ

### 7.2 Preset & Profiles

- Hệ thống phải hỗ trợ preset theo brand/channel.
- Mỗi preset phải có khả năng lưu:
  - logo mặc định
  - audio replacement policy
  - subtitle style preset
  - rule layout cơ bản
  - export preset cơ bản
- Hệ thống phải cho phép save, load, duplicate preset.
- Người dùng phải có thể áp dụng một preset cho toàn bộ job.

### 7.3 Batch Processing

- Hệ thống phải hỗ trợ xử lý 5-20 video trong một job ở V1.
- Hệ thống phải có queue xử lý nền.
- Hệ thống phải thể hiện trạng thái từng video:
  - Imported
  - InputNeedsReview
  - Processing
  - Review Needed
  - Ready to Export
  - Exporting
  - Done
  - Failed
- Hệ thống nên có khả năng resume và retry cơ bản cho job bị lỗi hoặc bị gián đoạn.

#### Mô hình persistence tối thiểu cho V1

- hệ thống phải persist job manifest gồm danh sách video, preset đang áp dụng, mapping file, và task đã chọn
- hệ thống phải persist trạng thái từng video trong queue
- hệ thống phải persist segment flags sinh ra từ bước auto process
- hệ thống phải persist toàn bộ quick-fix mà người dùng đã áp dụng trong review
- khi ứng dụng mở lại sau crash hoặc mất điện, job phải khôi phục được đến bước gần nhất đã hoàn tất
- V1 không bắt buộc resume một file export đang chạy dở ở giữa tiến trình encode; có thể cho phép export lại video đó từ đầu
- nếu người dùng đổi preset hoặc đổi file input sau khi đã review, hệ thống phải cảnh báo rằng các kết quả detect hoặc quick-fix cũ có thể không còn hợp lệ

### 7.4 Logo Detection / Replacement

- Hệ thống phải hỗ trợ detect logo cũ ở mức cơ bản cho các trường hợp phổ biến.
- Hệ thống phải thay logo cũ bằng cách overlay logo mới để che phủ vùng logo cũ.
- Hệ thống phải sinh confidence hoặc risk flag khi detect không chắc chắn.
- Người dùng phải có thể chỉnh tay vị trí và kích thước logo ở segment bị flag.
- V1 không bắt buộc tracking nâng cao cho logo chuyển động phức tạp.

### 7.5 Subtitle Removal / Replacement

- Hệ thống phải detect vùng subtitle hardcoded cũ theo segment ở mức cơ bản.
- V1 phải ưu tiên blur/mask hoặc box fill đơn giản để xử lý subtitle cũ.
- Inpaint chỉ áp dụng cho các case dễ nếu khả thi, không phải tiêu chuẩn mặc định.
- Hệ thống phải import SRT mới và render subtitle mới lên video.
- Khi task có audio replacement, hệ thống phải coi SRT mới là subtitle đi kèm với audio mới.
- Người dùng phải có thể chỉnh nhanh:
  - vị trí subtitle mới
  - scale cỡ chữ
  - style preset
- V1 không cung cấp full subtitle style editor.

### 7.6 Audio Replacement

- Hệ thống phải hỗ trợ thay toàn bộ audio của video bằng file audio mới đã map.
- Audio replacement trong V1 là toàn-video, không theo segment.
- V1 không bắt buộc audio offset, resync, hoặc audio timeline editing.
- Khi export, audio gốc phải bị thay hoàn toàn bằng audio mới đối với các video dùng task có audio replacement.
- Encode summary phải thể hiện audio track đầu ra cơ bản, bao gồm ít nhất codec và trạng thái audio source đã dùng.

### 7.7 Smart Review Layer

- Hệ thống phải dùng segment list làm UI chính cho review.
- Timeline nhẹ chỉ đóng vai trò hỗ trợ, không phải trung tâm thao tác.
- Mỗi segment phải có trạng thái hoặc mức rủi ro dễ nhìn.
- Hệ thống phải hỗ trợ before/after preview cho segment bị flag.
- Hệ thống phải cho phép apply fix cho một segment hoặc nhiều segment được chọn.

#### Quy tắc gắn cờ và mức rủi ro

V1 dùng ba mức rủi ro chính:

- `Low Risk`: hệ thống khá chắc chắn, segment có thể auto-pass và chỉ hiện trong spot-check tùy chọn
- `Medium Risk`: hệ thống có nghi ngờ nhưng vẫn có output khả dụng, segment nên được user xem lại trước khi export
- `High Risk`: hệ thống không đủ tin cậy hoặc phát hiện xung đột rõ ràng, segment bắt buộc phải được review hoặc xác nhận trước khi export

Các input cho risk scoring ở V1 nên bao gồm:

- confidence detect logo
- confidence detect subtitle region
- số lần layout thay đổi trong video
- xung đột mapping hoặc thiếu input
- quick-fix trước đó bị invalid do thay đổi preset hay input

#### Điều kiện pass ở cấp video

- video được coi là `Ready to Export` khi không còn segment `High Risk`
- video có thể vẫn giữ `Medium Risk` nếu user đã review và xác nhận chấp nhận output
- video giữ trạng thái `Review Needed` nếu còn bất kỳ segment bắt buộc nào chưa được xử lý

### 7.8 Export & Delivery

- Hệ thống phải export ra MP4 ở V1.
- Hệ thống phải ưu tiên chất lượng ổn định hơn tốc độ tối đa.
- Hệ thống phải cho phép export hàng loạt sau khi review xong.
- Hệ thống phải sinh encode summary cơ bản cho từng video.

### 7.9 Quality & Reporting

- Hệ thống phải sinh risk report cơ bản cho từng video.
- Báo cáo V1 phải bao gồm:
  - trạng thái job
  - encode summary
  - audio source summary
  - số segment bị flag
  - before/after spot check cho vùng đã chỉnh hoặc nghi ngờ
- V1 không bắt buộc VMAF, PSNR, SSIM.

## 8. UX Requirements

### 8.1 UX Direction

V1 áp dụng mô hình hybrid:

- Wizard/task flow là trải nghiệm mặc định
- Review panel/editor nhẹ xuất hiện khi cần sửa exception

### 8.2 Onboarding

- Người mới phải bắt đầu được trong vòng 5 phút đầu.
- Không được yêu cầu học tool trước khi chạy job đầu tiên.
- Màn hình đầu phải ưu tiên task-based wording thay vì thuật ngữ editor phức tạp.

### 8.3 Progressive Disclosure

- Chỉ hiển thị các hành động cơ bản ở lớp đầu.
- Các panel chỉnh sửa chỉ mở khi video có segment cần review hoặc khi người dùng chủ động sửa.
- Không hiển thị full technical controls upfront.

### 8.4 Exception-based Review

- Hệ thống phải đẩy người dùng tới đúng segment nghi ngờ thay vì yêu cầu xem toàn bộ video.
- Segment list là nơi điều hướng chính trong màn hình review.
- Timeline nhẹ chỉ để định vị và hỗ trợ context.

#### Ranh giới giữa wizard mode và review mode

- wizard mode bao phủ các bước nhập file, chọn preset, kiểm tra mapping, và chạy auto process
- audio mapping được xử lý ở wizard mode, không xử lý trong review mode
- nếu job không có segment `High Risk`, người dùng có thể đi thẳng sang export hoặc mở spot-check tùy chọn
- nếu job có bất kỳ segment `High Risk`, hệ thống phải điều hướng người dùng sang review mode trước khi export
- review mode chỉ mở full quick-fix panel khi người dùng chọn một segment cụ thể
- người dùng mới không nên bị hiển thị đồng thời toàn bộ controls của logo, subtitle cũ, và subtitle mới khi chưa chọn segment

### 8.5 Quick Fix UX

- User phải sửa được đa số lỗi thường gặp trong vài thao tác ngắn.
- Các quick fix của V1 chỉ gồm:
  - move/resize logo
  - move/resize blur or mask box
  - đổi vị trí subtitle mới
  - đổi scale subtitle
  - đổi subtitle style preset

## 9. Core Screens

### 9.1 Start Screen

- Hỏi “Bạn muốn làm gì?”
- Cho phép chọn task chính.
- Có vùng drag-and-drop cho video, logo, audio, SRT.
- Có đường đi nhanh cho preset sẵn có.

### 9.2 Preset Selection

- Hiển thị danh sách preset theo brand/channel.
- Cho xem nhanh logo, audio policy, subtitle style, và export preset của preset đang chọn.

### 9.3 Job Setup / Pre-run Review

- Liệt kê toàn bộ video trong job.
- Hiển thị mapping video-logo-audio-SRT-preset.
- Cho phép chạy auto process.

### 9.4 Processing Queue

- Hiển thị tiến độ từng video.
- Thể hiện rõ video nào cần review.
- Thể hiện rõ video nào còn thiếu audio mapping nếu task yêu cầu audio.

### 9.5 Exception Review

- Segment list ở panel chính bên trái.
- Preview before/after ở trung tâm.
- Quick fix panel ở bên phải.

### 9.6 Export & Report

- Cho phép export toàn bộ các video đã sẵn sàng.
- Hiển thị báo cáo sau xử lý cho từng video.
- Hiển thị audio source đầu ra đã dùng cho từng video.

## 10. Non-functional Requirements

- Ứng dụng phải phản hồi tốt trong UI, không block toàn bộ app khi xử lý nền.
- Hệ thống phải có logging đủ để debug pipeline.
- Hệ thống phải ổn định cho batch 5-20 video/job.
- Hệ thống phải có cơ chế resume hoặc retry cơ bản cho job lỗi.
- Chất lượng output phải ổn định và không được hy sinh rõ rệt chỉ để tăng tốc export.

## 11. Success Metrics

V1 được xem là thành công khi:

- Người dùng thuộc persona Chị Mai có thể hoàn thành một job re-branding theo batch với ít chỉnh tay hơn đáng kể so với workflow thủ công.
- Người dùng kiểu Em Hà có thể chạy job đầu tiên mà không cần học một editor phức tạp.
- Người dùng chỉ cần review một số segment nghi ngờ thay vì xem lại toàn bộ video.
- Output có chất lượng đủ ổn định để đưa vào vận hành thực tế.

### 11.1 Chỉ số kiểm chứng đề xuất cho V1

- median số segment cần review trên mỗi video trong job chuẩn không vượt quá 20% tổng segment
- median thời gian quick-fix cho một segment bị flag không vượt quá 30 giây ở các case phổ biến
- ít nhất 70% video trong job chuẩn không cần sửa quá 3 segment
- tỷ lệ export thành công của các video `Ready to Export` đạt ít nhất 95%
- tỷ lệ job bị chặn do lỗi mapping input sau bước pre-run review phải giảm dần về dưới 10% sau khi người dùng quen preset và naming convention

## 12. MVP Scope

### 12.1 In Scope

- Import video + logo + audio + SRT
- Batch 5-20 video/job
- Preset theo brand/channel
- Logo detect/replace cơ bản
- Audio replacement toàn-video
- Subtitle region detect cơ bản
- Blur/mask subtitle cũ
- Render subtitle mới từ SRT
- Exception review theo segment list
- Export MP4
- Encode summary + risk report cơ bản

### 12.2 Out of Scope

- Audio offset/resync nâng cao
- Fingerprint variation
- Full timeline editor
- Full subtitle styling editor
- Deep inpaint nâng cao
- Metric QC như VMAF/PSNR/SSIM
- Upload/distribution integration

## 13. Phase 2 Candidates

- Tracking logo động nâng cao
- Subtitle placement thông minh tránh object/face
- Inpaint tốt hơn cho vùng subtitle cũ
- QC metric nâng cao
- Audio segment editing và subtitle resync nâng cao
- Fingerprint variation với guardrail rõ ràng

## 14. Rủi Ro & Giả Định

### 14.1 Rủi ro

- Detect subtitle hardcoded sẽ biến động mạnh theo nhiều nguồn video khác nhau.
- Logo detect cơ bản có thể không đủ cho logo bán trong suốt, đổi kích thước, hoặc di chuyển phức tạp.
- Nếu số segment bị flag quá nhiều thì lợi ích “review exception” sẽ giảm mạnh.
- Blur/mask có thể đủ cho nhiều case, nhưng sẽ không đẹp ở mọi nền cảnh.

### 14.2 Giả định

- Phần lớn case V1 nằm trong nhóm layout tương đối lặp lại và đủ phù hợp với preset theo brand.
- Người dùng chấp nhận quick-fix review thay vì kỳ vọng full automation.
- MP4 là định dạng output đủ dùng cho nhu cầu V1.
- Audio replacement ở V1 là thay toàn bộ audio gốc bằng audio mới, không yêu cầu sync nâng cao.

## 15. Kết Luận

Desktop Video Rebranding App V1 nên được xây dựng như một semi-auto batch workflow tool, không phải một full editor. Giá trị cốt lõi nằm ở khả năng nhập lô video, áp dụng preset thương hiệu, thay logo, thay audio, xử lý subtitle, tự động xử lý phần phổ biến, và kéo người dùng vào đúng các segment cần sửa trước khi export chất lượng ổn định.
