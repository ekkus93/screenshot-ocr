import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";

export function useWindowControls() {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    const window = getCurrentWindow();
    let unlisten: (() => void) | undefined;
    void window.isMaximized().then(setIsMaximized);
    void window
      .onResized(() => {
        void window.isMaximized().then(setIsMaximized);
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, []);

  const minimize = useCallback(() => {
    void getCurrentWindow().minimize();
  }, []);

  const toggleMaximize = useCallback(() => {
    void getCurrentWindow().toggleMaximize();
  }, []);

  const close = useCallback(() => {
    void getCurrentWindow().close();
  }, []);

  return { isMaximized, minimize, toggleMaximize, close };
}
