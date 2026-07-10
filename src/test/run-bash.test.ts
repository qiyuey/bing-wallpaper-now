/// <reference types="node" />

import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

describe("Bash launcher", () => {
  it("runs Bash without requiring it on the Windows PATH", () => {
    const launcher = join(process.cwd(), "scripts", "run-bash.mjs");
    const result = spawnSync(
      process.execPath,
      [launcher, "-c", "printf bash-launcher-ok"],
      {
        cwd: process.cwd(),
        encoding: "utf8",
      },
    );

    expect(result.status).toBe(0);
    expect(result.stdout).toBe("bash-launcher-ok");
  });
});
