import { invoke } from "@tauri-apps/api/core";
import type { ReviewGatingResponse } from "../start-flow/types";

type ReviewGatingPanelProps = {
  jobId: string;
  videoId: string | null;
  status: string | null;
  blockerCount: number;
  onGatingLoaded: (canProceed: boolean, blockers: { segmentId: string; timeRange: string; issueType: string }[]) => void;
  onVideoMarkedReady: () => void;
};

export function ReviewGatingPanel({
  jobId,
  videoId,
  status,
  blockerCount,
  onGatingLoaded,
  onVideoMarkedReady,
}: ReviewGatingPanelProps) {
  const refresh = async () => {
    if (!videoId) {
      return;
    }

    const response = await invoke<ReviewGatingResponse>("check_video_review_gating", {
      jobId,
      videoId,
    });
    onGatingLoaded(response.result.canProceed, response.result.blockers);
  };

  const markReady = async () => {
    if (!videoId) {
      return;
    }

    const response = await invoke<ReviewGatingResponse>("mark_video_ready", {
      jobId,
      videoId,
    });
    onGatingLoaded(response.result.canProceed, response.result.blockers);
    if (response.result.canProceed) {
      onVideoMarkedReady();
    }
  };

  return (
    <div className="segment-review-panel">
      <div className="segment-review-panel__header">
        <div>
          <p className="start-screen__eyebrow">Review Gating</p>
          <h3>Video ready gate</h3>
        </div>
        <span className={`mapping-pill mapping-pill--${(status ?? "unknown").toLowerCase()}`}>{status ?? "Unknown"}</span>
      </div>
      {blockerCount > 0 ? (
        <div className="segment-review-banner">
          Con {blockerCount} doan High Risk chua xu ly hoac accept.
        </div>
      ) : (
        <div className="segment-review-banner">Khong con blocker High Risk nao.</div>
      )}
      <div className="job-setup-screen__actions">
        <button type="button" className="action-button" onClick={() => void refresh()} disabled={!videoId}>
          Kiem tra blocker
        </button>
        <button type="button" className="action-button action-button--primary" onClick={() => void markReady()} disabled={!videoId}>
          Danh dau xong review
        </button>
      </div>
    </div>
  );
}
