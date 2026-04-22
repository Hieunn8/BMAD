import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import { useJobStore } from "../../store/jobStore";
import { useExportStore } from "../../store/exportStore";
import { useReviewStore } from "../../store/reviewStore";
import { DragDropZone } from "./DragDropZone";
import { JobListItem } from "./JobListItem";
import {
  type AppTask,
  type ClassifiedImportResult,
  type CreateJobResponse,
  type ListJobsResponse,
  type LoadJobResponse,
  type RecentJobSummary,
  TASK_OPTIONS,
  groupAcceptedFiles,
  mergeAcceptedFiles,
  suggestTaskFromRoles,
} from "./types";

export function StartScreen() {
  const currentJob = useJobStore((state) => state.currentJob);
  const importedFiles = useJobStore((state) => state.importedFiles);
  const rejectedFiles = useJobStore((state) => state.rejectedFiles);
  const selectedTask = useJobStore((state) => state.selectedTask);
  const addFiles = useJobStore((state) => state.addFiles);
  const loadJob = useJobStore((state) => state.loadJob);
  const resetJob = useJobStore((state) => state.resetJob);
  const setJob = useJobStore((state) => state.setJob);
  const setSelectedTask = useJobStore((state) => state.setSelectedTask);
  const closeReviewWorkspace = useReviewStore((state) => state.closeWorkspace);
  const closeExportScreen = useExportStore((state) => state.closeScreen);
  const [hintMessage, setHintMessage] = useState<string | null>(null);
  const [isImporting, setIsImporting] = useState(false);
  const [isResumingJobId, setIsResumingJobId] = useState<string | null>(null);
  const [recentJobs, setRecentJobs] = useState<RecentJobSummary[]>([]);
  const [recentJobsError, setRecentJobsError] = useState<string | null>(null);
  const [workspaceError, setWorkspaceError] = useState<string | null>(null);

  const groupedFiles = useMemo(() => groupAcceptedFiles(importedFiles), [importedFiles]);

  useEffect(() => {
    let mounted = true;

    const loadRecentJobs = async () => {
      try {
        const response = await invoke<ListJobsResponse>("list_jobs");
        if (mounted) {
          setRecentJobs(Array.isArray(response?.jobs) ? response.jobs : []);
          setRecentJobsError(null);
        }
      } catch (error) {
        if (mounted) {
          setRecentJobsError(error instanceof Error ? error.message : String(error));
        }
      }
    };

    void loadRecentJobs();
    return () => {
      mounted = false;
    };
  }, []);

  useEffect(() => {
    if (!selectedTask) {
      const suggestion = suggestTaskFromRoles(importedFiles.map((file) => file.role));

      if (suggestion) {
        setSelectedTask(suggestion);
        setHintMessage(`Đã gợi ý task phù hợp: ${TASK_OPTIONS.find((task) => task.id === suggestion)?.title}.`);
      }
    }
  }, [importedFiles, selectedTask, setSelectedTask]);

  const syncDraftJob = async (
    acceptedFiles: ClassifiedImportResult["acceptedFiles"],
    task: AppTask | null
  ) => {
    const response = await invoke<CreateJobResponse>("create_job", {
      acceptedFiles,
      existingCreatedAt: currentJob?.createdAt ?? null,
      existingJobId: currentJob?.jobId ?? null,
      selectedTask: task,
    });

    setJob(response.job);
  };

  const handleFilesDropped = async (paths: string[]) => {
    setIsImporting(true);
    setWorkspaceError(null);

    try {
      const result = await invoke<ClassifiedImportResult>("import_assets", { filePaths: paths });
      addFiles(result);

      const nextAcceptedFiles = mergeAcceptedFiles(importedFiles, result.acceptedFiles);
      const nextTask = selectedTask ?? suggestTaskFromRoles(nextAcceptedFiles.map((file) => file.role));

      if (!selectedTask && nextTask) {
        setSelectedTask(nextTask);
      }

      if (nextAcceptedFiles.length > 0) {
        await syncDraftJob(nextAcceptedFiles, nextTask);
      }
    } catch (error) {
      setWorkspaceError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsImporting(false);
    }
  };

  const handleTaskSelection = async (task: AppTask) => {
    setSelectedTask(task);
    setHintMessage(null);

    if (importedFiles.length > 0) {
      await syncDraftJob(importedFiles, task);
    }
  };

  const handleResumeJob = async (jobId: string) => {
    setIsResumingJobId(jobId);
    setWorkspaceError(null);

    try {
      const response = await invoke<LoadJobResponse>("load_job", { jobId });
      closeReviewWorkspace();
      closeExportScreen();
      loadJob(response);
      setHintMessage(response.warningMessage);
    } catch (error) {
      setWorkspaceError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsResumingJobId(null);
    }
  };

  const handleCreateNewJob = () => {
    closeReviewWorkspace();
    closeExportScreen();
    resetJob();
    setHintMessage("Da reset workspace de tao job moi.");
    setWorkspaceError(null);
  };

  return (
    <section className="start-screen">
      <div className="start-screen__hero">
        <p className="start-screen__eyebrow">Start Flow</p>
        <h2 className="start-screen__headline">Bạn muốn làm gì?</h2>
        <p className="start-screen__copy">
          Tạo draft job mới, kéo tất cả đầu vào cần thiết vào cùng một nơi, và để hệ thống
          phân loại file trước khi sang bước mapping.
        </p>
      </div>

      <div className="start-screen__actions">
        <button type="button" className="action-button" onClick={handleCreateNewJob}>
          Tao job moi
        </button>
      </div>

      <section className="workspace-panel">
        <div className="workspace-panel__header">
          <h3>Recent Jobs</h3>
          <span>{recentJobs.length} job</span>
        </div>
        {recentJobsError ? <div className="inline-error">{recentJobsError}</div> : null}
        {recentJobs.length === 0 ? (
          <p className="segment-review-empty">Chua co job nao de resume.</p>
        ) : (
          <div className="recent-job-list">
            {recentJobs.map((job) => (
              <JobListItem
                key={job.jobId}
                job={job}
                isLoading={isResumingJobId === job.jobId}
                onResume={handleResumeJob}
              />
            ))}
          </div>
        )}
      </section>

      <div className="task-grid">
        {TASK_OPTIONS.map((task) => (
          <button
            key={task.id}
            className={`task-card${selectedTask === task.id ? " task-card--selected" : ""}`}
            type="button"
            onClick={() => void handleTaskSelection(task.id)}
          >
            <span className="task-card__title">{task.title}</span>
            <span className="task-card__description">{task.description}</span>
          </button>
        ))}
      </div>

      <DragDropZone onFilesDropped={handleFilesDropped} />

      <div className="start-screen__meta">
        {isImporting ? <span className="inline-note">Đang import và phân loại file...</span> : null}
        {hintMessage ? <span className="inline-note">{hintMessage}</span> : null}
        {currentJob ? (
          <span className="inline-note">
            Draft job hiện tại: <strong>{currentJob.jobId}</strong>
          </span>
        ) : null}
      </div>

      {workspaceError ? <div className="inline-error">{workspaceError}</div> : null}

      {rejectedFiles.length > 0 ? (
        <div className="workspace-panel">
          <h3>File bị từ chối</h3>
          <ul className="workspace-list">
            {rejectedFiles.map((file) => (
              <li key={`${file.path}-${file.reason}`} className="workspace-list__item workspace-list__item--error">
                <span>{file.fileName}</span>
                <span>{file.reason}</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      <div className="workspace-grid">
        <WorkspaceSection title="Videos" files={groupedFiles.video} />
        <WorkspaceSection title="Logo" files={groupedFiles.logo} />
        <WorkspaceSection title="Audio" files={groupedFiles.audio} />
        <WorkspaceSection title="SRT" files={groupedFiles.srt} />
      </div>
    </section>
  );
}

function WorkspaceSection({
  files,
  title,
}: {
  files: Array<{ fileName: string; path: string }>;
  title: string;
}) {
  return (
    <section className="workspace-panel">
      <div className="workspace-panel__header">
        <h3>{title}</h3>
        <span>{files.length} file</span>
      </div>
      <ul className="workspace-list">
        {files.length === 0 ? (
          <li className="workspace-list__item workspace-list__item--empty">Chưa có file nào.</li>
        ) : (
          files.map((file) => (
            <li key={file.path} className="workspace-list__item">
              <span>{file.fileName}</span>
              <code>{file.path}</code>
            </li>
          ))
        )}
      </ul>
    </section>
  );
}
