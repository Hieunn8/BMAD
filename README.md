# BMAD Workspace

Kho nay la workspace BMAD da cai san de lam viec voi AI coding agents, uu tien tieng Viet, phuc vu ca **Codex** va **Claude Code**.

## Muc tieu

- Cung cap bo skill va workflow BMAD dong bo cho nhieu agent.
- To chuc tai lieu va artifact theo tung giai doan (analysis, planning, implementation).
- Chuan hoa cach thuc tu ideation -> planning -> dev -> review.

## Pham vi repo

Repo da bao gom day du:

- `.agents/` - skills va dinh nghia agent cho he sinh thai Codex.
- `.claude/` - skills tuong ung cho Claude Code.
- `_bmad/` - cau hinh module BMAD (`core`, `bmm`) va manifest cai dat.
- `_bmad-output/` - noi xuat ket qua khi chay cac workflow.
- `docs/` - kho tri thuc du an, tai lieu domain/tech/business.

## Thong tin cai dat BMAD

Theo `_bmad/_config/manifest.yaml`:

- BMAD version: `6.3.0`
- Modules: `core`, `bmm`
- IDE targets: `claude-code`, `codex`
- Communication language: `Vietnamese`
- Document output language: `Vietnamese`

## Cau truc thu muc

```text
BMAD/
|- .agents/
|  `- skills/                      # Skill pack cho Codex
|- .claude/
|  `- skills/                      # Skill pack cho Claude Code
|- _bmad/
|  |- _config/
|  |  |- manifest.yaml             # Metadata cai dat BMAD
|  |  |- bmad-help.csv             # Bang command/workflow huong dan
|  |  |- agent-manifest.csv
|  |  |- skill-manifest.csv
|  |  `- files-manifest.csv
|  |- core/
|  |  |- config.yaml
|  |  `- module-help.csv
|  `- bmm/
|     |- config.yaml
|     `- module-help.csv
|- _bmad-output/                   # Artifact sinh ra khi thuc thi
`- docs/                           # Tri thuc du an (co the bo sung tuy y)
```

## Workflow de xuat

BMAD khuyen nghi pipeline:

1. **Analysis**: Domain Research / Market Research / Technical Research / Product Brief.
2. **Planning**: Create PRD -> (tuy chon) Create UX -> Create Architecture -> Create Epics & Stories.
3. **Implementation**: Sprint Planning -> Create Story -> Dev Story -> Code Review -> Retrospective.
4. **Anytime tools**: Quick Dev, Document Project, Distillator, Checkpoint, Party Mode.

Ban co the tham khao chi tiet command mapping trong:

- `_bmad/_config/bmad-help.csv`

## Su dung voi Codex va Claude Code

### 1) Clone repo

```bash
git clone https://github.com/Hieunn8/BMAD.git
cd BMAD
```

### 2) Mo workspace bang agent tool ban dang dung

- Neu dung Codex: mo terminal tai root repo va bat dau go yeu cau theo skill/workflow BMAD.
- Neu dung Claude Code: mo cung root repo de he thong tu nap `.claude/skills`.

### 3) Luu artifact dung cho

- Planning artifacts: `_bmad-output/planning-artifacts`
- Implementation artifacts: `_bmad-output/implementation-artifacts`
- Project knowledge: `docs/`

## Cac nhom skill noi bat

- Strategy & Discovery: `bmad-product-brief`, `bmad-prfaq`, `bmad-domain-research`, `bmad-market-research`, `bmad-technical-research`
- Planning: `bmad-create-prd`, `bmad-create-ux-design`, `bmad-create-architecture`, `bmad-create-epics-and-stories`
- Delivery: `bmad-sprint-planning`, `bmad-create-story`, `bmad-dev-story`, `bmad-code-review`, `bmad-retrospective`
- Utilities: `bmad-quick-dev`, `bmad-document-project`, `bmad-distillator`, `bmad-checkpoint-preview`, `bmad-party-mode`

## Nguyen tac van hanh de tranh loi

- Lam viec tai root repo de agent nhin thay day du `.agents`, `.claude`, `_bmad`.
- Khong xoa/doi ten cac file manifest trong `_bmad/_config` neu khong can thiet.
- Tach artifact va tai lieu ra khoi source code (de trong `_bmad-output` va `docs`).
- Neu can cap nhat skill pack, commit dong bo ca `.agents` va `.claude`.

## Quyen va bao mat

- Kiem tra ky truoc khi commit cac file co du lieu nhay cam.
- Neu tao them script automation, uu tien doc/ghi trong pham vi repo.
- Khi push len remote public, dam bao khong de lo token, key, credential.

## Dong gop

1. Tao branch moi.
2. Cap nhat skill/config/doc theo nhu cau.
3. Kiem tra lai `README.md`, manifest, va output path.
4. Tao commit ro rang, push va mo Pull Request.

## License

Chua khai bao license rieng. Neu can chia se cong khai, nen bo sung file `LICENSE`.
