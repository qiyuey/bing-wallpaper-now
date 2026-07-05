import { useEffect, useRef, RefObject } from "react";

export function useLiquidGlass<T extends HTMLElement>(): RefObject<T | null> {
  const ref = useRef<T>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (window.matchMedia("(pointer: coarse)").matches) return;

    const onMove = (e: globalThis.MouseEvent) => {
      const { left, top, width, height } = el.getBoundingClientRect();
      const deg =
        Math.atan2(
          e.clientY - (top + height / 2),
          e.clientX - (left + width / 2),
        ) *
        (180 / Math.PI);
      el.style.setProperty("--glass-angle", `${deg}deg`);
    };

    document.addEventListener("mousemove", onMove, { passive: true });
    return () => document.removeEventListener("mousemove", onMove);
  }, []);

  return ref;
}
