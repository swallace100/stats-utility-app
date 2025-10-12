import js from "@eslint/js";
import tsPlugin from "@typescript-eslint/eslint-plugin";
import tsParser from "@typescript-eslint/parser";
import importPlugin from "eslint-plugin-import";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import unusedImports from "eslint-plugin-unused-imports";
import globals from "globals";

export default [
  // ignore heavy/build dirs
  {
    ignores: [
      "node_modules/**",
      "dist/**",
      "build/**",
      "coverage/**",
      "**/target/**",
    ],
  },
  {
    files: ["**/*.{js,jsx,ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2023,
      sourceType: "module",
      globals: { ...globals.browser, ...globals.node },
      parser: tsParser,
      parserOptions: {
        // if your tsconfig is in frontend/tsconfig.json, ESLint will pick it up.
        // If you use a special tsconfig for linting, set: project: "./tsconfig.eslint.json"
      },
    },
    plugins: {
      "@typescript-eslint": tsPlugin,
      react,
      "react-hooks": reactHooks,
      import: importPlugin,
      "unused-imports": unusedImports,
    },
    settings: { react: { version: "detect" } },
    rules: {
      // JS recommended
      ...js.configs.recommended.rules,

      // TS recommended (non-type-aware to avoid project config headaches)
      ...tsPlugin.configs.recommended.rules,

      // React
      ...react.configs.recommended.rules,
      "react/react-in-jsx-scope": "off",

      // Hooks
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",

      // Imports & hygiene
      "unused-imports/no-unused-imports": "error",
      "import/order": [
        "warn",
        {
          "newlines-between": "always",
          alphabetize: { order: "asc", caseInsensitive: true },
        },
      ],
    },
  },
];
