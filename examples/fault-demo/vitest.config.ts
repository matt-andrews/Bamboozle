import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // Integration tests against a real Docker container. The flaky-route
    // test runs 50 requests sequentially; give it room.
    testTimeout: 30_000,
    hookTimeout: 30_000,
    // Run sequentially — these tests share Bamboozle state via the control
    // API, and parallel execution would make call-count assertions racy.
    pool: "forks",
    poolOptions: {
      forks: { singleFork: true },
    },
  },
});
