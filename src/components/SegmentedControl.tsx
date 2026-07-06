import {
  useRef,
  useLayoutEffect,
  useState,
  useMemo,
  useCallback,
  type ReactNode,
} from "react";
import styles from "./SegmentedControl.module.css";

interface SegmentedControlProps<T extends string> {
  name?: string;
  options: { value: T; label: ReactNode }[];
  value: T;
  onChange: (value: T) => void;
}

export function SegmentedControl<T extends string>({
  name,
  options,
  value,
  onChange,
}: SegmentedControlProps<T>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [indicator, setIndicator] = useState({ left: 0, width: 0 });

  const optionValues = useMemo(() => options.map((o) => o.value), [options]);

  const updateIndicator = useCallback(() => {
    const container = containerRef.current;
    if (!container) return [];

    const idx = optionValues.indexOf(value);
    const items = container.querySelectorAll<HTMLElement>(`.${styles.option}`);
    const item = items[idx];

    if (item) {
      const next = {
        left: item.offsetLeft,
        width: item.offsetWidth,
      };
      setIndicator((prev) => {
        if (prev.left === next.left && prev.width === next.width) {
          return prev;
        }
        return next;
      });
    }

    return Array.from(items);
  }, [optionValues, value]);

  useLayoutEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const items = updateIndicator();
    const resizeObserver = new ResizeObserver(() => {
      updateIndicator();
    });

    resizeObserver.observe(container);
    items.forEach((item) => resizeObserver.observe(item));

    return () => {
      resizeObserver.disconnect();
    };
  }, [updateIndicator]);

  return (
    <div className={styles.container} ref={containerRef}>
      <div
        className={styles.indicator}
        style={{
          transform: `translateX(${indicator.left}px)`,
          width: `${indicator.width}px`,
        }}
      />
      {options.map((opt) => (
        <label
          key={opt.value}
          className={`${styles.option} ${opt.value === value ? styles.active : ""}`}
        >
          <input
            type="radio"
            name={name}
            checked={opt.value === value}
            onChange={() => onChange(opt.value)}
            className={styles.input}
          />
          <span>{opt.label}</span>
        </label>
      ))}
    </div>
  );
}
