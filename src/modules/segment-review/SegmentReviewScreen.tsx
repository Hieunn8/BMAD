import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { useJobStore } from "../../store/jobStore";
import { useExportStore } from "../../store/exportStore";
import { useReviewStore } from "../../store/reviewStore";
import type {
  ReviewContextResponse,
  ReviewGatingResponse,
  ReviewSegment,
  VideoReadyToExportEvent,
  VideoReviewContextResponse,
} from "../start-flow/types";
import { PreviewWorkspace } from "./PreviewWorkspace";
import { QuickFixPanel } from "./QuickFixPanel";
import { ReviewGatingPanel } from "./ReviewGatingPanel";
import { SegmentList, filterAndSortSegments } from "./SegmentList";
import { VideoList } from "./VideoList";

export function SegmentReviewScreen() {
  const currentJob = useJobStore((state) => state.currentJob);
  const setJob = useJobStore((state) => state.setJob);
  const openExportScreen = useExportStore((state) => state.openScreen);
  const {
    closeWorkspace,
    filters,
    gatingByVideo,
    hydrateContext,
    hydrateVideoContext,
    isLoading,
    logoOverlayVisible,
    pendingLogoFix,
    pendingSubtitleFix,
    previewMode,
    previewPaths,
    resetFilters,
    selectedSegmentId,
    selectedSegmentIds,
    selectedVideoId,
    setFilter,
    setGatingResult,
    setLoading,
    setLogoOverlayVisible,
    setPendingLogoFix,
    setPendingSubtitleFix,
    setPreviewMode,
    setSelectedSegmentId,
    setSelectedVideoId,
    setShowAllVideos,
    setValidationPreviewPath,
    showAllVideos,
    sourcePaths,
    toggleSegmentSelection,
    updateSegment,
    updateSegments,
    updateVideoSummaryStatus,
    validationPreviewPath,
    videoList,
    videoStates,
    segmentsByVideo,
  } = useReviewStore();

  useEffect(() => {
    if (!currentJob) {
      return;
    }

    let mounted = true;

    const loadContext = async () => {
      setLoading(true);
      try {
        const response = await invoke<ReviewContextResponse>("get_review_context", {
          jobId: currentJob.jobId,
          selectedVideoId,
          showAllVideos,
        });
        if (mounted) {
          hydrateContext(response.context);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    void loadContext();
    return () => {
      mounted = false;
    };
  }, [currentJob?.jobId, hydrateContext, selectedVideoId, setLoading, showAllVideos]);

  useEffect(() => {
    if (!currentJob || !selectedVideoId) {
      return;
    }

    let mounted = true;

    const loadVideo = async () => {
      const response = await invoke<VideoReviewContextResponse>("get_video_preview", {
        jobId: currentJob.jobId,
        videoId: selectedVideoId,
      });

      if (mounted) {
        hydrateVideoContext(response.context);
      }
    };

    const loadGating = async () => {
      const response = await invoke<ReviewGatingResponse>("check_video_review_gating", {
        jobId: currentJob.jobId,
        videoId: selectedVideoId,
      });

      if (mounted) {
        setGatingResult(selectedVideoId, response.result);
      }
    };

    if (!segmentsByVideo[selectedVideoId] || !previewPaths[selectedVideoId]) {
      void loadVideo();
    }
    void loadGating();

    return () => {
      mounted = false;
    };
  }, [
    currentJob?.jobId,
    hydrateVideoContext,
    previewPaths,
    segmentsByVideo,
    selectedVideoId,
    setGatingResult,
  ]);

  useEffect(() => {
    const unlisten = listen<VideoReadyToExportEvent>("videoReadyToExport", (event) => {
      updateVideoSummaryStatus(event.payload.videoId, "ReadyToExport");
    });

    return () => {
      void unlisten.then((dispose) => dispose());
    };
  }, [updateVideoSummaryStatus]);

  if (!currentJob) {
    return null;
  }

  const segments = selectedVideoId ? segmentsByVideo[selectedVideoId] ?? [] : [];
  const filteredSegments = filterAndSortSegments(segments, filters);
  const selectedSegment =
    segments.find((segment) => segment.id === selectedSegmentId) ??
    filteredSegments.find((segment) => segment.id === selectedSegmentId) ??
    null;
  const gating = selectedVideoId ? gatingByVideo[selectedVideoId] ?? null : null;
  const blockerCount = gating?.blockers.length ?? 0;

  const handleVideoMarkedReady = () => {
    if (!selectedVideoId || !currentJob) {
      return;
    }

    updateVideoSummaryStatus(selectedVideoId, "ReadyToExport");
    const nextVideoItems = currentJob.videoItems.map((video) =>
      video.videoId === selectedVideoId ? { ...video, status: "ReadyToExport" } : video
    );
    const nextStatus = nextVideoItems.every((video) => video.status === "ReadyToExport" || video.status === "Failed")
      ? "ReadyToExport"
      : currentJob.status;
    setJob({
      ...currentJob,
      status: nextStatus,
      videoItems: nextVideoItems,
    });
  };

  const handleOpenExport = () => {
    closeWorkspace();
    openExportScreen();
  };

  return (
    <section className="segment-review-screen">
      <div className="segment-review-screen__hero">
        <div>
          <p className="start-screen__eyebrow">Review Workspace</p>
          <h2 className="start-screen__headline">Xu ly cac video co exception truoc khi export</h2>
        </div>
        <div className="segment-review-screen__meta">
          <span className="inline-note">Job: {currentJob.jobId}</span>
          <span className="inline-note">{videoList.length} video trong review workspace</span>
          {isLoading ? <span className="inline-note">Dang tai review context...</span> : null}
          {currentJob.status === "ReadyToExport" ? (
            <button type="button" className="action-button action-button--small" onClick={handleOpenExport}>
              Mo export screen
            </button>
          ) : null}
          <button type="button" className="action-button action-button--small" onClick={closeWorkspace}>
            Dong workspace
          </button>
        </div>
      </div>

      <div className="segment-review-screen__grid">
        <div className="segment-review-screen__column">
          <VideoList
            showAllVideos={showAllVideos}
            selectedVideoId={selectedVideoId}
            videoList={videoList}
            onToggleShowAll={setShowAllVideos}
            onSelectVideo={setSelectedVideoId}
          />
          <SegmentList
            filters={filters}
            segments={segments}
            selectedSegmentId={selectedSegmentId}
            selectedSegmentIds={selectedSegmentIds}
            onFilterChange={setFilter}
            onResetFilters={resetFilters}
            onActivateSegment={setSelectedSegmentId}
            onToggleSegment={toggleSegmentSelection}
          />
        </div>

        <PreviewWorkspace
          jobId={currentJob.jobId}
          previewMode={previewMode}
          previewPath={selectedVideoId ? previewPaths[selectedVideoId] ?? null : null}
          sourcePath={selectedVideoId ? sourcePaths[selectedVideoId] ?? null : null}
          validationPreviewPath={validationPreviewPath}
          selectedSegment={selectedSegment as ReviewSegment | null}
          segments={filteredSegments}
          videoStatus={selectedVideoId ? videoStates[selectedVideoId] ?? null : null}
          logoOverlayVisible={logoOverlayVisible}
          pendingLogoFix={pendingLogoFix}
          pendingSubtitleFix={pendingSubtitleFix}
          onPreviewModeChange={setPreviewMode}
          onToggleLogoOverlay={setLogoOverlayVisible}
          onPendingLogoFixChange={setPendingLogoFix}
          onPendingSubtitleFixChange={setPendingSubtitleFix}
          onValidationPreviewReady={setValidationPreviewPath}
        />

        <div className="segment-review-screen__column">
          <QuickFixPanel
            jobId={currentJob.jobId}
            selectedSegment={selectedSegment}
            selectedSegmentIds={selectedSegmentIds}
            pendingLogoFix={pendingLogoFix}
            pendingSubtitleFix={pendingSubtitleFix}
            onPendingLogoFixChange={setPendingLogoFix}
            onPendingSubtitleFixChange={setPendingSubtitleFix}
            onSegmentsUpdated={(updated) => {
              if (selectedVideoId) {
                updateSegments(selectedVideoId, updated);
              }
            }}
            onSegmentUpdated={(segment) => {
              if (selectedVideoId) {
                updateSegment(selectedVideoId, segment);
              }
            }}
          />
          <ReviewGatingPanel
            jobId={currentJob.jobId}
            videoId={selectedVideoId}
            status={selectedVideoId ? videoStates[selectedVideoId]?.status ?? null : null}
            blockerCount={blockerCount}
            onGatingLoaded={(canProceed, blockers) => {
              if (selectedVideoId) {
                setGatingResult(selectedVideoId, { canProceed, blockers });
              }
            }}
            onVideoMarkedReady={handleVideoMarkedReady}
          />
          {gating?.blockers.length ? (
            <div className="segment-review-panel">
              <div className="segment-review-panel__header">
                <div>
                  <p className="start-screen__eyebrow">Blockers</p>
                  <h3>High Risk chua giai quyet</h3>
                </div>
              </div>
              <div className="review-video-list">
                {gating.blockers.map((blocker) => (
                  <div key={blocker.segmentId} className="segment-review-banner">
                    <strong>{blocker.timeRange}</strong> • {blocker.issueType}
                  </div>
                ))}
              </div>
            </div>
          ) : null}
        </div>
      </div>
    </section>
  );
}
