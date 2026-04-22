import type { ReviewFilterState } from "../../store/reviewStore";

type FilterBarProps = {
  filters: ReviewFilterState;
  matchCount: number;
  totalCount: number;
  onFilterChange: <K extends keyof ReviewFilterState>(key: K, value: ReviewFilterState[K]) => void;
  onReset: () => void;
};

export function FilterBar({
  filters,
  matchCount,
  totalCount,
  onFilterChange,
  onReset,
}: FilterBarProps) {
  return (
    <div className="segment-filter-bar">
      <div className="segment-filter-bar__controls">
        <label>
          <span>Risk</span>
          <select
            value={filters.riskLevel}
            onChange={(event) => onFilterChange("riskLevel", event.target.value as ReviewFilterState["riskLevel"])}
          >
            <option value="All">All</option>
            <option value="High">High</option>
            <option value="Medium">Medium</option>
            <option value="Low">Low</option>
          </select>
        </label>
        <label>
          <span>Issue</span>
          <select
            value={filters.issueType}
            onChange={(event) => onFilterChange("issueType", event.target.value as ReviewFilterState["issueType"])}
          >
            <option value="All">All</option>
            <option value="LogoPosition">Logo</option>
            <option value="SubtitleRegion">Subtitle</option>
          </select>
        </label>
        <label>
          <span>Status</span>
          <select
            value={filters.status}
            onChange={(event) => onFilterChange("status", event.target.value as ReviewFilterState["status"])}
          >
            <option value="All">All</option>
            <option value="Unreviewed">Unreviewed</option>
            <option value="Accepted">Accepted</option>
            <option value="Modified">Modified</option>
            <option value="Blocked">Blocked</option>
          </select>
        </label>
      </div>
      <div className="segment-filter-bar__meta">
        <span className="inline-note">Dang hien {matchCount} / {totalCount} doan</span>
        <button type="button" className="action-button action-button--small" onClick={onReset}>
          Reset filter
        </button>
      </div>
    </div>
  );
}
