import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useJobStore } from "../../store/jobStore";
import { useReviewStore } from "../../store/reviewStore";
import type {
  InputFileReplacedAfterReviewEvent,
  JobReadinessResponse,
  JobStartedEvent,
  MappingResponse,
  MappingUpdatedEvent,
  StartJobResponse,
} from "../start-flow/types";
import { MappingTable } from "./MappingTable";

export function JobSetupScreen() {
  const currentJob = useJobStore((state) => state.currentJob);
  const mappingRows = useJobStore((state) => state.mappingRows);
  const preset = useJobStore((state) => state.preset);
  const readinessState = useJobStore((state) => state.readinessState);
  const setJob = useJobStore((state) => state.setJob);
  const setMappingRows = useJobStore((state) => state.setMappingRows);
  const setReadinessState = useJobStore((state) => state.setReadinessState);
  const openReviewWorkspace = useReviewStore((state) => state.openWorkspace);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isStarting, setIsStarting] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [reviewWarning, setReviewWarning] = useState<string | null>(null);

  useEffect(() => {
    if (!currentJob || !preset) {
      return;
    }

    let mounted = true;

    const loadMappings = async () => {
      setIsLoading(true);
      setErrorMessage(null);

      try {
        const response = await invoke<MappingResponse>("auto_map_job", {
          jobId: currentJob.jobId,
        });

        if (!mounted) {
          return;
        }

        setJob(response.job);
        setMappingRows(response.rows);
        await loadReadiness(response.job.jobId);
      } catch (error) {
        if (mounted) {
          setErrorMessage(error instanceof Error ? error.message : String(error));
        }
      } finally {
        if (mounted) {
          setIsLoading(false);
        }
      }
    };

    void loadMappings();

    return () => {
      mounted = false;
    };
  }, [currentJob?.jobId, preset?.presetId, preset, setJob, setMappingRows]);

  useEffect(() => {
    const unlistenUpdated = listen<MappingUpdatedEvent>("mappingUpdated", (event) => {
      setJob(event.payload.job);
      setMappingRows(event.payload.rows);
      void loadReadiness(event.payload.job.jobId);
    });

    const unlistenJobStarted = listen<JobStartedEvent>("jobStarted", (event) => {
      setJob(event.payload.job);
    });

    const unlistenReviewWarning = listen<InputFileReplacedAfterReviewEvent>(
      "inputFileReplacedAfterReview",
      (event) => {
        setReviewWarning(event.payload.message);
      }
    );

    return () => {
      void unlistenUpdated.then((dispose) => dispose());
      void unlistenJobStarted.then((dispose) => dispose());
      void unlistenReviewWarning.then((dispose) => dispose());
    };
  }, [setJob, setMappingRows, setReadinessState]);

  if (!currentJob || !preset) {
    return null;
  }

  async function loadReadiness(jobId: string) {
    const response = await invoke<JobReadinessResponse>("get_job_readiness", { jobId });
    setReadinessState(response.readiness);
  }

  const applyMapping = async (videoId: string, field: "logo" | "audio" | "srt", path: string) => {
    setIsSaving(true);
    setErrorMessage(null);

    try {
      const response = await invoke<MappingResponse>("fix_mapping", {
        jobId: currentJob.jobId,
        videoId,
        field,
        filePath: path,
      });

      setJob(response.job);
      setMappingRows(response.rows);
      await loadReadiness(response.job.jobId);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSaving(false);
    }
  };

  const readyCount = mappingRows.filter((row) => row.status === "Ready").length;

  const startJob = async () => {
    setIsStarting(true);
    setErrorMessage(null);

    try {
      const response = await invoke<StartJobResponse>("start_job", {
        jobId: currentJob.jobId,
      });

      setJob(response.job);

      if (!response.started && response.blockers.length > 0) {
        setErrorMessage(response.blockers.join(" "));
      }
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsStarting(false);
    }
  };

  return (
    <section className="job-setup-screen">
      <div className="job-setup-screen__hero">
        <div>
          <p className="start-screen__eyebrow">Job Setup</p>
          <h2 className="start-screen__headline">Review mapping truoc khi chay batch</h2>
        </div>
        <div className="job-setup-screen__meta">
          <span className="inline-note">
            {readyCount}/{mappingRows.length} video san sang
          </span>
          <span className="inline-note">Preset: {preset.brandName}</span>
        </div>
      </div>

      {isLoading ? <div className="inline-note">Dang tinh mapping cho tung video...</div> : null}
      {reviewWarning ? <div className="inline-error">{reviewWarning}</div> : null}
      {errorMessage ? <div className="inline-error">{errorMessage}</div> : null}
      {readinessState && !readinessState.isReady ? (
        <div className="inline-error">{readinessState.blockers.join(" ")}</div>
      ) : null}

      {mappingRows.length > 0 ? (
        <MappingTable
          isSaving={isSaving}
          onApply={applyMapping}
          readinessState={readinessState}
          rows={mappingRows}
        />
      ) : (
        <div className="workspace-panel">
          <p className="job-setup-screen__empty">
            Chua co dong mapping nao. Import video va chon preset de he thong bat dau doi chieu input.
          </p>
        </div>
      )}

      <div className="job-setup-screen__actions">
        <button
          type="button"
          className="action-button action-button--primary"
          disabled={!readinessState?.isReady || isStarting}
          onClick={() => void startJob()}
        >
          {isStarting ? "Dang bat dau..." : "Chay tu dong"}
        </button>
        {currentJob.status === "ReviewPending" || currentJob.status === "ReadyToExport" ? (
          <button
            type="button"
            className="action-button"
            onClick={() => openReviewWorkspace()}
          >
            Mo review workspace
          </button>
        ) : null}
        {!readinessState?.isReady ? (
          <span className="inline-note">Can giai quyet tat ca blocker truoc khi chay.</span>
        ) : (
          <span className="inline-note">Job da san sang de chay batch.</span>
        )}
      </div>
    </section>
  );
}
