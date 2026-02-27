#!/usr/bin/env node

/**
 * winload-rust-bin — CLI entry point
 *
 * 定位当前平台的预编译二进制并透传所有参数执行。
 * 用户通过 `npx winload-rust-bin` 或全局安装后直接 `winload` 即可运行。
 */

"use strict";

const { spawnSync } = require("child_process");
const { getBinaryPath } = require("./index.js");

const bin = getBinaryPath();

// Show implementation info for help & version flags
const args = process.argv.slice(2);
if (args.includes("--help") || args.includes("-h") || args.includes("--version") || args.includes("-V")) {
  console.error("ℹ️  This is the Rust binary edition (installed from npm)\n");
}

const result = spawnSync(bin, args, {
  stdio: "inherit",
  windowsHide: false,
});

process.exit(result.status ?? 1);
