import { create } from "zustand";
import type {
  LogoFix,
  ReviewContext,
  ReviewGatingResult,
  ReviewSegment,
  ReviewVideoContext,
  ReviewVideoSummary,
  SubtitleFix,
  VideoProcessingState,
} from "../modules/start-flow/types";

export type ReviewFilterState = {
  riskLevel: "All" | "High" | "Medium" | "Low";
  issueType: "All" | "LogoPosition" | "SubtitleRegion" | "SubtitleStyle";
  status: "All" | "Unreviewed" | "Accepted" | "Modified" | "Blocked";
};

export type PreviewMode = "before" | "after" | "compare";

const DEFAULT_FILTERS: ReviewFilterState = {
  riskLevel: "All",
  issueType: "All",
  status: "All",
};

type ReviewState = {
  isOpen: boolean;
  isLoading: boolean;
  showAllVideos: boolean;
  selectedVideoId: string | null;
  selectedSegmentId: string | null;
  selectedSegmentIds: string[];
  lastRangeAnchorId: string | null;
  videoList: ReviewVideoSummary[];
  segmentsByVideo: Record<string, ReviewSegment[]>;
  videoStates: Record<string, VideoProcessingState | null>;
  previewPaths: Record<string, string>;
  sourcePaths: Record<string, string>;
  filters: ReviewFilterState;
  previewMode: PreviewMode;
  logoOverlayVisible: boolean;
  pendingLogoFix: LogoFix | null;
  pendingSubtitleFix: SubtitleFix | null;
  gatingByVideo: Record<string, ReviewGatingResult | null>;
  validationPreviewPath: string | null;
  openWorkspace: () => void;
  closeWorkspace: () => void;
  setLoading: (isLoading: boolean) => void;
  setShowAllVideos: (value: boolean) => void;
  setSelectedVideoId: (videoId: string | null) => void;
  hydrateContext: (context: ReviewContext) => void;
  hydrateVideoContext: (context: ReviewVideoContext) => void;
  setSelectedSegmentId: (segmentId: string | null) => void;
  toggleSegmentSelection: (segmentId: string, orderedIds: string[], withRange?: boolean) => void;
  setFilter: <K extends keyof ReviewFilterState>(key: K, value: ReviewFilterState[K]) => void;
  resetFilters: () => void;
  setPreviewMode: (mode: PreviewMode) => void;
  setLogoOverlayVisible: (value: boolean) => void;
  setPendingLogoFix: (value: LogoFix | null) => void;
  setPendingSubtitleFix: (value: SubtitleFix | null) => void;
  syncPendingFixesFromSelection: () => void;
  updateSegments: (videoId: string, updatedSegments: ReviewSegment[]) => void;
  updateSegment: (videoId: string, updatedSegment: ReviewSegment) => void;
  setGatingResult: (videoId: string, result: ReviewGatingResult | null) => void;
  setValidationPreviewPath: (path: string | null) => void;
  updateVideoSummaryStatus: (videoId: string, status: string) => void;
};

function toLogoFix(segment: ReviewSegment | null): LogoFix | null {
  if (!segment?.boundingBox) {
    return null;
  }

  return (
    segment.quickFixState.logoFix ?? {
      x: segment.boundingBox.x,
      y: segment.boundingBox.y,
      width: segment.boundingBox.width,
      height: segment.boundingBox.height,
    }
  );
}

function toSubtitleFix(segment: ReviewSegment | null): SubtitleFix | null {
  if (!segment?.boundingBox) {
    return null;
  }

  return (
    segment.quickFixState.subtitleFix ?? {
      oldRegion: {
        x: segment.boundingBox.x,
        y: segment.boundingBox.y,
        width: segment.boundingBox.width,
        height: segment.boundingBox.height,
        mode: "blur",
      },
      newPosition: {
        x: segment.boundingBox.x,
        y: segment.boundingBox.y,
      },
      newScale: 1,
      stylePreset: "default",
    }
  );
}

function syncPendingFromSelectedSegment(state: ReviewState) {
  const selectedVideoId = state.selectedVideoId;
  const selectedSegment = selectedVideoId
    ? state.segmentsByVideo[selectedVideoId]?.find((segment) => segment.id === state.selectedSegmentId) ?? null
    : null;

  return {
    pendingLogoFix: selectedSegment?.source === "logo" ? toLogoFix(selectedSegment) : null,
    pendingSubtitleFix: selectedSegment?.source === "subtitle" ? toSubtitleFix(selectedSegment) : null,
    validationPreviewPath: null,
  };
}

export const useReviewStore = create<ReviewState>((set) => ({
  isOpen: false,
  isLoading: false,
  showAllVideos: false,
  selectedVideoId: null,
  selectedSegmentId: null,
  selectedSegmentIds: [],
  lastRangeAnchorId: null,
  videoList: [],
  segmentsByVideo: {},
  videoStates: {},
  previewPaths: {},
  sourcePaths: {},
  filters: DEFAULT_FILTERS,
  previewMode: "after",
  logoOverlayVisible: true,
  pendingLogoFix: null,
  pendingSubtitleFix: null,
  gatingByVideo: {},
  validationPreviewPath: null,
  openWorkspace: () => set({ isOpen: true }),
  closeWorkspace: () =>
    set({
      isOpen: false,
      isLoading: false,
      showAllVideos: false,
      selectedVideoId: null,
      selectedSegmentId: null,
      selectedSegmentIds: [],
      lastRangeAnchorId: null,
      videoList: [],
      segmentsByVideo: {},
      videoStates: {},
      previewPaths: {},
      sourcePaths: {},
      filters: DEFAULT_FILTERS,
      previewMode: "after",
      logoOverlayVisible: true,
      pendingLogoFix: null,
      pendingSubtitleFix: null,
      gatingByVideo: {},
      validationPreviewPath: null,
    }),
  setLoading: (isLoading) => set({ isLoading }),
  setShowAllVideos: (value) => set({ showAllVideos: value }),
  setSelectedVideoId: (videoId) =>
    set((state) => {
      const nextState: Partial<ReviewState> = {
        selectedVideoId: videoId,
        selectedSegmentId: null,
        selectedSegmentIds: [],
        lastRangeAnchorId: null,
        validationPreviewPath: null,
      };
      return {
        ...nextState,
        ...syncPendingFromSelectedSegment({ ...state, ...nextState } as ReviewState),
      };
    }),
  hydrateContext: (context) =>
    set((state) => {
      const nextSegments = { ...state.segmentsByVideo };
      const nextVideoStates = { ...state.videoStates };
      const nextPreviewPaths = { ...state.previewPaths };
      const nextSourcePaths = { ...state.sourcePaths };

      if (context.selectedVideo) {
        nextSegments[context.selectedVideo.videoId] = context.selectedVideo.segments;
        nextVideoStates[context.selectedVideo.videoId] = context.selectedVideo.videoStatus;
        nextPreviewPaths[context.selectedVideo.videoId] = context.selectedVideo.previewPath;
        nextSourcePaths[context.selectedVideo.videoId] = context.selectedVideo.sourcePath;
      }

      const baseState = {
        ...state,
        videoList: context.videoList,
        selectedVideoId: context.selectedVideoId,
        segmentsByVideo: nextSegments,
        videoStates: nextVideoStates,
        previewPaths: nextPreviewPaths,
        sourcePaths: nextSourcePaths,
      } as ReviewState;

      return {
        videoList: context.videoList,
        selectedVideoId: context.selectedVideoId,
        segmentsByVideo: nextSegments,
        videoStates: nextVideoStates,
        previewPaths: nextPreviewPaths,
        sourcePaths: nextSourcePaths,
        ...syncPendingFromSelectedSegment(baseState),
      };
    }),
  hydrateVideoContext: (context) =>
    set((state) => {
      const baseState = {
        ...state,
        segmentsByVideo: {
          ...state.segmentsByVideo,
          [context.videoId]: context.segments,
        },
        videoStates: {
          ...state.videoStates,
          [context.videoId]: context.videoStatus,
        },
        previewPaths: {
          ...state.previewPaths,
          [context.videoId]: context.previewPath,
        },
        sourcePaths: {
          ...state.sourcePaths,
          [context.videoId]: context.sourcePath,
        },
        selectedVideoId: context.videoId,
      } as ReviewState;

      return {
        segmentsByVideo: baseState.segmentsByVideo,
        videoStates: baseState.videoStates,
        previewPaths: baseState.previewPaths,
        sourcePaths: baseState.sourcePaths,
        selectedVideoId: context.videoId,
        ...syncPendingFromSelectedSegment(baseState),
      };
    }),
  setSelectedSegmentId: (segmentId) =>
    set((state) => {
      const baseState = {
        ...state,
        selectedSegmentId: segmentId,
        selectedSegmentIds: segmentId ? [segmentId] : [],
        lastRangeAnchorId: segmentId,
      } as ReviewState;

      return {
        selectedSegmentId: segmentId,
        selectedSegmentIds: segmentId ? [segmentId] : [],
        lastRangeAnchorId: segmentId,
        ...syncPendingFromSelectedSegment(baseState),
      };
    }),
  toggleSegmentSelection: (segmentId, orderedIds, withRange) =>
    set((state) => {
      let selected = state.selectedSegmentIds;
      const anchor = state.lastRangeAnchorId ?? state.selectedSegmentId ?? segmentId;

      if (withRange) {
        const start = orderedIds.indexOf(anchor);
        const end = orderedIds.indexOf(segmentId);
        if (start >= 0 && end >= 0) {
          const [from, to] = start < end ? [start, end] : [end, start];
          selected = Array.from(new Set([...selected, ...orderedIds.slice(from, to + 1)]));
        }
      } else if (selected.includes(segmentId)) {
        selected = selected.filter((id) => id !== segmentId);
      } else {
        selected = [...selected, segmentId];
      }

      return {
        selectedSegmentIds: selected,
        lastRangeAnchorId: segmentId,
      };
    }),
  setFilter: (key, value) =>
    set((state) => ({
      filters: {
        ...state.filters,
        [key]: value,
      },
    })),
  resetFilters: () => set({ filters: DEFAULT_FILTERS }),
  setPreviewMode: (mode) => set({ previewMode: mode }),
  setLogoOverlayVisible: (value) => set({ logoOverlayVisible: value }),
  setPendingLogoFix: (value) => set({ pendingLogoFix: value, validationPreviewPath: null }),
  setPendingSubtitleFix: (value) => set({ pendingSubtitleFix: value, validationPreviewPath: null }),
  syncPendingFixesFromSelection: () => set((state) => syncPendingFromSelectedSegment(state)),
  updateSegments: (videoId, updatedSegments) =>
    set((state) => {
      const current = state.segmentsByVideo[videoId] ?? [];
      const map = new Map(current.map((segment) => [segment.id, segment]));

      for (const segment of updatedSegments) {
        map.set(segment.id, segment);
      }

      const baseState = {
        ...state,
        segmentsByVideo: {
          ...state.segmentsByVideo,
          [videoId]: current.map((segment) => map.get(segment.id) ?? segment),
        },
      } as ReviewState;

      return {
        segmentsByVideo: baseState.segmentsByVideo,
        ...syncPendingFromSelectedSegment(baseState),
      };
    }),
  updateSegment: (videoId, updatedSegment) =>
    set((state) => {
      const current = state.segmentsByVideo[videoId] ?? [];
      const nextSegments = current.map((segment) =>
        segment.id === updatedSegment.id ? updatedSegment : segment
      );
      const baseState = {
        ...state,
        segmentsByVideo: {
          ...state.segmentsByVideo,
          [videoId]: nextSegments,
        },
      } as ReviewState;

      return {
        segmentsByVideo: baseState.segmentsByVideo,
        ...syncPendingFromSelectedSegment(baseState),
      };
    }),
  setGatingResult: (videoId, result) =>
    set((state) => ({
      gatingByVideo: {
        ...state.gatingByVideo,
        [videoId]: result,
      },
    })),
  setValidationPreviewPath: (path) => set({ validationPreviewPath: path }),
  updateVideoSummaryStatus: (videoId, status) =>
    set((state) => ({
      videoList: state.videoList.map((video) =>
        video.videoId === videoId
          ? {
              ...video,
              status,
              reviewRequired: status !== "ReadyToExport" && video.reviewRequired,
            }
          : video
      ),
      videoStates: {
        ...state.videoStates,
        [videoId]: state.videoStates[videoId]
          ? {
              ...state.videoStates[videoId],
              status,
              currentStep: status === "ReadyToExport" ? "review-complete" : state.videoStates[videoId]?.currentStep ?? "done",
            }
          : state.videoStates[videoId],
      },
    })),
}));
