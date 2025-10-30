import { useState, useEffect } from "react";
import { getCurrentTagline, getDailyTagline, TAGLINES } from "../config/taglines";

/**
 * 动态标语模式
 */
export type TaglineMode = "time-based" | "daily" | "random";

/**
 * 使用动态标语的 Hook
 * @param {TaglineMode} mode 标语显示模式
 *   - "time-based": 根据时间段显示不同文案（每小时变化）
 *   - "daily": 每天固定一个文案（当天不变）
 *   - "random": 完全随机（每次刷新变化）
 * @param {number} updateInterval 更新间隔（毫秒），默认 60000（1分钟）
 * @returns {string} 当前标语
 */
export function useDynamicTagline(
  mode: TaglineMode = "time-based",
  updateInterval: number = 60000
): string {
  const [tagline, setTagline] = useState<string>(() => {
    switch (mode) {
      case "time-based":
        return getCurrentTagline();
      case "daily":
        return getDailyTagline();
      case "random":
        return TAGLINES[Math.floor(Math.random() * TAGLINES.length)];
      default:
        return TAGLINES[0];
    }
  });

  useEffect(() => {
    // 如果模式是 daily，不需要定时更新
    if (mode === "daily") {
      return;
    }

    // 设置定时器更新标语
    const intervalId = setInterval(() => {
      switch (mode) {
        case "time-based":
          setTagline(getCurrentTagline());
          break;
        case "random":
          setTagline(TAGLINES[Math.floor(Math.random() * TAGLINES.length)]);
          break;
      }
    }, updateInterval);

    return () => {
      clearInterval(intervalId);
    };
  }, [mode, updateInterval]);

  return tagline;
}

