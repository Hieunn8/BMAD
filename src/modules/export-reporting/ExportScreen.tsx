import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { useReviewStore } from "../../store/reviewStore";
import { useJobStore } from "../../store/jobStore";
import { useExportStore } from "../../store/exportStore";
import type {
  BatchExportCompletedEvent,
  ExportProgressEvent,
  ExportReadinessResponse,
  JobExportSummaryReportResponse,
  JobUpdatedEvent,
  SetExportOutputFolderResponse,
  StartExportResponse,
  VideoExportCompletedEvent,
  VideoExportStartedEvent,
  VideoReportResponse,
} from "../start-flow/types";
import { ExportReadinessList } from "./ExportReadinessList";
import { ReportScreen } from "./ReportScreen";

export function ExportScreen() {
  const currentJob = useJobStore((state) => state.currentJob);
  const setJob = useJobStore((state) => state.setJob);
  const {
    batchSummary,
    batchVideos,
    blockedVideos,
    closeScreen,
    errorByVideo,
    finishBatch,
    hydrate,
    isExporting,
    isLoading,
    jobSummaryReport,
    markVideoCompleted,
    markVideoStarted,
    openScreen,
    outputFolder,
    outputPathByVideo,
    presetSummary,
    progressByVideo,
    readyVideos,
    reportsByVideo,
    setJobSummaryReport,
    setLoading,
    setOutputFolder,
    setVideoReport,
    startBatch,
    statusByVideo,
    updateProgress,
  } = useExportStore();
  const openReviewWorkspace = useReviewStore((state) => state.openWorkspace);
  const setSelectedVideoId = useReviewStore((state) => state.setSelectedVideoId);
  const setShowAllVideos = useReviewStore((state) => state.setShowAllVideos);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [infoMessage, setInfoMessage] = useState<string | null>(null);

  useEffect(() => {
    if (!currentJob) {
      return;
    }

    let mounted = true;

    const loadReadiness = async () => {
      setLoading(true);
      try {
        const response = await invoke<ExportReadinessResponse>("get_export_readiness", {
          jobId: currentJob.jobId,
        });
        if (mounted) {
          hydrate(response.result);
        }
      } catch (error) {
        if (mounted) {
          setErrorMessage(error instanceof Error ? error.message : String(error));
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    const loadSummaryReport = async () => {
      if (!["Exported", "ExportedWithFailures", "ExportFailed"].includes(currentJob.status)) {
        return;
      }
      try {
        const response = await invoke<JobExportSummaryReportResponse>("get_job_export_summary_report", {
          jobId: currentJob.jobId,
        });
        if (mounted) {
          setJobSummaryReport(response.summary);
          response.summary.reports.forEach((report) => setVideoReport(report));
        }
      } catch {
        // Ignore missing reports during initial load; they will appear after export completes.
      }
    };

    void loadReadiness();
    void loadSummaryReport();
    return () => {
      mounted = false;
    };
  }, [currentJob?.jobId, currentJob?.status, hydrate, setJobSummaryReport, setLoading, setVideoReport]);

  useEffect(() => {
    if (!currentJob) {
      return;
    }

    const unlistenJobUpdated = listen<JobUpdatedEvent>("jobUpdated", (event) => {
      if (event.payload.job.jobId === currentJob.jobId) {
        setJob(event.payload.job);
      }
    });
    const unlistenStarted = listen<VideoExportStartedEvent>("videoExportStarted", (event) => {
      markVideoStarted(event.payload.videoId);
      setErrorMessage(null);
      setInfoMessage(`Dang export ${event.payload.videoId}`);
    });
    const unlistenProgress = listen<ExportProgressEvent>("exportProgress", (event) => {
      updateProgress(event.payload.videoId, event.payload.percent);
    });
    const unlistenCompleted = listen<VideoExportCompletedEvent>("videoExportCompleted", (event) => {
      markVideoCompleted({
        videoId: event.payload.videoId,
        success: event.payload.success,
        outputPath: event.payload.outputPath,
        errorMessage: event.payload.errorMessage,
      });
      void invoke<VideoReportResponse>("get_report", {
        jobId: currentJob.jobId,
        videoId: event.payload.videoId,
      })
        .then((response) => {
          setVideoReport(response.report);
        })
        .catch((error) => {
          setErrorMessage(error instanceof Error ? error.message : String(error));
        });
      if (!event.payload.success && event.payload.errorMessage) {
        setErrorMessage(event.payload.errorMessage);
      }
    });
    const unlistenBatch = listen<BatchExportCompletedEvent>("batchExportCompleted", (event) => {
      if (event.payload.jobId !== currentJob.jobId) {
        return;
      }
      finishBatch({
        total: event.payload.total,
        success: event.payload.success,
        failed: event.payload.failed,
      });
      setInfoMessage(`Export xong: ${event.payload.success}/${event.payload.total} video thanh cong`);
      void invoke<ExportReadinessResponse>("get_export_readiness", {
        jobId: currentJob.jobId,
      })
        .then((response) => {
          hydrate(response.result);
        })
        .catch((error) => {
          setErrorMessage(error instanceof Error ? error.message : String(error));
        });
      void invoke<JobExportSummaryReportResponse>("get_job_export_summary_report", {
        jobId: currentJob.jobId,
      })
        .then((response) => {
          setJobSummaryReport(response.summary);
          response.summary.reports.forEach((report) => setVideoReport(report));
        })
        .catch((error) => {
          setErrorMessage(error instanceof Error ? error.message : String(error));
        });
    });

    return () => {
      void unlistenJobUpdated.then((dispose) => dispose());
      void unlistenStarted.then((dispose) => dispose());
      void unlistenProgress.then((dispose) => dispose());
      void unlistenCompleted.then((dispose) => dispose());
      void unlistenBatch.then((dispose) => dispose());
    };
  }, [
    currentJob?.jobId,
    finishBatch,
    hydrate,
    markVideoCompleted,
    markVideoStarted,
    setJob,
    setJobSummaryReport,
    setVideoReport,
    updateProgress,
  ]);

  if (!currentJob) {
    return null;
  }

  const pickOutputFolder = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: outputFolder ?? currentJob.outputFolder,
      title: "Chon output folder",
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    try {
      const response = await invoke<SetExportOutputFolderResponse>("set_export_output_folder", {
        jobId: currentJob.jobId,
        outputFolder: selected,
      });
      setJob(response.job);
      setOutputFolder(selected);
      setErrorMessage(null);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    }
  };

  const startExport = async () => {
    try {
      const exportTargets = readyVideos.slice();
      const response = await invoke<StartExportResponse>("start_export", {
        jobId: currentJob.jobId,
      });
      setJob(response.job);
      if (response.started) {
        openScreen();
        startBatch(exportTargets);
        setInfoMessage(`Bat dau export ${exportTargets.length} video`);
        setErrorMessage(null);
      }
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    }
  };

  const openReview = () => {
    closeScreen();
    openReviewWorkspace();
  };

  const openReviewFromReport = (videoId: string) => {
    closeScreen();
    setShowAllVideos(true);
    setSelectedVideoId(videoId);
    openReviewWorkspace();
  };

  const progressVideos = batchVideos.length > 0 ? batchVideos : readyVideos;
  const completedCount = Object.values(statusByVideo).filter((status) => status === "Exported").length;
  const derivedSummary =
    jobSummaryReport ??
    (Object.keys(reportsByVideo).length > 0
      ? {
          jobId: currentJob.jobId,
          totalVideos: Object.keys(reportsByVideo).length,
          success: Object.values(reportsByVideo).filter((report) => report.finalStatus === "Exported").length,
          failed: Object.values(reportsByVideo).filter((report) => report.finalStatus === "Failed").length,
          totalOutputSizeMb: Object.values(reportsByVideo).reduce(
            (total, report) => total + (report.encodeSummary?.outputSizeMb ?? 0),
            0
          ),
          reports: Object.values(reportsByVideo),
          generatedAt: new Date().toISOString(),
        }
      : null);

  return (
    <section className="segment-review-screen export-screen">
      <div className="segment-review-screen__hero">
        <div>
          <p className="start-screen__eyebrow">Export Readiness</p>
          <h2 className="start-screen__headline">Kiem tra batch truoc khi export</h2>
        </div>
        <div className="segment-review-screen__meta">
          <span className="inline-note">Job: {currentJob.jobId}</span>
          {isLoading ? <span className="inline-note">Dang tai export readiness...</span> : null}
          <button type="button" className="action-button action-button--small" onClick={openReview}>
            Mo review workspace
          </button>
        </div>
      </div>

      {errorMessage ? <div className="inline-error">{errorMessage}</div> : null}
      {infoMessage ? <div className="inline-note">{infoMessage}</div> : null}

      <section className="workspace-panel">
        <div className="workspace-panel__header">
          <h3>Export Settings</h3>
          <span>{presetSummary ? `${presetSummary.codec} - CRF ${presetSummary.crf}` : "No preset"}</span>
        </div>
        <div className="export-screen__settings">
          <div className="workspace-panel">
            <strong>Output Folder</strong>
            <p>{outputFolder ?? "Chua chon output folder"}</p>
            <button type="button" className="action-button" onClick={() => void pickOutputFolder()} disabled={isExporting}>
              Chon output folder
            </button>
          </div>
          <div className="workspace-panel">
            <strong>Export Preset</strong>
            <p>{presetSummary?.label ?? "Chua co preset export"}</p>
          </div>
          <div className="workspace-panel">
            <strong>Batch Progress</strong>
            {batchSummary ? (
              <p>
                {batchSummary.success}/{batchSummary.total} thanh cong, {batchSummary.failed} that bai
              </p>
            ) : (
              <p>
                {completedCount}/{progressVideos.length} video exported
              </p>
            )}
          </div>
        </div>
      </section>

      <ExportReadinessList
        blockedVideos={blockedVideos}
        errorByVideo={errorByVideo}
        outputPathByVideo={outputPathByVideo}
        progressByVideo={progressByVideo}
        readyVideos={progressVideos}
        statusByVideo={statusByVideo}
      />

      <ReportScreen summary={derivedSummary} onOpenReview={openReviewFromReport} />

      <div className="job-setup-screen__actions">
        <button
          type="button"
          className="action-button action-button--primary"
          disabled={readyVideos.length === 0 || isExporting}
          onClick={() => void startExport()}
        >
          {isExporting ? "Dang export..." : "Export All Ready Videos"}
        </button>
        {blockedVideos.length > 0 ? (
          <span className="inline-note">Con {blockedVideos.length} video dang bi chan.</span>
        ) : (
          <span className="inline-note">Tat ca video da san sang export.</span>
        )}
      </div>
    </section>
  );
}
