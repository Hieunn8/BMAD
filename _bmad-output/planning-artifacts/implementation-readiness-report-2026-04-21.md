---
stepsCompleted: ["step-01-document-discovery", "step-02-prd-analysis", "step-03-epic-coverage-validation", "step-04-ux-alignment", "step-05-epic-quality-review", "step-06-final-assessment"]
documentsIncluded:
  prd: "_bmad-output/planning-artifacts/prd.md"
  architecture: "_bmad-output/planning-artifacts/architecture.md"
  epics: "_bmad-output/planning-artifacts/epics.md"
  ux: "_bmad-output/planning-artifacts/ux-design-specification.md"
---

# Implementation Readiness Assessment Report

**Date:** 2026-04-21
**Project:** BMAD

---

## PRD Analysis

### Functional Requirements

**FR1:** Hệ thống phải cho phép import một hoặc nhiều video trong cùng một job.
**FR2:** Hệ thống phải cho phép import logo, audio mới, và SRT phục vụ re-branding.
**FR3:** Hệ thống phải tự nhận diện loại file cơ bản từ extension và gán đúng vai trò trong pipeline.
**FR4:** Hệ thống phải cho phép người dùng lưu cấu hình job trước khi chạy.
**FR5:** V1 chỉ hỗ trợ mỗi video map với đúng một audio output và đúng một SRT output.
**FR6:** Hệ thống phải cố gắng map SRT vào video bằng exact base filename match trước.
**FR7:** Hệ thống phải cố gắng map audio vào video bằng exact base filename match trước.
**FR8:** Nếu có đúng một SRT khớp với một video, hệ thống tự gán mapping đó.
**FR9:** Nếu có đúng một audio khớp với một video, hệ thống tự gán mapping đó.
**FR10:** Nếu một video không có SRT khớp, video đó được gắn trạng thái `Input Needs Review` trước khi chạy.
**FR11:** Nếu một video không có audio khớp cho task yêu cầu audio, video đó được gắn trạng thái `Input Needs Review` trước khi chạy.
**FR12:** Nếu nhiều SRT cùng có khả năng khớp với một video, hệ thống không được tự chọn ngẫu nhiên mà phải yêu cầu user xác nhận.
**FR13:** Nếu nhiều file audio cùng có khả năng khớp với một video, hệ thống không được tự chọn ngẫu nhiên mà phải yêu cầu user xác nhận.
**FR14:** Trước khi bấm `Chạy tự động`, người dùng phải thấy toàn bộ mapping video-logo-audio-SRT-preset và có thể sửa tay các mapping chưa đúng.
**FR15:** Job không được phép bắt đầu nếu task đã chọn yêu cầu subtitle nhưng vẫn còn video chưa map SRT hợp lệ.
**FR16:** Job không được phép bắt đầu nếu task đã chọn yêu cầu audio nhưng vẫn còn video chưa map audio hợp lệ.
**FR17:** Hệ thống phải hỗ trợ preset theo brand/channel.
**FR18:** Mỗi preset phải có khả năng lưu: logo mặc định, audio replacement policy, subtitle style preset, rule layout cơ bản, export preset cơ bản.
**FR19:** Hệ thống phải cho phép save, load, duplicate preset.
**FR20:** Người dùng phải có thể áp dụng một preset cho toàn bộ job.
**FR21:** Hệ thống phải hỗ trợ xử lý 5-20 video trong một job ở V1.
**FR22:** Hệ thống phải có queue xử lý nền.
**FR23:** Hệ thống phải thể hiện trạng thái từng video: Imported, InputNeedsReview, Processing, Review Needed, Ready to Export, Exporting, Done, Failed.
**FR24:** Hệ thống nên có khả năng resume và retry cơ bản cho job bị lỗi hoặc bị gián đoạn.
**FR25:** Hệ thống phải persist job manifest gồm danh sách video, preset đang áp dụng, mapping file, và task đã chọn.
**FR26:** Hệ thống phải persist trạng thái từng video trong queue.
**FR27:** Hệ thống phải persist segment flags sinh ra từ bước auto process.
**FR28:** Hệ thống phải persist toàn bộ quick-fix mà người dùng đã áp dụng trong review.
**FR29:** Khi ứng dụng mở lại sau crash hoặc mất điện, job phải khôi phục được đến bước gần nhất đã hoàn tất.
**FR30:** Nếu người dùng đổi preset hoặc đổi file input sau khi đã review, hệ thống phải cảnh báo rằng các kết quả detect hoặc quick-fix cũ có thể không còn hợp lệ.
**FR31:** Hệ thống phải hỗ trợ detect logo cũ ở mức cơ bản cho các trường hợp phổ biến.
**FR32:** Hệ thống phải thay logo cũ bằng cách overlay logo mới để che phủ vùng logo cũ.
**FR33:** Hệ thống phải sinh confidence hoặc risk flag khi detect không chắc chắn.
**FR34:** Người dùng phải có thể chỉnh tay vị trí và kích thước logo ở segment bị flag.
**FR35:** Hệ thống phải detect vùng subtitle hardcoded cũ theo segment ở mức cơ bản.
**FR36:** V1 phải ưu tiên blur/mask hoặc box fill đơn giản để xử lý subtitle cũ.
**FR37:** Hệ thống phải import SRT mới và render subtitle mới lên video.
**FR38:** Khi task có audio replacement, hệ thống phải coi SRT mới là subtitle đi kèm với audio mới.
**FR39:** Người dùng phải có thể chỉnh nhanh: vị trí subtitle mới, scale cỡ chữ, style preset.
**FR40:** Hệ thống phải hỗ trợ thay toàn bộ audio của video bằng file audio mới đã map.
**FR41:** Khi export, audio gốc phải bị thay hoàn toàn bằng audio mới đối với các video dùng task có audio replacement.
**FR42:** Encode summary phải thể hiện audio track đầu ra cơ bản, bao gồm ít nhất codec và trạng thái audio source đã dùng.
**FR43:** Hệ thống phải dùng segment list làm UI chính cho review.
**FR44:** Timeline nhẹ chỉ đóng vai trò hỗ trợ, không phải trung tâm thao tác.
**FR45:** Mỗi segment phải có trạng thái hoặc mức rủi ro dễ nhìn.
**FR46:** Hệ thống phải hỗ trợ before/after preview cho segment bị flag.
**FR47:** Hệ thống phải cho phép apply fix cho một segment hoặc nhiều segment được chọn.
**FR48:** V1 dùng ba mức rủi ro: Low Risk (auto-pass), Medium Risk (nên review), High Risk (bắt buộc review trước export).
**FR49:** Hệ thống phải export ra MP4 ở V1.
**FR50:** Hệ thống phải ưu tiên chất lượng ổn định hơn tốc độ tối đa.
**FR51:** Hệ thống phải cho phép export hàng loạt sau khi review xong.
**FR52:** Hệ thống phải sinh encode summary cơ bản cho từng video.
**FR53:** Hệ thống phải sinh risk report cơ bản cho từng video.
**FR54:** Báo cáo V1 phải bao gồm: trạng thái job, encode summary, audio source summary, số segment bị flag, before/after spot check cho vùng đã chỉnh hoặc nghi ngờ.

**Tổng FRs: 54**

---

### Non-Functional Requirements

**NFR1:** Ứng dụng phải phản hồi tốt trong UI, không block toàn bộ app khi xử lý nền. *(Responsiveness)*
**NFR2:** Hệ thống phải có logging đủ để debug pipeline. *(Observability)*
**NFR3:** Hệ thống phải ổn định cho batch 5-20 video/job. *(Stability/Scalability)*
**NFR4:** Hệ thống phải có cơ chế resume hoặc retry cơ bản cho job lỗi. *(Reliability)*
**NFR5:** Chất lượng output phải ổn định và không được hy sinh rõ rệt chỉ để tăng tốc export. *(Output Quality)*

**Tổng NFRs: 5**

---

### Additional Requirements

**UX Requirements (Section 8):**
- UX1: Người mới phải bắt đầu được trong vòng 5 phút đầu (onboarding).
- UX2: Không được yêu cầu học tool trước khi chạy job đầu tiên.
- UX3: Màn hình đầu phải ưu tiên task-based wording thay vì thuật ngữ editor phức tạp.
- UX4: Chỉ hiển thị các hành động cơ bản ở lớp đầu (progressive disclosure).
- UX5: Các panel chỉnh sửa chỉ mở khi video có segment cần review hoặc khi người dùng chủ động sửa.
- UX6: User phải sửa được đa số lỗi thường gặp trong vài thao tác ngắn (quick fix UX).

**Success Metrics / Operational Thresholds (Section 3.4 & 11.1):**
- ≥70% video trong job chuẩn không cần quick-fix quá 3 segment.
- ≤20% tổng segment bị flag cần review.
- Thời gian quick-fix trung bình ≤30 giây/segment.
- Tỷ lệ export thất bại <5%.
- Tỷ lệ export thành công cho video `Ready to Export` ≥95%.

**Constraints (Section 14.2):**
- Phần lớn case V1 nằm trong nhóm layout tương đối lặp lại.
- Người dùng chấp nhận quick-fix review thay vì kỳ vọng full automation.
- MP4 là định dạng output đủ dùng cho nhu cầu V1.
- Audio replacement là thay toàn bộ audio gốc, không yêu cầu sync nâng cao.

---

---

## Epic Coverage Validation

### Coverage Matrix

| PRD FR | Tóm tắt yêu cầu | Epics FR / Story | Trạng thái |
|--------|----------------|------------------|-----------|
| FR1 | Import nhiều video trong cùng job | Epics FR2 → Epic 1 Story 1.1 | ✓ Covered |
| FR2 | Import logo, audio, SRT | Epics FR2 → Epic 1 Story 1.1 | ✓ Covered |
| FR3 | Nhận diện file từ extension, lưu job config | Epics FR3 → Epic 1 Story 1.1, 1.4 | ✓ Covered |
| FR4 | Lưu cấu hình job trước khi chạy | Epics FR3 → Epic 1 Story 1.4 | ✓ Covered |
| FR5 | 1 video = 1 audio + 1 SRT tối đa | Epics FR4 → Epic 1 | ✓ Covered |
| FR6 | Map SRT bằng exact base filename | Epics FR5 → Epic 1 Story 1.3 | ✓ Covered |
| FR7 | Map audio bằng exact base filename | Epics FR5 → Epic 1 Story 1.3 | ✓ Covered |
| FR8 | 1 SRT khớp → tự gán | Epics FR5 → Epic 1 Story 1.3 | ✓ Covered |
| FR9 | 1 audio khớp → tự gán | Epics FR5 → Epic 1 Story 1.3 | ✓ Covered |
| FR10 | Không có SRT → InputNeedsReview | Epics FR5 → Epic 1 Story 1.3 | ✓ Covered |
| FR11 | Không có audio → InputNeedsReview | Epics FR5 → Epic 1 Story 1.3 | ✓ Covered |
| FR12 | Nhiều SRT khớp → yêu cầu user xác nhận | Epics FR5 → Epic 1 Story 1.3 | ✓ Covered |
| FR13 | Nhiều audio khớp → yêu cầu user xác nhận | Epics FR5 → Epic 1 Story 1.3 | ✓ Covered |
| FR14 | Hiển thị toàn bộ mapping trước khi chạy | Epics FR5, FR19 → Epic 1 Story 1.3, 1.4 | ✓ Covered |
| FR15 | Block job nếu thiếu SRT bắt buộc | Epics FR5 → Epic 1 Story 1.4 | ✓ Covered |
| FR16 | Block job nếu thiếu audio bắt buộc | Epics FR5 → Epic 1 Story 1.4 | ✓ Covered |
| FR17 | Preset theo brand/channel | Epics FR6 → Epic 1 Story 1.2 | ✓ Covered |
| FR18 | Nội dung preset (logo, audio policy, subtitle style, layout, export) | Epics FR7 → Epic 1 Story 1.2 | ✓ Covered |
| FR19 | Save, load, duplicate preset | Epics FR6 → Epic 1 | ⚠️ Partial — Story 1.2 chỉ cover *select & apply*, không có story riêng cho preset management (create/edit/duplicate) |
| FR20 | Áp dụng preset cho toàn bộ job | Epics FR6 → Epic 1 Story 1.2 | ✓ Covered |
| FR21 | Batch 5-20 video/job | Epics FR8 → Epic 2 Story 2.1 | ✓ Covered |
| FR22 | Queue xử lý nền | Epics FR8 → Epic 2 Story 2.1 | ✓ Covered |
| FR23 | Hiển thị trạng thái từng video (8 states) | Epics FR8 → Epic 2 Story 2.5 | ✓ Covered |
| FR24 | Resume và retry cơ bản | Epics FR8 → Epic 4 Story 4.1 | ✓ Covered |
| FR25 | Persist job manifest | Epics FR9 → Epic 4 Story 4.1 | ✓ Covered |
| FR26 | Persist per-video state | Epics FR9 → Epic 4 Story 4.1 | ✓ Covered |
| FR27 | Persist segment flags | Epics FR9 → Epic 4 Story 4.1 | ✓ Covered |
| FR28 | Persist quick-fix state | Epics FR9 → Epic 4 Story 4.4 | ✓ Covered |
| FR29 | Khôi phục job sau crash/mất điện | Epics FR9 → Epic 4 Story 4.1 | ✓ Covered |
| FR30 | Cảnh báo khi đổi preset hoặc file input sau review | Epics FR9 → Epic 1 Story 1.2 | ⚠️ Partial — Story 1.2 chỉ cảnh báo cho *đổi preset*, không cover *đổi file input* |
| FR31 | Detect logo cũ cơ bản | Epics FR10 → Epic 2 Story 2.3 | ✓ Covered |
| FR32 | Overlay logo mới đè lên logo cũ | Epics FR10 → Epic 2 Story 2.3 | ✓ Covered |
| FR33 | Sinh confidence/risk flag khi detect không chắc | Epics FR10 → Epic 2 Story 2.3 | ✓ Covered |
| FR34 | User chỉnh tay vị trí/kích thước logo ở segment flag | Epics FR10 → Epic 3 Story 3.4 | ✓ Covered |
| FR35 | Detect subtitle hardcoded cũ theo segment | Epics FR11 → Epic 2 Story 2.4 | ✓ Covered |
| FR36 | Blur/mask/box fill subtitle cũ | Epics FR11 → Epic 2 Story 2.4 | ✓ Covered |
| FR37 | Import SRT mới và render subtitle mới | Epics FR12 → Epic 2 Story 2.4 | ✓ Covered |
| FR38 | SRT đi kèm audio khi có audio replacement | Epics FR12 → Epic 2 Story 2.4 | ✓ Covered |
| FR39 | Quick fix: vị trí/scale/style subtitle mới | Epics FR13 → Epic 3 Story 3.5 | ✓ Covered |
| FR40 | Thay toàn bộ audio bằng audio mới đã map | Epics FR14 → Epic 2 Story 2.2 | ✓ Covered |
| FR41 | Audio gốc phải bị thay hoàn toàn khi export | Epics FR14 → Epic 2 Story 2.2 | ✓ Covered |
| FR42 | Encode summary thể hiện codec và audio source | Epics FR18 → Epic 4 Story 4.4 | ✓ Covered |
| FR43 | Segment list là UI chính cho review | Epics FR15 → Epic 3 Story 3.1 | ✓ Covered |
| FR44 | Timeline nhẹ chỉ là hỗ trợ | Epics FR15 → Epic 3 Story 3.1 | ✓ Covered |
| FR45 | Mỗi segment có risk level dễ nhìn | Epics FR16 → Epic 3 Story 3.2 | ✓ Covered |
| FR46 | Before/after preview cho segment flag | Epics FR16 → Epic 3 Story 3.3 | ✓ Covered |
| FR47 | Apply fix cho một hoặc nhiều segment | Epics FR16 → Epic 3 Story 3.4, 3.5 | ✓ Covered |
| FR48 | Ba mức rủi ro: Low/Medium/High Risk với điều kiện pass | Epics FR16 → Epic 3 Story 3.6 | ✓ Covered |
| FR49 | Export ra MP4 | Epics FR17 → Epic 4 Story 4.3 | ✓ Covered |
| FR50 | Ưu tiên chất lượng ổn định hơn tốc độ | Epics NFR3 | ✓ Covered (as NFR) |
| FR51 | Export hàng loạt sau khi review xong | Epics FR17 → Epic 4 Story 4.3 | ✓ Covered |
| FR52 | Sinh encode summary cơ bản cho từng video | Epics FR18 → Epic 4 Story 4.4 | ✓ Covered |
| FR53 | Sinh risk report cơ bản cho từng video | Epics FR18 → Epic 4 Story 4.4 | ✓ Covered |
| FR54 | Report bao gồm: job status, encode summary, audio source, segment count, spot check | Epics FR18 → Epic 4 Story 4.4 | ✓ Covered |

**Epics FR1 (Task selection workflow)** — Epics document bổ sung thêm FR này (từ PRD Section 6.1, không số hóa trong Section 7). Đây là yêu cầu hợp lệ được thêm vào bởi epics → ✅

---

### Missing Requirements

#### ⚠️ Partially Covered FRs

**PRD FR19 — Preset Management (Save/Load/Duplicate)**
- Yêu cầu: "Hệ thống phải cho phép save, load, duplicate preset."
- Hiện trạng: Epics FR6 claim coverage, nhưng không có story nào cho preset management flow (tạo preset mới, chỉnh sửa preset, duplicate). Story 1.2 chỉ cover *selecting & applying* preset đã có.
- Impact: Nếu không có story riêng, dev có thể implement select/apply nhưng bỏ sót create/duplicate.
- Đề xuất: Thêm Story 1.5 hoặc expand Story 1.2 để cover preset CRUD operations.

**PRD FR30 — Cảnh báo khi đổi file input sau review**
- Yêu cầu: "nếu người dùng đổi preset hoặc đổi file input sau khi đã review, hệ thống phải cảnh báo..."
- Hiện trạng: Story 1.2 chỉ cover cảnh báo khi *đổi preset*. Cảnh báo khi *đổi file input* (ví dụ: thay logo, SRT, audio file sau khi đã review) không được address trong bất kỳ story nào.
- Impact: User có thể thay logo/SRT sau khi review xong mà không biết quick fixes cũ đã mất hiệu lực.
- Đề xuất: Cập nhật Story 1.3 hoặc Story 3.1 để cover cảnh báo khi thay đổi file input sau khi job đã có review data.

#### ℹ️ Missing Story (Implied Gap)

**Processing Queue UI Screen (PRD Section 9.4)**
- PRD yêu cầu: "Thể hiện rõ video nào còn thiếu audio mapping nếu task yêu cầu audio" trong processing queue screen.
- Hiện trạng: Story 2.5 cover persisting và cập nhật status. Không có story riêng về Processing Queue screen UI. UX-DR7 có requirements nhưng không được implement qua story.
- Impact: Low — UX-DR7 đã có requirements. Developer có thể implement từ UX-DR7 mà không cần story riêng.
- Đề xuất: Xem xét thêm acceptance criteria vào Story 2.5 về việc hiển thị trạng thái thiếu audio mapping.

---

### Coverage Statistics

- **Tổng PRD FRs:** 54
- **FRs được cover đầy đủ:** 52
- **FRs được cover một phần:** 2 (FR19, FR30)
- **FRs không được cover:** 0
- **Coverage percentage:** ~96% (52/54 fully covered, 2 partial)
- **Epics bổ sung thêm:** 1 FR hợp lệ (Task selection workflow từ Section 6.1)

---

### PRD Completeness Assessment

PRD được viết rõ ràng, có cấu trúc tốt, và bao phủ đầy đủ các yêu cầu chức năng cốt lõi. Các điểm mạnh:
- Scope V1 được giới hạn rõ ràng (non-goals được liệt kê tường minh).
- Mapping rules cho file input được đặc tả chi tiết.
- Review mode có state machine rõ ràng (Low/Medium/High Risk).
- Persistence model được mô tả cụ thể.

Một số điểm cần theo dõi trong bước tiếp theo:
- PRD không đặc tả rõ platform target (Windows? macOS? Cross-platform?).
- Không có yêu cầu cụ thể về format file input được hỗ trợ (codec video, format audio).
- Quick fix UX (Section 8.5) và FR39 cần được kiểm tra xem có được cover đủ trong epics.

---

## UX Alignment Assessment

### UX Document Status

**Tìm thấy:** `_bmad-output/planning-artifacts/ux-design-specification.md`
- Input documents: PRD + Brainstorming session
- Ghi chú: UX document chưa include architecture.md làm input document, nhưng nội dung vẫn align tốt.

---

### UX ↔ PRD Alignment Issues

#### ✅ Aligned

- Vision "wizard-first, review-exception-centered" → Nhất quán với PRD Section 3.2 và 8.1.
- 4 task cards (Thay logo, Thay audio, Thay subtitle, Thay logo+audio+subtitle) → Nhất quán với PRD Section 6.1.
- Audio mapping ở Setup Mode, không xuất hiện trong review → Nhất quán với PRD Section 6.4 và 7.6.
- Three-column review layout (Segment List / Preview / Quick Fix) → Nhất quán với PRD Section 9.5.
- Risk color system (Low/Medium/High/Done) + icon + label → Nhất quán với PRD Section 7.7.
- Before/after toggle + split preview → Nhất quán với PRD FR46.
- Export screen shows blocked videos clearly → Nhất quán với PRD Section 9.6.

#### ⚠️ UX Additions Not Explicitly in PRD

**UX-ADD-1: Logo overlay toggle (bật/tắt logo overlay)**
- UX Section 6.5.3 Quick Fix Panel adds `bật/tắt logo overlay` control.
- PRD Section 8.5 quick fixes list chỉ có: move/resize logo. Không mention toggle.
- Impact: Minor — đây là UX enhancement hợp lý. Developer cần biết control này được expect.
- Đề xuất: Confirm trong stories/acceptance criteria cho Story 3.4.

**UX-ADD-2: "Mở job gần đây" (Open recent job)**
- UX Section 6.1 Start Screen có nút `Mở job gần đây`.
- PRD không có requirement cho job history/recent jobs.
- Impact: Trung bình — nếu implement, cần persistence cho recent job list. Không có AC nào cover requirement này.
- Đề xuất: Quyết định rõ: là in-scope hay out-scope cho V1? Nếu in-scope, cần thêm AC vào Story 4.1 hoặc story riêng.

**UX-ADD-3: Blur/Mask/Fill mode selection**
- UX Section 6.5.3 thêm control `chọn mode: blur/mask/fill` trong Quick Fix Panel subtitle old region.
- PRD Section 8.5 chỉ nói "move/resize blur or mask box" không đề cập chọn mode.
- Impact: Minor — PRD Section 7.5 có nhắc "blur/mask hoặc box fill" như options nhưng không nói user có thể switch. UX làm rõ hơn điều này.
- Đề xuất: Confirm behavior trong Story 3.5.

#### ⚠️ UX ↔ PRD Discrepancy

**UX-DISC-1: Processing Screen thiếu 2 states**
- PRD Section 7.3 định nghĩa 8 trạng thái video: Imported, InputNeedsReview, Processing, Review Needed, Ready to Export, **Exporting**, **Done**, Failed.
- UX Section 6.4 Processing Screen chỉ liệt kê 6 trạng thái, thiếu `Exporting` và `Done`.
- Impact: Trung bình — UI developer có thể bỏ sót 2 states này nếu implement từ UX spec.
- Đề xuất: Cập nhật UX spec để thêm `Exporting` và `Done` states, hoặc confirm trong Story 4.3/4.4.

---

### UX ↔ Architecture Alignment Issues

#### ✅ Aligned

- Tauri + React + TypeScript phù hợp với UX direction "modern, clean, không enterprise nặng".
- Segment-based review là ADR-01, nhất quán với UX Section 2.4 "Segment, not timeline".
- Hybrid preview (UI overlay + Rendered validation) support UX before/after toggle và overlay handles.
- `preset-service` trong architecture cover `load/save/duplicate` → align UX preset screen.
- Local-first, không ghi đè source → align UX safety assumptions.
- Error handling "fail one video not whole job" → align UX Processing Screen behavior.

#### ⚠️ Architecture Gaps vs UX Requirements

**ARCH-GAP-1: Preset management commands thiếu trong API layer**
- UX Preset Screen có actions: `Chọn preset này`, `Sửa preset`, `Tạo preset mới`.
- Architecture Section 13.1 (UI → Application commands) chỉ có `selectPreset`.
- Thiếu: `createPreset`, `editPreset`, `duplicatePreset` commands.
- `Preset Service` (Section 4, Component 3) có đề cập `load/save/duplicate` nhưng không được expose qua API layer.
- Impact: Cao — nếu API layer không có commands này, developer frontend sẽ không biết cách gọi.
- Đề xuất: Bổ sung `createPreset`, `editPreset`, `duplicatePreset` vào Architecture Section 13.1.

**ARCH-GAP-2: Không có event cho "file input changed after review"**
- PRD FR30 yêu cầu cảnh báo khi user đổi file input sau khi đã review.
- Architecture Section 13.2 (Application → UI events) không có event `inputMappingChangedAfterReview` hay tương đương.
- Impact: Trung bình — thiếu event này có thể khiến warning không được implement.
- Đề xuất: Thêm event `inputFileReplacedAfterReview` vào Architecture Section 13.2.

**ARCH-GAP-3: "Open recent job" không có support trong architecture**
- UX "Mở job gần đây" cần persistence cho recent job list.
- Architecture Section 9 (Persistence Design) không mention recent job index hay job listing.
- Impact: Tùy thuộc vào quyết định in/out-scope ở UX-ADD-2.
- Đề xuất: Resolve scope decision trước, sau đó update persistence design nếu cần.

---

### Warnings

⚠️ **UX spec input documents không include architecture.md** — UX được tạo trước hoặc độc lập với architecture. Handoff notes (Section 14) của UX tuy nhiên đúng hướng và align tốt về mặt technical.

ℹ️ **Platform target chưa được làm rõ** — cả PRD, UX, và Architecture đều không đặc tả rõ Windows/macOS hay cross-platform. Tauri hỗ trợ cross-platform nhưng đây nên được confirm rõ trong epics.

---

---

## Epic Quality Review

### Epic Structure Validation

#### Epic 1: Prepare Job and Map Assets
- **User-centric title:** ✓ — Mô tả việc người dùng có thể làm.
- **User outcome goal:** ✓ — "Người dùng có thể tạo một job re-branding hợp lệ..."
- **Value alone:** ✓ — Người dùng có thể setup job hoàn chỉnh trước khi chạy.
- **Independence:** ✓ — Standalone, không phụ thuộc epic khác.
- **FRs claimed:** FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR19

#### Epic 2: Run Automated Rebranding
- **User-centric title:** ✓
- **User outcome goal:** ✓ — "Người dùng có thể chạy pipeline re-branding tự động..."
- **Value alone:** ✓ — Người dùng có thể xử lý batch sau khi Epic 1 hoàn thành.
- **Independence:** ✓ — Chỉ phụ thuộc Epic 1 output.
- **FRs claimed:** FR8, FR10, FR11, FR12, FR14

#### Epic 3: Review Exceptions and Apply Targeted Fixes
- **User-centric title:** ✓
- **User outcome goal:** ✓ — "Người dùng có thể tập trung vào đúng segment bị flag..."
- **Value alone:** ✓ — Phụ thuộc Epic 2 output là hợp lý (sequential).
- **Independence:** ✓ — Không yêu cầu Epic 4.
- **FRs claimed:** FR13, FR15, FR16, FR20

#### Epic 4: Export Deliverables and Recover Jobs
- **User-centric title:** ✓
- **User outcome goal:** ✓ — "Người dùng có thể resume job, export, và xem báo cáo..."
- **Value alone:** ✓ — Phụ thuộc Epics 1-3 là hợp lý.
- **Independence:** ✓ — Không có circular dependency.
- **FRs claimed:** FR9, FR17, FR18
- 🔴 **CRITICAL:** Toàn bộ Epic 4 stories trong tài liệu có **encoding corruption** — văn bản tiếng Việt bị garbled (ký tự lạ). AC text gần như không đọc được cho người implement.

---

### Story Quality Assessment

#### 🔴 Critical Violations

**CRITICAL-1: Không có Project Setup / Initialization Story**
- Đây là greenfield project (Tauri + React + TypeScript + FFmpeg).
- Architecture Section 3 và 19 chỉ định rõ tech stack và build order.
- Không có story nào cho: "Set up initial project from Tauri template", "Configure dev environment", "Set up project structure", "Integrate FFmpeg binary".
- Impact: Developer bắt đầu không biết phải setup gì. Không có definition of "project running baseline".
- Đề xuất: Thêm Story 0.1 hoặc Story 1.0 "Set up initial project structure and development environment" vào Epic 1.

**CRITICAL-2: Epic 4 Stories có Encoding Corruption**
- Stories 4.1 đến 4.4 trong `epics.md` có ký tự corrupt (hiển thị dạng `?` thay cho tiếng Việt).
- ACs quan trọng như resume checkpoint, export batch, persist report đều bị ảnh hưởng.
- Impact: Developer không thể đọc AC một cách tin cậy — có thể dẫn đến implement thiếu hoặc sai.
- Đề xuất: Re-generate hoặc fix encoding của toàn bộ Epic 4 (4 stories).

---

#### 🟠 Major Issues

**MAJOR-1: Story 1.2 thiếu AC cho Preset CRUD**
- Story 1.2 chỉ cover `select preset` và `apply preset`.
- Không có AC cho: `Tạo preset mới`, `Sửa preset`, `Duplicate preset`.
- UX spec Section 6.2 và PRD FR19 đều đặc tả các actions này.
- Đề xuất: Expand Story 1.2 hoặc tạo Story 1.5 "Manage Brand Presets (Create/Edit/Duplicate)".

**MAJOR-2: Story 2.1 thiếu AC cho Background Queue / Non-blocking UI**
- Story 2.1 start batch processing nhưng không có AC kiểm chứng UI không bị block khi queue chạy nền.
- PRD NFR1 và Architecture explicitly require non-blocking UI.
- Đề xuất: Thêm AC: "Given queue is processing, When user interacts with UI, Then UI remains responsive and not blocked."

**MAJOR-3: Story 2.5 thiếu AC cho Processing Queue UI Display**
- Story 2.5 cover persisting status nhưng không cover UI display.
- UX-DR7 yêu cầu: progress tổng, progress per video, panel log ngắn.
- PRD Section 9.4 yêu cầu: "Thể hiện rõ video nào còn thiếu audio mapping nếu task yêu cầu audio."
- Đề xuất: Thêm AC cho Processing Queue screen display behavior vào Story 2.5 hoặc Story 2.1.

**MAJOR-4: Story 3.3 thiếu AC cho Split Preview và Overlay Handles**
- Story 3.3 cover before/after toggle và segment jump, nhưng thiếu:
  - Split preview mode (UX spec Section 6.5.2)
  - Overlay handles cho logo, blur/mask box, subtitle mới
- Impact: Developer không biết preview workspace cần hỗ trợ handles tương tác.
- Đề xuất: Thêm AC cho split mode và interactive overlay handles vào Story 3.3 hoặc split thành Story 3.3a/3.3b.

**MAJOR-5: Story 3.4 thiếu các AC quan trọng**
- Thiếu AC cho:
  - `bật/tắt logo overlay` (UX-ADD-1, xác nhận là expected UX control)
  - `Đánh dấu đã review` action
  - `Khôi phục mặc định` (reset to preset default)
- Đề xuất: Thêm 3 ACs vào Story 3.4.

**MAJOR-6: Story 3.5 thiếu các AC quan trọng**
- Thiếu AC cho:
  - `chọn mode: blur/mask/fill` cho subtitle old region (UX-ADD-3)
  - `Đánh dấu đã review` action
  - `Khôi phục mặc định` action
- Đề xuất: Thêm 3 ACs vào Story 3.5.

---

#### 🟡 Minor Concerns

**MINOR-1: Story 1.1 thiếu AC cho saving job configuration**
- FR4: "Hệ thống phải cho phép người dùng lưu cấu hình job trước khi chạy."
- Story 1.1 cover import nhưng không có AC kiểm chứng job config được lưu.
- Đề xuất: Thêm AC: "Given user imports files and selects task, When a draft job is created, Then job configuration is persisted locally."

**MINOR-2: Story 2.3 vague về "apply per preset rules"**
- AC: "áp dụng kết quả theo rule của preset hiện tại" không specify rõ "rule" ở đây là gì.
- Developer không biết logo overlay phải reference preset's `defaultLogoPath` và `layoutRules`.
- Đề xuất: Làm rõ AC để mention preset's logo asset và layout rules được áp dụng.

**MINOR-3: Story 4.3 thiếu quality vs speed preference**
- PRD FR50: "Hệ thống phải ưu tiên chất lượng ổn định hơn tốc độ tối đa."
- Không có AC nào đặt constraint này cho export pipeline.
- Đề xuất: Thêm implicit constraint vào Story 4.3 hoặc acceptance test.

**MINOR-4: Story 3.1 empty state cho non-flagged video chưa rõ**
- AC: "vẫn mở được review workspace" cho non-flagged video.
- Chưa specify UX của "Không có đoạn bắt buộc phải sửa" state (UX-DR17).
- Đề xuất: Thêm AC: "When review is opened for a video with no flags, Then system displays clear message: 'Không có đoạn bắt buộc phải sửa'."

**MINOR-5: "Mở job gần đây" chưa có story**
- UX spec Start Screen có nút này nhưng không có AC hay story nào cover.
- Cần scope decision trước (in/out V1), sau đó add story nếu in-scope.

---

### Best Practices Compliance Checklist

| Epic | Delivers User Value | Independently Completable | Stories Sized OK | No Forward Deps | Clear ACs | FR Traceability |
|------|--------------------|--------------------------|--------------------|-----------------|-----------|-----------------|
| Epic 1 | ✅ | ✅ | ⚠️ (thiếu setup story) | ✅ | ⚠️ (gap 1.2) | ✅ |
| Epic 2 | ✅ | ✅ | ✅ | ✅ | ⚠️ (gap 2.1, 2.5) | ✅ |
| Epic 3 | ✅ | ✅ | ✅ | ✅ | ⚠️ (gap 3.3, 3.4, 3.5) | ✅ |
| Epic 4 | ✅ | ✅ | ✅ | ✅ | 🔴 (encoding corrupt) | ✅ |

---

### Epic Quality Summary

| Severity | Count | Items |
|----------|-------|-------|
| 🔴 Critical | 2 | Missing project setup story; Epic 4 encoding corruption |
| 🟠 Major | 6 | Preset CRUD AC; Background queue AC; Processing Queue UI; Split preview/handles; Story 3.4 ACs; Story 3.5 ACs |
| 🟡 Minor | 5 | Story 1.1 save config; Story 2.3 vague rule; Story 4.3 quality; Story 3.1 empty state; Recent jobs story |

### UX Alignment Summary

| Loại | Tìm thấy | Mức độ |
|------|----------|--------|
| UX ↔ PRD additions (không conflict, chỉ mở rộng) | 3 | Minor-Medium |
| UX ↔ PRD discrepancy | 1 | Medium |
| Architecture API gaps vs UX | 3 | Medium-High |

---

## Summary and Recommendations

### Overall Readiness Status

## ⚠️ NEEDS WORK

Dự án có nền tảng planning chắc chắn — PRD rõ ràng, coverage rate cao (96%), architecture align tốt với UX. Tuy nhiên, **2 Critical issues và 6 Major issues** phải được giải quyết trước khi bắt đầu implementation để tránh rủi ro về delivery.

---

### Critical Issues Requiring Immediate Action

#### 🔴 CRITICAL-1: Epic 4 — Encoding Corruption (BLOCK)
- **Vấn đề:** Toàn bộ Epic 4 stories (4.1 đến 4.4) có ký tự bị corrupt trong `epics.md`. AC text gần như không đọc được.
- **Risk:** Developer implement Epic 4 sai hoặc thiếu do không đọc được requirement.
- **Action:** Re-generate hoặc fix encoding cho Stories 4.1–4.4 ngay trước khi sprint bắt đầu.

#### 🔴 CRITICAL-2: Thiếu Project Setup Story (BLOCK)
- **Vấn đề:** Greenfield project (Tauri + React + TypeScript + FFmpeg) không có story "setup initial project".
- **Risk:** Dev sprint 1 bắt đầu mà không có baseline shared project structure, gây ra divergence ngay từ đầu.
- **Action:** Thêm Story 1.0 hoặc Story 0.1 với scope: setup Tauri project, configure React + TypeScript, integrate FFmpeg binary, verify dev environment.

---

### Recommended Next Steps

**Ưu tiên 1 — Trước khi implement (MUST FIX):**

1. **Fix Epic 4 encoding** — Re-generate Stories 4.1–4.4 với full Vietnamese text.
2. **Add Project Setup Story** — Thêm story mới cho Tauri + React + FFmpeg project initialization.
3. **Add Preset Management Story** — Expand Story 1.2 hoặc tạo Story 1.5 cover create/edit/duplicate preset với ACs rõ ràng.

**Ưu tiên 2 — Trước khi sprint planning (SHOULD FIX):**

4. **Bổ sung ACs cho Story 2.1** — Thêm AC cho background queue / non-blocking UI.
5. **Bổ sung ACs cho Story 2.5** — Thêm AC cho Processing Queue UI display (progress tổng, per-video, log panel).
6. **Bổ sung ACs cho Story 3.3** — Thêm AC cho split preview mode và overlay handles.
7. **Bổ sung ACs cho Story 3.4** — Thêm AC cho logo toggle, đánh dấu reviewed, khôi phục mặc định.
8. **Bổ sung ACs cho Story 3.5** — Thêm AC cho blur/mask/fill mode selector, đánh dấu reviewed, khôi phục mặc định.
9. **Cập nhật Architecture Section 13.1** — Thêm commands: `createPreset`, `editPreset`, `duplicatePreset`.
10. **Cập nhật Architecture Section 13.2** — Thêm event: `inputFileReplacedAfterReview`.

**Ưu tiên 3 — Quyết định scope (DECIDE FIRST):**

11. **"Mở job gần đây"** — Confirm in-scope hay out-scope cho V1, sau đó add hoặc explicitly exclude trong stories.
12. **Platform target** — Confirm Windows-only hay cross-platform (macOS/Linux), để inform build pipeline setup.
13. **PRD FR30 file input change warning** — Expand Story 1.3 để cover cảnh báo khi thay đổi file input sau khi đã review.

**Ưu tiên 4 — Minor polish (NICE TO HAVE):**

14. Thêm AC cho Story 1.1 (save job config), Story 2.3 (clarify preset rules), Story 4.3 (quality vs speed), Story 3.1 (empty state).
15. Cập nhật UX spec để thêm `Exporting` và `Done` states vào Processing Screen.

---

### Final Note

Assessment này tìm thấy **16 issues** trên **5 categories**:

| Category | Issues | Critical | Major | Minor |
|----------|--------|----------|-------|-------|
| PRD Coverage | 2 | 0 | 2 | 0 |
| UX Alignment | 7 | 0 | 3 | 4 |
| Epic Quality | 13 | 2 | 6 | 5 |
| Architecture Gaps | 2 | 0 | 2 | 0 |
| Scope Decisions | 2 | 0 | 0 | 2 |

**Điểm mạnh của planning hiện tại:**
- PRD rõ ràng, scope V1 được giới hạn tốt với explicit non-goals.
- FR coverage rate: ~96% (52/54 fully covered).
- Architecture align chặt với UX và PRD.
- Epic structure là user-centric, không có technical milestones.
- Dependency ordering của 4 epics là hợp lý và không có circular dependency.

**Quyết định quan trọng trước khi implementation:**
- Giải quyết 2 Critical issues (Epic 4 encoding + project setup story) là bắt buộc.
- 6 Major AC gaps nên được fill trước sprint planning để tránh scope creep giữa sprint.

**Assessor:** BMAD Implementation Readiness Workflow  
**Date:** 2026-04-21  
**Report file:** `_bmad-output/planning-artifacts/implementation-readiness-report-2026-04-21.md`
