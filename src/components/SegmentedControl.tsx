import { useRef, useEffect, useState, useMemo, type ReactNode } from "react";
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

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    const idx = optionValues.indexOf(value);
    const items = container.querySelectorAll<HTMLElement>(`.${styles.option}`);
    const item = items[idx];
    if (item) {
      setIndicator({
        left: item.offsetLeft,
        width: item.offsetWidth,
      });
    }
  }, [value, optionValues]);

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
