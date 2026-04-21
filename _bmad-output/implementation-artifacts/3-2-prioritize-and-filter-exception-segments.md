# Story 3.2: Prioritize and Filter Exception Segments

Status: ready-for-dev

## Story

As a user,
I want ưu tiên và lọc các segment theo mức độ rủi ro và loại lỗi,
so that tôi có thể xử lý đúng các vấn đề nghiêm trọng nhất trước.

## Acceptance Criteria

1. Segment list hiển thị mỗi segment với: time range, issue type, risk level, review status.
2. Mặc định: sắp xếp `High Risk` trước rồi theo thời gian.
3. Lọc theo risk level, issue type, hoặc resolved/unresolved → chỉ hiện segments phù hợp.

## Tasks / Subtasks

- [ ] Implement segment list sorting (AC: 2)
  - [ ] Default sort: `High Risk` → `Medium Risk` → `Low Risk`, trong cùng mức risk → sort by startTime asc
  - [ ] Sort logic xử lý ở frontend (client-side), data đã được load vào reviewStore
- [ ] Implement segment list filtering (AC: 3)
  - [ ] Filter options: Risk Level (All / High / Medium / Low), Issue Type (All / Logo / Subtitle / SubtitleRegion), Status (All / Unreviewed / Accepted / Modified / Blocked)
  - [ ] Multi-filter: kết hợp risk + issue type + status cùng lúc
  - [ ] Filter state lưu trong reviewStore, không reset khi chuyển video
- [ ] Implement segment row component (AC: 1)
  - [ ] Hiển thị: time range (mm:ss–mm:ss), issue type icon + label, risk badge, review status badge
  - [ ] Click → select, emit `segmentSelected` internal event
  - [ ] Multi-select: Checkbox + Shift+Click range select (cần cho bulk fix trong Story 3.4, 3.5)
- [ ] Implement filter bar UI (AC: 3)
  - [ ] Row of filter chips/dropdowns phía trên segment list
  - [ ] Hiển thị số segment match filter hiện tại: "Đang hiện 5 / 12 đoạn"
  - [ ] Reset button để clear all filters

## Dev Notes

- **Client-side sort/filter**: tất cả segments đã được load vào `reviewStore`. Sort và filter chỉ là array operations — không gọi backend.
- **Risk levels** (NFR10): `High` = blockers bắt buộc phải xử lý; `Medium` = khuyến khích review; `Low` = tự động pass.
- **Multi-select** là foundation cho Story 3.4 và 3.5 (`Áp dụng cho các segment đã chọn`). Implement đúng ở đây để không refactor sau.
- **Issue type mapping**: Logo → `issueType = "LogoPosition"`; Subtitle removal → `"SubtitleRegion"`; Subtitle render → `"SubtitleStyle"` (nếu cần). Giữ consistent với Segment domain model.
- **Filter persistence**: user không nên phải re-set filter khi quay lại sau khi sửa một segment. Giữ filter state active.
- **Empty state**: nếu filter không có kết quả → hiển thị `"Không có đoạn nào khớp với bộ lọc hiện tại"` thay vì bảng rỗng.

### Project Structure Notes

- Frontend: `src/modules/segment-review/SegmentList.tsx` (sorting + filtering + multi-select), `src/modules/segment-review/FilterBar.tsx`
- Store: `src/store/reviewStore.ts` (`filterState`, `selectedSegmentIds`, `sortedSegments` computed)

### References

- [Source: epics.md#Story 3.2] Acceptance criteria
- [Source: prd.md#7.1] FR15: segment list là review UI chính; FR16: risk levels
- [Source: ux-design-specification.md#6.5] Segment list sort/filter (UX-DR9)
- [Source: architecture.md#5.3] Segment domain: riskLevel, issueType, reviewStatus

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
