use crate::models::{LocalWallpaper, MarketStatus};
use crate::{
    AppState, download_manager, get_effective_mkt, runtime_state, storage, update_cycle,
    wallpaper_manager,
};
use log::{error, info, warn};
use std::path::Path;
use std::path::PathBuf;
use tauri::{Emitter, Manager};

/// 设置桌面壁纸（异步非阻塞）
#[tauri::command]
pub(crate) async fn set_desktop_wallpaper(
    file_path: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let path = PathBuf::from(&file_path);

    let base_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };
    let base_dir_can = base_dir
        .canonicalize()
        .map_err(|e| format!("无法解析壁纸目录: {e}"))?;

    if !path.exists() {
        info!(
            target: "wallpaper",
            "壁纸文件不存在，尝试按需下载: {}",
            path.display()
        );
        if let Err(e) =
            download_manager::download_wallpaper_if_needed(&path, &base_dir_can, &app).await
        {
            return Err(format!("文件不存在且下载失败: {}", e));
        }
    }

    let target_can = path
        .canonicalize()
        .map_err(|e| format!("无法解析目标路径: {e}"))?;

    if !target_can.starts_with(&base_dir_can) {
        return Err("目标文件不在壁纸目录下，拒绝设置".into());
    }
    if !target_can.is_file() {
        return Err("目标文件不存在或不是普通文件".into());
    }

    let target_for_spawn = target_can.clone();
    let app_clone = app.clone();
    let (mkt_code, wallpaper_dir_for_record) = {
        let mkt = get_effective_mkt(&state).await;
        let dir = state.wallpaper_directory.lock().await.clone();
        (mkt, dir)
    };
    let set_end_date = path
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|filename| filename.strip_suffix(".jpg"))
        .map(|s| s.to_string());

    tauri::async_runtime::spawn(async move {
        let screen_orientations = wallpaper_manager::get_screen_orientations();
        let has_portrait_screen = screen_orientations.iter().any(|s| s.is_portrait);

        let base_dir = target_for_spawn.parent().unwrap_or(Path::new(""));
        let portrait_file = target_for_spawn
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| base_dir.join(format!("{}r.jpg", s)));

        let mut portrait_path = None;

        if has_portrait_screen && let Some(ref portrait_file_path) = portrait_file {
            if portrait_file_path.exists() {
                portrait_path = Some(portrait_file_path.clone());
            } else {
                info!(
                    target: "wallpaper",
                    "竖屏壁纸文件不存在，尝试按需下载: {}",
                    portrait_file_path.display()
                );
                let end_date = target_for_spawn
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());

                if let Some(_end_date) = end_date {
                    let wallpaper_dir = base_dir.to_path_buf();
                    info!(
                        target: "wallpaper",
                        "开始下载竖屏壁纸: {}",
                        portrait_file_path.display()
                    );
                    if let Err(e) = download_manager::download_wallpaper_if_needed(
                        portrait_file_path,
                        &wallpaper_dir,
                        &app_clone,
                    )
                    .await
                    {
                        warn!(target: "wallpaper", "按需下载竖屏壁纸失败: {e}，将仅设置横屏壁纸");
                    } else if portrait_file_path.exists() {
                        info!(
                            target: "wallpaper",
                            "竖屏壁纸下载成功，将使用竖屏壁纸: {}",
                            portrait_file_path.display()
                        );
                        portrait_path = Some(portrait_file_path.clone());
                    } else {
                        warn!(target: "wallpaper", "竖屏壁纸下载完成但文件不存在，将仅设置横屏壁纸");
                    }
                }
            }
        }

        if let Err(e) =
            wallpaper_manager::set_wallpaper(&target_for_spawn, portrait_path.as_deref())
        {
            error!(target: "wallpaper", "设置壁纸失败: {e}");
        } else {
            let state_clone = app_clone.state::<AppState>();
            let mut current_path = state_clone.current_wallpaper_path.lock().await;
            *current_path = Some(target_for_spawn.clone());
            drop(current_path);

            if let Some(set_end_date) = set_end_date
                && let Ok(latest_wallpapers) =
                    storage::get_local_wallpapers(&wallpaper_dir_for_record, &mkt_code).await
                && let Some(latest) = latest_wallpapers.first()
            {
                let mut runtime_state =
                    runtime_state::load_runtime_state(&app_clone).unwrap_or_default();
                runtime_state
                    .manually_set_latest_wallpapers
                    .insert(mkt_code.clone(), latest.end_date.clone());
                if let Err(e) = runtime_state::save_runtime_state(&app_clone, &runtime_state) {
                    warn!(target: "wallpaper", "保存手动设置记录失败: {e}");
                } else {
                    info!(target: "wallpaper",
                        "已记录用户手动设置时的最新壁纸：mkt={}, 设置壁纸={}, 当时最新壁纸={}",
                        mkt_code, set_end_date, latest.end_date);
                }
            }
        }
    });

    Ok(())
}

/// 获取已下载的壁纸列表
#[tauri::command]
pub(crate) async fn get_local_wallpapers(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Vec<LocalWallpaper>, String> {
    let wallpaper_dir = state.wallpaper_directory.lock().await.clone();

    let mkt = get_effective_mkt(&state).await;
    let (settings_mkt, resolved_language) = {
        let settings = state.settings.lock().await;
        (settings.mkt.clone(), settings.resolved_language.clone())
    };

    info!(
        target: "commands",
        "获取本地壁纸列表，mkt: {}, 目录: {}",
        mkt,
        wallpaper_dir.display()
    );

    let mut wallpapers = storage::get_local_wallpapers(&wallpaper_dir, &mkt)
        .await
        .map_err(|e| {
            error!(target: "commands", "获取本地壁纸列表失败: {}", e);
            e.to_string()
        })?;

    let mut actual_read_mkt = mkt.clone();

    if wallpapers.is_empty()
        && let Ok(available_keys) = storage::get_available_mkt_keys(&wallpaper_dir).await
        && !available_keys.is_empty()
    {
        let fallback_mkt = if available_keys.contains(&settings_mkt) {
            settings_mkt.clone()
        } else if available_keys.contains(&resolved_language) {
            resolved_language.clone()
        } else {
            available_keys[0].clone()
        };

        if fallback_mkt != mkt {
            warn!(
                target: "commands",
                "mkt fallback: effective_mkt={} 无数据，回退到 index 中可用的 mkt={}（可用 keys: {:?}）",
                mkt, fallback_mkt, available_keys
            );
            wallpapers = storage::get_local_wallpapers(&wallpaper_dir, &fallback_mkt)
                .await
                .map_err(|e| {
                    error!(target: "commands", "fallback 获取本地壁纸列表失败: {}", e);
                    e.to_string()
                })?;
            actual_read_mkt = fallback_mkt;
        }
    }

    let old_effective = {
        let guard = state.last_actual_mkt.lock().await;
        guard.clone().unwrap_or_else(|| settings_mkt.clone())
    };
    let old_mismatch = old_effective != settings_mkt;
    let new_mismatch = actual_read_mkt != settings_mkt;
    let new_actual_mkt = if new_mismatch {
        Some(actual_read_mkt.clone())
    } else {
        None
    };

    if old_effective != actual_read_mkt {
        *state.last_actual_mkt.lock().await = new_actual_mkt.clone();

        if let Ok(mut runtime_state) = runtime_state::load_runtime_state(&app) {
            runtime_state.last_actual_mkt = new_actual_mkt;
            if let Err(e) = runtime_state::save_runtime_state(&app, &runtime_state) {
                warn!(target: "commands", "持久化同步 last_actual_mkt 失败: {}", e);
            }
        }
    }

    if new_mismatch != old_mismatch {
        let status = MarketStatus {
            requested_mkt: settings_mkt.clone(),
            effective_mkt: actual_read_mkt.clone(),
            is_mismatch: new_mismatch,
        };
        if let Err(e) = app.emit("mkt-status-changed", &status) {
            warn!(target: "commands", "发送 mkt-status-changed 事件失败: {}", e);
        }
    }

    info!(
        target: "commands",
        "成功获取 {} 张本地壁纸（mkt: {}）",
        wallpapers.len(),
        actual_read_mkt
    );

    if wallpapers.is_empty() {
        warn!(
            target: "commands",
            "当前 mkt ({}) 的壁纸列表为空（fallback 后仍无数据），将触发异步更新",
            actual_read_mkt
        );
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = update_cycle::try_trigger_update_if_empty(&app_clone, &actual_read_mkt).await;
        });
    }

    let mut missing_wallpapers = Vec::new();
    for wallpaper in &wallpapers {
        let path = storage::get_wallpaper_path(&wallpaper_dir, &wallpaper.end_date);
        if !path.exists() {
            warn!(target: "commands", "壁纸文件不存在，将触发重新下载: {}", path.display());
            missing_wallpapers.push(wallpaper.clone());
        }
    }

    if !missing_wallpapers.is_empty() {
        warn!(
            target: "commands",
            "发现 {} 个缺失的壁纸文件，将触发重新下载",
            missing_wallpapers.len()
        );
        let wallpaper_dir_clone = wallpaper_dir.clone();
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            update_cycle::redownload_missing_wallpapers(
                missing_wallpapers,
                wallpaper_dir_clone,
                app_clone,
            )
            .await;
        });
    }

    Ok(wallpapers)
}
