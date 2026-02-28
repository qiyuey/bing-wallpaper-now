use crate::AppState;
use crate::models::MarketStatus;
use crate::utils;

/// 获取按区域分组的市场列表（前端动态渲染下拉选项）
#[tauri::command]
pub(crate) fn get_supported_mkts() -> Vec<utils::MarketGroup> {
    utils::get_market_groups()
}

/// 获取当前 market 状态
///
/// 前端通过此命令主动拉取 mkt 状态，而非依赖事件推送。
/// `effective_mkt` 与 `get_effective_mkt()` 返回值完全一致，确保单一 truth source。
#[tauri::command]
pub(crate) async fn get_market_status(
    state: tauri::State<'_, AppState>,
) -> Result<MarketStatus, String> {
    let requested = state.settings.lock().await.mkt.clone();
    let effective = crate::get_effective_mkt(&state).await;
    Ok(MarketStatus {
        is_mismatch: requested != effective,
        requested_mkt: requested,
        effective_mkt: effective,
    })
}
