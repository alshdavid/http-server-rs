#!/usr/bin/env node

import * as child_process from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import * as process from "node:process";
import * as url from "node:url";

const PACKAGE = `http-server-rs-${process.platform}-${process.arch}`;

const SUPPORTED = [
  "linux-x64",
  "linux-arm64",
  "darwin-arm64",
  "win32-x64",
  "win32-arm64",
];

function resolveBinary() {
  let manifestPath;
  try {
    manifestPath = url.fileURLToPath(
      import.meta.resolve(`${PACKAGE}/package.json`),
    );
  } catch {
    return undefined;
  }

  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  const bin =
    typeof manifest.bin === "string" ? manifest.bin : manifest.bin?.[PACKAGE];
  if (!bin) {
    return undefined;
  }

  return path.resolve(path.dirname(manifestPath), bin);
}

const override = process.env.HTTP_SERVER_RS_BIN_OVERRIDE;

let binary;

if (override) {
  if (!fs.existsSync(override)) {
    console.error(
      `http-server-rs: HTTP_SERVER_RS_BIN_OVERRIDE is set, but no file exists at:\n` +
        `  ${override}\n\n` +
        `Unset it to use the binary for ${process.platform}-${process.arch}.`,
    );
    process.exit(1);
  }
  binary = override;
} else {
  binary = resolveBinary();

  if (!binary) {
    console.error(
      `http-server-rs: no prebuilt binary for ${process.platform}-${process.arch}.\n\n` +
        `Expected the optional dependency "${PACKAGE}" to be installed.\n` +
        `If your platform is supported, npm may have been run with --omit=optional.\n\n` +
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
  process.exit(1);
}

if (result.signal) {
  process.kill(process.pid, result.signal);
}

process.exit(result.status ?? 1);
