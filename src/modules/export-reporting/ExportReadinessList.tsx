import type { ExportBlockedVideo, ExportReadyVideo } from "../start-flow/types";

type ExportReadinessListProps = {
  blockedVideos: ExportBlockedVideo[];
  errorByVideo: Record<string, string>;
  outputPathByVideo: Record<string, string>;
  progressByVideo: Record<string, number>;
  readyVideos: ExportReadyVideo[];
  statusByVideo: Record<string, string>;
};

export function ExportReadinessList({
  blockedVideos,
  errorByVideo,
  outputPathByVideo,
  progressByVideo,
  readyVideos,
  statusByVideo,
}: ExportReadinessListProps) {
  return (
    <div className="export-screen__grid">
      <section className="workspace-panel">
        <div className="workspace-panel__header">
          <h3>Ready To Export</h3>
          <span>{readyVideos.length} video</span>
        </div>
        <div className="recent-job-list">
          {readyVideos.length === 0 ? (
            <div className="segment-review-empty">Chua co video nao du dieu kien export.</div>
          ) : (
            readyVideos.map((video) => {
              const currentStatus = statusByVideo[video.videoId] ?? video.status;
              const progress = progressByVideo[video.videoId] ?? 0;
              return (
                <div key={video.videoId} className="recent-job-card">
                  <div className="recent-job-card__meta">
                    <div>
                      <strong>{video.videoName}</strong>
                      <p>{video.audioSummary}</p>
                      <p>{currentStatus === "Exporting" || currentStatus === "Exported" ? `${progress}%` : currentStatus}</p>
                      {outputPathByVideo[video.videoId] ? <p>{outputPathByVideo[video.videoId]}</p> : null}
                      {errorByVideo[video.videoId] ? <p>{errorByVideo[video.videoId]}</p> : null}
                    </div>
                    <span className={`mapping-pill mapping-pill--${currentStatus.toLowerCase()}`}>{currentStatus}</span>
                  </div>
                  <progress max={100} value={progress} />
                </div>
              );
            })
          )}
        </div>
      </section>

      <section className="workspace-panel">
        <div className="workspace-panel__header">
          <h3>Blocked</h3>
          <span>{blockedVideos.length} video</span>
        </div>
        <div className="recent-job-list">
          {blockedVideos.length === 0 ? (
            <div className="segment-review-empty">Khong co video bi chan.</div>
          ) : (
            blockedVideos.map((video) => (
              <div key={video.videoId} className="recent-job-card">
                <div className="recent-job-card__meta">
                  <div>
                    <strong>{video.videoName}</strong>
                    <p>{video.reason}</p>
                    <p>{video.audioSummary}</p>
                  </div>
                  <span className={`mapping-pill mapping-pill--${video.status.toLowerCase()}`}>{video.status}</span>
                </div>
              </div>
            ))
          )}
        </div>
      </section>
    </div>
  );
}
