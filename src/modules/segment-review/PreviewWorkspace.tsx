import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { forwardRef, useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties, MouseEvent as ReactMouseEvent } from "react";
import type {
  FramePreviewResponse,
  LogoFix,
  ReviewSegment,
  SubtitleFix,
  VideoProcessingState,
} from "../start-flow/types";
import type { PreviewMode } from "../../store/reviewStore";

type PreviewWorkspaceProps = {
  jobId: string;
  previewMode: PreviewMode;
  previewPath: string | null;
  sourcePath: string | null;
  validationPreviewPath: string | null;
  selectedSegment: ReviewSegment | null;
  segments: ReviewSegment[];
  videoStatus: VideoProcessingState | null;
  logoOverlayVisible: boolean;
  pendingLogoFix: LogoFix | null;
  pendingSubtitleFix: SubtitleFix | null;
  onPreviewModeChange: (mode: PreviewMode) => void;
  onToggleLogoOverlay: (value: boolean) => void;
  onPendingLogoFixChange: (value: LogoFix | null) => void;
  onPendingSubtitleFixChange: (value: SubtitleFix | null) => void;
  onValidationPreviewReady: (path: string | null) => void;
};

type OverlayBox = {
  x: number;
  y: number;
  width: number;
  height: number;
};

type DragState = {
  mode: "move" | "resize-se" | "move-subtitle-position";
  startX: number;
  startY: number;
  box: OverlayBox;
  scaleX: number;
  scaleY: number;
};

export function PreviewWorkspace({
  jobId,
  previewMode,
  previewPath,
  sourcePath,
  validationPreviewPath,
  selectedSegment,
  segments,
  videoStatus,
  logoOverlayVisible,
  pendingLogoFix,
  pendingSubtitleFix,
  onPreviewModeChange,
  onToggleLogoOverlay,
  onPendingLogoFixChange,
  onPendingSubtitleFixChange,
  onValidationPreviewReady,
}: PreviewWorkspaceProps) {
  const beforeRef = useRef<HTMLVideoElement | null>(null);
  const afterRef = useRef<HTMLVideoElement | null>(null);
  const activeVideoRef = useRef<HTMLVideoElement | null>(null);
  const syncGuardRef = useRef(false);
  const dragStateRef = useRef<DragState | null>(null);
  const [isLoadingValidatedPreview, setIsLoadingValidatedPreview] = useState(false);
  const [overlayRenderVersion, setOverlayRenderVersion] = useState(0);

  const activeOverlayBox = useMemo<OverlayBox | null>(() => {
    if (selectedSegment?.source === "logo") {
      return pendingLogoFix;
    }

    if (selectedSegment?.source === "subtitle" && pendingSubtitleFix?.oldRegion) {
      return pendingSubtitleFix.oldRegion;
    }

    return null;
  }, [pendingLogoFix, pendingSubtitleFix, selectedSegment?.source]);

  const resolvedAfterPath = validationPreviewPath ?? previewPath;
  const loopStart = (selectedSegment?.startMs ?? 0) / 1000;
  const loopEnd = ((selectedSegment?.endMs ?? selectedSegment?.startMs ?? 3000) + 250) / 1000;
  const showOverlay = Boolean(
    selectedSegment &&
      activeOverlayBox &&
      (selectedSegment.source !== "logo" || logoOverlayVisible)
  );
  const subtitlePositionStyle = computeSubtitlePositionStyle(
    afterRef.current,
    pendingSubtitleFix?.newPosition ?? null,
    overlayRenderVersion
  );

  useEffect(() => {
    const updateMetrics = () => setOverlayRenderVersion((value) => value + 1);
    updateMetrics();

    const video = afterRef.current;
    if (!video || typeof ResizeObserver === "undefined") {
      return;
    }

    const observer = new ResizeObserver(updateMetrics);
    observer.observe(video);
    window.addEventListener("resize", updateMetrics);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", updateMetrics);
    };
  }, [resolvedAfterPath, selectedSegment?.id]);

  useEffect(() => {
    const syncVideoToSegment = (video: HTMLVideoElement | null) => {
      if (!video || !selectedSegment) {
        return;
      }

      video.currentTime = selectedSegment.startMs / 1000;
      video.pause();
    };

    syncVideoToSegment(beforeRef.current);
    syncVideoToSegment(afterRef.current);
    activeVideoRef.current = previewMode === "before" ? beforeRef.current : afterRef.current;
  }, [previewMode, selectedSegment?.id, selectedSegment?.startMs]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const activeVideo = activeVideoRef.current;
      if (!activeVideo) {
        return;
      }

      if (event.code === "Space") {
        event.preventDefault();
        if (activeVideo.paused) {
          void activeVideo.play();
        } else {
          activeVideo.pause();
        }
      }

      if (event.key === "ArrowLeft") {
        activeVideo.currentTime = Math.max(0, activeVideo.currentTime - 1);
      }

      if (event.key === "ArrowRight") {
        activeVideo.currentTime = activeVideo.currentTime + 1;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const syncPeer = (source: HTMLVideoElement, peer?: HTMLVideoElement | null) => {
    activeVideoRef.current = source;
    if (!selectedSegment) {
      return;
    }

    if (source.currentTime >= loopEnd) {
      source.currentTime = loopStart;
      if (peer) {
        peer.currentTime = loopStart;
      }
      if (!source.paused) {
        void source.play();
      }
      if (peer && !peer.paused) {
        void peer.play();
      }
      return;
    }

    if (peer && Math.abs(peer.currentTime - source.currentTime) > 0.08) {
      syncGuardRef.current = true;
      peer.currentTime = source.currentTime;
      syncGuardRef.current = false;
    }
  };

  const syncPlaybackState = (source: HTMLVideoElement, peer?: HTMLVideoElement | null, action?: "play" | "pause") => {
    activeVideoRef.current = source;
    if (!peer || syncGuardRef.current) {
      return;
    }

    syncGuardRef.current = true;
    peer.currentTime = source.currentTime;
    if (action === "play") {
      void peer.play().catch(() => undefined);
    } else if (action === "pause") {
      peer.pause();
    }
    syncGuardRef.current = false;
  };

  const loadValidatedPreview = async () => {
    if (!selectedSegment) {
      return;
    }

    setIsLoadingValidatedPreview(true);
    try {
      const response = await invoke<FramePreviewResponse>("get_frame_preview", {
        jobId,
        segmentId: selectedSegment.id,
        timeSeconds: selectedSegment.startMs / 1000,
        pendingLogoFix,
        pendingSubtitleFix,
      });
      onValidationPreviewReady(response.result.cachePath);
    } finally {
      setIsLoadingValidatedPreview(false);
    }
  };

  const startDrag = (
    event: ReactMouseEvent<HTMLElement>,
    mode: "move" | "resize-se" | "move-subtitle-position"
  ) => {
    if (!afterRef.current) {
      return;
    }

    const video = afterRef.current;
    const scaleX = video.videoWidth > 0 && video.clientWidth > 0 ? video.videoWidth / video.clientWidth : 1;
    const scaleY = video.videoHeight > 0 && video.clientHeight > 0 ? video.videoHeight / video.clientHeight : 1;

    event.preventDefault();
    event.stopPropagation();
    dragStateRef.current = {
      mode,
      startX: event.clientX,
      startY: event.clientY,
      box:
        mode === "move-subtitle-position" && pendingSubtitleFix?.newPosition
          ? {
              x: pendingSubtitleFix.newPosition.x,
              y: pendingSubtitleFix.newPosition.y,
              width: 0,
              height: 0,
            }
          : { ...(activeOverlayBox ?? { x: 0, y: 0, width: 0, height: 0 }) },
      scaleX,
      scaleY,
    };
  };

  useEffect(() => {
    const handleMove = (event: MouseEvent) => {
      const dragState = dragStateRef.current;
      if (!dragState) {
        return;
      }

      const deltaX = Math.round((event.clientX - dragState.startX) * dragState.scaleX);
      const deltaY = Math.round((event.clientY - dragState.startY) * dragState.scaleY);

      if (selectedSegment?.source === "logo") {
        const nextValue =
          dragState.mode === "move"
            ? {
                ...dragState.box,
                x: Math.max(0, dragState.box.x + deltaX),
                y: Math.max(0, dragState.box.y + deltaY),
              }
            : {
                ...dragState.box,
                width: Math.max(16, dragState.box.width + deltaX),
                height: Math.max(16, dragState.box.height + deltaY),
              };
        onPendingLogoFixChange(nextValue);
        return;
      }

      if (selectedSegment?.source === "subtitle" && pendingSubtitleFix?.oldRegion) {
        if (dragState.mode === "move-subtitle-position") {
          onPendingSubtitleFixChange({
            ...pendingSubtitleFix,
            newPosition: {
              ...(pendingSubtitleFix.newPosition ?? { x: dragState.box.x, y: dragState.box.y }),
              x: Math.max(0, dragState.box.x + deltaX),
              y: Math.max(0, dragState.box.y + deltaY),
            },
          });
          return;
        }

        const nextRegion =
          dragState.mode === "move"
            ? {
                ...dragState.box,
                x: Math.max(0, dragState.box.x + deltaX),
                y: Math.max(0, dragState.box.y + deltaY),
              }
            : {
                ...dragState.box,
                width: Math.max(16, dragState.box.width + deltaX),
                height: Math.max(16, dragState.box.height + deltaY),
              };

        onPendingSubtitleFixChange({
          ...pendingSubtitleFix,
          oldRegion: {
            ...pendingSubtitleFix.oldRegion,
            ...nextRegion,
          },
        });
      }
    };

    const handleUp = () => {
      dragStateRef.current = null;
    };

    window.addEventListener("mousemove", handleMove);
    window.addEventListener("mouseup", handleUp);
    return () => {
      window.removeEventListener("mousemove", handleMove);
      window.removeEventListener("mouseup", handleUp);
    };
  }, [onPendingLogoFixChange, onPendingSubtitleFixChange, pendingSubtitleFix, selectedSegment?.source]);

  const overlayMetrics = computeOverlayMetrics(afterRef.current, activeOverlayBox, overlayRenderVersion);

  return (
    <div className="segment-review-panel segment-review-panel--preview">
      <div className="segment-review-panel__header">
        <div>
          <p className="start-screen__eyebrow">Preview Workspace</p>
          <h3>Xem context cua segment</h3>
        </div>
        <div className="preview-workspace__toolbar">
          <div className="preview-workspace__mode-toggle">
            {(["before", "after", "compare"] as PreviewMode[]).map((mode) => (
              <button
                key={mode}
                type="button"
                className={`action-button action-button--small${previewMode === mode ? " action-button--primary" : ""}`}
                onClick={() => onPreviewModeChange(mode)}
              >
                {mode === "before" ? "Truoc" : mode === "after" ? "Sau" : "So sanh"}
              </button>
            ))}
          </div>
          {selectedSegment?.source === "logo" ? (
            <label className="segment-review-toggle">
              <input
                type="checkbox"
                checked={logoOverlayVisible}
                onChange={(event) => onToggleLogoOverlay(event.target.checked)}
              />
              <span>Hien logo overlay</span>
            </label>
          ) : null}
          <button
            type="button"
            className="action-button action-button--small"
            disabled={!selectedSegment || isLoadingValidatedPreview}
            onClick={() => void loadValidatedPreview()}
          >
            {isLoadingValidatedPreview ? "Dang tao clip..." : "Validate after clip"}
          </button>
        </div>
      </div>

      <div className={`preview-workspace__stage preview-workspace__stage--${previewMode}`}>
        {(previewMode === "before" || previewMode === "compare") && sourcePath ? (
          <VideoPane
            label="Before"
            ref={beforeRef}
            path={sourcePath}
            showOverlay={false}
            overlayBox={null}
            overlayStyle={null}
            subtitlePositionStyle={null}
            onLoadedMetadata={() => setOverlayRenderVersion((value) => value + 1)}
            onTimeUpdate={(video) => syncPeer(video, previewMode === "compare" ? afterRef.current : null)}
            onPlay={(video) => syncPlaybackState(video, previewMode === "compare" ? afterRef.current : null, "play")}
            onPause={(video) => syncPlaybackState(video, previewMode === "compare" ? afterRef.current : null, "pause")}
          />
        ) : null}
        {(previewMode === "after" || previewMode === "compare") && resolvedAfterPath ? (
          <VideoPane
            label={validationPreviewPath ? "After (validated)" : "After"}
            ref={afterRef}
            path={resolvedAfterPath}
            showOverlay={showOverlay}
            overlayBox={showOverlay ? activeOverlayBox : null}
            overlayStyle={overlayMetrics}
            overlayTone={selectedSegment?.source === "subtitle" ? pendingSubtitleFix?.oldRegion?.mode ?? "blur" : "logo"}
            onOverlayMoveStart={(event) => startDrag(event, "move")}
            onOverlayResizeStart={(event) => startDrag(event, "resize-se")}
            subtitlePositionStyle={
              selectedSegment?.source === "subtitle" ? subtitlePositionStyle : null
            }
            onSubtitlePositionMoveStart={(event) => startDrag(event, "move-subtitle-position")}
            onLoadedMetadata={() => setOverlayRenderVersion((value) => value + 1)}
            onTimeUpdate={(video) => syncPeer(video, previewMode === "compare" ? beforeRef.current : null)}
            onPlay={(video) => syncPlaybackState(video, previewMode === "compare" ? beforeRef.current : null, "play")}
            onPause={(video) => syncPlaybackState(video, previewMode === "compare" ? beforeRef.current : null, "pause")}
          />
        ) : null}
      </div>

      <div className="preview-workspace__details">
        <div className="workspace-panel">
          <strong>Status</strong>
          <p>
            {videoStatus?.status ?? "Unknown"} • step {videoStatus?.currentStep ?? "n/a"}
          </p>
        </div>
        <div className="workspace-panel">
          <strong>Selected Segment</strong>
          {selectedSegment ? (
            <p>
              {selectedSegment.issueType} • {selectedSegment.riskLevel} • {selectedSegment.message}
            </p>
          ) : (
            <p>Chon mot segment de xem chi tiet.</p>
          )}
        </div>
      </div>

      <div className="preview-workspace__timeline">
        {segments.map((segment) => (
          <div
            key={segment.id}
            className={`preview-workspace__timeline-mark preview-workspace__timeline-mark--${segment.riskLevel.toLowerCase()}`}
            style={{
              left: `${timelinePosition(segment.startMs, segments)}%`,
            }}
            title={`${segment.issueType} - ${segment.riskLevel}`}
          />
        ))}
      </div>
    </div>
  );
}

type VideoPaneProps = {
  label: string;
  path: string;
  showOverlay: boolean;
  overlayBox: OverlayBox | null;
  overlayStyle: CSSProperties | null;
  overlayTone?: string;
  onTimeUpdate: (video: HTMLVideoElement) => void;
  onPlay: (video: HTMLVideoElement) => void;
  onPause: (video: HTMLVideoElement) => void;
  onLoadedMetadata: () => void;
  subtitlePositionStyle: CSSProperties | null;
  onOverlayMoveStart?: (event: ReactMouseEvent<HTMLElement>) => void;
  onOverlayResizeStart?: (event: ReactMouseEvent<HTMLElement>) => void;
  onSubtitlePositionMoveStart?: (event: ReactMouseEvent<HTMLElement>) => void;
};

const VideoPane = forwardRef<HTMLVideoElement, VideoPaneProps>(function VideoPane(
  {
    label,
    path,
    showOverlay,
    overlayBox,
    overlayStyle,
    overlayTone,
    onTimeUpdate,
    onPlay,
    onPause,
    onLoadedMetadata,
    subtitlePositionStyle,
    onOverlayMoveStart,
    onOverlayResizeStart,
    onSubtitlePositionMoveStart,
  },
  ref
) {
  return (
    <div className="preview-workspace__pane">
      <span className="inline-note">{label}</span>
      <div className="preview-workspace__video-shell">
        <video
          ref={ref}
          className="preview-workspace__video"
          controls
          src={convertFileSrc(path)}
          onLoadedMetadata={onLoadedMetadata}
          onTimeUpdate={(event) => onTimeUpdate(event.currentTarget)}
          onPlay={(event) => onPlay(event.currentTarget)}
          onPause={(event) => onPause(event.currentTarget)}
        />
        {showOverlay && overlayBox && overlayStyle ? (
          <div
            className={`preview-overlay preview-overlay--${overlayTone ?? "logo"}`}
            style={overlayStyle}
            onMouseDown={onOverlayMoveStart}
          >
            <span className="preview-overlay__label">
              {overlayTone === "logo" ? "Logo" : `Subtitle ${overlayTone ?? "blur"}`}
            </span>
            <div className="preview-overlay__handle" onMouseDown={onOverlayResizeStart} />
          </div>
        ) : null}
        {subtitlePositionStyle ? (
          <button
            type="button"
            className="preview-subtitle-anchor"
            style={subtitlePositionStyle}
            onMouseDown={onSubtitlePositionMoveStart}
            title="Keo de doi vi tri subtitle moi"
          >
            T
          </button>
        ) : null}
      </div>
    </div>
  );
});

function computeOverlayMetrics(
  video: HTMLVideoElement | null,
  overlayBox: OverlayBox | null,
  _version: number
): CSSProperties | null {
  if (!video || !overlayBox || video.videoWidth === 0 || video.videoHeight === 0) {
    return null;
  }

  const scaleX = video.clientWidth / video.videoWidth;
  const scaleY = video.clientHeight / video.videoHeight;

  return {
    left: `${overlayBox.x * scaleX}px`,
    top: `${overlayBox.y * scaleY}px`,
    width: `${Math.max(16, overlayBox.width * scaleX)}px`,
    height: `${Math.max(16, overlayBox.height * scaleY)}px`,
  };
}

function computeSubtitlePositionStyle(
  video: HTMLVideoElement | null,
  position: { x: number; y: number } | null,
  _version: number
): CSSProperties | null {
  if (!video || !position || video.videoWidth === 0 || video.videoHeight === 0) {
    return null;
  }

  const scaleX = video.clientWidth / video.videoWidth;
  const scaleY = video.clientHeight / video.videoHeight;

  return {
    left: `${position.x * scaleX}px`,
    top: `${position.y * scaleY}px`,
  };
}

function timelinePosition(startMs: number, segments: ReviewSegment[]) {
  const max = segments.reduce((acc, item) => Math.max(acc, item.endMs ?? item.startMs), 1);
  return Math.min(96, (startMs / Math.max(max, 1)) * 96);
}
