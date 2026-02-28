//! Tauri command handlers organized by domain.
//!
//! Not all `#[tauri::command]` functions live here — commands tightly coupled
//! to domain logic (e.g. `version_check::check_for_updates`,
//! `update_cycle::force_update`, `transfer::import_wallpapers`) stay in their
//! respective modules to avoid a thin-wrapper layer that adds indirection
//! without value.

pub(crate) mod mkt;
pub(crate) mod settings;
pub(crate) mod storage;
pub(crate) mod wallpaper;
pub(crate) mod window;
