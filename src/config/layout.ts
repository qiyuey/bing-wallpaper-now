// 布局配置常量
// 集中管理响应式断点和卡片布局配置

/**
 * 响应式断点配置
 */
export const BREAKPOINTS = {
  /** 极窄窗口（≤750px）显示1张 */
  NARROW: 750,
  /** 窄窗口（751-1024px）显示2张 */
  TABLET: 1024,
  /** 大屏幕笔记本及以上（≥1400px）显示4张 */
  DESKTOP_4K: 1400,
} as const;

/**
 * 每行卡片数量配置
 */
export const CARDS_PER_ROW = {
  /** 大屏幕笔记本及以上每行4张 */
  FOUR_K: 4,
  /** 桌面端每行3张 */
  DESKTOP: 3,
  /** 窄窗口每行2张 */
  NARROW: 2,
  /** 极窄窗口每行1张 */
  SINGLE: 1,
} as const;

/**
 * 卡片尺寸配置
 */
export const CARD_DIMENSIONS = {
  /** 卡片图片高度 */
  IMAGE_HEIGHT: 240,
  /** 卡片内边距 */
  PADDING_X: 24, // 1.5rem
  PADDING_Y: 20, // 1.25rem
  /** 卡片信息区域 */
  INFO_PADDING_X: 24, // 1.5rem
  INFO_PADDING_Y: 20, // 1.25rem
  /** 卡片操作区域 */
  ACTIONS_PADDING_X: 24, // 1.5rem
  ACTIONS_PADDING_BOTTOM: 24, // 1.5rem
  /** 标题字体大小 */
  TITLE_FONT_SIZE: 18, // 1.125rem
  TITLE_LINE_HEIGHT: 1.3,
  TITLE_MARGIN_BOTTOM: 6, // 0.375rem
  /** 副标题字体大小 */
  SUBTITLE_FONT_SIZE: 13, // 0.8125rem
  /** 副标题行高（继承 :root 的 line-height: 1.6） */
  SUBTITLE_LINE_HEIGHT: 1.6,
  /** 按钮高度 */
  BUTTON_HEIGHT: 38,
} as const;

/**
 * 间距配置
 */
export const SPACING = {
  /** 卡片行间距 */
  ROW_GAP_NARROW: 16, // 1rem
  ROW_GAP_DESKTOP: 32, // 2rem
  /** 行底部间距（减小以让卡片更紧凑） */
  ROW_MARGIN_BOTTOM: 8, // 0.5rem
  /** 行底部内边距（减小以让卡片更紧凑） */
  ROW_PADDING_BOTTOM: 8, // 0.5rem
} as const;

/**
 * 计算虚拟列表行高
 * 公式：图片高度 + 信息区域 + 操作区域 + 行间距
 *
 * 详细计算：
 * - 图片高度：240px
 * - 信息区域：顶部内边距 + 标题高度 + 标题间距 + 副标题高度 + 底部内边距
 * - 操作区域：底部内边距 + 按钮高度
 * - 行间距：行底部边距 + 行底部内边距（8px + 8px = 16px）
 */
export function calculateRowHeight(): number {
  const imageHeight = CARD_DIMENSIONS.IMAGE_HEIGHT;

  // 信息区域高度
  const infoHeight =
    CARD_DIMENSIONS.INFO_PADDING_Y * 2 + // 上下内边距
    Math.ceil(
      CARD_DIMENSIONS.TITLE_FONT_SIZE * CARD_DIMENSIONS.TITLE_LINE_HEIGHT,
    ) + // 标题实际高度（考虑 line-height）
    CARD_DIMENSIONS.TITLE_MARGIN_BOTTOM + // 标题底部间距
    Math.ceil(
      CARD_DIMENSIONS.SUBTITLE_FONT_SIZE * CARD_DIMENSIONS.SUBTITLE_LINE_HEIGHT,
    ); // 副标题实际高度（考虑继承的 line-height: 1.6）

  // 操作区域高度
  const actionsHeight =
    CARD_DIMENSIONS.ACTIONS_PADDING_BOTTOM + CARD_DIMENSIONS.BUTTON_HEIGHT;

  // 行间距
  const rowSpacing = SPACING.ROW_MARGIN_BOTTOM + SPACING.ROW_PADDING_BOTTOM;

  return imageHeight + infoHeight + actionsHeight + rowSpacing;
}

/**
 * 根据窗口宽度决定每行卡片数
 */
export function getCardsPerRow(width: number): number {
  if (width <= BREAKPOINTS.NARROW) {
    return CARDS_PER_ROW.SINGLE;
  } else if (width <= BREAKPOINTS.TABLET) {
    return CARDS_PER_ROW.NARROW;
  } else if (width >= BREAKPOINTS.DESKTOP_4K) {
    return CARDS_PER_ROW.FOUR_K;
  } else {
    return CARDS_PER_ROW.DESKTOP;
  }
}
