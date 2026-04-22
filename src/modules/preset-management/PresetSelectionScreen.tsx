import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { useJobStore } from "../../store/jobStore";
import type {
  BrandPreset,
  CreatePresetResponse,
  DuplicatePresetResponse,
  EditPresetResponse,
  ListPresetsResponse,
  PresetDraft,
  SelectPresetResponse,
} from "../start-flow/types";
import { PresetCard } from "./PresetCard";
import { PresetForm } from "./PresetForm";

const EMPTY_PRESET: PresetDraft = {
  brandName: "",
  defaultLogoPath: "",
  audioReplacementPolicy: "ReplaceAll",
  subtitleStylePreset: "",
  layoutRules: "",
  exportPreset: "MP4 H264 CRF20",
  notes: "",
};

function normalizeAudioPolicy(value: string) {
  if (value === "ReplaceAll" || value === "Replace all spoken tracks with channel voiceover") {
    return "ReplaceAll";
  }

  return "NoReplacement";
}

function normalizeExportPreset(value: string) {
  if (value === "MP4 H264 CRF18" || value === "4K archive master") {
    return "MP4 H264 CRF18";
  }

  if (value === "MP4 H264 CRF23" || value === "1080x1920 vertical short") {
    return "MP4 H264 CRF23";
  }

  return "MP4 H264 CRF20";
}

export function PresetSelectionScreen() {
  const currentJob = useJobStore((state) => state.currentJob);
  const preset = useJobStore((state) => state.preset);
  const setJob = useJobStore((state) => state.setJob);
  const setPreset = useJobStore((state) => state.setPreset);
  const [presets, setPresets] = useState<BrandPreset[]>([]);
  const [selectedPresetId, setSelectedPresetId] = useState<string | null>(preset?.presetId ?? null);
  const [isLoading, setIsLoading] = useState(false);
  const [isApplying, setIsApplying] = useState(false);
  const [isSavingPreset, setIsSavingPreset] = useState(false);
  const [formMode, setFormMode] = useState<"create" | "edit" | null>(null);
  const [formWarningMessage, setFormWarningMessage] = useState<string | null>(null);
  const [warningMessage, setWarningMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (!currentJob) {
      return;
    }

    let mounted = true;

    const loadPresets = async () => {
      setIsLoading(true);
      setErrorMessage(null);

      try {
        const response = await invoke<ListPresetsResponse>("list_presets");

        if (!mounted) {
          return;
        }

        setPresets(response.presets);
        setSelectedPresetId((currentSelected) => {
          if (currentSelected) {
            return currentSelected;
          }

          return currentJob.presetId ?? response.presets[0]?.presetId ?? null;
        });
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

    void loadPresets();

    return () => {
      mounted = false;
    };
  }, [currentJob]);

  useEffect(() => {
    if (preset?.presetId) {
      setSelectedPresetId(preset.presetId);
    }
  }, [preset]);

  const loadPresets = async () => {
    const response = await invoke<ListPresetsResponse>("list_presets");
    setPresets(response.presets);
    setSelectedPresetId((currentSelected) => currentSelected ?? currentJob?.presetId ?? response.presets[0]?.presetId ?? null);
  };

  if (!currentJob) {
    return null;
  }

  const selectedPreset =
    presets.find((candidate) => candidate.presetId === selectedPresetId) ?? presets[0] ?? null;

  const applyPreset = async (confirmOverride: boolean) => {
    if (!selectedPreset || !currentJob) {
      return;
    }

    setIsApplying(true);
    setErrorMessage(null);

    try {
      const response = await invoke<SelectPresetResponse>("select_preset", {
        confirmOverride,
        jobId: currentJob.jobId,
        presetId: selectedPreset.presetId,
      });

      setWarningMessage(response.warningMessage);

      if (response.applied && response.job) {
        setJob(response.job);
        setPreset(response.preset);
      }
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsApplying(false);
    }
  };

  const initialFormValue: PresetDraft =
    formMode === "edit" && selectedPreset
      ? {
          brandName: selectedPreset.brandName,
          defaultLogoPath: selectedPreset.defaultLogoPath,
          audioReplacementPolicy: normalizeAudioPolicy(selectedPreset.audioReplacementPolicy),
          subtitleStylePreset: selectedPreset.subtitleStylePreset,
          layoutRules: selectedPreset.layoutRules,
          exportPreset: normalizeExportPreset(selectedPreset.exportPreset),
          notes: selectedPreset.notes,
        }
      : EMPTY_PRESET;

  const submitPreset = async (value: PresetDraft, confirmOverride: boolean) => {
    setIsSavingPreset(true);
    setErrorMessage(null);

    try {
      if (formMode === "create") {
        const response = await invoke<CreatePresetResponse>("create_preset", { preset: value });
        await loadPresets();
        setSelectedPresetId(response.preset.presetId);
        setFormMode(null);
        setFormWarningMessage(null);
      }

      if (formMode === "edit" && selectedPreset) {
        const response = await invoke<EditPresetResponse>("edit_preset", {
          preset: value,
          presetId: selectedPreset.presetId,
          currentJobId: currentJob.jobId,
          confirmOverride,
        });

        setFormWarningMessage(response.warningMessage);

        if (response.saved) {
          await loadPresets();
          setFormMode(null);
          setFormWarningMessage(null);

          if (preset?.presetId === selectedPreset.presetId && response.preset) {
            setPreset(response.preset);
          }
        }
      }
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSavingPreset(false);
    }
  };

  const duplicatePreset = async () => {
    if (!selectedPreset) {
      return;
    }

    setIsSavingPreset(true);
    setErrorMessage(null);

    try {
      const response = await invoke<DuplicatePresetResponse>("duplicate_preset", {
        presetId: selectedPreset.presetId,
      });
      await loadPresets();
      setSelectedPresetId(response.preset.presetId);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSavingPreset(false);
    }
  };

  return (
    <section className="preset-screen">
      <div className="preset-screen__hero">
        <div>
          <p className="start-screen__eyebrow">Preset Selection</p>
          <h2 className="start-screen__headline">Chon preset theo brand va channel</h2>
        </div>
        <div className="preset-screen__meta">
          <span className="inline-note">Draft job: {currentJob.jobId}</span>
          {preset ? <span className="inline-note">Dang ap dung: {preset.brandName}</span> : null}
        </div>
      </div>

      {isLoading ? <div className="inline-note">Dang tai danh sach preset...</div> : null}
      {warningMessage ? <div className="inline-error">{warningMessage}</div> : null}
      {errorMessage ? <div className="inline-error">{errorMessage}</div> : null}

      <div className="preset-layout">
        <div className="preset-list">
          {presets.map((candidate) => (
            <button
              key={candidate.presetId}
              type="button"
              className={`preset-list__item${
                selectedPreset?.presetId === candidate.presetId ? " preset-list__item--selected" : ""
              }`}
              onClick={() => {
                setSelectedPresetId(candidate.presetId);
                setWarningMessage(null);
              }}
            >
              <span className="preset-list__name">{candidate.brandName}</span>
              <span className="preset-list__label">{candidate.exportPreset}</span>
            </button>
          ))}
        </div>

        {selectedPreset ? <PresetCard isSelected={preset?.presetId === selectedPreset.presetId} preset={selectedPreset} /> : null}
      </div>

      <div className="preset-actions">
        <button type="button" className="action-button action-button--primary" onClick={() => void applyPreset(false)} disabled={!selectedPreset || isApplying}>
          {isApplying ? "Dang ap dung..." : "Chon preset nay"}
        </button>
        <button
          type="button"
          className="action-button"
          disabled={!selectedPreset || isSavingPreset}
          onClick={() => {
            setFormMode("edit");
            setFormWarningMessage(null);
          }}
        >
          Sua preset
        </button>
        <button
          type="button"
          className="action-button"
          disabled={isSavingPreset}
          onClick={() => {
            setFormMode("create");
            setFormWarningMessage(null);
          }}
        >
          Tao preset moi
        </button>
        <button
          type="button"
          className="action-button"
          disabled={!selectedPreset || isSavingPreset}
          onClick={() => void duplicatePreset()}
        >
          Duplicate preset
        </button>
        {warningMessage ? (
          <button type="button" className="action-button action-button--danger" onClick={() => void applyPreset(true)} disabled={isApplying}>
            Xac nhan doi preset
          </button>
        ) : null}
      </div>

      {formMode ? (
        <PresetForm
          initialValue={initialFormValue}
          isSaving={isSavingPreset}
          mode={formMode}
          onCancel={() => {
            setFormMode(null);
            setFormWarningMessage(null);
          }}
          onSubmit={submitPreset}
          warningMessage={formWarningMessage}
        />
      ) : null}
    </section>
  );
}
