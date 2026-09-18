#!/usr/bin/env node

import * as child_process from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import * as process from "node:process";
import * as url from "node:url";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

const SUPPORTED = [
  "linux-x64",
  "linux-arm64",
  "darwin-arm64",
  "win32-x64",
  "win32-arm64",
];

const TARGET = `${process.platform}-${process.arch}`;
const EXTENSION = process.platform === "win32" ? ".exe" : "";

const override = process.env.HTTP_SERVER_RS_BIN_OVERRIDE;

let binary;

if (override) {
  if (!fs.existsSync(override)) {
    console.error(
      `http-server-rs: HTTP_SERVER_RS_BIN_OVERRIDE is set, but no file exists at:\n` +
        `  ${override}\n\n` +
        `Unset it to use the bundled binary for ${TARGET}.`,
    );
    process.exit(1);
  }
  binary = override;
} else {
  binary = path.join(__dirname, `http-server-rs-${TARGET}${EXTENSION}`);

  if (!fs.existsSync(binary)) {
    console.error(
      `http-server-rs: no bundled binary for ${TARGET}.\n\n` +
        `Expected it at:\n` +
        `  ${binary}\n\n` +
        `Supported platforms: ${SUPPORTED.join(", ")}\n` +
        `Set HTTP_SERVER_RS_BIN_OVERRIDE to run a binary from an explicit path.\n` +
        `https://github.com/alshdavid/http-server-rs`,
    );
    process.exit(1);
  }
}

const result = child_process.spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: true,
});

if (result.error) {
  console.error(`http-server-rs: failed to execute ${binary}`);
  console.error(result.error.message);
  process.exit(1);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status ?? 1);
