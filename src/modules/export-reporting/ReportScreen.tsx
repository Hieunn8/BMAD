import type { JobExportSummaryReport } from "../start-flow/types";
import { VideoReportCard } from "./VideoReportCard";

type ReportScreenProps = {
  onOpenReview: (videoId: string) => void;
  summary: JobExportSummaryReport | null;
};

export function ReportScreen({ onOpenReview, summary }: ReportScreenProps) {
  if (!summary || summary.reports.length === 0) {
    return null;
  }

  return (
    <section className="workspace-panel">
      <div className="workspace-panel__header">
        <h3>Export Reports</h3>
        <span>
          {summary.success}/{summary.totalVideos} success | {summary.failed} failed | {summary.totalOutputSizeMb} MB
        </span>
      </div>
      <div className="recent-job-list">
        {summary.reports.map((report) => (
          <VideoReportCard key={report.videoId} report={report} onOpenReview={onOpenReview} />
        ))}
      </div>
    </section>
  );
}
