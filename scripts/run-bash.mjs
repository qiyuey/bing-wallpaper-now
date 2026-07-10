#!/usr/bin/env node

import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = process.argv.slice(2);

if (args.length === 0) {
  console.error("Usage: node scripts/run-bash.mjs <script> [...args]");
  process.exit(1);
}

let bash = "bash";
if (process.platform === "win32") {
  const candidates = [
    process.env.SHELL,
    "C:\\Program Files\\Git\\bin\\bash.exe",
    "C:\\Program Files\\Git\\usr\\bin\\bash.exe",
    join(homedir(), "AppData", "Local", "Programs", "Git", "bin", "bash.exe"),
  ].filter(Boolean);
  bash = candidates.find((candidate) => existsSync(candidate));

  if (!bash) {
    console.error("Git Bash was not found. Install Git for Windows first.");
    process.exit(1);
  }
}

const result = spawnSync(bash, args, {
  cwd: projectRoot,
  env: process.env,
  stdio: "inherit",
});

if (result.error) {
  console.error(`Unable to start Bash: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
