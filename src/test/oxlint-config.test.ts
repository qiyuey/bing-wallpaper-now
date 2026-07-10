/// <reference types="node" />

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

describe("Oxlint configuration", () => {
  it("rejects conditional React Hook calls", () => {
    const fixtureDirectory = mkdtempSync(
      join(process.cwd(), "src/test/.oxlint-"),
    );
    const fixturePath = join(fixtureDirectory, "ConditionalHook.tsx");
    const cliPath = join(
      process.cwd(),
      "node_modules",
      "oxlint",
      "bin",
      "oxlint",
    );

    writeFileSync(
      fixturePath,
      `import { useState } from "react";

export function ConditionalHook({ enabled }: { enabled: boolean }) {
  if (enabled) {
    useState(0);
  }
  return null;
}
`,
    );

    try {
      const result = spawnSync(process.execPath, [cliPath, fixturePath], {
        cwd: process.cwd(),
        encoding: "utf8",
      });
      const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;

      expect(result.status).not.toBe(0);
      expect(output).toContain("react-hooks(rules-of-hooks)");
    } finally {
      rmSync(fixtureDirectory, { recursive: true, force: true });
    }
  });
});
