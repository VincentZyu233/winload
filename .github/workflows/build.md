# Build & Release Workflow

> **[📖 中文文档](build.zh-cn.md)**

## 📋 Overview

The CI/CD pipeline is driven entirely by **commit message keywords**. Push to `main` with the right keyword and GitHub Actions takes care of the rest.

## 🔑 Keywords

| Keyword in commit message | Build (6 platforms) | GitHub Release | Scoop Bucket |
|---------------------------|:---:|:---:|:---:|
| *(none)* | ❌ | ❌ | ❌ |
| `build action` | ✅ | ❌ | ❌ |
| `build release` | ✅ | ✅ | ❌ |
| `publish from release` | ❌ | ❌ | ✅ |
| `build publish` | ✅ | ✅ | ✅ |

> **Note:** `publish from release` fetches binaries from an existing Release without rebuilding. `build publish` does the full pipeline.

> **Note:** Pull Requests always trigger a build (no release or publish).

## 🚀 Usage Examples

```bash
# Just build, verify compilation across all platforms
git commit --allow-empty -m "ci: test cross-compile (build action)"

# Build + create GitHub Release with artifacts
git commit -m "release: v0.2.0 (build release)"

# Only update Scoop bucket from the latest existing Release (no rebuild)
git commit --allow-empty -m "ci: update scoop (publish from release)"

# Full pipeline: build + release + publish to Scoop
git commit -m "release: v0.2.0 (build publish)"
```

## 🏗️ Build Targets

| Platform | Architecture | Target | Notes |
|----------|:---:|--------|-------|
| Windows | x64 | `x86_64-pc-windows-msvc` | Native MSVC |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | Cross-compiled on x64 runner |
| Linux | x64 | `x86_64-unknown-linux-musl` | Static linking (musl), portable |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | Built on ubuntu-22.04 for lower GLIBC |
| macOS | x64 | `x86_64-apple-darwin` | Built on Apple Silicon runner |
| macOS | ARM64 | `aarch64-apple-darwin` | Native Apple Silicon |

## 📦 Pipeline Stages

```
check ──→ build ──→ release ──→ publish-scoop
  │         │         │              │
  │         │         │              └─ Download binaries from Release
  │         │         │                 Generate winload.json
  │         │         │                 Push to scoop-bucket repo
  │         │         │
  │         │         └─ Download artifacts
  │         │            Delete old release/tag
  │         │            Generate release notes
  │         │            Create GitHub Release
  │         │
  │         └─ Compile for 6 platform targets
  │            Upload build artifacts
  │
  └─ Parse commit message keywords
     Extract version from Cargo.toml
```

## 🍺 Scoop Publish

The `publish` keyword triggers an update to the [scoop-bucket](https://github.com/VincentZyuApps/scoop-bucket) repository:

1. Downloads Windows x64 and ARM64 binaries from the latest GitHub Release
2. Computes SHA256 hashes
3. Generates `winload.json` manifest (with both `64bit` and `arm64` architecture support)
4. Pushes to `VincentZyuApps/scoop-bucket`

### Prerequisite

A repository secret `SCOOP_BUCKET_TOKEN` must be set in **Settings → Secrets → Actions**, containing a GitHub PAT with `repo` scope.

## 📌 Version

The version is automatically extracted from `rust/Cargo.toml` and used for:
- Release tag name (e.g. `v0.1.5`)
- Artifact filenames (e.g. `winload-windows-x86_64-v0.1.5.exe`)
- Scoop manifest version field
