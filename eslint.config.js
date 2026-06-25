import js from "@eslint/js";
import eslintReact from "@eslint-react/eslint-plugin";
import reactHooksPlugin from "eslint-plugin-react-hooks";
import prettierConfig from "eslint-config-prettier";
import tseslint from "typescript-eslint";

export default [
  // 基础 JavaScript 推荐规则
  js.configs.recommended,

  // 配置文件（根目录）
  {
    files: ["*.config.{js,ts}", "*.setup.{js,ts}"],
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
      },
      globals: {
        console: "readonly",
        process: "readonly",
        global: "readonly",
      },
    },
    plugins: {
      "@typescript-eslint": tseslint.plugin,
    },
    rules: {
      "@typescript-eslint/no-unused-vars": "off",
      "@typescript-eslint/no-explicit-any": "off",
      "no-unused-vars": "off",
    },
  },

  // TypeScript + React 文件配置
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: {
          jsx: true,
        },
      },
      globals: {
        console: "readonly",
        process: "readonly",
        window: "readonly",
        document: "readonly",
        alert: "readonly",
        setInterval: "readonly",
        clearInterval: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        HTMLElement: "readonly",
        HTMLImageElement: "readonly",
        HTMLDivElement: "readonly",
        ResizeObserver: "readonly",
        KeyboardEvent: "readonly",
        global: "readonly",
      },
    },
    plugins: {
      "@eslint-react": eslintReact,
      "@typescript-eslint": tseslint.plugin,
      "react-hooks": reactHooksPlugin,
    },
    rules: {
      // TypeScript 规则
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/explicit-module-boundary-types": "off",

      // React 规则
      ...eslintReact.configs["recommended-typescript"].rules,
      "@eslint-react/exhaustive-deps": "off",
      "@eslint-react/no-context-provider": "off",
      "@eslint-react/no-unnecessary-use-prefix": "off",
      "@eslint-react/no-use-context": "off",
      "@eslint-react/purity": "off",
      "@eslint-react/rules-of-hooks": "off",
      "@eslint-react/set-state-in-effect": "off",
      "@eslint-react/use-state": "off",

      // React Hooks 规则
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",

      // 通用规则
      "no-console": ["warn", { allow: ["warn", "error"] }],
      "no-debugger": "warn",
      "no-unused-vars": "off", // 使用 TypeScript 的版本
    },
    settings: eslintReact.configs["recommended-typescript"].settings,
  },

  // Prettier 配置（必须在最后，避免规则冲突）
  prettierConfig,
];
