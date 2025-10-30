import { createContext, useContext, useState, useEffect, ReactNode } from "react";
import { Language, TranslationKey, translations, getActualLanguage } from "./translations";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../types";

interface I18nContextType {
  language: Language;
  actualLanguage: "zh-CN" | "en-US";
  t: (key: TranslationKey) => string;
  setLanguage: (lang: Language) => Promise<void>;
}

const I18nContext = createContext<I18nContextType | undefined>(undefined);

interface I18nProviderProps {
  children: ReactNode;
}

export function I18nProvider({ children }: I18nProviderProps) {
  const [language, setLanguageState] = useState<Language>("auto");
  const [loading, setLoading] = useState(true);

  // 从设置中加载语言
  useEffect(() => {
    (async () => {
      try {
        const settings = await invoke<AppSettings>("get_settings");
        if (settings?.language) {
          setLanguageState((settings.language as Language) || "auto");
        }
      } catch (err) {
        console.error("Failed to load language setting:", err);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const actualLanguage = getActualLanguage(language);

  const t = (key: TranslationKey): string => {
    return translations[actualLanguage][key] || key;
  };

  const setLanguage = async (lang: Language) => {
    setLanguageState(lang);
    try {
      // 获取当前设置并更新语言
      const currentSettings = await invoke<AppSettings>("get_settings");
      await invoke("update_settings", {
        newSettings: {
          ...currentSettings,
          language: lang,
        },
      });
    } catch (err) {
      console.error("Failed to save language setting:", err);
    }
  };

  // 加载中时显示默认语言
  if (loading) {
    return (
      <I18nContext.Provider
        value={{
          language: "auto",
          actualLanguage: getActualLanguage("auto"),
          t: (key: TranslationKey) => translations[getActualLanguage("auto")][key] || key,
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

