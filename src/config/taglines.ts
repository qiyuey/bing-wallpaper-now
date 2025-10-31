// 动态标语配置
// 提供多种标语选项，可以根据时间、随机等方式显示
// 注意：标语会根据当前语言自动切换

import { detectSystemLanguage } from "../i18n/translations";

/**
 * 标语文案集合（中文）
 */
const TAGLINES_ZH = [
  "哪怕前路渺茫，也要让心中有光。",
  "世界以痛吻我，要我报之以歌。",
  "不要因为走得太远，而忘了当初为什么出发。",
  "生活不止眼前的苟且，还有诗和远方。",
  "即使被世界辜负，也不要辜负自己的热爱。",
  "黑夜给了我黑色的眼睛，我却用它寻找光明。",
  "凡心所向，素履以往。生如逆旅，一苇以航。",
  "我不能选择怎么生，怎么死，但我能决定怎么爱，怎么活。",
  "生活不能等待别人来安排，要自己去争取和奋斗。",
  "山川是不卷收的文章，日月为你掌灯伴读。",
] as const;

/**
 * 标语文案集合（英文）
 */
const TAGLINES_EN = [
  "Not all those who wander are lost.",
  "We are all in the gutter, but some of us are looking at the stars.",
  "Stay hungry, stay foolish.",
  "To strive, to seek, to find, and not to yield.",
  "The only journey is the one within.",
  "Do not go gentle into that good night.",
  "The future belongs to those who believe in the beauty of their dreams.",
  "Perhaps all the dragons in our lives are princesses who are only waiting to see us act, just once, with beauty and courage.",
  "And the end of all our exploring will be to arrive where we started and to know the place for the first time.",
  "Not I, nor anyone else can travel that road for you. You must travel it by yourself.",
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

/**
 * 获取所有标语数组
 * @param {"zh-CN" | "en-US"} lang 语言代码
 * @returns {readonly string[]} 标语数组
 */
export function getAllTaglines(lang?: "zh-CN" | "en-US"): readonly string[] {
  const language = lang || detectSystemLanguage();
  return language === "zh-CN" ? TAGLINES_ZH : TAGLINES_EN;
}
