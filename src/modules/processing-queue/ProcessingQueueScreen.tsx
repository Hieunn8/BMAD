import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { useJobStore } from "../../store/jobStore";
import { useReviewStore } from "../../store/reviewStore";
import type {
  JobProcessingCompletedEvent,
  JobStartedEvent,
  JobUpdatedEvent,
  ProcessingStepUpdateEvent,
  VideoProcessingCompletedEvent,
  VideoProcessingEvent,
  VideoReadyToExportEvent,
} from "../start-flow/types";

const LOG_MAX = 50;

type VideoStep = {
  step: string;
  status: string;
  message: string;
};

type LogEntry = {
  id: number;
  videoId: string;
  message: string;
};

let logSeq = 0;

export function ProcessingQueueScreen() {
  const currentJob = useJobStore((state) => state.currentJob);
  const setJob = useJobStore((state) => state.setJob);
  const openReviewWorkspace = useReviewStore((state) => state.openWorkspace);

  const [videoSteps, setVideoSteps] = useState<Map<string, VideoStep>>(new Map());
  const [videoOutcomes, setVideoOutcomes] = useState<Map<string, string>>(new Map());
  const [logEntries, setLogEntries] = useState<LogEntry[]>([]);
  const [jobSummary, setJobSummary] = useState<{ reviewNeeded: number; readyToExport: number; failed: number } | null>(null);
  const logEndRef = useRef<HTMLDivElement>(null);
  const currentJobRef = useRef(currentJob);

  useEffect(() => {
    currentJobRef.current = currentJob;
  }, [currentJob]);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logEntries]);

  useEffect(() => {
    const appendLog = (videoId: string, message: string) => {
      setLogEntries((prev) => {
        const entry: LogEntry = { id: logSeq++, videoId, message };
        const next = [...prev, entry];
        return next.length > LOG_MAX ? next.slice(-LOG_MAX) : next;
      });
    };

    const unlistenStarted = listen<JobStartedEvent>("jobStarted", (event) => {
      setJob(event.payload.job);
    });

    const unlistenUpdated = listen<JobUpdatedEvent>("jobUpdated", (event) => {
      setJob(event.payload.job);
    });

    const unlistenVideoStart = listen<VideoProcessingEvent>("videoProcessingStarted", (event) => {
      appendLog(event.payload.videoId, `[${event.payload.videoId}] Bat dau xu ly`);
    });

    const unlistenStepUpdate = listen<ProcessingStepUpdateEvent>("processingStepUpdate", (event) => {
      const { videoId, step, status, message } = event.payload;
      setVideoSteps((prev) => {
        const next = new Map(prev);
        next.set(videoId, { step, status, message });
        return next;
      });
      appendLog(videoId, `[${videoId}] ${step} → ${status}: ${message}`);
    });

    const unlistenVideoCompleted = listen<VideoProcessingCompletedEvent>("videoProcessingCompleted", (event) => {
      const { videoId, outcome, segmentCount } = event.payload;
      setVideoOutcomes((prev) => {
        const next = new Map(prev);
        next.set(videoId, outcome);
        return next;
      });
      appendLog(videoId, `[${videoId}] Hoan thanh: ${outcome} (${segmentCount} segment)`);
    });

    const unlistenJobCompleted = listen<JobProcessingCompletedEvent>("jobProcessingCompleted", (event) => {
      const { summary } = event.payload;
      setJobSummary({ reviewNeeded: summary.reviewNeeded, readyToExport: summary.readyToExport, failed: summary.failed });
    });

    const unlistenVideoReady = listen<VideoReadyToExportEvent>("videoReadyToExport", (event) => {
      const stateJob = currentJobRef.current;
      if (stateJob) {
        setJob({
          ...stateJob,
          videoItems: stateJob.videoItems.map((video) =>
            video.videoId === event.payload.videoId
              ? { ...video, status: "ReadyToExport" }
              : video
          ),
        });
      }
      setVideoOutcomes((prev) => {
        const next = new Map(prev);
        next.set(event.payload.videoId, "ReadyToExport");
        return next;
      });
      appendLog(event.payload.videoId, `[${event.payload.videoId}] Review xong: ReadyToExport`);
    });

    return () => {
      void unlistenStarted.then((d) => d());
      void unlistenUpdated.then((d) => d());
      void unlistenVideoStart.then((d) => d());
      void unlistenStepUpdate.then((d) => d());
      void unlistenVideoCompleted.then((d) => d());
      void unlistenJobCompleted.then((d) => d());
      void unlistenVideoReady.then((d) => d());
    };
  }, [setJob]);

  if (!currentJob) {
    return null;
  }

  const totalVideos = currentJob.videoItems.length;
  const doneStatuses = new Set(["ReviewNeeded", "ReadyToExport", "Failed", "Processed"]);
  const doneCount = currentJob.videoItems.filter((v) => doneStatuses.has(v.status)).length;
  const progressPct = totalVideos > 0 ? Math.round((doneCount / totalVideos) * 100) : 0;

  return (
    <section className="processing-queue-screen">
      <div className="processing-queue-screen__hero">
        <div>
          <p className="start-screen__eyebrow">Processing Queue</p>
          <h2 className="start-screen__headline">Job dang chay tren background</h2>
        </div>
        <div className="processing-queue-screen__meta">
          <span className="inline-note">Job: {currentJob.jobId}</span>
          <span className="inline-note">
            {doneCount}/{totalVideos} video da xu ly
          </span>
        </div>
      </div>

      {/* Overall progress bar */}
      <div className="processing-queue-screen__progress">
        <div className="processing-queue-screen__progress-bar">
          <div
            className="processing-queue-screen__progress-fill"
            style={{ width: `${progressPct}%` }}
          />
        </div>
        <span className="inline-note">{progressPct}%</span>
      </div>

      {/* Per-video list */}
      <div className="processing-queue-list">
        {currentJob.videoItems.map((video) => {
          const stepInfo = videoSteps.get(video.videoId);
          const outcome = videoOutcomes.get(video.videoId);
          const displayStatus = outcome ?? video.status;
          const statusSlug = displayStatus.toLowerCase().replace(/\s+/g, "-");

          return (
            <div key={video.videoId} className="processing-queue-list__item">
              <div className="processing-queue-list__item-info">
                <strong>{video.sourcePath.split(/[/\\]/).pop()}</strong>
                {stepInfo && (
                  <p className="processing-queue-list__step-label">
                    {stepInfo.step} — {stepInfo.message}
                  </p>
                )}
              </div>
              <div className={`mapping-pill mapping-pill--${statusSlug}`}>
                {displayStatus}
              </div>
            </div>
          );
        })}
      </div>

      {/* Job summary (shown after completion) */}
      {jobSummary && (
        <div className="processing-queue-screen__summary">
          <span className="inline-note">Hoan thanh:</span>
          <span className="inline-note review-needed">{jobSummary.reviewNeeded} can review</span>
          <span className="inline-note ready">{jobSummary.readyToExport} san sang export</span>
          {jobSummary.failed > 0 && (
            <span className="inline-note failed">{jobSummary.failed} loi</span>
          )}
          <button type="button" className="action-button action-button--small" onClick={() => openReviewWorkspace()}>
            Mo review workspace
          </button>
        </div>
      )}

      {/* Log panel (ring buffer, max 50 entries) */}
      <div className="processing-queue-screen__log">
        <p className="start-screen__eyebrow">Log</p>
        <div className="processing-queue-screen__log-body">
          {logEntries.map((entry) => (
            <div key={entry.id} className="processing-queue-screen__log-entry">
              {entry.message}
            </div>
          ))}
          <div ref={logEndRef} />
        </div>
      </div>
    </section>
  );
}
