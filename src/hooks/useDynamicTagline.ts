import { useState, useEffect } from "react";
import { getCurrentTagline, getDailyTagline } from "../config/taglines";
import { detectSystemLanguage } from "../i18n/translations";

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
 * @param {"zh-CN" | "en-US"} lang 语言代码，如果不提供则自动检测
 * @returns {string} 当前标语
 */
export function useDynamicTagline(
  mode: TaglineMode = "time-based",
  updateInterval: number = 60000,
  lang?: "zh-CN" | "en-US"
): string {
  const currentLang = lang || detectSystemLanguage();
  
  const [tagline, setTagline] = useState<string>(() => {
    switch (mode) {
      case "time-based":
        return getCurrentTagline(undefined, currentLang);
      case "daily":
        return getDailyTagline(currentLang);
      case "random":
        const taglines = currentLang === "zh-CN" 
          ? ["世界之美 · 每日相遇", "探索世界的每一个角落", "让每一天都有新的开始"]
          : ["Beauty of the World · Daily Encounter", "Explore Every Corner of the World", "A New Beginning Every Day"];
        return taglines[Math.floor(Math.random() * taglines.length)];
      default:
        return getCurrentTagline(undefined, currentLang);
    }
  });

  // 当语言变化时，立即更新标语
  useEffect(() => {
    switch (mode) {
      case "time-based":
        setTagline(getCurrentTagline(undefined, currentLang));
        break;
      case "daily":
        setTagline(getDailyTagline(currentLang));
        break;
      case "random":
        const taglines = currentLang === "zh-CN" 
          ? ["世界之美 · 每日相遇", "探索世界的每一个角落", "让每一天都有新的开始"]
          : ["Beauty of the World · Daily Encounter", "Explore Every Corner of the World", "A New Beginning Every Day"];
        setTagline(taglines[Math.floor(Math.random() * taglines.length)]);
        break;
    }
  }, [mode, currentLang]);

  useEffect(() => {
    // 如果模式是 daily，不需要定时更新
    if (mode === "daily") {
      return;
    }
    
    // 设置定时器更新标语
    const intervalId = setInterval(() => {
      switch (mode) {
        case "time-based":
          setTagline(getCurrentTagline(undefined, currentLang));
          break;
        case "random":
          const taglines = currentLang === "zh-CN" 
            ? ["世界之美 · 每日相遇", "探索世界的每一个角落", "让每一天都有新的开始"]
            : ["Beauty of the World · Daily Encounter", "Explore Every Corner of the World", "A New Beginning Every Day"];
          setTagline(taglines[Math.floor(Math.random() * taglines.length)]);
          break;
      }
    }, updateInterval);

    return () => {
      clearInterval(intervalId);
    };
  }, [mode, updateInterval, currentLang]);

  return tagline;
}

