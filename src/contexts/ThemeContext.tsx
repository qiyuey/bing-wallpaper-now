import {
  createContext,
  useContext,
  useEffect,
  useLayoutEffect,
  useState,
  ReactNode,
  useRef,
} from "react";
import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark" | "system";
export const THEME_STORAGE_KEY = "bing-wallpaper-now.theme";

interface ThemeContextType {
  theme: Theme;
  actualTheme: "light" | "dark";
  setTheme: (theme: Theme) => Promise<void>;
  applyThemeToUI: (theme: Theme) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

function isTheme(value: unknown): value is Theme {
  return value === "light" || value === "dark" || value === "system";
}

function getSystemTheme(): "light" | "dark" {
  if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return "dark";
  }
  return "light";
}

function readStoredTheme(): Theme {
  try {
    const storedTheme = window.localStorage.getItem(THEME_STORAGE_KEY);
    return isTheme(storedTheme) ? storedTheme : "system";
  } catch {
    return "system";
  }
}

function writeStoredTheme(theme: Theme) {
  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // localStorage can be unavailable in restricted webviews; theme still works in memory.
  }
}

function resolveTheme(theme: Theme): "light" | "dark" {
  return theme === "system" ? getSystemTheme() : theme;
}

function applyTheme(theme: "light" | "dark") {
  document.documentElement.setAttribute("data-theme", theme);
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(() => readStoredTheme());
  const [actualTheme, setActualTheme] = useState<"light" | "dark">(() =>
    resolveTheme(readStoredTheme()),
  );

  // Apply a synchronous fallback theme before the first paint.
  // Persisted settings still take precedence once get_settings resolves.
  useLayoutEffect(() => {
    applyTheme(resolveTheme(readStoredTheme()));
  }, []);

  // Initialize theme from settings
  useEffect(() => {
    const initTheme = async () => {
      try {
        const settings = await invoke<{
          theme: string;
          auto_update: boolean;
          save_directory: string | null;
          launch_at_startup: boolean;
        }>("get_settings");

        if (!settings || typeof settings !== "object") {
          throw new Error("settings unavailable");
        }

        const savedTheme = isTheme(settings.theme) ? settings.theme : "system";
        setThemeState(savedTheme);

        const resolvedTheme = resolveTheme(savedTheme);
        setActualTheme(resolvedTheme);
        writeStoredTheme(savedTheme);
        applyTheme(resolvedTheme);
      } catch (error) {
        console.error("Failed to load theme from settings:", error);
        const fallbackTheme = readStoredTheme();
        const resolvedTheme = resolveTheme(fallbackTheme);
        setThemeState(fallbackTheme);
        setActualTheme(resolvedTheme);
        applyTheme(resolvedTheme);
      }
    };

    initTheme();
  }, []);

  // Use ref to keep the latest theme value without recreating the listener
  const themeRef = useRef(theme);

  useEffect(() => {
    themeRef.current = theme;
  }, [theme]);

  // Listen for system theme changes
  // Use empty dependency array to ensure listener is only created once
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const handleChange = (e: { matches: boolean }) => {
      if (themeRef.current === "system") {
        const newTheme = e.matches ? "dark" : "light";
        setActualTheme(newTheme);
        applyTheme(newTheme);
      }
    };

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, []); // Empty dependency array - listener created once

  // Apply theme to UI only, without saving to backend
  const applyThemeToUI = (newTheme: Theme) => {
    setThemeState(newTheme);
    const resolvedTheme = resolveTheme(newTheme);
    setActualTheme(resolvedTheme);
    writeStoredTheme(newTheme);
    applyTheme(resolvedTheme);
  };

  const setTheme = async (newTheme: Theme) => {
    try {
      // Get current settings
      const settings = await invoke<{
        theme: string;
        auto_update: boolean;
        save_directory: string | null;
        launch_at_startup: boolean;
      }>("get_settings");

      // Update theme in settings - 使用驼峰命名 newSettings
      await invoke("update_settings", {
        newSettings: {
          auto_update: settings.auto_update,
          save_directory: settings.save_directory,
          launch_at_startup: settings.launch_at_startup,
          theme: newTheme,
        },
      });

      // Update local state
      applyThemeToUI(newTheme);
    } catch (error) {
      console.error("Failed to save theme:", error);
      throw error;
    }
  };

  return (
    <ThemeContext.Provider
      value={{ theme, actualTheme, setTheme, applyThemeToUI }}
    >
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return context;
}
