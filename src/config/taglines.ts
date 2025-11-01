// 动态标语配置
// 提供多种标语选项，可以根据时间、随机等方式显示
// 注意：标语会根据当前语言自动切换

import { detectSystemLanguage } from "../i18n/translations";

/**
 * 标语文案集合（中文）
 */
const TAGLINES_ZH = [
  "哪怕前路渺茫，也要让心中有光。",
  "生命充满劳碌，但依然要诗意地栖居，因为生活远比诗歌精彩。",
  "生活的不确定性正是我们希望的来源。",
  "碑是那么小，与其说是为了纪念，更像是为了忘却。",
  "即使被世界辜负，也不要辜负自己的热爱。",
  "重要的是你的目光，而不是你看见的东西。",
  "这世界并不缺少魅力，不缺少值得为之醒来的黎明。",
  "你的不同不是问题，而是答案。",
  "愿你在自己的路上，披星戴月，风雨兼程。",
  "山川是不卷收的文章，日月为你掌灯伴读。",
  "只要你行动，你的脑中自然会开始浮现计划，脚踏实地的感觉也会带给你自信。",
  "低谷是变好的开始，只要积蓄力量往前走，怎么走都是往上。",
  "我时常回到童年，用一片童心来思考问题，很多繁难的问题就变得易解。",
  "让自己进入一片雪，一片叶，一片云，让自己平和安乐是一种修行。",
  "你既无青春也无老年，而只像饭后的一场睡眠，把两者梦见。",
  "每一次困难的中心，都蕴藏着机会。",
  "愿你的视觉时刻都新。智者，即对一切事物都感到新奇的人。",
  "不是在一瞬间就能脱胎换骨的，生命原是一次又一次的试探。",
  "雪珠声声入耳，一如古柏，我身依然故我。",
  "我们等待着戈多，在等待的过程中发现戈多就是等待本身。",
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
    return taglines[Math.floor(Math.random() * 7)]; // 随机选择前7个
  } else if (currentHour >= 12 && currentHour < 18) {
    // 下午（12:00-18:00）：使用探索发现的文案
    return taglines[7 + Math.floor(Math.random() * 7)]; // 随机选择中间7个
  } else {
    // 晚上（18:00-6:00）：使用安静温馨的文案
    return taglines[14 + Math.floor(Math.random() * 6)]; // 随机选择后6个
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
