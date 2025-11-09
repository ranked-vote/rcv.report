export default {
  testEnvironment: "node",
  transform: {},
  moduleNameMapper: {},
  testMatch: ["**/tests/**/*.test.{js,mjs}"],
  collectCoverageFrom: [
    "scripts/**/*.mjs",
    "src/**/*.{js,ts,svelte}",
    "!src/**/*.d.ts",
  ],
  testTimeout: 30000,
};
