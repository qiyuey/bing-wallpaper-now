/**
 * Enhanced ESLint configuration
 * Includes integrated Prettier formatting and refined TypeScript/React/Import rules.
 *
 * Install dependencies:
 *   npm i -D eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin eslint-plugin-react eslint-plugin-react-hooks eslint-plugin-import prettier eslint-config-prettier eslint-plugin-prettier
 *
 * Run:
 *   npx eslint "src/**/*.{ts,tsx}"
 *
 * Format check (after adding prettier script):
 *   npx prettier --check .
 */

module.exports = {
  root: true,
  env: {
    browser: true,
    es2023: true,
    node: true,
  },
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
    project: undefined, // set to ./tsconfig.json if you want project-aware rules
    ecmaFeatures: {
      jsx: true,
    },
  },
  plugins: [
    "react",
    "react-hooks",
    "@typescript-eslint",
    "import",
    "prettier",
  ],
  extends: [
    "eslint:recommended",
    "plugin:react/recommended",
    "plugin:react-hooks/recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:import/recommended",
    "plugin:import/typescript",
    "plugin:prettier/recommended",
  ],
  settings: {
    react: {
      version: "detect",
    },
    "import/resolver": {
      typescript: {
        alwaysTryTypes: true,
      },
    },
  },
  overrides: [
    {
      files: ["*.cjs", "*.config.js", "*.config.cjs"],
      env: { node: true },
      rules: {
        "@typescript-eslint/no-var-requires": "off",
      },
    },
    {
      files: ["*.d.ts"],
      rules: {
        "@typescript-eslint/no-unused-vars": "off",
      },
    },
  ],
  rules: {
    // Core TypeScript
    "@typescript-eslint/explicit-module-boundary-types": "off",
    "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
    "@typescript-eslint/consistent-type-imports": ["warn", { prefer: "type-imports", fixStyle: "separate-type-imports" }],
    "@typescript-eslint/no-floating-promises": "warn",

    // React / JSX
    "react/prop-types": "off",
    "react/jsx-uses-react": "off",
    "react/react-in-jsx-scope": "off",

    // Hooks
    "react-hooks/rules-of-hooks": "error",
    "react-hooks/exhaustive-deps": "warn",

    // Imports
    "import/order": [
      "warn",
      {
        groups: ["builtin", "external", "internal", ["parent", "sibling"], "index", "object", "type"],
        "newlines-between": "always",
        alphabetize: { order: "asc", caseInsensitive: true },
      },
    ],
    "import/no-unresolved": "error",

    // General Hygiene
    "no-console": ["warn", { allow: ["warn", "error", "info"] }],

    // Prettier integration (will surface formatting issues as lint errors)
    "prettier/prettier": "error",
  },
  ignorePatterns: [
    "dist/",
    "node_modules/",
    "src-tauri/target/",
    "*.d.ts",
    "*.svg",
    "*.ico",
  ],
};
