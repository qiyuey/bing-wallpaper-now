// UI 图标配置
// 统一管理 SVG 图标尺寸和样式

/**
 * 图标尺寸配置
 */
export const ICON_SIZES = {
  /** 标准图标尺寸（按钮图标） */
  STANDARD: 20,
  /** 小图标尺寸 */
  SMALL: 16,
  /** 大图标尺寸 */
  LARGE: 24,
  /** 超大图标尺寸 */
  XLARGE: 32,
} as const;

/**
 * SVG 图标通用属性
 */
export const ICON_PROPS = {
  /** 标准图标尺寸配置 */
  standard: {
    width: ICON_SIZES.STANDARD,
    height: ICON_SIZES.STANDARD,
    viewBox: "0 0 24 24",
    strokeWidth: 2,
  },
  /** 小图标尺寸配置 */
  small: {
    width: ICON_SIZES.SMALL,
    height: ICON_SIZES.SMALL,
    viewBox: "0 0 16 16",
    strokeWidth: 2,
  },
} as const;

/**
 * 获取标准图标属性
 */
export function getStandardIconProps() {
  return { ...ICON_PROPS.standard };
}

/**
 * 获取小图标属性
 */
export function getSmallIconProps() {
  return { ...ICON_PROPS.small };
}
