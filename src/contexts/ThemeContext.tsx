import {
  createContext,
  useContext,
  useEffect,
  useState,
  ReactNode,
  useRef,
} from "react";
import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark" | "system";

interface ThemeContextType {
  theme: Theme;
  actualTheme: "light" | "dark";
  setTheme: (theme: Theme) => Promise<void>;
  applyThemeToUI: (theme: Theme) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

function getSystemTheme(): "light" | "dark" {
  if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return "dark";
  }
  return "light";
}

function applyTheme(theme: "light" | "dark") {
  document.documentElement.setAttribute("data-theme", theme);
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>("system");
  const [actualTheme, setActualTheme] = useState<"light" | "dark">("light");

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

        const savedTheme = (settings.theme ?? "system") as Theme;
        setThemeState(savedTheme);

        const resolvedTheme =
          savedTheme === "system" ? getSystemTheme() : savedTheme;
        setActualTheme(resolvedTheme);
        applyTheme(resolvedTheme);
      } catch (error) {
        console.error("Failed to load theme from settings:", error);
        // Fallback to system theme
        const systemTheme = getSystemTheme();
        setActualTheme(systemTheme);
        applyTheme(systemTheme);
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
    const resolvedTheme = newTheme === "system" ? getSystemTheme() : newTheme;
    setActualTheme(resolvedTheme);
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
