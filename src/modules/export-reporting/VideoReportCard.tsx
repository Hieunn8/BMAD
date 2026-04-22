import { convertFileSrc } from "@tauri-apps/api/core";
import type { VideoReport } from "../start-flow/types";

type VideoReportCardProps = {
  onOpenReview: (videoId: string) => void;
  report: VideoReport;
};

export function VideoReportCard({ onOpenReview, report }: VideoReportCardProps) {
  return (
    <article className="workspace-panel">
      <div className="workspace-panel__header">
        <h3>{report.videoName}</h3>
        <span className={`mapping-pill mapping-pill--${report.finalStatus.toLowerCase()}`}>{report.finalStatus}</span>
      </div>

      <p>Output: {report.outputPath ?? "Chua co output"}</p>
      <p>
        Encode:{" "}
        {report.encodeSummary
          ? `${report.encodeSummary.codec} | CRF ${report.encodeSummary.crf} | ${report.encodeSummary.outputSizeMb} MB`
          : "Chua co encode summary"}
      </p>
      <p>
        Segments: total {report.segmentStats.total}, flagged {report.segmentStats.flagged}, modified{" "}
        {report.segmentStats.modified}, accepted {report.segmentStats.accepted}
      </p>
      <p>
        Audio: {report.audioSource.policy}
        {report.audioSource.audioFilePath ? ` | ${report.audioSource.audioFilePath}` : ""}
      </p>

      {report.spotCheckThumbnails.length > 0 ? (
        <div className="export-report-card__thumb-grid">
          {report.spotCheckThumbnails.map((item) => (
            <div key={item.segmentId} className="export-report-card__thumb-pair">
              <strong>{item.segmentId}</strong>
              <div className="export-report-card__thumb-row">
                {item.beforePath ? <img src={convertFileSrc(item.beforePath)} alt={`${item.segmentId} before`} /> : null}
                {item.afterPath ? <img src={convertFileSrc(item.afterPath)} alt={`${item.segmentId} after`} /> : null}
              </div>
            </div>
          ))}
        </div>
      ) : null}

      <button type="button" className="action-button action-button--small" onClick={() => onOpenReview(report.videoId)}>
        Xem lai video nay
      </button>
    </article>
  );
}
