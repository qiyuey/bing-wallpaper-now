import {
  createContext,
  useContext,
  useEffect,
  useState,
  ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";

export type Theme = "light" | "dark" | "system";

interface ThemeContextType {
  theme: Theme;
  actualTheme: "light" | "dark";
  setTheme: (theme: Theme) => Promise<void>;
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
          keep_image_count: number;
          launch_at_startup: boolean;
        }>("get_settings");

        const savedTheme = (settings.theme || "system") as Theme;
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

  // Listen for system theme changes
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const handleChange = (e: { matches: boolean }) => {
      if (theme === "system") {
        const newTheme = e.matches ? "dark" : "light";
        setActualTheme(newTheme);
        applyTheme(newTheme);
      }
    };

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [theme]);

  const setTheme = async (newTheme: Theme) => {
    try {
      // Get current settings
      const settings = await invoke<{
        theme: string;
        auto_update: boolean;
        save_directory: string | null;
        keep_image_count: number;
        launch_at_startup: boolean;
      }>("get_settings");

      // Update theme in settings - 使用驼峰命名 newSettings
      await invoke("update_settings", {
        newSettings: {
          auto_update: settings.auto_update,
          save_directory: settings.save_directory,
          keep_image_count: settings.keep_image_count,
          launch_at_startup: settings.launch_at_startup,
          theme: newTheme,
        },
      });

      // Update local state
      setThemeState(newTheme);

      // Apply the resolved theme
      const resolvedTheme = newTheme === "system" ? getSystemTheme() : newTheme;
      setActualTheme(resolvedTheme);
      applyTheme(resolvedTheme);
    } catch (error) {
      console.error("Failed to save theme:", error);
      throw error;
    }
  };

  return (
    <ThemeContext.Provider value={{ theme, actualTheme, setTheme }}>
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
