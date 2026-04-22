import { create } from "zustand";
import type {
  AcceptedFile,
  BrandPreset,
  AppTask,
  ClassifiedImportResult,
  DraftJob,
  JobReadiness,
  LoadJobResponse,
  MappingRow,
} from "../modules/start-flow/types";
import { mergeAcceptedFiles } from "../modules/start-flow/types";

type JobState = {
  currentJob: DraftJob | null;
  importedFiles: AcceptedFile[];
  mappingRows: MappingRow[];
  readinessState: JobReadiness | null;
  preset: BrandPreset | null;
  rejectedFiles: ClassifiedImportResult["rejectedFiles"];
  selectedTask: AppTask | null;
  setJob: (job: DraftJob) => void;
  loadJob: (jobData: LoadJobResponse) => void;
  resetJob: () => void;
  addFiles: (result: ClassifiedImportResult) => void;
  setMappingRows: (rows: MappingRow[]) => void;
  setReadinessState: (readiness: JobReadiness | null) => void;
  setPreset: (preset: BrandPreset | null) => void;
  setSelectedTask: (task: AppTask | null) => void;
};

export const useJobStore = create<JobState>((set) => ({
  currentJob: null,
  importedFiles: [],
  mappingRows: [],
  readinessState: null,
  preset: null,
  rejectedFiles: [],
  selectedTask: null,
  setJob: (job) =>
    set({
      currentJob: job,
      importedFiles: job.importedFiles,
      selectedTask: (job.selectedTask as AppTask | null) ?? null,
    }),
  loadJob: (jobData) =>
    set({
      currentJob: jobData.job,
      importedFiles: jobData.job.importedFiles,
      mappingRows: [],
      readinessState: null,
      preset: jobData.preset,
      rejectedFiles: [],
      selectedTask: (jobData.job.selectedTask as AppTask | null) ?? null,
    }),
  resetJob: () =>
    set({
      currentJob: null,
      importedFiles: [],
      mappingRows: [],
      readinessState: null,
      preset: null,
      rejectedFiles: [],
      selectedTask: null,
    }),
  addFiles: (result) =>
    set((state) => ({
      importedFiles: mergeAcceptedFiles(state.importedFiles, result.acceptedFiles),
      rejectedFiles: result.rejectedFiles,
    })),
  setMappingRows: (rows) => set({ mappingRows: rows }),
  setReadinessState: (readiness) => set({ readinessState: readiness }),
  setPreset: (preset) => set({ preset }),
  setSelectedTask: (task) => set({ selectedTask: task }),
}));
