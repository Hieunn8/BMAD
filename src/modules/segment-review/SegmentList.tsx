import type { ReviewSegment } from "../start-flow/types";
import type { ReviewFilterState } from "../../store/reviewStore";
import { FilterBar } from "./FilterBar";

type SegmentListProps = {
  filters: ReviewFilterState;
  segments: ReviewSegment[];
  selectedSegmentId: string | null;
  selectedSegmentIds: string[];
  onFilterChange: <K extends keyof ReviewFilterState>(key: K, value: ReviewFilterState[K]) => void;
  onResetFilters: () => void;
  onActivateSegment: (segmentId: string) => void;
  onToggleSegment: (segmentId: string, orderedIds: string[], withRange?: boolean) => void;
};

const riskOrder: Record<string, number> = {
  High: 0,
  Medium: 1,
  Low: 2,
};

export function filterAndSortSegments(segments: ReviewSegment[], filters: ReviewFilterState) {
  return segments
    .filter((segment) => {
      if (filters.riskLevel !== "All" && segment.riskLevel !== filters.riskLevel) {
        return false;
      }

      if (filters.issueType !== "All" && segment.issueType !== filters.issueType) {
        return false;
      }

      if (filters.status === "Blocked") {
        return segment.riskLevel === "High" && segment.reviewStatus === "Unreviewed";
      }

      if (filters.status !== "All" && segment.reviewStatus !== filters.status) {
        return false;
      }

      return true;
    })
    .sort((left, right) => {
      const riskDelta = (riskOrder[left.riskLevel] ?? 99) - (riskOrder[right.riskLevel] ?? 99);
      if (riskDelta !== 0) {
        return riskDelta;
      }
      return left.startMs - right.startMs;
    });
}

export function SegmentList({
  filters,
  segments,
  selectedSegmentId,
  selectedSegmentIds,
  onFilterChange,
  onResetFilters,
  onActivateSegment,
  onToggleSegment,
}: SegmentListProps) {
  const filteredSegments = filterAndSortSegments(segments, filters);
  const orderedIds = filteredSegments.map((segment) => segment.id);

  return (
    <div className="segment-review-panel">
      <div className="segment-review-panel__header">
        <div>
          <p className="start-screen__eyebrow">Segments</p>
          <h3>Danh sach exception</h3>
        </div>
      </div>

      <FilterBar
        filters={filters}
        matchCount={filteredSegments.length}
        totalCount={segments.length}
        onFilterChange={onFilterChange}
        onReset={onResetFilters}
      />

      {filteredSegments.length === 0 ? (
        <div className="segment-review-empty">
          Khong co doan nao khop voi bo loc hien tai.
        </div>
      ) : (
        <div className="segment-list">
          {filteredSegments.map((segment) => {
            const selected = selectedSegmentIds.includes(segment.id);
            const riskSlug = segment.riskLevel.toLowerCase();
            const reviewSlug = segment.reviewStatus.toLowerCase();

            return (
              <div
                key={segment.id}
                className={`segment-row${segment.id === selectedSegmentId ? " segment-row--active" : ""}`}
              >
                <div className="segment-row__check">
                  <input
                    type="checkbox"
                    checked={selected}
                    readOnly
                    onClick={(event) =>
                      onToggleSegment(segment.id, orderedIds, event.shiftKey)
                    }
                  />
                </div>
                <button
                  type="button"
                  className="segment-row__button"
                  onClick={() => onActivateSegment(segment.id)}
                >
                  <div className="segment-row__body">
                    <div className="segment-row__topline">
                      <strong>{formatRange(segment.startMs, segment.endMs)}</strong>
                      <span className={`mapping-pill mapping-pill--${riskSlug}`}>{segment.riskLevel}</span>
                    </div>
                    <div className="segment-row__meta">
                      <span>{segment.issueType}</span>
                      <span className={`segment-row__status segment-row__status--${reviewSlug}`}>
                        {segment.reviewStatus}
                      </span>
                    </div>
                    <p>{segment.message}</p>
                  </div>
                </button>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function formatRange(startMs: number, endMs: number | null) {
  const start = formatTime(startMs);
  const end = formatTime(endMs ?? startMs);
  return `${start}-${end}`;
}

function formatTime(value: number) {
  const totalSeconds = Math.max(0, Math.floor(value / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}
