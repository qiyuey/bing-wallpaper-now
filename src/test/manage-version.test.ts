/// <reference types="node" />

import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const launcher = join(process.cwd(), "scripts", "run-bash.mjs");

function resolveReleaseTarget(version: string, level?: string) {
  const command = [
    "source scripts/lib/version.sh",
    `version_resolve_release_target ${JSON.stringify(version)} ${JSON.stringify(level ?? "")}`,
  ].join("; ");

  return spawnSync(process.execPath, [launcher, "-c", command], {
    cwd: process.cwd(),
    encoding: "utf8",
  });
}

describe("release version resolution", () => {
  it("releases an existing development version without another bump", () => {
    const result = resolveReleaseTarget("1.5.7-0");

    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe("1.5.7");
  });

  it.each([
    ["patch", "1.5.7"],
    ["minor", "1.6.0"],
    ["major", "2.0.0"],
  ])(
    "supports a direct %s release from a production version",
    (level, target) => {
      const result = resolveReleaseTarget("1.5.6", level);

      expect(result.status).toBe(0);
      expect(result.stdout.trim()).toBe(target);
    },
  );

  it("requires a release level for a production version", () => {
    const result = resolveReleaseTarget("1.5.6");

    expect(result.status).not.toBe(0);
  });

  it("rejects a second bump for an existing development version", () => {
    const result = resolveReleaseTarget("1.5.7-0", "patch");

    expect(result.status).not.toBe(0);
  });
});
