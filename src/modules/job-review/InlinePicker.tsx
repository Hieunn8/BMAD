import { useEffect, useId, useState } from "react";
import type { MappingFieldState } from "../start-flow/types";

type InlinePickerProps = {
  fieldKey: "logo" | "audio" | "srt";
  fieldLabel: string;
  fieldState: MappingFieldState;
  isSaving: boolean;
  onApply: (field: "logo" | "audio" | "srt", path: string) => Promise<void>;
};

export function InlinePicker({
  fieldKey,
  fieldLabel,
  fieldState,
  isSaving,
  onApply,
}: InlinePickerProps) {
  const selectId = useId();
  const [selectedPath, setSelectedPath] = useState(fieldState.currentPath ?? fieldState.options[0]?.path ?? "");

  useEffect(() => {
    setSelectedPath(fieldState.currentPath ?? fieldState.options[0]?.path ?? "");
  }, [fieldState.currentPath, fieldState.options]);

  const canApply = selectedPath.length > 0 && selectedPath !== (fieldState.currentPath ?? "");

  return (
    <div className="mapping-cell">
      <div className={`mapping-pill mapping-pill--${fieldState.status.toLowerCase()}`}>{fieldState.status}</div>
      <div className="mapping-cell__current">{fieldState.currentPath ?? "Chua gan file"}</div>
      <label className="mapping-cell__picker" htmlFor={selectId}>
        <span>{fieldLabel}</span>
        <select
          id={selectId}
          value={selectedPath}
          onChange={(event) => setSelectedPath(event.target.value)}
          disabled={isSaving || fieldState.options.length === 0}
        >
          <option value="">Chon file</option>
          {fieldState.options.map((option) => (
            <option key={`${fieldKey}-${option.path}`} value={option.path}>
              {option.fileName}
            </option>
          ))}
        </select>
      </label>
      <p className="mapping-cell__message">{fieldState.message}</p>
      <button
        type="button"
        className="action-button action-button--small"
        disabled={!canApply || isSaving}
        onClick={() => void onApply(fieldKey, selectedPath)}
      >
        {isSaving ? "Dang luu..." : "Ap dung"}
      </button>
    </div>
  );
}
