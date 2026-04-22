import type { JobReadiness, MappingRow } from "../start-flow/types";
import { InlinePicker } from "./InlinePicker";

type MappingTableProps = {
  isSaving: boolean;
  onApply: (videoId: string, field: "logo" | "audio" | "srt", path: string) => Promise<void>;
  readinessState: JobReadiness | null;
  rows: MappingRow[];
};

export function MappingTable({ isSaving, onApply, readinessState, rows }: MappingTableProps) {
  const readinessMap = new Map(readinessState?.videos.map((video) => [video.videoId, video]) ?? []);

  return (
    <div className="mapping-table">
      <div className="mapping-table__header">
        <span>Ten video</span>
        <span>Task</span>
        <span>Preset</span>
        <span>Logo</span>
        <span>Audio</span>
        <span>SRT</span>
        <span>Trang thai</span>
      </div>

      {rows.map((row) => (
        <div className="mapping-table__row" key={row.videoId}>
          <div className="mapping-table__meta">
            <strong>{row.videoName}</strong>
          </div>
          <div className="mapping-table__meta">{row.task ?? "Chua chon task"}</div>
          <div className="mapping-table__meta">{row.presetName ?? "Chua co preset"}</div>
          <InlinePicker
            fieldKey="logo"
            fieldLabel="Logo"
            fieldState={row.logo}
            isSaving={isSaving}
            onApply={(field, path) => onApply(row.videoId, field, path)}
          />
          <InlinePicker
            fieldKey="audio"
            fieldLabel="Audio"
            fieldState={row.audio}
            isSaving={isSaving}
            onApply={(field, path) => onApply(row.videoId, field, path)}
          />
          <InlinePicker
            fieldKey="srt"
            fieldLabel="SRT"
            fieldState={row.srt}
            isSaving={isSaving}
            onApply={(field, path) => onApply(row.videoId, field, path)}
          />
          <div className="mapping-table__status">
            <div className="mapping-table__status-stack">
              <div className={`mapping-pill mapping-pill--${row.status.toLowerCase().replace(/\s+/g, "-")}`}>
                {row.status}
              </div>
              <div
                className={`mapping-pill ${
                  readinessMap.get(row.videoId)?.isReady ? "mapping-pill--ready" : "mapping-pill--blocked"
                }`}
              >
                {readinessMap.get(row.videoId)?.isReady ? "Ready" : "Blocked"}
              </div>
              {readinessMap.get(row.videoId)?.blockers.length ? (
                <ul className="mapping-table__blockers">
                  {readinessMap.get(row.videoId)?.blockers.map((blocker) => (
                    <li key={`${row.videoId}-${blocker}`}>{blocker}</li>
                  ))}
                </ul>
              ) : null}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
