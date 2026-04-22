import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { PresetDraft } from "../start-flow/types";

type PresetFormProps = {
  initialValue: PresetDraft;
  isSaving: boolean;
  mode: "create" | "edit";
  onCancel: () => void;
  onSubmit: (value: PresetDraft, confirmOverride: boolean) => Promise<void>;
  warningMessage: string | null;
};

const AUDIO_POLICY_OPTIONS = [
  { label: "ReplaceAll", value: "ReplaceAll" },
  { label: "NoReplacement", value: "NoReplacement" },
];

const EXPORT_PRESET_OPTIONS = ["MP4 H264 CRF18", "MP4 H264 CRF20", "MP4 H264 CRF23"];

export function PresetForm({
  initialValue,
  isSaving,
  mode,
  onCancel,
  onSubmit,
  warningMessage,
}: PresetFormProps) {
  const [formValue, setFormValue] = useState<PresetDraft>(initialValue);
  const [validationError, setValidationError] = useState<string | null>(null);

  useEffect(() => {
    setFormValue(initialValue);
    setValidationError(null);
  }, [initialValue]);

  const submit = async (confirmOverride: boolean) => {
    if (!formValue.brandName.trim()) {
      setValidationError("Brand Name khong duoc de trong.");
      return;
    }

    if (!formValue.defaultLogoPath.trim()) {
      setValidationError("Default Logo khong duoc de trong.");
      return;
    }

    setValidationError(null);
    await onSubmit(formValue, confirmOverride);
  };

  const pickLogo = async () => {
    const selected = await open({
      multiple: false,
      title: "Chon logo mac dinh",
      filters: [
        {
          name: "Logo",
          extensions: ["png", "jpg", "jpeg", "svg"],
        },
      ],
    });

    if (typeof selected === "string") {
      setFormValue((current) => ({ ...current, defaultLogoPath: selected }));
      setValidationError(null);
    }
  };

  return (
    <section className="preset-form">
      <div className="preset-form__hero">
        <div>
          <p className="preset-card__eyebrow">{mode === "create" ? "Create Preset" : "Edit Preset"}</p>
          <h3>{mode === "create" ? "Tao preset moi" : "Cap nhat preset da chon"}</h3>
        </div>
      </div>

      {validationError ? <div className="inline-error">{validationError}</div> : null}
      {warningMessage ? <div className="inline-error">{warningMessage}</div> : null}

      <div className="preset-form__grid">
        <label className="preset-form__field">
          <span>Brand Name</span>
          <input
            type="text"
            value={formValue.brandName}
            onChange={(event) => setFormValue((current) => ({ ...current, brandName: event.target.value }))}
          />
        </label>

        <label className="preset-form__field">
          <span>Default Logo</span>
          <div className="preset-form__logo-picker">
            <input type="text" value={formValue.defaultLogoPath} readOnly />
            <button type="button" className="action-button action-button--small" onClick={() => void pickLogo()}>
              Chon file
            </button>
          </div>
        </label>

        <label className="preset-form__field">
          <span>Audio Replacement Policy</span>
          <select
            value={formValue.audioReplacementPolicy}
            onChange={(event) =>
              setFormValue((current) => ({ ...current, audioReplacementPolicy: event.target.value }))
            }
          >
            {AUDIO_POLICY_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>

        <label className="preset-form__field">
          <span>Subtitle Style Preset</span>
          <input
            type="text"
            value={formValue.subtitleStylePreset}
            onChange={(event) =>
              setFormValue((current) => ({ ...current, subtitleStylePreset: event.target.value }))
            }
          />
        </label>

        <label className="preset-form__field">
          <span>Export Preset</span>
          <select
            value={formValue.exportPreset}
            onChange={(event) => setFormValue((current) => ({ ...current, exportPreset: event.target.value }))}
          >
            {EXPORT_PRESET_OPTIONS.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </label>

        <label className="preset-form__field preset-form__field--full">
          <span>Layout Rules</span>
          <textarea
            rows={3}
            value={formValue.layoutRules}
            onChange={(event) => setFormValue((current) => ({ ...current, layoutRules: event.target.value }))}
          />
        </label>

        <label className="preset-form__field preset-form__field--full">
          <span>Notes</span>
          <textarea
            rows={3}
            value={formValue.notes}
            onChange={(event) => setFormValue((current) => ({ ...current, notes: event.target.value }))}
          />
        </label>
      </div>

      <div className="preset-actions">
        <button
          type="button"
          className="action-button action-button--primary"
          disabled={isSaving}
          onClick={() => void submit(false)}
        >
          {isSaving ? "Dang luu..." : "Luu preset"}
        </button>
        {warningMessage ? (
          <button
            type="button"
            className="action-button action-button--danger"
            disabled={isSaving}
            onClick={() => void submit(true)}
          >
            Xac nhan luu thay doi
          </button>
        ) : null}
        <button type="button" className="action-button" disabled={isSaving} onClick={onCancel}>
          Huy
        </button>
      </div>
    </section>
  );
}
