use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::wallpaper::LocalWallpaper;

/// 壁纸元数据索引（单一文件存储）
///
/// 索引版本号说明：
/// - v4: 使用短字段名和紧凑格式，壁纸按 `wallpapers_by_language` 分组
/// - v5: 将 `wallpapers_by_language` 重命名为 `mkt`，语义更准确
///
/// 迁移说明：
/// - v4 → v5：自动备份旧文件为 `index.json.v4.bak`，将 `wallpapers_by_language` 迁移为 `mkt`
/// - 通过 `#[serde(alias = "wallpapers_by_language")]` 保证反序列化兼容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperIndex {
    /// 版本号（用于兼容性检查）
    pub version: u32,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 按市场代码（mkt）分组的壁纸列表
    /// 外层 key = mkt（如 "zh-CN", "en-US", "ja-JP"），内层 key = end_date
    /// 使用 end_date 作为 key，因为文件名也使用 end_date（Bing 的 startdate 是昨天，enddate 才是今天）
    /// 使用 IndexMap 以保持插入顺序，确保 JSON 序列化时按日期排序
    #[serde(alias = "wallpapers_by_language")]
    pub mkt: IndexMap<String, IndexMap<String, LocalWallpaper>>,
}

impl Default for WallpaperIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperIndex {
    /// 索引版本常量
    ///
    /// v4: 使用短字段名和紧凑格式
    /// v5: wallpapers_by_language → mkt
    pub const VERSION: u32 = 5;

    /// 支持从此版本迁移升级（v4 → v5）
    pub const MIGRATE_FROM_VERSION: u32 = 4;

    /// 创建新索引
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            last_updated: Utc::now(),
            mkt: IndexMap::new(),
        }
    }

    /// 获取指定 mkt 的壁纸列表
    pub fn get_wallpapers_for_mkt(&self, mkt: &str) -> Vec<LocalWallpaper> {
        self.mkt
            .get(mkt)
            .map(|wp_map| {
                let mut wallpapers: Vec<_> = wp_map.values().cloned().collect();
                wallpapers.sort_by(|a, b| b.end_date.cmp(&a.end_date));
                wallpapers
            })
            .unwrap_or_default()
    }

    /// 批量添加或更新指定 mkt 的壁纸
    ///
    /// 插入时会按日期降序排序，确保 JSON 序列化时保持顺序。
    /// 返回实际新增的条目数（不含覆盖已存在的条目）。
    pub fn upsert_wallpapers_for_mkt(
        &mut self,
        mkt: &str,
        wallpapers: Vec<LocalWallpaper>,
    ) -> usize {
        if wallpapers.is_empty() {
            return 0;
        }
        let mkt_map = self.mkt.entry(mkt.to_string()).or_default();

        let mut new_count = 0;
        for wallpaper in wallpapers {
            let key = wallpaper.end_date.clone();
            if !mkt_map.contains_key(&key) {
                new_count += 1;
            }
            mkt_map.insert(key, wallpaper);
        }

        // 按日期降序排序（最新的在前）
        mkt_map.sort_by(|k1, _, k2, _| k2.cmp(k1));

        // 对外层（mkt）也按字典序排序，确保 JSON 中的 mkt 顺序一致
        self.mkt.sort_keys();

        self.last_updated = Utc::now();
        new_count
    }

    /// 对所有 mkt 和日期进行排序，确保 JSON 序列化时保持顺序
    pub fn sort_all(&mut self) {
        // 对每个 mkt 的壁纸按日期降序排序
        for mkt_wallpapers in self.mkt.values_mut() {
            mkt_wallpapers.sort_by(|k1, _, k2, _| k2.cmp(k1));
        }
        // 对外层（mkt）按字典序排序
        self.mkt.sort_keys();
    }

    /// 获取所有语言的壁纸（用于清理操作）
    /// 返回所有语言中唯一的 end_date 对应的壁纸列表
    /// 如果有多个语言存在相同 end_date，优先选择字典序靠前的语言
    pub fn get_all_wallpapers_unique(&self) -> Vec<LocalWallpaper> {
        use std::collections::{BTreeMap, HashSet};
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        // 使用 BTreeMap 按语言代码排序，确保一致性
        let lang_order: BTreeMap<_, _> = self.mkt.iter().collect();

        // 按语言代码顺序遍历，优先选择字典序靠前的语言
        for (_, lang_wallpapers) in lang_order {
            for wallpaper in lang_wallpapers.values() {
                if seen.insert(wallpaper.end_date.clone()) {
                    result.push(wallpaper.clone());
                }
            }
        }

        // 按 end_date 降序排序（最新的在前）
        result.sort_by(|a, b| b.end_date.cmp(&a.end_date));
        result
    }

    /// 限制索引大小，保留最新的条目
    ///
    /// 如果索引总数超过 `max_count`，会删除最旧的条目。
    /// 优先保留最新的条目，按 end_date 降序排序。
    ///
    /// # Arguments
    /// * `max_count` - 最大索引数量
    pub fn limit_index_size(&mut self, max_count: usize) {
        // 获取所有唯一的 end_date，按降序排序（最新的在前）
        let all_unique = self.get_all_wallpapers_unique();

        // 如果总数不超过限制，不需要清理
        if all_unique.len() <= max_count {
            return;
        }

        // 需要删除的 end_date 列表（最旧的）
        let to_remove: Vec<String> = all_unique
            .iter()
            .skip(max_count)
            .map(|w| w.end_date.clone())
            .collect();

        log::info!(
            "索引数据超过限制 ({} > {})，删除 {} 条最旧的索引条目",
            all_unique.len(),
            max_count,
            to_remove.len()
        );

        // 从所有语言中删除这些 end_date
        for lang_wallpapers in self.mkt.values_mut() {
            for end_date in &to_remove {
                lang_wallpapers.shift_remove(end_date);
            }
        }

        // 移除空的语言分组
        self.mkt
            .retain(|_, lang_wallpapers| !lang_wallpapers.is_empty());

        self.last_updated = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助函数：创建一个 LocalWallpaper
    fn make_wallpaper(end_date: &str, title: &str) -> LocalWallpaper {
        LocalWallpaper {
            title: title.to_string(),
            copyright: format!("Copyright for {}", title),
            copyright_link: "https://example.com".to_string(),
            end_date: end_date.to_string(),
            urlbase: format!("/th?id=OHR.{}", title),
        }
    }

    #[test]
    fn test_wallpaper_index_new() {
        let index = WallpaperIndex::new();
        assert_eq!(index.version, WallpaperIndex::VERSION);
        assert!(index.mkt.is_empty());
    }

    #[test]
    fn test_wallpaper_index_default() {
        let index = WallpaperIndex::default();
        assert_eq!(index.version, WallpaperIndex::VERSION);
        assert!(index.mkt.is_empty());
    }

    #[test]
    fn test_get_wallpapers_for_mkt_empty() {
        let index = WallpaperIndex::new();
        let wallpapers = index.get_wallpapers_for_mkt("zh-CN");
        assert!(wallpapers.is_empty());
    }

    #[test]
    fn test_get_wallpapers_for_mkt_nonexistent() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt("zh-CN", vec![make_wallpaper("20240102", "Test")]);
        // 查询不存在的 mkt 应返回空
        let wallpapers = index.get_wallpapers_for_mkt("en-US");
        assert!(wallpapers.is_empty());
    }

    #[test]
    fn test_get_wallpapers_for_mkt_returns_sorted_desc() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240101", "Old"),
                make_wallpaper("20240103", "New"),
                make_wallpaper("20240102", "Mid"),
            ],
        );

        let wallpapers = index.get_wallpapers_for_mkt("zh-CN");
        assert_eq!(wallpapers.len(), 3);
        assert_eq!(wallpapers[0].end_date, "20240103"); // 最新在前
        assert_eq!(wallpapers[1].end_date, "20240102");
        assert_eq!(wallpapers[2].end_date, "20240101");
    }

    #[test]
    fn test_upsert_wallpapers_for_mkt_empty_vec() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt("zh-CN", vec![]);
        // 空插入不应创建 mkt 条目
        assert!(index.mkt.is_empty());
    }

    #[test]
    fn test_upsert_wallpapers_for_mkt_dedup_by_end_date() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240102", "First"),
                make_wallpaper("20240102", "Second"), // 同一 end_date
            ],
        );

        let wallpapers = index.get_wallpapers_for_mkt("zh-CN");
        assert_eq!(wallpapers.len(), 1);
        // 后插入的应覆盖先插入的
        assert_eq!(wallpapers[0].title, "Second");
    }

    #[test]
    fn test_upsert_wallpapers_for_mkt_update_existing() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt("zh-CN", vec![make_wallpaper("20240102", "Original")]);

        // 更新同一 end_date 的壁纸
        index.upsert_wallpapers_for_mkt("zh-CN", vec![make_wallpaper("20240102", "Updated")]);

        let wallpapers = index.get_wallpapers_for_mkt("zh-CN");
        assert_eq!(wallpapers.len(), 1);
        assert_eq!(wallpapers[0].title, "Updated");
    }

    #[test]
    fn test_upsert_wallpapers_for_mkt_sorts_mkt_keys() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt("zh-CN", vec![make_wallpaper("20240102", "ZH")]);
        index.upsert_wallpapers_for_mkt("en-US", vec![make_wallpaper("20240102", "EN")]);

        let keys: Vec<&String> = index.mkt.keys().collect();
        // 外层 mkt 应按字典序排列
        assert_eq!(keys, vec!["en-US", "zh-CN"]);
    }

    #[test]
    fn test_sort_all() {
        let mut index = WallpaperIndex::new();

        // 先插入 zh-CN 再插入 en-US
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240101", "Old"),
                make_wallpaper("20240103", "New"),
            ],
        );
        index.upsert_wallpapers_for_mkt(
            "en-US",
            vec![
                make_wallpaper("20240101", "Old EN"),
                make_wallpaper("20240102", "Mid EN"),
            ],
        );

        // sort_all 应对每个 mkt 内按日期降序，外层按 mkt 字典序
        index.sort_all();

        let keys: Vec<&String> = index.mkt.keys().collect();
        assert_eq!(keys, vec!["en-US", "zh-CN"]);

        // 验证 zh-CN 内部顺序
        let zh_dates: Vec<&String> = index.mkt["zh-CN"].keys().collect();
        assert_eq!(zh_dates, vec!["20240103", "20240101"]);

        // 验证 en-US 内部顺序
        let en_dates: Vec<&String> = index.mkt["en-US"].keys().collect();
        assert_eq!(en_dates, vec!["20240102", "20240101"]);
    }

    #[test]
    fn test_get_all_wallpapers_unique_single_mkt() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240101", "Day1"),
                make_wallpaper("20240102", "Day2"),
                make_wallpaper("20240103", "Day3"),
            ],
        );

        let unique = index.get_all_wallpapers_unique();
        assert_eq!(unique.len(), 3);
        // 应按 end_date 降序排列
        assert_eq!(unique[0].end_date, "20240103");
        assert_eq!(unique[1].end_date, "20240102");
        assert_eq!(unique[2].end_date, "20240101");
    }

    #[test]
    fn test_get_all_wallpapers_unique_cross_mkt_dedup() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240101", "ZH Day1"),
                make_wallpaper("20240102", "ZH Day2"),
            ],
        );
        index.upsert_wallpapers_for_mkt(
            "en-US",
            vec![
                make_wallpaper("20240102", "EN Day2"), // 与 zh-CN 同一天
                make_wallpaper("20240103", "EN Day3"),
            ],
        );

        let unique = index.get_all_wallpapers_unique();
        // 应只有 3 个唯一的 end_date
        assert_eq!(unique.len(), 3);

        // 20240102 的壁纸应来自字典序靠前的 mkt (en-US < zh-CN)
        let day2 = unique.iter().find(|w| w.end_date == "20240102").unwrap();
        assert_eq!(day2.title, "EN Day2");
    }

    #[test]
    fn test_get_all_wallpapers_unique_empty() {
        let index = WallpaperIndex::new();
        let unique = index.get_all_wallpapers_unique();
        assert!(unique.is_empty());
    }

    #[test]
    fn test_limit_index_size_no_op_when_under_limit() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240101", "Day1"),
                make_wallpaper("20240102", "Day2"),
            ],
        );

        index.limit_index_size(10);

        // 不超过限制，应保持不变
        let wallpapers = index.get_wallpapers_for_mkt("zh-CN");
        assert_eq!(wallpapers.len(), 2);
    }

    #[test]
    fn test_limit_index_size_exact_limit() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240101", "Day1"),
                make_wallpaper("20240102", "Day2"),
            ],
        );

        index.limit_index_size(2);

        // 恰好等于限制，应保持不变
        let wallpapers = index.get_wallpapers_for_mkt("zh-CN");
        assert_eq!(wallpapers.len(), 2);
    }

    #[test]
    fn test_limit_index_size_removes_oldest() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240101", "Day1"),
                make_wallpaper("20240102", "Day2"),
                make_wallpaper("20240103", "Day3"),
                make_wallpaper("20240104", "Day4"),
            ],
        );

        index.limit_index_size(2);

        let wallpapers = index.get_wallpapers_for_mkt("zh-CN");
        assert_eq!(wallpapers.len(), 2);
        // 应保留最新的两个
        assert_eq!(wallpapers[0].end_date, "20240104");
        assert_eq!(wallpapers[1].end_date, "20240103");
    }

    #[test]
    fn test_limit_index_size_cross_mkt() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240101", "ZH Old"),
                make_wallpaper("20240103", "ZH New"),
            ],
        );
        index.upsert_wallpapers_for_mkt(
            "en-US",
            vec![
                make_wallpaper("20240101", "EN Old"),
                make_wallpaper("20240102", "EN Mid"),
            ],
        );

        // 唯一日期共 3 个：20240101, 20240102, 20240103
        // 保留最新 2 个（20240103, 20240102），删除 20240101
        index.limit_index_size(2);

        let zh = index.get_wallpapers_for_mkt("zh-CN");
        let en = index.get_wallpapers_for_mkt("en-US");

        // zh-CN: 只保留 20240103（20240101 被删除）
        assert_eq!(zh.len(), 1);
        assert_eq!(zh[0].end_date, "20240103");

        // en-US: 只保留 20240102（20240101 被删除）
        assert_eq!(en.len(), 1);
        assert_eq!(en[0].end_date, "20240102");
    }

    #[test]
    fn test_limit_index_size_removes_empty_mkt() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt("zh-CN", vec![make_wallpaper("20240103", "ZH New")]);
        index.upsert_wallpapers_for_mkt("en-US", vec![make_wallpaper("20240101", "EN Old")]);

        // 唯一日期: 20240103, 20240101，保留最新 1 个
        index.limit_index_size(1);

        // en-US 的壁纸全部被删除，应被移除
        assert!(!index.mkt.contains_key("en-US"));
        // zh-CN 保留
        assert_eq!(index.get_wallpapers_for_mkt("zh-CN").len(), 1);
    }

    #[test]
    fn test_limit_index_size_empty_index() {
        let mut index = WallpaperIndex::new();
        index.limit_index_size(5);
        assert!(index.mkt.is_empty());
    }

    #[test]
    fn test_wallpaper_index_serialization_roundtrip() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt("zh-CN", vec![make_wallpaper("20240102", "Test")]);

        let json = serde_json::to_string(&index).unwrap();
        let deserialized: WallpaperIndex = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, index.version);
        assert_eq!(deserialized.mkt.len(), 1);
        let wallpapers = deserialized.get_wallpapers_for_mkt("zh-CN");
        assert_eq!(wallpapers.len(), 1);
        assert_eq!(wallpapers[0].title, "Test");
    }
}
