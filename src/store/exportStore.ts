import { create } from "zustand";
import type {
  ExportBlockedVideo,
  ExportPresetSummary,
  ExportReadyVideo,
  JobExportSummaryReport,
  VideoReport,
} from "../modules/start-flow/types";

type BatchSummary = {
  failed: number;
  success: number;
  total: number;
};

type ExportState = {
  isOpen: boolean;
  isLoading: boolean;
  isExporting: boolean;
  readyVideos: ExportReadyVideo[];
  blockedVideos: ExportBlockedVideo[];
  batchVideos: ExportReadyVideo[];
  outputFolder: string | null;
  presetSummary: ExportPresetSummary | null;
  progressByVideo: Record<string, number>;
  statusByVideo: Record<string, string>;
  outputPathByVideo: Record<string, string>;
  errorByVideo: Record<string, string>;
  batchSummary: BatchSummary | null;
  reportsByVideo: Record<string, VideoReport>;
  jobSummaryReport: JobExportSummaryReport | null;
  openScreen: () => void;
  closeScreen: () => void;
  setLoading: (value: boolean) => void;
  hydrate: (payload: {
    readyVideos: ExportReadyVideo[];
    blockedVideos: ExportBlockedVideo[];
    outputFolder: string;
    presetSummary: ExportPresetSummary | null;
  }) => void;
  setOutputFolder: (value: string) => void;
  startBatch: (videos: ExportReadyVideo[]) => void;
  markVideoStarted: (videoId: string) => void;
  updateProgress: (videoId: string, percent: number) => void;
  markVideoCompleted: (payload: {
    videoId: string;
    success: boolean;
    outputPath?: string | null;
    errorMessage?: string | null;
  }) => void;
  finishBatch: (summary: BatchSummary) => void;
  setVideoReport: (report: VideoReport) => void;
  setJobSummaryReport: (summary: JobExportSummaryReport | null) => void;
};

export const useExportStore = create<ExportState>((set) => ({
  isOpen: false,
  isLoading: false,
  isExporting: false,
  readyVideos: [],
  blockedVideos: [],
  batchVideos: [],
  outputFolder: null,
  presetSummary: null,
  progressByVideo: {},
  statusByVideo: {},
  outputPathByVideo: {},
  errorByVideo: {},
  batchSummary: null,
  reportsByVideo: {},
  jobSummaryReport: null,
  openScreen: () => set({ isOpen: true }),
  closeScreen: () =>
    set({
      isOpen: false,
      isLoading: false,
      isExporting: false,
      readyVideos: [],
      blockedVideos: [],
      batchVideos: [],
      outputFolder: null,
      presetSummary: null,
      progressByVideo: {},
      statusByVideo: {},
      outputPathByVideo: {},
      errorByVideo: {},
      batchSummary: null,
      reportsByVideo: {},
      jobSummaryReport: null,
    }),
  setLoading: (value) => set({ isLoading: value }),
  hydrate: (payload) =>
    set({
      readyVideos: payload.readyVideos,
      blockedVideos: payload.blockedVideos,
      outputFolder: payload.outputFolder,
      presetSummary: payload.presetSummary,
    }),
  setOutputFolder: (value) => set({ outputFolder: value }),
  startBatch: (videos) =>
    set(() => ({
      isExporting: true,
      batchVideos: videos,
      batchSummary: null,
      progressByVideo: Object.fromEntries(videos.map((video) => [video.videoId, 0])),
      statusByVideo: Object.fromEntries(videos.map((video) => [video.videoId, "Queued"])),
      outputPathByVideo: {},
      errorByVideo: {},
      reportsByVideo: {},
      jobSummaryReport: null,
    })),
  markVideoStarted: (videoId) =>
    set((state) => ({
      statusByVideo: { ...state.statusByVideo, [videoId]: "Exporting" },
      progressByVideo: { ...state.progressByVideo, [videoId]: Math.max(state.progressByVideo[videoId] ?? 0, 1) },
    })),
  updateProgress: (videoId, percent) =>
    set((state) => ({
      progressByVideo: { ...state.progressByVideo, [videoId]: percent },
    })),
  markVideoCompleted: ({ videoId, success, outputPath, errorMessage }) =>
    set((state) => ({
      statusByVideo: { ...state.statusByVideo, [videoId]: success ? "Exported" : "Failed" },
      progressByVideo: { ...state.progressByVideo, [videoId]: success ? 100 : state.progressByVideo[videoId] ?? 0 },
      outputPathByVideo: outputPath ? { ...state.outputPathByVideo, [videoId]: outputPath } : state.outputPathByVideo,
      errorByVideo: errorMessage ? { ...state.errorByVideo, [videoId]: errorMessage } : state.errorByVideo,
    })),
  finishBatch: (summary) =>
    set({
      isExporting: false,
      batchSummary: summary,
    }),
  setVideoReport: (report) =>
    set((state) => ({
      reportsByVideo: {
        ...state.reportsByVideo,
        [report.videoId]: report,
      },
    })),
  setJobSummaryReport: (summary) => set({ jobSummaryReport: summary }),
}));
