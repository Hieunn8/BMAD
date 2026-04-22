import { invoke } from "@tauri-apps/api/core";
import type {
  ApplySegmentsResponse,
  LogoFix,
  ReviewSegment,
  ReviewSegmentResponse,
  SubtitleFix,
} from "../start-flow/types";

type QuickFixPanelProps = {
  jobId: string;
  selectedSegment: ReviewSegment | null;
  selectedSegmentIds: string[];
  pendingLogoFix: LogoFix | null;
  pendingSubtitleFix: SubtitleFix | null;
  onPendingLogoFixChange: (value: LogoFix | null) => void;
  onPendingSubtitleFixChange: (value: SubtitleFix | null) => void;
  onSegmentsUpdated: (segments: ReviewSegment[]) => void;
  onSegmentUpdated: (segment: ReviewSegment) => void;
};

export function QuickFixPanel({
  jobId,
  selectedSegment,
  selectedSegmentIds,
  pendingLogoFix,
  pendingSubtitleFix,
  onPendingLogoFixChange,
  onPendingSubtitleFixChange,
  onSegmentsUpdated,
  onSegmentUpdated,
}: QuickFixPanelProps) {
  if (!selectedSegment) {
    return (
      <div className="segment-review-panel">
        <div className="segment-review-panel__header">
          <div>
            <p className="start-screen__eyebrow">Quick Fix Panel</p>
            <h3>Chon segment de thao tac</h3>
          </div>
        </div>
        <p className="segment-review-placeholder">
          Chon mot segment de move, resize, apply fix, accept hoac reset.
        </p>
      </div>
    );
  }

  const multiSelection = selectedSegmentIds.length > 1;

  const applyLogoFix = async (segmentIds: string[]) => {
    if (!pendingLogoFix) {
      return;
    }

    const response = await invoke<ApplySegmentsResponse>("apply_logo_fix", {
      jobId,
      payload: {
        segmentIds,
        logoFix: pendingLogoFix,
      },
    });
    onSegmentsUpdated(response.result.updatedSegments);
    if (response.result.warningMessage) {
      window.alert(response.result.warningMessage);
    }
  };

  const applySubtitleFix = async (segmentIds: string[]) => {
    if (!pendingSubtitleFix) {
      return;
    }

    const response = await invoke<ApplySegmentsResponse>("apply_subtitle_fix", {
      jobId,
      payload: {
        segmentIds,
        subtitleFix: pendingSubtitleFix,
      },
    });
    onSegmentsUpdated(response.result.updatedSegments);
    if (response.result.warningMessage) {
      window.alert(response.result.warningMessage);
    }
  };

  const markAccepted = async () => {
    const response = await invoke<ReviewSegmentResponse>("mark_segment_accepted", {
      jobId,
      segmentId: selectedSegment.id,
    });
    onSegmentUpdated(response.segment);
  };

  const reset = async () => {
    const command = selectedSegment.source === "logo" ? "reset_logo_fix" : "reset_subtitle_fix";
    const response = await invoke<ReviewSegmentResponse>(command, {
      jobId,
      segmentId: selectedSegment.id,
    });
    onSegmentUpdated(response.segment);
  };

  return (
    <div className="segment-review-panel">
      <div className="segment-review-panel__header">
        <div>
          <p className="start-screen__eyebrow">Quick Fix Panel</p>
          <h3>{selectedSegment.issueType}</h3>
        </div>
        {multiSelection ? <span className="inline-note">{selectedSegmentIds.length} segment dang chon</span> : null}
      </div>

      {selectedSegment.source === "logo" && pendingLogoFix ? (
        <div className="quick-fix-form">
          <NumberField label="X" value={pendingLogoFix.x} onChange={(value) => onPendingLogoFixChange({ ...pendingLogoFix, x: value })} />
          <NumberField label="Y" value={pendingLogoFix.y} onChange={(value) => onPendingLogoFixChange({ ...pendingLogoFix, y: value })} />
          <NumberField label="Width" value={pendingLogoFix.width} onChange={(value) => onPendingLogoFixChange({ ...pendingLogoFix, width: value })} />
          <NumberField label="Height" value={pendingLogoFix.height} onChange={(value) => onPendingLogoFixChange({ ...pendingLogoFix, height: value })} />
        </div>
      ) : null}

      {selectedSegment.source === "subtitle" && pendingSubtitleFix ? (
        <div className="quick-fix-form">
          <NumberField
            label="Old Region X"
            value={pendingSubtitleFix.oldRegion?.x ?? 0}
            onChange={(value) =>
              onPendingSubtitleFixChange({
                ...pendingSubtitleFix,
                oldRegion: {
                  ...(pendingSubtitleFix.oldRegion ?? { x: 0, y: 0, width: 160, height: 80, mode: "blur" }),
                  x: value,
                },
              })
            }
          />
          <NumberField
            label="Old Region Y"
            value={pendingSubtitleFix.oldRegion?.y ?? 0}
            onChange={(value) =>
              onPendingSubtitleFixChange({
                ...pendingSubtitleFix,
                oldRegion: {
                  ...(pendingSubtitleFix.oldRegion ?? { x: 0, y: 0, width: 160, height: 80, mode: "blur" }),
                  y: value,
                },
              })
            }
          />
          <NumberField
            label="Region Width"
            value={pendingSubtitleFix.oldRegion?.width ?? 0}
            onChange={(value) =>
              onPendingSubtitleFixChange({
                ...pendingSubtitleFix,
                oldRegion: {
                  ...(pendingSubtitleFix.oldRegion ?? { x: 0, y: 0, width: 160, height: 80, mode: "blur" }),
                  width: value,
                },
              })
            }
          />
          <NumberField
            label="Region Height"
            value={pendingSubtitleFix.oldRegion?.height ?? 0}
            onChange={(value) =>
              onPendingSubtitleFixChange({
                ...pendingSubtitleFix,
                oldRegion: {
                  ...(pendingSubtitleFix.oldRegion ?? { x: 0, y: 0, width: 160, height: 80, mode: "blur" }),
                  height: value,
                },
              })
            }
          />
          <NumberField
            label="New Subtitle X"
            value={pendingSubtitleFix.newPosition?.x ?? 0}
            onChange={(value) =>
              onPendingSubtitleFixChange({
                ...pendingSubtitleFix,
                newPosition: {
                  ...(pendingSubtitleFix.newPosition ?? { x: 0, y: 0 }),
                  x: value,
                },
              })
            }
          />
          <NumberField
            label="New Subtitle Y"
            value={pendingSubtitleFix.newPosition?.y ?? 0}
            onChange={(value) =>
              onPendingSubtitleFixChange({
                ...pendingSubtitleFix,
                newPosition: {
                  ...(pendingSubtitleFix.newPosition ?? { x: 0, y: 0 }),
                  y: value,
                },
              })
            }
          />
          <label className="quick-fix-form__field">
            <span>Mode</span>
            <select
              value={pendingSubtitleFix.oldRegion?.mode ?? "blur"}
              onChange={(event) =>
                onPendingSubtitleFixChange({
                  ...pendingSubtitleFix,
                  oldRegion: {
                    ...(pendingSubtitleFix.oldRegion ?? { x: 0, y: 0, width: 160, height: 80, mode: "blur" }),
                    mode: event.target.value,
                  },
                })
              }
            >
              <option value="blur">blur</option>
              <option value="mask">mask</option>
              <option value="fill">fill</option>
            </select>
          </label>
          <label className="quick-fix-form__field">
            <span>Scale</span>
            <input
              type="number"
              min={0.2}
              step={0.1}
              value={pendingSubtitleFix.newScale ?? 1}
              onChange={(event) =>
                onPendingSubtitleFixChange({
                  ...pendingSubtitleFix,
                  newScale: Number(event.target.value),
                })
              }
            />
          </label>
          <label className="quick-fix-form__field">
            <span>Style Preset</span>
            <input
              type="text"
              value={pendingSubtitleFix.stylePreset ?? ""}
              onChange={(event) =>
                onPendingSubtitleFixChange({
                  ...pendingSubtitleFix,
                  stylePreset: event.target.value,
                })
              }
            />
          </label>
        </div>
      ) : null}

      <div className="job-setup-screen__actions">
        {selectedSegment.source === "logo" ? (
          <>
            <button type="button" className="action-button action-button--primary" onClick={() => void applyLogoFix([selectedSegment.id])}>
              Ap dung cho segment nay
            </button>
            <button
              type="button"
              className="action-button"
              disabled={!multiSelection}
              onClick={() => void applyLogoFix(selectedSegmentIds)}
            >
              Ap dung cho cac segment da chon
            </button>
          </>
        ) : (
          <>
            <button type="button" className="action-button action-button--primary" onClick={() => void applySubtitleFix([selectedSegment.id])}>
              Ap dung cho segment nay
            </button>
            <button
              type="button"
              className="action-button"
              disabled={!multiSelection}
              onClick={() => void applySubtitleFix(selectedSegmentIds)}
            >
              Ap dung cho cac segment da chon
            </button>
          </>
        )}
        <button type="button" className="action-button" onClick={() => void markAccepted()}>
          Danh dau da review
        </button>
        <button type="button" className="action-button action-button--danger" onClick={() => void reset()}>
          Khoi phuc mac dinh
        </button>
      </div>
    </div>
  );
}

function NumberField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="quick-fix-form__field">
      <span>{label}</span>
      <input type="number" value={value} onChange={(event) => onChange(Number(event.target.value))} />
    </label>
  );
}
