use std::path::Path;

#[cfg(all(unix, not(target_os = "macos")))]
use anyhow::{Context, Result, anyhow};
#[cfg(all(unix, not(target_os = "macos")))]
use log::info;
#[cfg(all(unix, not(target_os = "macos")))]
use std::env;
#[cfg(all(unix, not(target_os = "macos")))]
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinuxWallpaperBackend {
    Gnome,
    Cinnamon,
    KdePlasma,
    Xfce,
}

#[derive(Debug, Clone, Default)]
struct LinuxDesktopContext {
    xdg_current_desktop: Option<String>,
    desktop_session: Option<String>,
    xdg_session_type: Option<String>,
}

impl LinuxDesktopContext {
    #[cfg(all(unix, not(target_os = "macos")))]
    fn from_env() -> Self {
        Self {
            xdg_current_desktop: env::var("XDG_CURRENT_DESKTOP")
                .ok()
                .filter(|v| !v.is_empty()),
            desktop_session: env::var("DESKTOP_SESSION").ok().filter(|v| !v.is_empty()),
            xdg_session_type: env::var("XDG_SESSION_TYPE").ok().filter(|v| !v.is_empty()),
        }
    }

    fn marker(&self) -> String {
        [
            self.xdg_current_desktop.as_deref(),
            self.desktop_session.as_deref(),
            self.xdg_session_type.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(":")
        .to_lowercase()
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn set_wallpaper(image_path: &Path) -> Result<()> {
    let target_path = image_path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid path encoding"))?;

    let context = LinuxDesktopContext::from_env();
    let backend = select_backend(&context).ok_or_else(|| {
        anyhow!(
            "unsupported Linux desktop environment: XDG_CURRENT_DESKTOP={:?}, DESKTOP_SESSION={:?}, XDG_SESSION_TYPE={:?}",
            context.xdg_current_desktop,
            context.desktop_session,
            context.xdg_session_type
        )
    })?;

    info!(target: "wallpaper", "设置 Linux 壁纸为 {:?}，backend={:?}, context={:?}",
        target_path, backend, context
    );

    set_with_backend(backend, image_path)
        .with_context(|| format!("Linux {:?} wallpaper backend failed", backend))
}

fn select_backend(context: &LinuxDesktopContext) -> Option<LinuxWallpaperBackend> {
    let marker = context.marker();

    if marker.contains("kde") || marker.contains("plasma") {
        Some(LinuxWallpaperBackend::KdePlasma)
    } else if marker.contains("xfce") {
        Some(LinuxWallpaperBackend::Xfce)
    } else if marker.contains("cinnamon") {
        Some(LinuxWallpaperBackend::Cinnamon)
    } else if marker.contains("gnome") || marker.contains("unity") || marker.contains("budgie") {
        Some(LinuxWallpaperBackend::Gnome)
    } else {
        None
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn set_with_backend(backend: LinuxWallpaperBackend, image_path: &Path) -> Result<()> {
    match backend {
        LinuxWallpaperBackend::Gnome => set_gsettings_wallpaper(
            "org.gnome.desktop.background",
            &["picture-uri", "picture-uri-dark"],
            image_path,
        ),
        LinuxWallpaperBackend::Cinnamon => set_gsettings_wallpaper(
            "org.cinnamon.desktop.background",
            &["picture-uri"],
            image_path,
        ),
        LinuxWallpaperBackend::KdePlasma => set_kde_wallpaper(image_path),
        LinuxWallpaperBackend::Xfce => set_xfce_wallpaper(image_path),
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn set_gsettings_wallpaper(schema: &str, keys: &[&str], image_path: &Path) -> Result<()> {
    let uri = file_uri(image_path);
    let mut errors = Vec::new();

    for key in keys {
        if let Err(error) = run_command("gsettings", &["set", schema, key, &uri]) {
            errors.push(format!("{schema}.{key}: {error:#}"));
        }
    }

    if errors.len() == keys.len() {
        Err(anyhow!(
            "all gsettings writes failed: {}",
            errors.join("; ")
        ))
    } else {
        Ok(())
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn set_kde_wallpaper(image_path: &Path) -> Result<()> {
    let uri = js_string_literal(&file_uri(image_path));
    let script = format!(
        r#"
var allDesktops = desktops();
for (var i = 0; i < allDesktops.length; i++) {{
    var desktop = allDesktops[i];
    desktop.wallpaperPlugin = "org.kde.image";
    desktop.currentConfigGroup = Array("Wallpaper", "org.kde.image", "General");
    desktop.writeConfig("Image", {uri});
    if (typeof desktop.reloadConfig === "function") {{
        desktop.reloadConfig();
    }}
}}
"#
    );

    let mut errors = Vec::new();
    for command in ["qdbus6", "qdbus", "qdbus-qt6"] {
        match run_command(
            command,
            &[
                "org.kde.plasmashell",
                "/PlasmaShell",
                "org.kde.PlasmaShell.evaluateScript",
                &script,
            ],
        ) {
            Ok(()) => return Ok(()),
            Err(error) => errors.push(format!("{command}: {error:#}")),
        }
    }

    Err(anyhow!(
        "all KDE Plasma DBus commands failed: {}",
        errors.join("; ")
    ))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn set_xfce_wallpaper(image_path: &Path) -> Result<()> {
    let target_path = image_path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid path encoding"))?;
    let output = Command::new("xfconf-query")
        .args(["-c", "xfce4-desktop", "-l"])
        .output()
        .context("failed to list XFCE desktop properties with xfconf-query")?;

    if !output.status.success() {
        return Err(anyhow!(
            "xfconf-query -l failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let properties = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.ends_with("/last-image"))
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if properties.is_empty() {
        return Err(anyhow!("no XFCE last-image properties found"));
    }

    let mut errors = Vec::new();
    for property in &properties {
        if let Err(error) = run_command(
            "xfconf-query",
            &["-c", "xfce4-desktop", "-p", property, "-s", target_path],
        ) {
            errors.push(format!("{property}: {error:#}"));
        }
    }

    if errors.len() == properties.len() {
        Err(anyhow!(
            "all XFCE wallpaper writes failed: {}",
            errors.join("; ")
        ))
    } else {
        Ok(())
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn run_command(program: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("failed to run {program}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "{} exited with {}: {}",
            program,
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn normalize_unix_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
        .to_string()
}

fn file_uri(path: &Path) -> String {
    let normalized = normalize_unix_path(path);
    let mut uri = String::from("file://");

    for byte in normalized.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                uri.push(*byte as char)
            }
            _ => uri.push_str(&format!("%{byte:02X}")),
        }
    }

    uri
}

fn js_string_literal(value: &str) -> String {
    let mut escaped = String::from("\"");

    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }

    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(current: &str, session: &str, session_type: &str) -> LinuxDesktopContext {
        LinuxDesktopContext {
            xdg_current_desktop: (!current.is_empty()).then(|| current.to_string()),
            desktop_session: (!session.is_empty()).then(|| session.to_string()),
            xdg_session_type: (!session_type.is_empty()).then(|| session_type.to_string()),
        }
    }

    #[test]
    fn selects_kde_from_current_desktop() {
        assert_eq!(
            select_backend(&context("KDE", "plasma", "wayland")),
            Some(LinuxWallpaperBackend::KdePlasma)
        );
    }

    #[test]
    fn selects_xfce_from_desktop_session() {
        assert_eq!(
            select_backend(&context("", "xfce", "x11")),
            Some(LinuxWallpaperBackend::Xfce)
        );
    }

    #[test]
    fn selects_gnome_family() {
        assert_eq!(
            select_backend(&context("GNOME:GNOME-Classic", "", "wayland")),
            Some(LinuxWallpaperBackend::Gnome)
        );
        assert_eq!(
            select_backend(&context("Budgie", "", "x11")),
            Some(LinuxWallpaperBackend::Gnome)
        );
    }

    #[test]
    fn selects_cinnamon_before_gnome_family() {
        assert_eq!(
            select_backend(&context("X-Cinnamon", "", "x11")),
            Some(LinuxWallpaperBackend::Cinnamon)
        );
    }

    #[test]
    fn unknown_desktop_is_unsupported() {
        assert_eq!(select_backend(&context("", "sway", "wayland")), None);
    }

    #[test]
    fn escapes_file_uri_bytes() {
        assert_eq!(
            file_uri(Path::new("/tmp/Bing Wallpaper 你好.jpg")),
            "file:///tmp/Bing%20Wallpaper%20%E4%BD%A0%E5%A5%BD.jpg"
        );
    }

    #[test]
    fn escapes_javascript_string_literal() {
        assert_eq!(
            js_string_literal("file:///tmp/a\"b\\c.jpg"),
            "\"file:///tmp/a\\\"b\\\\c.jpg\""
        );
    }
}
