# Gas Town Input Requirements

## Mục Tiêu

Tài liệu này mô tả chính xác **đầu vào tốt** để Gas Town có thể thực thi hiệu quả ngay, đặc biệt trong các workflow:

- epic -> child tasks -> convoy -> sling -> done -> refinery
- parallel execution
- dependency-aware execution
- merge queue + verification
- handoff khi full context

Tài liệu này cũng trả lời câu hỏi:

- Gas Town có hỗ trợ task test không?
- Test nên nằm ở đâu trong plan?
- Cần mô tả test như thế nào để worker và refinery thực thi đúng?

## 1. Kết Luận Ngắn

Gas Town không cần một “plan document” cố định, nhưng để chạy tốt thì đầu vào phải:

1. **đủ rõ để giao việc**
2. **đủ nhỏ để một worker có thể hoàn thành**
3. **đủ cấu trúc để biểu diễn dependency**
4. **đủ cụ thể để biết khi nào công việc xong**
5. **đủ kiểm chứng để refinery hoặc worker có thể verify**

Nói ngắn gọn:

- **Input tốt cho Gas Town không phải là plan dài**
- mà là **work items rõ, có scope, có acceptance, có dependency, có verification**

## 2. Gas Town Có Cần Plan Rất Chi Tiết Không?

**Không bắt buộc phải cực kỳ chi tiết**, nhưng phải đạt một ngưỡng tối thiểu để trở thành:

- epic
- task
- bug
- feature
- workflow step

Gas Town mạnh nhất khi input đã qua giai đoạn:

- brainstorming
- khám phá mơ hồ
- tranh luận chiến lược cấp cao

và đã sang giai đoạn:

- decomposition
- assignment
- implementation
- verification
- merge

## 3. Input Tốt Nhất Cho Gas Town Có Dạng Gì?

Ba dạng input phù hợp nhất:

### 3.1 Epic + Child Tasks

Đây là dạng tốt nhất nếu feature lớn.

Ví dụ:

- Epic: `Auth overhaul`
- Child 1: `Add login API`
- Child 2: `Create session table migration`
- Child 3: `Build login UI`
- Child 4: `Add integration tests`
- Child 5: `Update auth docs`

Kèm dependency:

- UI depends on API
- integration tests depend on API + UI
- docs depend on API behavior being stable

### 3.2 Standalone Tasks + Convoy

Nếu không cần epic, bạn vẫn có thể có nhiều task độc lập:

- fix bug A
- fix bug B
- update docs C
- add test D

Rồi gom chúng vào convoy để track batch.

### 3.3 Formula / Molecule

Nếu công việc có pattern lặp rõ:

- TDD cycle
- release
- patrol
- review
- migration procedure

thì nên encode bằng formula/molecule.

## 4. Input Tối Thiểu Mỗi Task Nên Có

Một task tốt cho Gas Town nên có ít nhất:

### 4.1 Title rõ

Ví dụ tốt:

- `Add POST /auth/login endpoint`
- `Create user_sessions migration`
- `Add integration test for invalid login`

Ví dụ xấu:

- `Fix auth`
- `Make login better`
- `Clean up backend`

### 4.2 Mục tiêu cụ thể

Phải trả lời được:

- task này tạo ra cái gì?
- sửa cái gì?
- output cuối là gì?

### 4.3 Scope rõ

Phải giới hạn:

- module nào
- service nào
- UI nào
- file/domain nào

Ví dụ:

- chỉ backend auth module
- không đổi UI
- không đổi schema ngoài bảng `user_sessions`

### 4.4 Acceptance criteria

Đây là phần rất quan trọng.

Task nên nói rõ:

- điều kiện nào thì coi là xong
- hành vi nào phải đúng
- test nào phải pass
- build/lint có phải pass không

### 4.5 Dependency

Nếu task phụ thuộc task khác, phải khai báo rõ.

Nếu không, hệ thống sẽ khó:

- xác định wave đầu
- unblock wave sau
- tránh dispatch sai thứ tự

### 4.6 Verification expectation

Nên chỉ rõ:

- unit test cần thêm không
- integration test cần thêm không
- build/lint/test command nào là chuẩn

## 5. Template Task Khuyên Dùng

Đây là format thực tế tốt cho Gas Town:

```text
Title: Add login API
Type: task

Goal:
- Add POST /auth/login
- Validate email/password
- Return JWT + refresh token

Scope:
- backend/auth only
- no frontend changes
- no database schema changes

Acceptance Criteria:
- endpoint exists and returns 200 on valid credentials
- returns 401 on invalid password
- unit tests added for auth service
- integration test covers happy path + invalid password
- OpenAPI spec updated

Dependencies:
- None

Verification:
- go test ./...
- golangci-lint run
- go build ./...
```

## 6. Template Epic Khuyên Dùng

```text
Title: Auth overhaul
Type: epic

Goal:
- deliver full login flow across backend, UI, tests, and docs

Children:
- Add login API
- Add session table migration
- Build login UI
- Add integration tests
- Update auth docs

Dependency Notes:
- login UI depends on login API
- integration tests depend on API + UI
- docs can start late, after behavior is stable

Landing Strategy:
- use integration branch for the epic
```

## 7. Input Xấu Trông Như Thế Nào?

### 7.1 Quá mơ hồ

Ví dụ:

- `Improve auth`
- `Rewrite backend`
- `Make code cleaner`

Vấn đề:

- không biết scope
- không biết xong là gì
- không biết task con là gì

### 7.2 Quá to

Ví dụ:

- `Build billing system`
- `Re-architect notification platform`

Vấn đề:

- không thể giao trực tiếp cho một polecat ổn định
- dễ full context
- khó acceptance
- khó merge

### 7.3 Không có dependency

Ví dụ:

- tạo 8 task nhưng không chỉ ra task nào phải đi trước

Vấn đề:

- dispatch sai thứ tự
- worker làm việc bị block giữa chừng
- convoy không phản ánh đúng wave

### 7.4 Không có verification

Ví dụ:

- chỉ mô tả code change nhưng không nói test/build/lint kỳ vọng gì

Vấn đề:

- worker khó biết mức hoàn tất
- refinery chỉ kiểm tra mức mặc định của rig
- dễ merge code “xong theo cảm giác”

## 8. Task Nên Nhỏ Đến Mức Nào?

Một task tốt nên:

- giải quyết một mục tiêu rõ
- thường nằm trong một phạm vi module/domain tương đối hẹp
- có thể hoàn thành trong một vòng làm việc của một worker
- không buộc agent phải giữ quá nhiều trạng thái trong context

Heuristic thực dụng:

- nếu task cần nhiều hơn một chuỗi reasoning lớn và nhiều lần handoff chỉ để “hiểu việc”, task đó có thể đang quá lớn
- nếu task chạm quá nhiều domain cùng lúc, nên tách
- nếu acceptance criteria dài như một mini-spec lớn, nên chia nhỏ

## 9. Gas Town Có Task Test Các Task Không?

**Có.**

Nhưng cần hiểu đúng:

- Gas Town **có thể orchestration việc test**
- Gas Town **có merge queue verification**
- Gas Town **có thể có task riêng cho test**
- Gas Town **có thể encode workflow TDD/test trong formulas**

Nhưng Gas Town **không tự động đảm bảo mọi task đều có test hợp lý nếu đầu vào không mô tả rõ**.

## 10. Test Nằm Ở Đâu Trong Gas Town?

Test có thể xuất hiện ở 3 lớp:

### 10.1 Test là một phần acceptance của task implement

Ví dụ:

- task implement API phải kèm unit test
- task implement UI phải kèm component test

Đây là cách tốt nhất cho thay đổi nhỏ và rõ phạm vi.

### 10.2 Test là task riêng

Ví dụ:

- `Add integration tests for auth flow`
- `Add regression tests for migration edge cases`

Cách này phù hợp khi:

- test đủ lớn để thành deliverable riêng
- test phụ thuộc nhiều task upstream
- cần tách worker implement và worker verification

### 10.3 Test là merge gate của Refinery

Rig có thể cấu hình:

- `lint_command`
- `test_command`
- `build_command`
- `typecheck_command`

Như vậy dù task không tạo bead test riêng, refinery vẫn có verification gate trước khi merge.

## 11. Khi Nào Nên Tạo Task Test Riêng?

Nên tạo task test riêng khi:

1. test là deliverable đáng kể
2. test phụ thuộc nhiều child tasks khác
3. cần làm rõ trách nhiệm verification
4. muốn chạy wave verification riêng sau wave implementation
5. regression risk cao

Ví dụ:

- Epic: auth overhaul
- Child tasks:
  - login API
  - login UI
  - session migration
  - auth integration test suite

Ở đây `auth integration test suite` nên là task riêng.

## 12. Khi Nào Không Cần Task Test Riêng?

Không cần tách bead test riêng khi:

1. test rất nhỏ và gắn chặt với code change
2. unit test có thể là acceptance mặc định của task
3. merge gates đã đủ để chặn code lỗi

Ví dụ:

- đổi một validator nhỏ
- fix một bug parse input
- thêm một helper function

Lúc đó chỉ cần ghi rõ trong acceptance:

- “add/update unit tests”

## 13. Cấu Trúc Input Tốt Khi Có Test

### Mẫu 1: Test nằm trong task implement

```text
Title: Add login API
Type: task

Goal:
- Add POST /auth/login

Acceptance Criteria:
- endpoint returns 200 for valid login
- endpoint returns 401 for invalid password
- unit tests added for auth service
- integration test updated for login route

Verification:
- go test ./...
- golangci-lint run
- go build ./...
```

### Mẫu 2: Test là task riêng

```text
Title: Add auth integration tests
Type: task

Goal:
- cover login, invalid password, expired session refresh

Dependencies:
- Add login API
- Build login UI

Acceptance Criteria:
- integration suite covers auth happy path
- invalid password path covered
- refresh token path covered
- CI test command passes

Verification:
- go test ./integration/...
```

## 14. Đầu Vào Tốt Để Gas Town Có Thể Thực Thi Ngay

Nếu muốn “đưa vào là chạy được ngay”, mỗi task nên đạt checklist sau:

### Checklist bắt buộc

1. **Title rõ**
2. **Type rõ**
3. **Goal rõ**
4. **Scope rõ**
5. **Acceptance criteria rõ**
6. **Dependencies rõ hoặc explicitly none**

### Checklist nên có

7. **Verification commands**
8. **Test expectation**
9. **Landing strategy**
10. **Owner hint hoặc runtime hint nếu cần**

## 15. Mẫu Input Sẵn Sàng Cho Gas Town

```text
Epic: Auth overhaul

Child Task 1:
Title: Add login API
Type: task
Goal:
- Add POST /auth/login that returns JWT and refresh token
Scope:
- backend/auth only
Acceptance:
- valid credentials return 200
- invalid password returns 401
- unit tests added
- auth OpenAPI updated
Dependencies:
- None
Verification:
- go test ./...
- golangci-lint run

Child Task 2:
Title: Build login UI
Type: task
Goal:
- Add login form wired to auth API
Scope:
- frontend/auth pages only
Acceptance:
- valid login redirects to dashboard
- invalid login shows error
- component test added
Dependencies:
- Add login API
Verification:
- pnpm test
- pnpm build

Child Task 3:
Title: Add auth integration tests
Type: task
Goal:
- Add end-to-end auth verification across API and UI
Scope:
- integration test suite only
Acceptance:
- happy path covered
- invalid password covered
- expired session flow covered
Dependencies:
- Add login API
- Build login UI
Verification:
- pnpm test:e2e
```

Đây là loại input mà Gas Town có thể:

- chuyển thành beads
- nhận diện dependency
- dispatch theo wave
- track qua convoy
- merge qua refinery

## 16. Quy Tắc Thực Dụng

Nếu muốn Gas Town chạy tốt, hãy coi mỗi task như một “hợp đồng giao việc”:

- worker phải hiểu phải làm gì
- witness phải hiểu worker có đang bị block không
- refinery phải hiểu kiểm tra cái gì
- convoy phải hiểu khi nào batch hoàn tất

Một task không đủ rõ sẽ làm hỏng cả bốn tầng này.

## 17. Kết Luận

### Đầu vào tốt cho Gas Town là:

- rõ mục tiêu
- rõ scope
- rõ acceptance
- rõ dependency
- rõ verification

### Gas Town có hỗ trợ test không?

**Có.**

Theo ba cách:

1. test là acceptance trong task implement
2. test là task riêng
3. test là merge gate của refinery

### Cách làm tốt nhất

- test nhỏ -> để trong task implement
- test lớn / integration / regression -> tách thành task riêng
- luôn cấu hình merge queue gates ở rig

Như vậy Gas Town mới thực thi ổn định, không chỉ “chạy task”, mà còn **chạy task có kiểm chứng**.
