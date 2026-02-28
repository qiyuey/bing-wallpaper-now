use crate::models::{LocalWallpaper, MarketStatus};
use crate::{
    AppState, bing_api, download_manager, get_effective_mkt, runtime_state, storage,
    wallpaper_manager,
};
use chrono::Local;
use log::{error, info, warn};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// 重新下载缺失的壁纸文件
pub(crate) async fn redownload_missing_wallpapers(
    missing_wallpapers: Vec<LocalWallpaper>,
    wallpaper_dir: PathBuf,
    app: tauri::AppHandle,
) {
    info!(target: "commands", "开始重新下载 {} 张缺失的壁纸", missing_wallpapers.len());

    for wallpaper in missing_wallpapers {
        // 如果 urlbase 为空，无法重新下载
        if wallpaper.urlbase.is_empty() {
            warn!(target: "commands", "壁纸缺少 urlbase 信息，无法重新下载: {}", wallpaper.end_date);
            continue;
        }

        // 构建完整的图片 URL
        let image_url = bing_api::get_wallpaper_url(&wallpaper.urlbase, "UHD");

        // 构建保存路径（使用 end_date，因为文件名使用 end_date）
        let save_path = wallpaper_dir.join(format!("{}.jpg", wallpaper.end_date));

        match download_manager::download_image(&image_url, &save_path).await {
            Ok(()) => {
                info!(target: "commands", "成功重新下载壁纸: {}", save_path.display());
                // 发送事件通知前端
                let _ = app.emit("image-downloaded", &wallpaper.end_date);
            }
            Err(e) => {
                error!(target: "commands", "重新下载壁纸失败 {}: {}", wallpaper.end_date, e);
            }
        }
    }
}

/// 单次更新循环：下载、保存、清理、可选应用最新壁纸（含重试与共享客户端）
pub(crate) async fn run_update_cycle(app: &AppHandle) {
    run_update_cycle_internal(app, false).await;
}

/// 检查指定 mkt 的索引是否为空，如果为空且没有更新正在进行，则触发强制更新
///
/// 这个函数用于处理首次启动时索引为空的情况。
/// 通过先检查索引，再检查更新标志，避免不必要的锁竞争。
/// run_update_cycle_internal 内部有并发保护，会确保只有一个更新真正执行。
///
/// # Arguments
/// * `app` - Tauri app handle
/// * `mkt` - 市场代码（用于日志和索引检查）
///
/// # Returns
/// `true` 如果成功触发更新，`false` 如果索引不为空或更新已在进行中
pub(crate) async fn try_trigger_update_if_empty(app: &AppHandle, mkt: &str) -> bool {
    let state = app.state::<AppState>();

    // 先快速检查索引（不需要持有 update_in_progress 锁）
    let wallpaper_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };

    let existing_wallpapers = storage::get_local_wallpapers(&wallpaper_dir, mkt)
        .await
        .unwrap_or_default();

    if !existing_wallpapers.is_empty() {
        // 索引不为空，不需要更新
        return false;
    }

    // 索引为空，检查是否已有更新在进行
    // 注意：这里只检查，不设置标志，让 run_update_cycle_internal 来处理
    // 这样可以避免与 run_update_cycle_internal 内部的并发保护冲突
    let is_updating = {
        let flag = state.update_in_progress.lock().await;
        *flag
    };

    if is_updating {
        info!(
            target: "commands",
            "当前 mkt ({}) 的索引为空，但已有更新在进行中，跳过触发",
            mkt
        );
        return false;
    }

    // 启动更新任务
    // run_update_cycle_internal 内部有并发保护，会确保只有一个更新真正执行
    let app_clone = app.clone();
    let mkt_clone = mkt.to_string();

    tauri::async_runtime::spawn(async move {
        info!(
            target: "commands",
            "当前 mkt ({}) 的索引为空，触发更新",
            mkt_clone
        );
        run_update_cycle_internal(&app_clone, true).await;
    });

    true
}

/// 检查索引是否为空，如果为空则触发强制更新（用于启动时）
///
/// 这个函数用于启动时检查索引，确保首次启动时能正确加载数据。
///
/// # Arguments
/// * `app` - Tauri app handle
///
/// # Returns
/// `true` 如果索引为空且需要强制更新，`false` 如果索引不为空
pub(crate) async fn check_and_trigger_update_if_needed(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();

    // 获取当前 effective_mkt 和壁纸目录
    let (wallpaper_dir, mkt) = {
        let dir = state.wallpaper_directory.lock().await.clone();
        let mkt = get_effective_mkt(&state).await;
        (dir, mkt)
    };

    let existing_wallpapers = storage::get_local_wallpapers(&wallpaper_dir, &mkt)
        .await
        .unwrap_or_default();

    if existing_wallpapers.is_empty() {
        info!(target: "auto_update", "启动时检测到索引为空，执行强制更新");
        run_update_cycle_internal(app, true).await;
        true
    } else {
        // 索引不为空，执行常规更新（可能因为智能检查而跳过）
        run_update_cycle(app).await;
        false
    }
}

/// 应用最新壁纸（如果需要）
/// 只有在 auto_update 设置开启时才会自动应用
async fn apply_latest_wallpaper_if_needed(app: &AppHandle, state: &AppState, wallpaper_dir: &Path) {
    // 一次性获取 auto_update，然后读 effective_mkt（减少锁间设置变化的窗口）
    let should_apply = state.settings.lock().await.auto_update;
    if !should_apply {
        return;
    }
    let mkt = get_effective_mkt(state).await;

    let latest_wallpapers = storage::get_local_wallpapers(wallpaper_dir, &mkt)
        .await
        .unwrap_or_default();
    if let Some(first) = latest_wallpapers.first() {
        // 检查用户是否手动设置过壁纸，且当前最新壁纸和手动设置时的最新壁纸相同
        let runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();
        if runtime_state
            .manually_set_latest_wallpapers
            .get(&mkt)
            .is_some_and(|manually_set_end_date| manually_set_end_date == &first.end_date)
        {
            info!(
                target: "update",
                "跳过自动应用：当前 mkt ({}) 的最新壁纸 ({}) 和用户手动设置时的最新壁纸相同",
                mkt,
                first.end_date
            );
            return;
        }

        let path = storage::get_wallpaper_path(wallpaper_dir, &first.end_date);

        // 检测屏幕方向，获取竖屏壁纸路径
        let screen_orientations = wallpaper_manager::get_screen_orientations();
        let has_portrait_screen = screen_orientations.iter().any(|s| s.is_portrait);
        let portrait_path = if has_portrait_screen {
            let portrait_file = wallpaper_dir.join(format!("{}r.jpg", first.end_date));
            if portrait_file.exists() {
                Some(portrait_file)
            } else {
                None
            }
        } else {
            None
        };

        // 检查当前壁纸是否已经是目标壁纸
        let current_path_guard = state.current_wallpaper_path.lock().await;
        let needs_set = current_path_guard
            .as_ref()
            .map(|p| p != &path)
            .unwrap_or(true);
        drop(current_path_guard);

        if needs_set {
            // 如果文件不存在，尝试按需下载
            if !path.exists() {
                info!(
                    target: "update",
                    "最新壁纸文件不存在，尝试按需下载: {}",
                    path.display()
                );
                if let Err(e) =
                    download_manager::download_wallpaper_if_needed(&path, wallpaper_dir, app).await
                {
                    error!(target: "update", "按需下载壁纸失败: {e}，跳过设置壁纸");
                    return; // 下载失败，不设置壁纸
                }
            }

            // 如果竖屏壁纸不存在，尝试按需下载
            if let Some(ref portrait_file) = portrait_path
                && !portrait_file.exists()
            {
                info!(
                    target: "update",
                    "竖屏壁纸文件不存在，尝试按需下载: {}",
                    portrait_file.display()
                );
                if let Err(e) = download_manager::download_wallpaper_if_needed(
                    portrait_file,
                    wallpaper_dir,
                    app,
                )
                .await
                {
                    warn!(target: "update", "按需下载竖屏壁纸失败: {e}，将仅设置横屏壁纸");
                }
            }

            if let Err(e) = wallpaper_manager::set_wallpaper(&path, portrait_path.as_deref()) {
                error!(target: "update", "设置壁纸失败: {e}");
            } else {
                let mut current_path = state.current_wallpaper_path.lock().await;
                *current_path = Some(path);
            }
        }
    }
    // app 参数保留用于未来可能的扩展（如发送事件通知）
    let _ = app;
}

/// 带重试的 Bing 图片获取
async fn fetch_bing_images_with_retry(mkt: &str) -> Option<bing_api::BingFetchResult> {
    let mut result_opt = None;
    const MAX_RETRIES: u32 = 3;
    const MAX_BACKOFF_SECS: u64 = 16; // 最大延迟 16 秒

    info!(target: "update", "开始获取 Bing 图片（市场代码: {}, 最大重试次数: {}）", mkt, MAX_RETRIES);

    for attempt in 0..MAX_RETRIES {
        info!(target: "update", "Bing API 请求第 {} 次尝试（共 {} 次）", attempt + 1, MAX_RETRIES);

        match bing_api::fetch_bing_images(8, 0, mkt).await {
            Ok(v) => {
                info!(target: "update", "Bing API 请求成功（第 {} 次尝试）: 获取到 {} 张图片, actual_mkt={:?}", attempt + 1, v.images.len(), v.actual_mkt);
                result_opt = Some(v);
                break;
            }
            Err(e) => {
                if attempt < MAX_RETRIES - 1 {
                    // 优化：限制最大延迟时间，避免等待时间过长
                    let base_backoff = 1 << attempt; // 指数退避：1, 2, 4
                    let backoff = base_backoff.min(MAX_BACKOFF_SECS); // 限制最大 16 秒
                    warn!(target: "update",
                        "获取 Bing 图片失败(第 {} 次): {}，{}s 后重试",
                        attempt + 1,
                        e,
                        backoff
                    );
                    tokio::time::sleep(Duration::from_secs(backoff)).await;
                } else {
                    error!(target: "update",
                        "获取 Bing 图片失败(第 {} 次): {}，已达最大重试次数",
                        attempt + 1,
                        e
                    );
                }
            }
        }
    }

    match &result_opt {
        Some(result) => {
            info!(target: "update", "Bing API 获取完成: 成功获取 {} 张图片", result.images.len());
        }
        None => {
            error!(target: "update", "Bing API 获取失败: 所有重试均失败");
        }
    }

    result_opt
}

/// 内部更新循环实现
/// @param force_update: 是否强制更新（忽略智能检查）
pub(crate) async fn run_update_cycle_internal(app: &AppHandle, force_update: bool) {
    let state = app.state::<AppState>();

    // 并发保护：若已有更新在进行，直接跳过
    {
        let mut flag = state.update_in_progress.lock().await;
        if *flag {
            return;
        }
        *flag = true;
    }

    // 核心逻辑在 async block 中：所有 return 只退出此 block，
    // 确保下方的 update_in_progress 重置一定会执行。
    let _: () = async {
        let dir = {
            let d = state.wallpaper_directory.lock().await;
            d.clone()
        };

        let request_mkt = state.settings.lock().await.mkt.clone();
        let read_mkt = get_effective_mkt(&state).await;

        let existing_wallpapers = storage::get_local_wallpapers(&dir, &read_mkt)
            .await
            .unwrap_or_default();

        if !force_update {
            let runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();

            if runtime_state::can_skip_api_request(&runtime_state, &dir, &read_mkt).await {
                info!(target: "update", "使用缓存策略跳过 API 请求，直接使用本地壁纸");
                apply_latest_wallpaper_if_needed(app, &state, &dir).await;
                return;
            }

            if !runtime_state::should_update_today(&runtime_state) {
                if runtime_state::has_today_wallpaper(&dir, &read_mkt).await {
                    info!(target: "update", "跳过更新：今天已更新且本地有今日壁纸");
                    apply_latest_wallpaper_if_needed(app, &state, &dir).await;
                    return;
                }
                info!(target: "update", "今天已更新但本地没有今日壁纸，继续更新");
            }

            let mut runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();
            let _ = runtime_state::update_last_check_time(app, &mut runtime_state);
        } else {
            info!(target: "update", "强制更新模式，跳过智能检查");
        }

        if let Err(e) = storage::ensure_wallpaper_directory(&dir).await {
            error!(target: "update", "创建目录失败: {e}");
            return;
        }

        let fetch_result = match fetch_bing_images_with_retry(&request_mkt).await {
            Some(v) => v,
            None => {
                error!(target: "update", "多次重试仍失败，跳过本次循环");
                return;
            }
        };

        let images = fetch_result.images;
        let save_mkt = fetch_result
            .actual_mkt
            .as_deref()
            .unwrap_or(&request_mkt)
            .to_string();

        // 更新 last_actual_mkt（内存 + 持久化），确保后续读取路径与写入一致
        // 使用边沿触发：仅在 mismatch 状态发生变化时（false→true / true→false）才发事件
        {
            let old_effective = {
                let guard = state.last_actual_mkt.lock().await;
                guard.clone().unwrap_or_else(|| request_mkt.clone())
            };
            let old_mismatch = old_effective != request_mkt;
            let new_mismatch = save_mkt != request_mkt;

            let new_actual_mkt = if new_mismatch {
                info!(
                    target: "update",
                    "mkt 不一致：请求={}, 实际={}, 将使用实际 mkt 保存元数据",
                    request_mkt, save_mkt
                );
                Some(save_mkt.clone())
            } else {
                None
            };

            *state.last_actual_mkt.lock().await = new_actual_mkt.clone();

            if let Ok(mut runtime_state) = runtime_state::load_runtime_state(app) {
                runtime_state.last_actual_mkt = new_actual_mkt;
                if let Err(e) = runtime_state::save_runtime_state(app, &runtime_state) {
                    warn!(target: "update", "持久化 last_actual_mkt 失败: {}", e);
                }
            }

            if new_mismatch != old_mismatch {
                let status = MarketStatus {
                    requested_mkt: request_mkt.clone(),
                    effective_mkt: save_mkt.clone(),
                    is_mismatch: new_mismatch,
                };
                if let Err(e) = app.emit("mkt-status-changed", &status) {
                    warn!(target: "update", "发送 mkt-status-changed 事件失败: {}", e);
                }
                info!(
                    target: "update",
                    "mkt 状态变化：mismatch {} → {}",
                    old_mismatch, new_mismatch
                );
            }
        }

        let metadata_list: Vec<LocalWallpaper> = images
            .iter()
            .map(|image| LocalWallpaper::from(image.clone()))
            .collect();

        let is_first_launch = existing_wallpapers.is_empty();

        let screen_orientations = wallpaper_manager::get_screen_orientations();
        let has_portrait_screen = screen_orientations.iter().any(|s| s.is_portrait);
        let latest_wallpaper_for_portrait = if has_portrait_screen && !metadata_list.is_empty() {
            Some(metadata_list[0].clone())
        } else {
            None
        };

        if !metadata_list.is_empty() {
            let count = metadata_list.len();
            match storage::save_wallpapers_metadata(metadata_list, &dir, &save_mkt).await {
                Err(e) => {
                    if is_first_launch {
                        error!(target: "update", "保存元数据失败: {e}");
                    } else {
                        warn!(target: "update", "更新元数据失败: {e}");
                    }
                }
                Ok(result) => {
                    info!(
                        target: "update",
                        "已{}壁纸元数据（{} 条，新增 {} 条）",
                        if is_first_launch { "保存" } else { "更新" },
                        count,
                        result.new_count
                    );
                    if is_first_launch {
                        if let Err(e) = app.emit("wallpaper-updated", ()) {
                            warn!(target: "update", "通知前端失败: {e}");
                        }
                        info!(target: "update", "元信息已保存并通知前端，图片将按需下载");
                    }
                }
            }
        }

        if let Some(ref latest_wallpaper) = latest_wallpaper_for_portrait
            && !latest_wallpaper.urlbase.is_empty()
        {
            let portrait_file_path = dir.join(format!("{}r.jpg", latest_wallpaper.end_date));

            if !portrait_file_path.exists() {
                let portrait_url =
                    bing_api::get_wallpaper_url(&latest_wallpaper.urlbase, "1080x1920");
                let end_date = latest_wallpaper.end_date.clone();
                info!(
                    target: "update",
                    "检测到竖屏显示器，开始下载竖屏壁纸: {}",
                    portrait_file_path.display()
                );

                let app_clone = app.clone();
                let portrait_path_clone = portrait_file_path.clone();
                tauri::async_runtime::spawn(async move {
                    match download_manager::download_image(&portrait_url, &portrait_path_clone)
                        .await
                    {
                        Ok(()) => {
                            info!(
                                target: "update",
                                "竖屏壁纸下载成功: {}",
                                portrait_path_clone.display()
                            );
                            let _ = app_clone.emit("image-downloaded", end_date);
                        }
                        Err(e) => {
                            error!(
                                target: "update",
                                "竖屏壁纸下载失败: {}",
                                e
                            );
                        }
                    }
                });
            }
        }

        apply_latest_wallpaper_if_needed(app, &state, &dir).await;

        info!(target: "update", "完成一次更新循环");
        {
            let mut last = state.last_update_time.lock().await;
            *last = Some(Local::now());
        }

        {
            let mut runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();
            let _ = runtime_state::update_last_successful_time(app, &mut runtime_state);
        }

        if !is_first_launch && let Err(e) = app.emit("wallpaper-updated", ()) {
            warn!(target: "update", "通知前端失败: {e}");
        }
    }
    .await;

    // 统一重置 update_in_progress，无论上方逻辑如何退出
    {
        let mut flag = state.update_in_progress.lock().await;
        *flag = false;
    }
}

/// 手动强制执行一次更新
#[tauri::command]
pub(crate) async fn force_update(app: tauri::AppHandle) -> Result<(), String> {
    // 调用强制更新版本，跳过智能检查
    run_update_cycle_internal(&app, true).await;
    Ok(())
}
