import {
  memo,
  useCallback,
  useState,
  useEffect,
  useRef,
  type CSSProperties,
} from "react";
import { List, RowComponentProps, useDynamicRowHeight } from "react-window";
import { WallpaperCard } from "./WallpaperCard";
import { LocalWallpaper } from "../types";
import {
  CARDS_PER_ROW,
  calculateRowHeight,
  getCardsPerRow,
} from "../config/layout";
import { CONTAINER } from "../config/ui";
import { useI18n } from "../i18n/I18nContext";
import styles from "./WallpaperGrid.module.css";
import spinnerStyles from "../styles/spinner.module.css";

interface WallpaperGridProps {
  wallpapers: LocalWallpaper[];
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  loading?: boolean;
  wallpaperDirectory: string;
}

// 仅作为 useDynamicRowHeight 的 defaultRowHeight 初值
// 真实行高由 react-window 内部 ResizeObserver 测量，避免计算误差
const DEFAULT_ROW_HEIGHT = calculateRowHeight();

// 骨架屏组件
const SkeletonCard = memo(() => (
  <div className={styles.cardSkeleton}>
    <div className={styles.skeletonImage} />
    <div className={`${styles.skeletonText} ${styles.skeletonTextTitle}`} />
    <div className={`${styles.skeletonText} ${styles.skeletonTextSubtitle}`} />
    <div className={styles.skeletonButton} />
  </div>
));

SkeletonCard.displayName = "SkeletonCard";

// Row 渲染组件的额外数据类型
interface RowData {
  wallpapers: LocalWallpaper[];
  cardsPerRow: number;
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  wallpaperDirectory: string;
}

// Row 提取到模块作用域，避免每次 WallpaperGrid 渲染时创建新函数引用，
// 否则 react-window 内部 memo(Row) 会失效，导致整行频繁卸载/重挂载。
function Row({
  index,
  style,
  wallpapers,
  cardsPerRow,
  onSetWallpaper,
  wallpaperDirectory,
}: RowComponentProps<RowData>) {
  const startIndex = index * cardsPerRow;
  const rowWallpapers = wallpapers.slice(startIndex, startIndex + cardsPerRow);

  // 把列数通过 CSS 自定义属性透传给 .wallpaper-row，
  // 让 CSS Grid 的列数与 React 的数据切片完全同步（避免 auto-fill 错位）
  const rowStyle = {
    ...style,
    "--cards-per-row": cardsPerRow,
  } as CSSProperties;

  return (
    <div style={rowStyle} className={styles.row}>
      {rowWallpapers.map((wallpaper) => (
        <div
          key={`${wallpaper.end_date}-${index}`}
          className={styles.rowItem}
        >
          <WallpaperCard
            wallpaper={wallpaper}
            onSetWallpaper={onSetWallpaper}
            wallpaperDirectory={wallpaperDirectory}
          />
        </div>
      ))}
    </div>
  );
}

export const WallpaperGrid = memo(function WallpaperGrid({
  wallpapers,
  onSetWallpaper,
  loading = false,
  wallpaperDirectory,
}: WallpaperGridProps) {
  const { t } = useI18n();
  const containerRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(0);
  const [containerHeight, setContainerHeight] = useState(
    CONTAINER.DEFAULT_HEIGHT,
  );
  const [cardsPerRow, setCardsPerRow] = useState<number>(CARDS_PER_ROW.DESKTOP);

  // 监听容器尺寸变化（使用 ResizeObserver 自动获取可用高度）
  useEffect(() => {
    const updateSize = () => {
      if (containerRef.current) {
        const width = containerRef.current.offsetWidth;
        const height = containerRef.current.offsetHeight;
        setContainerWidth(width);
        setContainerHeight(height);

        // 使用配置函数根据宽度决定每行卡片数
        setCardsPerRow(getCardsPerRow(width));
      }
    };

    updateSize();

    // 使用 ResizeObserver 监听容器尺寸变化
    const resizeObserver = new ResizeObserver(() => {
      updateSize();
    });

    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    // 也监听窗口 resize 作为备份
    window.addEventListener("resize", updateSize);

    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("resize", updateSize);
    };
  }, []);

  const handleSetWallpaper = useCallback(
    (wallpaper: LocalWallpaper) => {
      onSetWallpaper(wallpaper);
    },
    [onSetWallpaper],
  );

  // 通过 ResizeObserver 自动测量每行实际渲染高度，
  // 切换 cardsPerRow（窗口宽度跨断点）时以 key 强制清空缓存重新测量，
  // 避免固定 ROW_HEIGHT 与实际渲染高度不一致引起的行间"幻影/重叠"。
  const rowHeight = useDynamicRowHeight({
    defaultRowHeight: DEFAULT_ROW_HEIGHT,
    key: cardsPerRow,
  });

  if (loading) {
    return (
      <div ref={containerRef} className={styles.container}>
        <div className={styles.gridLoading}>
          <div className={spinnerStyles.spinner} data-testid="spinner"></div>
        </div>
      </div>
    );
  }

  if (wallpapers.length === 0) {
    return (
      <div ref={containerRef} className={styles.container}>
        <div className={styles.gridEmpty}>
          <div className={styles.gridEmptyIcon}>🖼️</div>
          <p>{t("noWallpapers")}</p>
          <p className={styles.gridEmptyHint}>{t("noWallpapersHint")}</p>
        </div>
      </div>
    );
  }

  // 计算总行数
  const rowCount = Math.ceil(wallpapers.length / cardsPerRow);

  return (
    <div ref={containerRef} className={styles.container}>
      {containerWidth > 0 && containerHeight > 0 && (
        <List<RowData>
          rowCount={rowCount}
          rowHeight={rowHeight}
          className={styles.virtualList}
          style={{ width: containerWidth, height: containerHeight }}
          rowComponent={Row}
          rowProps={{
            wallpapers,
            cardsPerRow,
            onSetWallpaper: handleSetWallpaper,
            wallpaperDirectory,
          }}
        />
      )}
    </div>
  );
});
