import { useEffect, useState } from "react";
import { listen, TauriEvent } from "@tauri-apps/api/event";

type DragDropZoneProps = {
  onFilesDropped: (paths: string[]) => Promise<void>;
};

export function DragDropZone({ onFilesDropped }: DragDropZoneProps) {
  const [isDropActive, setIsDropActive] = useState(false);

  useEffect(() => {
    let mounted = true;

    const register = async () => {
      const unlistenDrop = await listen<unknown>(TauriEvent.DRAG_DROP, async (event) => {
        const payload = event.payload as { paths?: string[] } | string[];
        const paths = Array.isArray(payload) ? payload : payload?.paths ?? [];

        if (!mounted || paths.length === 0) {
          return;
        }

        setIsDropActive(false);
        await onFilesDropped(paths);
      });

      const unlistenEnter = await listen(TauriEvent.DRAG_ENTER, () => {
        if (mounted) {
          setIsDropActive(true);
        }
      });

      const unlistenLeave = await listen(TauriEvent.DRAG_LEAVE, () => {
        if (mounted) {
          setIsDropActive(false);
        }
      });

      return async () => {
        await Promise.all([unlistenDrop(), unlistenEnter(), unlistenLeave()]);
      };
    };

    const cleanupPromise = register();

    return () => {
      mounted = false;
      void cleanupPromise.then((cleanup) => cleanup());
    };
  }, [onFilesDropped]);

  return (
    <div className={`drop-zone${isDropActive ? " drop-zone--active" : ""}`}>
      <div className="drop-zone__inner">
        <p className="drop-zone__title">Kéo thả video, logo, audio và SRT vào đây</p>
        <p className="drop-zone__hint">
          Dùng file drop của Tauri để import trực tiếp từ máy local vào draft job.
        </p>
      </div>
    </div>
  );
}
