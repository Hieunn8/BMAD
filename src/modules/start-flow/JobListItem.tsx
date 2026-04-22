import type { RecentJobSummary } from "./types";

type JobListItemProps = {
  job: RecentJobSummary;
  isLoading: boolean;
  onResume: (jobId: string) => void;
};

export function JobListItem({ job, isLoading, onResume }: JobListItemProps) {
  const statusSlug = job.status.toLowerCase().replace(/\s+/g, "-");
  const createdAt = new Date(job.createdAt);
  const lastModified = new Date(job.lastModified);

  return (
    <div className="recent-job-card">
      <div className="recent-job-card__meta">
        <div>
          <strong>{job.jobId}</strong>
          <p>
            Tao luc {createdAt.toLocaleString()} • cap nhat {lastModified.toLocaleString()}
          </p>
        </div>
        <span className={`mapping-pill mapping-pill--${statusSlug}`}>{job.status}</span>
      </div>
      <div className="recent-job-card__actions">
        <span className="inline-note">{job.videoCount} video</span>
        <button
          type="button"
          className="action-button action-button--small"
          disabled={isLoading}
          onClick={() => onResume(job.jobId)}
        >
          {isLoading ? "Dang tai..." : "Tiep tuc"}
        </button>
      </div>
    </div>
  );
}
