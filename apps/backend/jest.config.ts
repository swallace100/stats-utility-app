import type { Config } from "jest";

const config: Config = {
  preset: "ts-jest",
  testEnvironment: "node",
  testMatch: ["**/tests/**/*.test.ts"],
  moduleFileExtensions: ["ts", "js", "json"],
  roots: ["<rootDir>"],
  // so imports like "../src/app" work
  moduleNameMapper: {
    "^(\\.{1,2}/.*)$": "$1"
  }
};

export default config;
