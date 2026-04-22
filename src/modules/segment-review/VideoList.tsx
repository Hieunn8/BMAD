import type { ReviewVideoSummary } from "../start-flow/types";

type VideoListProps = {
  showAllVideos: boolean;
  selectedVideoId: string | null;
  videoList: ReviewVideoSummary[];
  onToggleShowAll: (next: boolean) => void;
  onSelectVideo: (videoId: string) => void;
};

export function VideoList({
  showAllVideos,
  selectedVideoId,
  videoList,
  onToggleShowAll,
  onSelectVideo,
}: VideoListProps) {
  const selectedVideo = videoList.find((video) => video.videoId === selectedVideoId);

  return (
    <div className="segment-review-panel">
      <div className="segment-review-panel__header">
        <div>
          <p className="start-screen__eyebrow">Review Queue</p>
          <h3>Video can xem</h3>
        </div>
        <label className="segment-review-toggle">
          <input
            type="checkbox"
            checked={showAllVideos}
            onChange={(event) => onToggleShowAll(event.target.checked)}
          />
          <span>Hien thi tat ca video</span>
        </label>
      </div>

      {selectedVideo && !selectedVideo.reviewRequired ? (
        <div className="segment-review-banner">
          Khong co doan bat buoc phai sua. Day la review tuy chon.
        </div>
      ) : null}

      <div className="review-video-list">
        {videoList.map((video) => (
          <button
            key={video.videoId}
            type="button"
            className={`review-video-list__item${video.videoId === selectedVideoId ? " review-video-list__item--active" : ""}`}
            onClick={() => onSelectVideo(video.videoId)}
          >
            <div>
              <strong>{video.videoName}</strong>
              <p>{video.status}</p>
            </div>
            <div className="review-video-list__meta">
              <span className={`mapping-pill mapping-pill--${video.reviewRequired ? "blocked" : "ready"}`}>
                {video.reviewRequired ? "Can review" : "Optional"}
              </span>
              <span className="inline-note">{video.segmentCount} segment</span>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
