---
validationTarget: 'D:\WORKING\BMAD\_bmad-output\planning-artifacts\prd.md'
validationDate: '2026-04-20'
inputDocuments:
  - D:\WORKING\BMAD\_bmad-output\planning-artifacts\prd.md
  - D:\WORKING\BMAD\_bmad-output\brainstorming\brainstorming-session-2026-04-20-1600.md
validationStepsCompleted:
  - discovery
validationStatus: COMPLETED
---

# PRD Validation Report

**PRD Being Validated:** `D:\WORKING\BMAD\_bmad-output\planning-artifacts\prd.md`  
**Validation Date:** 2026-04-20

## Input Documents

- PRD: `prd.md`
- Brainstorming artifact: `brainstorming-session-2026-04-20-1600.md`

## Validation Findings

### 1. Measurability is too weak for several core requirements

The PRD repeatedly uses terms such as "cơ bản", "đủ tốt", "ổn định", and "một số edge case" without defining thresholds or operational criteria. This makes it hard to validate success, implement risk scoring, or align engineering decisions.

Affected areas include:

- persona expectation of "đủ tốt" detection
- logo detection and subtitle detection quality
- quality stability and output acceptability
- success metrics that are directionally correct but not measurable

Impact:

- product, UX, QA, and engineering may each interpret V1 success differently
- architecture cannot choose the right detection-review boundary confidently

Recommendation:

- define V1 target thresholds for:
  - acceptable auto-pass rate
  - maximum review-needed segment rate per video
  - acceptable export failure rate
  - acceptable manual-fix time per flagged segment

### 2. Review-exception logic is central to the product, but the gating rules are not specified

The PRD correctly centers the workflow on `auto process -> review exceptions -> export`, but it never defines what qualifies a segment as risky, what confidence range triggers review, or when a video can move from `Review Needed` to `Ready to Export`.

Impact:

- the queue state model is underspecified
- UX cannot know how aggressive or conservative the review screen should be
- QA/reporting cannot produce consistent output

Recommendation:

- specify:
  - risk scoring inputs
  - review threshold rules
  - segment severity labels
  - video-level pass criteria after review

### 3. Batch input mapping is underspecified for the core V1 use case

The PRD says each video maps to one SRT, but the batch import flow does not define how this mapping happens in practice when many videos and many SRT files are dragged in together.

Missing rules include:

- filename matching strategy
- behavior when one video has no matching SRT
- behavior when multiple SRTs appear to match one video
- whether users can manually correct mappings before auto process

Impact:

- this is a core operational flow for persona Chị Mai
- implementation may become fragile or confusing without explicit matching rules

Recommendation:

- define a deterministic mapping strategy for V1, such as:
  - exact base filename match first
  - then user review for unresolved files
  - block job start when required mappings are missing

### 4. Crash safety and resume are mentioned, but not defined at the workflow level

The PRD says the system should support `resume` or `retry` for interrupted jobs, but it does not specify which artifacts or processing stages must persist.

Missing details:

- whether per-video detection outputs are cached
- whether review fixes persist before export
- whether export resumes from partially completed queue items
- whether rerun invalidates previous review decisions

Impact:

- crash-safe processing is part of the product promise for batch work
- architecture cannot design job persistence cleanly without clearer boundaries

Recommendation:

- define minimum resumability for V1:
  - persist job manifest
  - persist per-video status
  - persist segment flags
  - persist user review edits
  - do not require partial export resume if that is too expensive for V1

### 5. Core screens are clear, but the PRD does not yet define the decision boundary between wizard mode and review mode tightly enough

The PRD correctly chooses a hybrid UX, but it still leaves ambiguity around when users remain in a simple task flow and when they are dropped into the review workspace.

Missing details:

- can users force review even if no risk segments are detected
- can users skip review for low-risk jobs
- is review mandatory for all jobs with any flagged segment
- how much of the quick-fix UI is exposed by default to novice users

Impact:

- UX direction is set, but interaction design is not yet constrained enough for implementation

Recommendation:

- add explicit mode-transition rules between:
  - setup
  - auto process
  - required review
  - optional spot check
  - export-ready

## Validation Verdict

**Status:** Pass with revision needed

The PRD is strong enough to continue into UX design or architecture, but it should be tightened before epics/stories if you want cleaner implementation slicing and less ambiguity in review logic.

## Suggested Next Edits

1. Add measurable V1 success thresholds.
2. Define exception-review gating rules.
3. Define batch file mapping behavior.
4. Define minimum crash-safe persistence model.
5. Clarify mode transitions between wizard flow and review flow.
