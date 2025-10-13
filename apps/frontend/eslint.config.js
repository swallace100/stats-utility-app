import js from "@eslint/js";
import ts from "typescript-eslint";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import importPlugin from "eslint-plugin-import";
import unusedImports from "eslint-plugin-unused-imports";
import globals from "globals";

export default [
  // Ignore generated / cache dirs & tooling configs
  {
    ignores: [
      "**/*.d.ts",
      "**/node_modules/**",
      "**/dist/**",
      "**/build/**",
      "**/coverage/**",
      "**/.vite/**",
      "**/.turbo/**",
      "**/.cache/**",
      "vite.config.*",
      "tailwind.config.*",
      "postcss.config.*",
      "eslint.config.*",
    ],
  },

  // Only lint TS/TSX in src with the TS parser (no project to avoid path issues)
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      parser: ts.parser,
      parserOptions: {
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: { jsx: true },
      },
      globals: {
        ...globals.browser,
        ...globals.node,
        fetch: "readonly",
        Request: "readonly",
        Response: "readonly",
        RequestInit: "readonly",
        FormData: "readonly",
        URL: "readonly",
      },
    },
    plugins: {
      "@typescript-eslint": ts.plugin,
      react,
      "react-hooks": reactHooks,
      import: importPlugin,
      "unused-imports": unusedImports,
    },
    settings: { react: { version: "detect" } },
    rules: {
      ...js.configs.recommended.rules,
      ...ts.configs.recommended.rules, // non-type-aware → no tsconfig path headaches
      ...react.configs.recommended.rules,
      "react/react-in-jsx-scope": "off",
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
      "unused-imports/no-unused-imports": "error",
      "import/order": [
        "warn",
        {
          "newlines-between": "always",
          alphabetize: { order: "asc", caseInsensitive: true },
        },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
    },
  },
];
