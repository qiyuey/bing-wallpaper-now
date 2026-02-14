import {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";
import {
  Language,
  TranslationKey,
  translations,
  detectSystemLanguage,
} from "./translations";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../types";

/** 实际用于渲染的语言类型（不含 "auto"） */
type ResolvedLanguage = "zh-CN" | "en-US";

function resolveLanguagePreference(lang: Language): ResolvedLanguage {
  return lang === "auto" ? detectSystemLanguage() : lang;
}

interface I18nContextType {
  /** 用户选择的语言偏好（可能是 "auto"），用于设置 UI 回显 */
  language: Language;
  /** 后端解析后的实际语言（始终是 "zh-CN" 或 "en-US"），用于 i18n 翻译 */
  actualLanguage: ResolvedLanguage;
  t: (key: TranslationKey) => string;
  /**
   * 同步前端 i18n 状态（不写后端）。
   *
   * **前置条件**：调用者必须已通过 `updateSettings` 将新语言写入后端。
   * 本函数只从后端 `get_settings` 读取最新的 `resolved_language` 并更新前端状态，
   * 不会调用 `update_settings`，避免双写导致的重复副作用（广播、托盘更新等）。
   */
  setLanguage: (lang: Language) => Promise<void>;
}

const I18nContext = createContext<I18nContextType | undefined>(undefined);

interface I18nProviderProps {
  children: ReactNode;
}

export function I18nProvider({ children }: I18nProviderProps) {
  // 用户的语言偏好（可能是 "auto"），用于设置 UI 回显
  const [language, setLanguageState] = useState<Language>("auto");
  // 后端解析后的实际语言，用于 i18n 翻译
  // 初始值使用前端系统语言检测作为临时默认值，避免加载闪烁
  const [actualLanguage, setActualLanguage] = useState<ResolvedLanguage>(
    detectSystemLanguage(),
  );
  const [loading, setLoading] = useState(true);

  // 从后端加载语言设置（前端语言以后端返回的 resolved_language 为准）
  useEffect(() => {
    (async () => {
      try {
        const settings = await invoke<AppSettings>("get_settings");
        // language: 用户偏好（"auto" | "zh-CN" | "en-US"），用于设置 UI
        if (settings?.language) {
          setLanguageState(settings.language as Language);
        }
        // resolved_language: 后端解析后的实际语言，用于 i18n
        if (
          settings?.resolved_language === "zh-CN" ||
          settings?.resolved_language === "en-US"
        ) {
          setActualLanguage(settings.resolved_language);
        }
      } catch (err) {
        console.error("Failed to load language setting:", err);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const t = (key: TranslationKey): string => {
    return translations[actualLanguage][key] || key;
  };

  const setLanguage = async (lang: Language) => {
    // 只负责同步前端 i18n 状态，不写后端
    // 后端写入由 Settings.tsx 的 handleChange → updateSettings 统一完成，
    // 避免双写导致的重复副作用（广播、托盘更新等）
    try {
      // 从后端获取 resolved_language（后端已由调用者保存最新值）
      const settings = await invoke<AppSettings>("get_settings");
      if (
        settings?.language === "auto" ||
        settings?.language === "zh-CN" ||
        settings?.language === "en-US"
      ) {
        setLanguageState(settings.language);
      } else {
        setLanguageState(lang);
      }
      if (
        settings?.resolved_language === "zh-CN" ||
        settings?.resolved_language === "en-US"
      ) {
        setActualLanguage(settings.resolved_language);
      } else {
        setActualLanguage(resolveLanguagePreference(lang));
      }
    } catch (err) {
      console.error("Failed to sync language state:", err);
      // 后端读取失败时保持前端状态自洽，避免 language / actualLanguage 分裂。
      setLanguageState(lang);
      setActualLanguage(resolveLanguagePreference(lang));
    }
  };

  // 加载中时使用前端系统语言检测的默认值（仅用于避免闪烁）
  if (loading) {
    const defaultLang = detectSystemLanguage();
    return (
      <I18nContext.Provider
        value={{
          language: "auto",
          actualLanguage: defaultLang,
          t: (key: TranslationKey) => translations[defaultLang][key] || key,
          setLanguage,
        }}
      >
        {children}
      </I18nContext.Provider>
    );
  }

  return (
    <I18nContext.Provider value={{ language, actualLanguage, t, setLanguage }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within I18nProvider");
  }
  return context;
}
