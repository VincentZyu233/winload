# 建置與發佈工作流

> **[📖 English](build.md)**
> **[📖 简体中文(大陆)](build.zh-cn.md)**
> **[📖 繁體中文(台灣)](bulid.zh-tw.md)**

## 📋 概述

CI/CD 流程完全由 **commit 資訊中的關鍵字** 驅動。推送至 `main` 分支時，只需在 commit message 中包含對應關鍵字，GitHub Actions 就會自動完成後續作業。

## 🔑 關鍵字

| Commit 資訊中的關鍵字 | 建置（6 平台） | GitHub Release | Scoop Bucket |
|----------------------|:---:|:---:|:---:|
| *（無關鍵字）* | ❌ | ❌ | ❌ |
| `build action` | ✅ | ❌ | ❌ |
| `build release` | ✅ | ✅ | ❌ |
| `publish from release` | ❌ | ❌ | ✅ |
| `build publish` | ✅ | ✅ | ✅ |

> **說明:** `publish from release` 從現有的 Release 抓取二進位檔發布，不會重新建置。`build publish` 則是完整的流程。

> **說明:** Pull Request 都會觸發建置（不會發布或推送套件管理工具）。PR 中 commit message 的關鍵字會被**忽略**——工作流程會無條件設定 `should_build=true`、`should_release=false`、`should_publish=false`，並跳過關鍵字解析。

## 🚀 用法範例

```bash
# 僅建置，驗證所有平台的編譯
git commit --allow-empty -m "ci: test cross-compile (build action)"

# 建置 + 建立 GitHub Release
git commit -m "release: v0.2.0 (build release)"

# 僅更新 Scoop bucket（從現有的最新 Release 抓取二進位檔，不重新建置）
git commit --allow-empty -m "ci: update scoop (publish from release)"

# 完整流程：建置 + 發布 Release + 推送 Scoop
git commit -m "release: v0.2.0 (build publish)"
```

## 🏗️ 建置目標

| 平台 | 架構 | Target | 說明 |
|------|:---:|--------|------|
| Windows | x64 | `x86_64-pc-windows-msvc` | 原生 MSVC 編譯 |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | 在 x64 runner 上交叉編譯 |
| Linux | x64 | `x86_64-unknown-linux-musl` | musl 靜態連結，可攜帶 |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | 在 ubuntu-22.04 上編譯，降低 GLIBC 需求 |
| macOS | x64 | `x86_64-apple-darwin` | 在 Apple Silicon runner 上編譯 |
| macOS | ARM64 | `aarch64-apple-darwin` | 原生 Apple Silicon |

## 📦 流程階段

```
check ──→ build ──→ release ──→ publish-scoop
  │         │         │              │
  │         │         │              └─ 從 Release 下載二進位檔
  │         │         │                 生成 winload.json
  │         │         │                 推送至 scoop-bucket 儲存庫
  │         │         │
  │         │         └─ 下載建置產物
  │         │            刪除舊的 release/tag
  │         │            生成 release notes
  │         │            建立 GitHub Release
  │         │
  │         └─ 編譯 6 個平台目標
  │            上傳建置產物
  │
  └─ 解析 commit 資訊關鍵字
     從 Cargo.toml 擷取版本號
```

## 🍺 Scoop 發佈

`publish` 關鍵字會觸發 [scoop-bucket](https://github.com/VincentZyuApps/scoop-bucket) 儲存庫的更新：

1. 從最新的 GitHub Release 下載 Windows x64 和 ARM64 二進位檔案
2. 計算 SHA256 雜湊值
3. 生成 `winload.json` 清單檔案（包含 `64bit` 和 `arm64` 兩種架構）
4. 推送至 `VincentZyuApps/scoop-bucket` 儲存庫

### 前置條件

需在儲存庫的 **Settings → Secrets → Actions** 中設定 `SCOOP_BUCKET_TOKEN` 金鑰，值為具備 `repo` 權限的 GitHub Personal Access Token。

## 📌 版本號

版本號自動從 `rust/Cargo.toml` 中擷取，用於：
- Release 標籤名（如 `v0.1.5`）
- 產物檔名（如 `winload-windows-x86_64-v0.1.5.exe`）
- Scoop 清單檔案中的版本欄位
