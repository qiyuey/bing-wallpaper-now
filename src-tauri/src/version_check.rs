use crate::runtime_state;
use log::info;
use tauri::AppHandle;

/// 添加版本到"不再提醒"列表（保存最大版本）
#[tauri::command]
pub(crate) async fn add_ignored_update_version(
    app: AppHandle,
    version: String,
) -> Result<(), String> {
    let mut runtime_state = runtime_state::load_runtime_state(&app)
        .map_err(|e| format!("Failed to load runtime state: {}", e))?;

    let should_update = runtime_state
        .ignored_update_version
        .as_ref()
        .map(|ignored| {
            let ignored_ver = semver::Version::parse(ignored);
            let new_ver = semver::Version::parse(&version);
            match (ignored_ver, new_ver) {
                (Ok(i), Ok(n)) => n > i,
                _ => true,
            }
        })
        .unwrap_or(true);

    if should_update {
        runtime_state.ignored_update_version = Some(version.clone());
        runtime_state::save_runtime_state(&app, &runtime_state)
            .map_err(|e| format!("Failed to save runtime state: {}", e))?;
        info!(
            target: "version_check",
            "Updated ignored update version to: {}",
            version
        );
    }

    Ok(())
}

/// 检查版本是否应该被忽略（版本小于等于忽略的版本）
#[tauri::command]
pub(crate) async fn is_version_ignored(app: AppHandle, version: String) -> Result<bool, String> {
    let runtime_state = runtime_state::load_runtime_state(&app)
        .map_err(|e| format!("Failed to load runtime state: {}", e))?;

    match runtime_state.ignored_update_version {
        Some(ref ignored_version) => {
            let ignored_ver = semver::Version::parse(ignored_version);
            let check_ver = semver::Version::parse(&version);
            match (ignored_ver, check_ver) {
                (Ok(i), Ok(c)) => Ok(c <= i),
                _ => Ok(false),
            }
        }
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_version_comparison_with_semver() {
        let v1 = semver::Version::parse("1.0.0").unwrap();
        let v2 = semver::Version::parse("1.0.1").unwrap();
        assert!(v2 > v1);

        let dev = semver::Version::parse("1.2.3-0").unwrap();
        let release = semver::Version::parse("1.2.3").unwrap();
        assert!(release > dev);

        let dev1 = semver::Version::parse("1.2.3-0").unwrap();
        let next = semver::Version::parse("1.2.4").unwrap();
        assert!(next > dev1);
    }
}
