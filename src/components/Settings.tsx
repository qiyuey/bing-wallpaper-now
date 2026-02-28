import { useState, useEffect, useCallback } from "react";
import { AppSettings, MarketStatus, MarketGroup } from "../types";
import { useSettings } from "../hooks/useSettings";
import { useTheme, Theme } from "../contexts/ThemeContext";
import { useI18n } from "../i18n/I18nContext";
import { showSystemNotification } from "../utils/notification";
import {
  TransferResult,
  TransferMessage,
  TransferTranslations,
  buildTransferMessage,
  buildTransferErrorMessage,
} from "../utils/transferHelpers";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createSafeUnlisten } from "../utils/eventListener";
import { EVENTS } from "../config/ui";

interface SettingsProps {
  onClose: () => void;
  version?: string;
  onLanguageChange?: () => void;
}

export function Settings({
  onClose,
  version,
  onLanguageChange,
}: SettingsProps) {
  const { settings, loading, updateSettings, getDefaultDirectory } =
    useSettings();
  const { applyThemeToUI } = useTheme();
  const { t, setLanguage } = useI18n();

  const [defaultDir, setDefaultDir] = useState<string>("");
  const [marketGroups, setMarketGroups] = useState<MarketGroup[]>([]);
  const [marketStatus, setMarketStatus] = useState<MarketStatus | null>(null);
  // dismiss 只控制当前打开的 Settings 面板内是否隐藏警告，
  // 下次重新打开 Settings 会重新 pull，如果仍然 mismatch 则重新显示
  const [dismissed, setDismissed] = useState(false);

  const [importing, setImporting] = useState(false);
  const [importMessage, setImportMessage] = useState<TransferMessage | null>(
    null,
  );

  const [exporting, setExporting] = useState(false);
  const [exportMessage, setExportMessage] = useState<TransferMessage | null>(
    null,
  );

  useEffect(() => {
    getDefaultDirectory()
      .then((dir) => {
        if (dir) setDefaultDir(dir);
      })
      .catch((err) => {
        // 静默处理错误，不影响组件渲染
        console.error("Failed to get default directory:", err);
      });
  }, [getDefaultDirectory]);

  // 从后端拉取 MarketStatus
  const fetchMarketStatus = useCallback(async () => {
    try {
      const status = await invoke<MarketStatus>("get_market_status");
      setMarketStatus(status);
      setDismissed(false); // 重新拉取时重置 dismiss 状态
    } catch (err) {
      console.error("Failed to fetch market status:", err);
    }
  }, []);

  // Settings 打开时从后端加载市场列表和 mkt 状态
  useEffect(() => {
    fetchMarketStatus();
    invoke<MarketGroup[]>("get_supported_mkts")
      .then(setMarketGroups)
      .catch((err) => console.error("Failed to load market groups:", err));
  }, [fetchMarketStatus]);

  // 监听 mkt-status-changed 事件，收到后重新 pull
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen(EVENTS.MKT_STATUS_CHANGED, () => {
          if (mounted) {
            fetchMarketStatus();
          }
        });
        const safeUnlisten = createSafeUnlisten(unlistenFn);

        if (mounted) {
          unlisten = safeUnlisten;
        } else {
          safeUnlisten();
        }
      } catch (e) {
        console.error("Failed to bind mkt-status-changed event:", e);
      }
    })();

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, [fetchMarketStatus]);

  // 后端 region ID → 翻译 key 映射
  const regionI18nKey: Record<string, Parameters<typeof t>[0]> = {
    asia_pacific: "marketRegionAsiaPacific",
    europe: "marketRegionEurope",
    americas: "marketRegionAmericas",
    africa: "marketRegionAfrica",
  };

  const handleChange = async (
    field: keyof AppSettings,
    value: string | number | boolean | null,
  ) => {
    if (!settings) return;

    try {
      const updatedSettings = { ...settings, [field]: value };
      await updateSettings(updatedSettings);

      // 如果是主题变化，立即应用到UI
      if (field === "theme" && typeof value === "string") {
        applyThemeToUI(value as Theme);
      }
      // 如果是语言变化，先同步 i18n context 再触发壁纸刷新
      if (field === "language" && typeof value === "string") {
        await setLanguage(value as "auto" | "zh-CN" | "en-US");
        if (onLanguageChange) {
          onLanguageChange();
        }
      }
    } catch (err) {
      console.error("Update settings error:", err);
      await showSystemNotification(
        t("settingsSaveError"),
        t("settingsSaveError") + ": " + err,
      );
    }
  };

  const handleSelectFolder = async () => {
    if (!settings) return;

    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: settings.save_directory ?? defaultDir,
        title: t("selectDirectory"),
      });

      if (selected && typeof selected === "string") {
        await handleChange("save_directory", selected);
      }
    } catch (err) {
      console.error("Failed to select folder:", err);
      await showSystemNotification(
        t("settingsFolderSelectError"),
        t("settingsFolderSelectError") + ": " + String(err),
      );
    }
  };

  const handleTransfer = async (
    command: string,
    paramKey: string,
    translations: TransferTranslations,
    setMessage: (msg: TransferMessage | null) => void,
    setLoading: (v: boolean) => void,
  ) => {
    setMessage(null);

    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: translations.selectDirectory,
      });

      if (!selected || typeof selected !== "string") return;

      setLoading(true);

      const result = await invoke<TransferResult>(command, {
        [paramKey]: selected,
      });

      setMessage(
        buildTransferMessage(result, translations, t("warningSeparator")),
      );
    } catch (err) {
      setMessage(buildTransferErrorMessage(err, translations));
    } finally {
      setLoading(false);
    }
  };

  const handleImport = () =>
    handleTransfer(
      "import_wallpapers",
      "sourceDir",
      {
        selectDirectory: t("importSelectDirectory"),
        success: t("importSuccess"),
        alreadyUpToDate: t("importAlreadyUpToDate"),
        metadataSkipped: t("importMetadataSkipped"),
        imagesFailed: t("importImagesFailed"),
        notDirectory: t("transferNotDirectory"),
        sameDirectory: t("importSameDirectory"),
        noData: t("importNoData"),
        error: t("importError"),
      },
      setImportMessage,
      setImporting,
    );

  const handleExport = () =>
    handleTransfer(
      "export_wallpapers",
      "targetDir",
      {
        selectDirectory: t("exportSelectDirectory"),
        success: t("exportSuccess"),
        alreadyUpToDate: t("exportAlreadyUpToDate"),
        metadataSkipped: t("exportMetadataSkipped"),
        imagesFailed: t("exportImagesFailed"),
        notDirectory: t("transferNotDirectory"),
        sameDirectory: t("exportSameDirectory"),
        noData: t("exportNoData"),
        error: t("exportError"),
      },
      setExportMessage,
      setExporting,
    );

  if (loading && !settings) {
    return <div className="settings-loading">{t("settingsLoading")}</div>;
  }

  return (
    <div className="settings-overlay">
      <div className="settings-modal">
        <div className="settings-header">
          <div className="settings-header-left">
            <h2>{t("settingsTitle")}</h2>
            {version && <span className="settings-version">v{version}</span>}
          </div>
          <button onClick={onClose} className="btn-close">
            ×
          </button>
        </div>

        <div className="settings-body">
          <div className="settings-section">
            <label className="settings-label checkbox-label">
              <input
                type="checkbox"
                checked={settings?.launch_at_startup ?? false}
                onChange={(e) =>
                  handleChange("launch_at_startup", e.target.checked)
                }
              />
              <span>{t("launchAtStartup")}</span>
            </label>
          </div>

          <div className="settings-section">
            <label className="settings-label checkbox-label">
              <input
                type="checkbox"
                checked={settings?.auto_update ?? true}
                onChange={(e) => handleChange("auto_update", e.target.checked)}
              />
              <span>{t("autoUpdate")}</span>
            </label>
          </div>

          <div className="settings-section">
            <label className="settings-label">{t("theme")}:</label>
            <div className="radio-group">
              <label className="radio-option">
                <input
                  type="radio"
                  name="theme"
                  value="system"
                  checked={(settings?.theme ?? "system") === "system"}
                  onChange={(e) =>
                    handleChange("theme", e.target.value as Theme)
                  }
                />
                <span>{t("themeSystem")}</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="theme"
                  value="light"
                  checked={(settings?.theme ?? "system") === "light"}
                  onChange={(e) =>
                    handleChange("theme", e.target.value as Theme)
                  }
                />
                <span>{t("themeLight")}</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="theme"
                  value="dark"
                  checked={(settings?.theme ?? "system") === "dark"}
                  onChange={(e) =>
                    handleChange("theme", e.target.value as Theme)
                  }
                />
                <span>{t("themeDark")}</span>
              </label>
            </div>
          </div>

          <div className="settings-section">
            <label className="settings-label">{t("language")}:</label>
            <div className="radio-group">
              <label className="radio-option">
                <input
                  type="radio"
                  name="language"
                  value="auto"
                  checked={(settings?.language ?? "auto") === "auto"}
                  onChange={(e) => handleChange("language", e.target.value)}
                />
                <span>{t("languageAuto")}</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="language"
                  value="zh-CN"
                  checked={(settings?.language ?? "auto") === "zh-CN"}
                  onChange={(e) => handleChange("language", e.target.value)}
                />
                <span>{t("languageZhCN")}</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="language"
                  value="en-US"
                  checked={(settings?.language ?? "auto") === "en-US"}
                  onChange={(e) => handleChange("language", e.target.value)}
                />
                <span>{t("languageEnUS")}</span>
              </label>
            </div>
          </div>

          <div className="settings-section">
            <label className="settings-label">
              {t("market")}:
              <span className="settings-hint">{t("marketHint")}</span>
            </label>
            <select
              className="settings-select"
              value={settings?.mkt ?? "zh-CN"}
              onChange={async (e) => {
                // 先等待保存完成，再触发壁纸刷新，避免刷新时仍读到旧 mkt
                await handleChange("mkt", e.target.value);
                // mkt 变更后主动刷新 MarketStatus（清除旧的 mismatch 警告）
                await fetchMarketStatus();
                if (onLanguageChange) {
                  onLanguageChange();
                }
              }}
            >
              {marketGroups.map((group) => (
                <optgroup
                  key={group.region}
                  label={t(regionI18nKey[group.region] ?? "market")}
                >
                  {group.markets.map((m) => (
                    <option key={m.code} value={m.code}>
                      {m.label} ({m.code})
                    </option>
                  ))}
                </optgroup>
              ))}
            </select>
            {marketStatus?.is_mismatch && !dismissed && (
              <div className="settings-mkt-warning">
                <span>
                  {t("marketMismatchWarning")
                    .replace("{actualMkt}", marketStatus.effective_mkt)
                    .replace("{requestedMkt}", marketStatus.requested_mkt)}
                </span>
                <button
                  className="btn-dismiss"
                  onClick={() => setDismissed(true)}
                  aria-label="dismiss"
                >
                  ×
                </button>
              </div>
            )}
          </div>

          <div className="settings-section">
            <div className="settings-label">{t("saveDirectory")}:</div>
            <div className="settings-dir-row">
              <div
                className="settings-dir-info"
                title={
                  settings?.save_directory ??
                  (defaultDir ? defaultDir : t("loading"))
                }
              >
                {settings?.save_directory ??
                  (defaultDir ? defaultDir : t("loading"))}
              </div>
              <button
                onClick={handleSelectFolder}
                className="btn btn-secondary btn-small"
                type="button"
              >
                {t("selectFolder")}
              </button>
            </div>
            {settings?.save_directory &&
              settings.save_directory !== defaultDir && (
                <button
                  onClick={() => handleChange("save_directory", null)}
                  className="btn btn-link btn-small"
                  type="button"
                >
                  {t("restoreDefault")}
                </button>
              )}
          </div>

          <div className="settings-section">
            <label className="settings-label">
              {t("importData")}:
              <span className="settings-hint">{t("importDataHint")}</span>
            </label>
            <div className="settings-dir-row">
              <button
                onClick={handleImport}
                className="btn btn-secondary btn-small"
                type="button"
                disabled={importing}
              >
                {importing ? t("importInProgress") : t("importSelectDirectory")}
              </button>
            </div>
            {importMessage && (
              <div
                className={
                  importMessage.type === "success"
                    ? "settings-transfer-success"
                    : "settings-transfer-error"
                }
              >
                {importMessage.text}
              </div>
            )}
          </div>

          <div className="settings-section">
            <label className="settings-label">
              {t("exportData")}:
              <span className="settings-hint">{t("exportDataHint")}</span>
            </label>
            <div className="settings-dir-row">
              <button
                onClick={handleExport}
                className="btn btn-secondary btn-small"
                type="button"
                disabled={exporting}
              >
                {exporting ? t("exportInProgress") : t("exportSelectDirectory")}
              </button>
            </div>
            {exportMessage && (
              <div
                className={
                  exportMessage.type === "success"
                    ? "settings-transfer-success"
                    : "settings-transfer-error"
                }
              >
                {exportMessage.text}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
