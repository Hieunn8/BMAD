import { useEffect } from "react";
import { useJobStore } from "../../store/jobStore";
import { JobSetupScreen } from "../job-review";
import { ExportScreen } from "../export-reporting";
import { PresetSelectionScreen } from "../preset-management";
import { ProcessingQueueScreen } from "../processing-queue";
import { SegmentReviewScreen } from "../segment-review";
import { StartScreen } from "../start-flow";
import { useAppShellStore } from "../../store/app-shell-store";
import { useReviewStore } from "../../store/reviewStore";
import { useExportStore } from "../../store/exportStore";

export function AppShell() {
  const ffmpegPath = useAppShellStore((state) => state.ffmpegPath);
  const status = useAppShellStore((state) => state.status);
  const error = useAppShellStore((state) => state.error);
  const loadFfmpegPath = useAppShellStore((state) => state.loadFfmpegPath);
  const currentJob = useJobStore((state) => state.currentJob);
  const preset = useJobStore((state) => state.preset);
  const isProcessing = currentJob?.status === "Processing";
  const isReviewOpen = useReviewStore((state) => state.isOpen);
  const closeWorkspace = useReviewStore((state) => state.closeWorkspace);
  const openWorkspace = useReviewStore((state) => state.openWorkspace);
  const isExportOpen = useExportStore((state) => state.isOpen);
  const openExportScreen = useExportStore((state) => state.openScreen);
  const closeExportScreen = useExportStore((state) => state.closeScreen);
  const isExportFlowStatus = ["ReadyToExport", "Exporting", "Exported", "ExportedWithFailures", "ExportFailed"].includes(
    currentJob?.status ?? ""
  );

  useEffect(() => {
    void loadFfmpegPath();
  }, [loadFfmpegPath]);

  useEffect(() => {
    if (currentJob?.status === "ReviewPending") {
      closeExportScreen();
      openWorkspace();
      return;
    }
    if (isExportFlowStatus && !isReviewOpen) {
      openExportScreen();
      return;
    }
    if (currentJob?.status !== "ReviewPending" && isReviewOpen) {
      closeWorkspace();
    }
    if (!isExportFlowStatus && isExportOpen) {
      closeExportScreen();
    }
  }, [closeExportScreen, closeWorkspace, currentJob?.status, isExportFlowStatus, isExportOpen, isReviewOpen, openExportScreen, openWorkspace]);

  return (
    <main className="app-shell">
      <section className="app-shell__frame">
        <header className="app-shell__hero">
          <p className="app-shell__eyebrow">Story 1.1 foundation</p>
          <h1 className="app-shell__title">Desktop Video Rebranding App</h1>
          <p className="app-shell__summary">
            Start flow đã sẵn sàng để tạo draft job mới, phân loại file từ Rust side, và
            persist cấu hình job ngay khi import đầu vào.
          </p>
        </header>

        <div className="app-shell__grid app-shell__grid--story">
          <div className="app-shell__flow-stack">
            <StartScreen />
            {currentJob && !isProcessing && !isReviewOpen && !isExportOpen ? <PresetSelectionScreen /> : null}
            {currentJob && preset && !isProcessing && !isReviewOpen && !isExportOpen ? <JobSetupScreen /> : null}
            {currentJob && isProcessing ? <ProcessingQueueScreen /> : null}
            {currentJob && isReviewOpen ? <SegmentReviewScreen /> : null}
            {currentJob && isExportOpen ? <ExportScreen /> : null}
          </div>

          <aside className="app-shell__panel">
            <h3>FFmpeg resource</h3>
            <div className="app-shell__status">
              <div className="app-shell__status-row">
                <span>Trạng thái</span>
                <strong>{status}</strong>
              </div>
              {ffmpegPath ? <code className="app-shell__path">{ffmpegPath}</code> : null}
              {error ? <div className="app-shell__error">{error}</div> : null}
            </div>
          </aside>
        </div>
      </section>
    </main>
  );
}
