export type FileRole = "video" | "logo" | "audio" | "srt";

export type AcceptedFile = {
  fileName: string;
  path: string;
  role: FileRole;
};

export type RejectedFile = {
  fileName: string;
  path: string;
  reason: string;
};

export type ClassifiedImportResult = {
  acceptedFiles: AcceptedFile[];
  rejectedFiles: RejectedFile[];
};

export type AppTask =
  | "replace-logo"
  | "replace-audio"
  | "replace-subtitle"
  | "replace-all";

export type BrandPreset = {
  presetId: string;
  brandName: string;
  defaultLogoPath: string;
  audioReplacementPolicy: string;
  subtitleStylePreset: string;
  layoutRules: string;
  exportPreset: string;
  notes: string;
};

export type PresetDraft = Omit<BrandPreset, "presetId">;

export type MappingOption = {
  fileName: string;
  path: string;
};

export type MappingFieldState = {
  currentPath: string | null;
  options: MappingOption[];
  status: string;
  message: string;
};

export type MappingRow = {
  videoId: string;
  videoName: string;
  task: AppTask | null;
  presetName: string | null;
  logo: MappingFieldState;
  audio: MappingFieldState;
  srt: MappingFieldState;
  status: string;
};

export type VideoReadiness = {
  videoId: string;
  isReady: boolean;
  blockers: string[];
};

export type JobReadiness = {
  isReady: boolean;
  blockers: string[];
  videos: VideoReadiness[];
};

export type DraftJob = {
  jobId: string;
  createdAt: string;
  selectedTask: AppTask | null;
  presetId: string | null;
  outputFolder: string;
  exportOutputFolder: string | null;
  status: string;
  videoItems: Array<{
    videoId: string;
    sourcePath: string;
    status: string;
  }>;
  importedFiles: AcceptedFile[];
};

export type CreateJobResponse = {
  job: DraftJob;
  manifestPath: string;
};

export type ListPresetsResponse = {
  presets: BrandPreset[];
};

export type SelectPresetResponse = {
  applied: boolean;
  job: DraftJob | null;
  preset: BrandPreset;
  warningMessage: string | null;
};

export type CreatePresetResponse = {
  preset: BrandPreset;
};

export type EditPresetResponse = {
  saved: boolean;
  preset: BrandPreset | null;
  warningMessage: string | null;
};

export type DuplicatePresetResponse = {
  preset: BrandPreset;
};

export type MappingResponse = {
  job: DraftJob;
  rows: MappingRow[];
};

export type MappingUpdatedEvent = MappingResponse;

export type InputFileReplacedAfterReviewEvent = {
  videoId: string;
  field: "logo" | "audio" | "srt";
  message: string;
};

export type JobReadinessResponse = {
  readiness: JobReadiness;
};

export type RecentJobSummary = {
  jobId: string;
  createdAt: string;
  status: string;
  videoCount: number;
  lastModified: string;
};

export type ListJobsResponse = {
  jobs: RecentJobSummary[];
};

export type LoadedSegmentState = {
  fileName: string;
  payload: unknown;
};

export type LoadJobResponse = {
  job: DraftJob;
  preset: BrandPreset | null;
  videoStates: VideoProcessingState[];
  segmentStates: LoadedSegmentState[];
  warningMessage: string | null;
  lastModified: string;
};

export type StartJobResponse = {
  started: boolean;
  job: DraftJob;
  blockers: string[];
};

export type JobStartedEvent = {
  job: DraftJob;
};

export type JobUpdatedEvent = {
  job: DraftJob;
};

export type VideoProcessingEvent = {
  jobId: string;
  videoId: string;
};

export type ProcessingStepUpdateEvent = {
  videoId: string;
  step: string;
  status: "started" | "completed" | "skipped" | "failed";
  message: string;
};

export type RiskDistribution = {
  high: number;
  medium: number;
  low: number;
};

export type VideoProcessingCompletedEvent = {
  videoId: string;
  outcome: "ReviewNeeded" | "ReadyToExport" | "Failed";
  segmentCount: number;
  riskDistribution: RiskDistribution;
};

export type JobSummary = {
  total: number;
  reviewNeeded: number;
  readyToExport: number;
  failed: number;
};

export type JobProcessingCompletedEvent = {
  jobId: string;
  summary: JobSummary;
};

export type ReviewSegment = {
  id: string;
  videoId: string;
  source: "logo" | "subtitle" | string;
  issueType: string;
  riskLevel: "High" | "Medium" | "Low" | string;
  reviewStatus: "Unreviewed" | "Accepted" | "Modified" | string;
  startMs: number;
  endMs: number | null;
  confidence: number;
  message: string;
  boundingBox: {
    x: number;
    y: number;
    width: number;
    height: number;
  } | null;
  quickFixState: QuickFixState;
};

export type LogoFix = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type SubtitleRegionFix = {
  x: number;
  y: number;
  width: number;
  height: number;
  mode: string;
};

export type SubtitlePositionFix = {
  x: number;
  y: number;
};

export type SubtitleFix = {
  oldRegion: SubtitleRegionFix | null;
  newPosition: SubtitlePositionFix | null;
  newScale: number | null;
  stylePreset: string | null;
};

export type QuickFixState = {
  logoFix: LogoFix | null;
  subtitleFix: SubtitleFix | null;
};

export type VideoProcessingState = {
  videoId: string;
  status: string;
  currentStep: string;
  completedSteps: string[];
  timestamp: string;
};

export type ReviewVideoSummary = {
  videoId: string;
  videoName: string;
  status: string;
  segmentCount: number;
  reviewRequired: boolean;
  optionalReview: boolean;
};

export type ReviewVideoContext = {
  videoId: string;
  sourcePath: string;
  segments: ReviewSegment[];
  videoStatus: VideoProcessingState;
  previewPath: string;
};

export type ReviewContext = {
  jobId: string;
  selectedVideoId: string | null;
  videoList: ReviewVideoSummary[];
  selectedVideo: ReviewVideoContext | null;
};

export type ReviewContextResponse = {
  context: ReviewContext;
};

export type VideoReviewContextResponse = {
  context: ReviewVideoContext;
};

export type ApplySegmentsResult = {
  updatedSegments: ReviewSegment[];
  warningMessage: string | null;
};

export type ApplySegmentsResponse = {
  result: ApplySegmentsResult;
};

export type ReviewSegmentResponse = {
  segment: ReviewSegment;
};

export type ReviewGatingBlocker = {
  segmentId: string;
  timeRange: string;
  issueType: string;
};

export type ReviewGatingResult = {
  canProceed: boolean;
  blockers: ReviewGatingBlocker[];
};

export type ReviewGatingResponse = {
  result: ReviewGatingResult;
};

export type FramePreviewResult = {
  cachePath: string;
};

export type FramePreviewResponse = {
  result: FramePreviewResult;
};

export type VideoReadyToExportEvent = {
  videoId: string;
};

export type ExportPresetSummary = {
  label: string;
  codec: string;
  crf: string;
};

export type ExportReadyVideo = {
  videoId: string;
  videoName: string;
  status: string;
  audioSummary: string;
};

export type ExportBlockedVideo = {
  videoId: string;
  videoName: string;
  status: string;
  reason: string;
  audioSummary: string;
};

export type ExportReadiness = {
  readyVideos: ExportReadyVideo[];
  blockedVideos: ExportBlockedVideo[];
  outputFolder: string;
  presetSummary: ExportPresetSummary | null;
};

export type ExportReadinessResponse = {
  result: ExportReadiness;
};

export type SetExportOutputFolderResponse = {
  job: DraftJob;
};

export type StartExportResponse = {
  job: DraftJob;
  started: boolean;
};

export type VideoExportStartedEvent = {
  videoId: string;
};

export type ExportProgressEvent = {
  videoId: string;
  percent: number;
};

export type VideoExportCompletedEvent = {
  videoId: string;
  success: boolean;
  outputPath: string | null;
  errorMessage: string | null;
};

export type BatchExportCompletedEvent = {
  jobId: string;
  total: number;
  success: number;
  failed: number;
};

export type PersistedEncodeSummary = {
  codec: string;
  crf: string;
  outputSizeMb: number;
  durationSeconds: number;
  bitrateKbps: number | null;
};

export type AudioSourceSummary = {
  policy: string;
  audioFilePath: string | null;
};

export type SegmentStats = {
  total: number;
  flagged: number;
  modified: number;
  accepted: number;
  highRiskRemaining: number;
};

export type SpotCheckThumbnail = {
  segmentId: string;
  beforePath: string | null;
  afterPath: string | null;
};

export type VideoReport = {
  videoId: string;
  videoName: string;
  finalStatus: string;
  encodeSummary: PersistedEncodeSummary | null;
  audioSource: AudioSourceSummary;
  segmentStats: SegmentStats;
  spotCheckThumbnails: SpotCheckThumbnail[];
  outputPath: string | null;
  reportGeneratedAt: string;
};

export type JobExportSummaryReport = {
  jobId: string;
  totalVideos: number;
  success: number;
  failed: number;
  totalOutputSizeMb: number;
  reports: VideoReport[];
  generatedAt: string;
};

export type VideoReportResponse = {
  report: VideoReport;
};

export type JobExportSummaryReportResponse = {
  summary: JobExportSummaryReport;
};

export const TASK_OPTIONS: Array<{ description: string; id: AppTask; title: string }> = [
  {
    id: "replace-logo",
    title: "Thay logo",
    description: "Dùng khi job chỉ cần cập nhật overlay logo thương hiệu.",
  },
  {
    id: "replace-audio",
    title: "Thay audio",
    description: "Dùng cho batch thay toàn bộ audio track của video.",
  },
  {
    id: "replace-subtitle",
    title: "Thay subtitle",
    description: "Dùng khi cần xử lý subtitle cũ và render SRT mới.",
  },
  {
    id: "replace-all",
    title: "Thay logo, audio, và subtitle",
    description: "Flow đầy đủ cho job re-branding V1.",
  },
];

export function suggestTaskFromRoles(roles: FileRole[]): AppTask | null {
  const uniqueRoles = new Set(roles);

  if (uniqueRoles.has("logo") && uniqueRoles.has("audio") && uniqueRoles.has("srt")) {
    return "replace-all";
  }

  if (uniqueRoles.has("audio") && !uniqueRoles.has("logo") && !uniqueRoles.has("srt")) {
    return "replace-audio";
  }

  if (uniqueRoles.has("srt") && !uniqueRoles.has("audio") && !uniqueRoles.has("logo")) {
    return "replace-subtitle";
  }

  if (uniqueRoles.has("logo") && !uniqueRoles.has("audio") && !uniqueRoles.has("srt")) {
    return "replace-logo";
  }

  return null;
}

export function groupAcceptedFiles(files: AcceptedFile[]) {
  return {
    audio: files.filter((file) => file.role === "audio"),
    logo: files.filter((file) => file.role === "logo"),
    srt: files.filter((file) => file.role === "srt"),
    video: files.filter((file) => file.role === "video"),
  };
}

export function mergeAcceptedFiles(
  currentFiles: AcceptedFile[],
  nextFiles: AcceptedFile[]
): AcceptedFile[] {
  const merged = [...currentFiles];

  for (const file of nextFiles) {
    if (!merged.some((existingFile) => existingFile.path === file.path)) {
      merged.push(file);
    }
  }

  return merged;
}
