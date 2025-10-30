// 动态标语配置
// 提供多种标语选项，可以根据时间、随机等方式显示
// 注意：标语会根据当前语言自动切换

import { detectSystemLanguage } from "../i18n/translations";

/**
 * 标语文案集合（中文）
 */
const TAGLINES_ZH = [
  "世界之美 · 每日相遇",
  "探索世界的每一个角落",
  "让每一天都有新的开始",
  "发现生活中的美好瞬间",
  "每一张壁纸，都是一个新的故事",
  "用美丽的画面开启新的一天",
  "让灵感从桌面开始",
  "世界那么大，每天看一看",
  "收藏世界的每一处风景",
  "用色彩装点你的每一天",
] as const;

/**
 * 标语文案集合（英文）
 */
const TAGLINES_EN = [
  "Beauty of the World · Daily Encounter",
  "Explore Every Corner of the World",
  "A New Beginning Every Day",
  "Discover Beautiful Moments in Life",
  "Every Wallpaper Tells a New Story",
  "Start Each Day with Beautiful Images",
  "Let Inspiration Begin from Your Desktop",
  "The World is Big, See It Every Day",
  "Collect Every Scenic View of the World",
  "Color Your Every Day",
] as const;

/**
 * 获取当前应该显示的标语
 * @param {number} hour 当前小时数（0-23），如果不提供则使用当前时间
 * @param {"zh-CN" | "en-US"} lang 语言代码
 * @returns {string} 标语文本
 */
export function getCurrentTagline(
  hour?: number,
  lang?: "zh-CN" | "en-US",
): string {
  const currentHour = hour ?? new Date().getHours();
  const language = lang || detectSystemLanguage();
  const taglines = language === "zh-CN" ? TAGLINES_ZH : TAGLINES_EN;

  // 根据时间段选择不同的标语策略
  if (currentHour >= 6 && currentHour < 12) {
    // 早上（6:00-12:00）：使用积极向上的文案
    return taglines[Math.floor(Math.random() * 3)]; // 随机选择前3个
  } else if (currentHour >= 12 && currentHour < 18) {
    // 下午（12:00-18:00）：使用探索发现的文案
    return taglines[3 + Math.floor(Math.random() * 3)]; // 随机选择中间3个
  } else {
    // 晚上（18:00-6:00）：使用安静温馨的文案
    return taglines[6 + Math.floor(Math.random() * 4)]; // 随机选择后4个
  }
}

/**
 * 根据日期种子获取当天的标语（每天固定，但不同日期不同）
 * @param {"zh-CN" | "en-US"} lang 语言代码
 * @returns {string} 标语文本
 */
export function getDailyTagline(lang?: "zh-CN" | "en-US"): string {
  const today = new Date();
  const dayOfYear = Math.floor(
    (today.getTime() - new Date(today.getFullYear(), 0, 0).getTime()) /
      1000 /
      60 /
      60 /
      24,
  );

  const language = lang || detectSystemLanguage();
  const taglines = language === "zh-CN" ? TAGLINES_ZH : TAGLINES_EN;

  // 使用日期作为种子，确保同一天显示相同的标语
  const seed = dayOfYear % taglines.length;
  return taglines[seed];
}
