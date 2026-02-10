# Winload 多平台发布自动化方案

> 通过 GitHub Actions + Commit Message 实现 Windows、Linux、macOS、Termux 等多平台包管理器的自动化发布

## 🎮 触发机制

基于 **commit message** 的三级触发：

| Commit Message | 行为 | 说明 |
|----------------|------|------|
| `build action` | 仅构建二进制 | 测试编译，不上传产物 |
| `build release` | 构建 + GitHub Release | 创建 Release，上传二进制文件 |
| `build publish` | 构建 + Release + 发布到包管理器 | 完整发布流程，更新所有平台 |

**优势：**
- ✅ 无需打 tag，更灵活
- ✅ 可以随时重新发布
- ✅ 清晰的发布意图

---

## 📦 目标平台

### Windows
- ✅ **Scoop** - 最推荐，用户友好
- ✅ **Winget** - Windows 官方包管理器
- ⚠️ **Chocolatey** - 需要审核，周期较长

### Linux
- ✅ **DEB** (Debian/Ubuntu) - apt 安装
- ✅ **RPM** (Fedora/RHEL) - dnf/yum 安装
- ✅ **APK** (Alpine) - apk 安装
- ✅ **AUR** (Arch) - paru/yay 安装

### macOS
- ✅ **Homebrew** - 主流方案

### Termux
- ✅ **Termux APK** - 独立打包

---

## 🚀 实施方案

### 版本号管理

**方案 1：从 Cargo.toml 提取（推荐）**
```bash
# 在 workflow 中自动读取
VERSION=$(grep '^version' rust/Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
```

**方案 2：从 commit message 提取**
```bash
# commit: "build publish v1.2.3"
VERSION=$(echo "$MSG" | grep -oP 'v\d+\.\d+\.\d+' | head -1)
```

---

### 阶段 1：增强现有 build.yml

#### 1.1 添加 `build publish` 支持

修改 `.github/workflows/build.yml` 的 check job：

```yaml
jobs:
  check:
    name: Check commit message
    runs-on: ubuntu-latest
    outputs:
      should_build: ${{ steps.flags.outputs.should_build }}
      should_release: ${{ steps.flags.outputs.should_release }}
      should_publish: ${{ steps.flags.outputs.should_publish }}  # 新增
      version: ${{ steps.flags.outputs.version }}                # 新增
    steps:
      - uses: actions/checkout@v4
      
      - name: Parse commit message
        id: flags
        run: |
          MSG="${{ github.event.head_commit.message }}"
          
          # PR 始终构建
          if [ "${{ github.event_name }}" = "pull_request" ]; then
            echo "should_build=true"    >> "$GITHUB_OUTPUT"
            echo "should_release=false" >> "$GITHUB_OUTPUT"
            echo "should_publish=false" >> "$GITHUB_OUTPUT"
            exit 0
          fi
          
          # 提取版本号（从 Cargo.toml）
          VERSION=$(grep '^version' rust/Cargo.toml | sed 's/.*"\(.*\)".*/v\1/')
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"
          echo "📦 Version: $VERSION"
          
          # "build publish" = 完整发布流程
          if echo "$MSG" | grep -qi "build publish"; then
            echo "should_build=true"    >> "$GITHUB_OUTPUT"
            echo "should_release=true"  >> "$GITHUB_OUTPUT"
            echo "should_publish=true"  >> "$GITHUB_OUTPUT"
          # "build release" = 构建 + GitHub Release
          elif echo "$MSG" | grep -qi "build release"; then
            echo "should_build=true"    >> "$GITHUB_OUTPUT"
            echo "should_release=true"  >> "$GITHUB_OUTPUT"
            echo "should_publish=false" >> "$GITHUB_OUTPUT"
          # "build action" = 仅构建
          elif echo "$MSG" | grep -qi "build action"; then
            echo "should_build=true"    >> "$GITHUB_OUTPUT"
            echo "should_release=false" >> "$GITHUB_OUTPUT"
            echo "should_publish=false" >> "$GITHUB_OUTPUT"
          else
            echo "should_build=false"   >> "$GITHUB_OUTPUT"
            echo "should_release=false" >> "$GITHUB_OUTPUT"
            echo "should_publish=false" >> "$GITHUB_OUTPUT"
          fi
```

---

#### 1.2 添加 Release Job

在现有 `build` job 后添加：

```yaml
  # ── 创建 GitHub Release ──────────────────────────────────
  release:
    name: Create GitHub Release
    needs: [check, build]
    if: needs.check.outputs.should_release == 'true'
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
      
      - name: Create Release
        id: create_release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ needs.check.outputs.version }}
          name: Release ${{ needs.check.outputs.version }}
          draft: false
          prerelease: false
          generate_release_notes: true
          files: artifacts/**/*
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

#### 1.3 添加 Publish Jobs

```yaml
  # ── 发布到包管理器 ──────────────────────────────────────
  publish-scoop:
    name: Publish to Scoop
    needs: [check, release]
    if: needs.check.outputs.should_publish == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: VincentZyu233/scoop-bucket
          token: ${{ secrets.SCOOP_TOKEN }}
          path: scoop-bucket
      
      - name: Update Scoop manifest
        run: |
          VERSION="${{ needs.check.outputs.version }}"
          HASH=$(sha256sum artifacts/winload-windows-x86_64.exe | cut -d' ' -f1)
          
          cat > scoop-bucket/bucket/winload.json <<EOF
          {
            "version": "${VERSION#v}",
            "description": "Network Load Monitor - nload for Windows",
            "homepage": "https://github.com/VincentZyu233/winload",
            "license": "MIT",
            "architecture": {
              "64bit": {
                "url": "https://github.com/VincentZyu233/winload/releases/download/$VERSION/winload-windows-x86_64.exe",
                "hash": "$HASH"
              }
            },
            "bin": [["winload-windows-x86_64.exe", "win-nload"]],
            "checkver": "github",
            "autoupdate": {
              "architecture": {
                "64bit": {
                  "url": "https://github.com/VincentZyu233/winload/releases/download/v\$version/winload-windows-x86_64.exe"
                }
              }
            }
          }
          EOF
          
          cd scoop-bucket
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add bucket/winload.json
          git commit -m "winload: Update to $VERSION"
          git push

  publish-homebrew:
    name: Publish to Homebrew
    needs: [check, release]
    if: needs.check.outputs.should_publish == 'true'
    runs-on: macos-latest
    steps:
      - name: Update Homebrew formula
        uses: dawidd6/action-homebrew-bump-formula@v3
        with:
          token: ${{ secrets.HOMEBREW_TOKEN }}
          tap: VincentZyu233/homebrew-tap
          formula: winload
          tag: ${{ needs.check.outputs.version }}
          force: false

  publish-aur:
    name: Publish to AUR
    needs: [check, release]
    if: needs.check.outputs.should_publish == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Generate PKGBUILD
        run: |
          VERSION="${{ needs.check.outputs.version }}"
          SHA256=$(sha256sum artifacts/winload-linux-x86_64 | cut -d' ' -f1)
          
          cat > PKGBUILD <<EOF
          pkgname=winload-bin
          pkgver=${VERSION#v}
          pkgrel=1
          pkgdesc="Network Load Monitor - nload for Linux"
          arch=('x86_64')
          url="https://github.com/VincentZyu233/winload"
          license=('MIT')
          provides=('win-nload')
          conflicts=('win-nload')
          source=("https://github.com/VincentZyu233/winload/releases/download/$VERSION/winload-linux-x86_64")
          sha256sums=('$SHA256')

          package() {
              install -Dm755 "\$srcdir/winload-linux-x86_64" "\$pkgdir/usr/bin/win-nload"
          }
          EOF
      
      - name: Publish to AUR
        uses: KSXGitHub/github-actions-deploy-aur@v2.7.0
        with:
          pkgname: winload-bin
          pkgbuild: ./PKGBUILD
          commit_username: VincentZyu
          commit_email: ${{ secrets.AUR_EMAIL }}
          ssh_private_key: ${{ secrets.AUR_SSH_PRIVATE_KEY }}
          commit_message: "Update to ${{ needs.check.outputs.version }}"
```

---

#### 1.2 Scoop (Windows) ⭐ 最简单，最推荐

**工作流程：**
1. 创建独立仓库 `winload-scoop` (或使用 scoop-bucket 模板)
2. 在 Release 时自动更新 manifest

**实现方式 A：使用 scoop-bucket (推荐)**
```yaml
# .github/workflows/update-scoop.yml
name: Update Scoop Manifest

on:
  release:
    types: [published]

jobs:
  update-scoop:
    runs-on: ubuntu-latest
    steps:
      - name: Update Scoop Bucket
        uses: ScoopInstaller/GithubActions@main
        with:
          app: winload
          manifest: |
            {
              "version": "${{ github.event.release.tag_name }}",
              "description": "Network Load Monitor",
              "homepage": "https://github.com/VincentZyu233/winload",
              "license": "MIT",
              "architecture": {
                "64bit": {
                  "url": "https://github.com/VincentZyu233/winload/releases/download/${{ github.event.release.tag_name }}/winload-windows-x86_64.exe",
                  "hash": "${{ steps.hash.outputs.hash }}"
                }
              },
              "bin": [["winload-windows-x86_64.exe", "win-nload"]]
            }
          bucket_repo: VincentZyu233/scoop-bucket
          token: ${{ secrets.SCOOP_TOKEN }}
```

**用户安装方式：**
```powershell
scoop bucket add vincentzyu https://github.com/VincentZyu233/scoop-bucket
scoop install winload
```

---

#### 1.3 Homebrew (macOS) ⭐

**工作流程：**
1. 创建 `homebrew-tap` 仓库
2. 自动生成 Formula

```yaml
# .github/workflows/update-homebrew.yml
name: Update Homebrew Formula

on:
  release:
    types: [published]

jobs:
  update-homebrew:
    runs-on: macos-latest
    steps:
      - name: Update Homebrew formula
        uses: dawidd6/action-homebrew-bump-formula@v3
        with:
          token: ${{ secrets.HOMEBREW_TOKEN }}
          tap: VincentZyu233/homebrew-tap
          formula: winload
          tag: ${{ github.ref }}
          force: false
```

**手动创建 Formula (首次)：**
```ruby
# Formula/winload.rb
class Winload < Formula
  desc "Network Load Monitor - nload for Windows/Linux/macOS"
  homepage "https://github.com/VincentZyu233/winload"
  version "0.1.0"
  
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-macos-aarch64"
      sha256 "xxx"
    else
      url "https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-macos-x86_64"
      sha256 "xxx"
    end
  end

  def install
    bin.install "winload-macos-aarch64" => "win-nload" if Hardware::CPU.arm?
    bin.install "winload-macos-x86_64" => "win-nload" unless Hardware::CPU.arm?
  end

  test do
    system "#{bin}/win-nload", "--version"
  end
end
```

**用户安装：**
```bash
brew tap VincentZyu233/tap
brew install winload
```

---

### 阶段 2：Linux 包管理器

#### 2.1 使用 cargo-deb 和 cargo-generate-rpm

在 `build` job 中添加包构建步骤：

```yaml
  build:
    name: Build ${{ matrix.name }}
    needs: check
    if: needs.check.outputs.should_build == 'true'
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: linux-x86_64
            build_packages: true  # 新增标记
          # ... 其他平台
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build binary
        working-directory: rust
        run: cargo build --release --target ${{ matrix.target }}
      
      # Linux 平台额外构建包
      - name: Install packaging tools
        if: matrix.build_packages == 'true'
        run: cargo install cargo-deb cargo-generate-rpm
      
      - name: Build DEB
        if: matrix.build_packages == 'true'
        working-directory: rust
        run: cargo deb --target ${{ matrix.target }} --no-build
      
      - name: Build RPM
        if: matrix.build_packages == 'true'
        working-directory: rust
        run: cargo generate-rpm --target ${{ matrix.target }}
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: |
            rust/target/${{ matrix.target }}/release/winload*
            rust/target/${{ matrix.target }}/debian/*.deb
            rust/target/${{ matrix.target }}/generate-rpm/*.rpm
```

**配置 Cargo.toml：**
```toml
[package.metadata.deb]
maintainer = "VincentZyu <your@email.com>"
copyright = "2026, VincentZyu"
license-file = ["../LICENSE", "0"]
extended-description = """\
A lightweight, real-time CLI tool for monitoring network bandwidth and traffic."""
depends = "$auto"
section = "utils"
priority = "optional"
assets = [
    ["target/release/winload", "usr/bin/win-nload", "755"],
]

[package.metadata.generate-rpm]
assets = [
    { source = "target/release/winload", dest = "/usr/bin/win-nload", mode = "755" },
]
```

**用户安装：**
```bash
# Debian/Ubuntu
wget https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload_0.1.0_amd64.deb
sudo dpkg -i winload_0.1.0_amd64.deb

# Fedora/RHEL
sudo rpm -i winload-0.1.0-1.x86_64.rpm
```

---

#### 2.2 AUR (Arch Linux) ⭐

**已集成到上面的 `publish-aur` job**

创建独立的 AUR 仓库配置（可选，用于手动维护）：
```bash
# aur-winload 仓库中的 PKGBUILD
pkgname=winload-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Network Load Monitor"
arch=('x86_64')
url="https://github.com/VincentZyu233/winload"
license=('MIT')
provides=('win-nload')
conflicts=('win-nload')
source=("https://github.com/VincentZyu233/winload/releases/download/v${pkgver}/winload-linux-x86_64")
sha256sums=('SKIP')

package() {
    install -Dm755 "$srcdir/winload-linux-x86_64" "$pkgdir/usr/bin/win-nload"
}
```

**用户安装：**
```bash
paru -S winload-bin
# 或
yay -S winload-bin
```

---

#### 2.3 Alpine APK

**使用 cargo-apk 或手动打包：**
```yaml
- name: Build Alpine APK
  run: |
    docker run --rm -v "$PWD:/build" alpine:latest sh -c "
      apk add --no-cache cargo rust build-base
      cd /build/rust
      cargo build --release --target x86_64-unknown-linux-musl
    "
```

**或使用 Alpine 专用打包工具 (需要额外维护 APKBUILD)**

---

### 阶段 3：高级平台

#### 3.1 Winget (Windows)

添加 `publish-winget` job：

```yaml
  publish-winget:
    name: Publish to Winget
    needs: [check, release]
    if: needs.check.outputs.should_publish == 'true'
    runs-on: windows-latest
    steps:
      - uses: vedantmgoyal2009/winget-releaser@v2
        with:
          identifier: VincentZyu.winload
          version: ${{ needs.check.outputs.version }}
          installers-regex: 'winload-windows-x86_64\.exe$'
          token: ${{ secrets.GITHUB_TOKEN }}
```

⚠️ **首次需要向 microsoft/winget-pkgs 提交 PR 创建 manifest**

---

#### 3.2 Chocolatey (Windows)

**流程复杂，需要：**
1. 手动创建 `.nuspec` 和 `chocolateyInstall.ps1`
2. 提交到 Chocolatey 社区审核
3. 通过后才能自动化更新

**不推荐作为首选，Scoop 更简单**

---

#### 3.3 Termux APK

**方案：构建 Termux 兼容的 APK**

```yaml
- name: Build for Termux
  run: |
    rustup target add aarch64-linux-android
    cargo build --release --target aarch64-linux-android
    
- name: Package for Termux
  run: |
    mkdir -p termux-package/data/data/com.termux/files/usr/bin
    cp target/aarch64-linux-android/release/winload \
       termux-package/data/data/com.termux/files/usr/bin/
```

**用户安装：**
```bash
# Termux
pkg install winload  # 需要提交到 termux-packages 仓库
```

---

## 🎯 推荐实施优先级

### 🥇 第一优先级（简单且用户多）
1. **Scoop** - 最简单，Windows 用户友好
2. **Homebrew** - macOS 主流方案
3. **GitHub Release** - 所有平台的基础

### 🥈 第二优先级（Linux 主流）
4. **DEB** - Ubuntu/Debian 用户
5. **AUR** - Arch Linux 用户
6. **RPM** - Fedora/RHEL 用户

### 🥉 第三优先级（可选）
7. **Winget** - 需要初次提交，但之后可自动化
8. **Alpine APK** - 相对小众
9. **Termux** - 需要单独维护

### ⏸️ 低优先级
10. **Chocolatey** - 审核周期长，不如 Scoop

---

## 📝 实施步骤

### Step 1: 准备工作
```bash
# 1. 确保 Cargo.toml 版本号正确
# rust/Cargo.toml
version = "0.1.0"

# 2. 添加打包元数据
[package.metadata.deb]
# ... (见上文配置)

[package.metadata.generate-rpm]
# ... (见上文配置)
```

### Step 2: 修改现有 build.yml
```bash
# 按照上文的方案修改 .github/workflows/build.yml
# - 添加 should_publish 输出
# - 添加 version 提取
# - 添加 release job
# - 添加 publish-* jobs
```

### Step 3: 创建依赖仓库
```bash
# Scoop Bucket
gh repo create scoop-bucket --public
mkdir -p bucket
echo "{}" > bucket/winload.json

# Homebrew Tap
gh repo create homebrew-tap --public
mkdir -p Formula
# Formula/winload.rb 见上文
```

### Step 4: 配置 Secrets
在 GitHub 仓库 Settings → Secrets 添加：
- `SCOOP_TOKEN` - GitHub PAT with repo 权限
- `HOMEBREW_TOKEN` - 同上
- `AUR_SSH_PRIVATE_KEY` - AUR SSH 私钥（需要先注册 AUR 账号）
- `AUR_EMAIL` - AUR 提交邮箱

### Step 5: 测试发布流程

**测试构建：**
```bash
git commit -m "test: build action - 测试编译"
git push
# 查看 Actions，应该只构建二进制
```

**测试 Release：**
```bash
git commit -m "feat: 新功能 build release"
git push
# 查看 Actions 和 Releases 页面，应该创建 Release
```

**完整发布：**
```bash
# 1. 确保 Cargo.toml 版本号正确
# 2. 提交代码
git add -A
git commit -m "chore: 发布 v0.1.0 build publish"
git push

# 3. 等待 Actions 完成
# 4. 验证：
# - GitHub Releases 有新版本
# - Scoop bucket 已更新
# - Homebrew tap 已更新
# - AUR 已更新
```

---

## 🔑 所需 Secrets

在 GitHub 仓库 Settings → Secrets and variables → Actions 添加：

| Secret Name | 用途 | 获取方式 |
|-------------|------|----------|
| `SCOOP_TOKEN` | 更新 scoop-bucket 仓库 | GitHub Settings → Developer settings → Personal access tokens → Generate new token (repo 权限) |
| `HOMEBREW_TOKEN` | 更新 homebrew-tap 仓库 | 同上 |
| `AUR_SSH_PRIVATE_KEY` | 推送到 AUR | `ssh-keygen -t ed25519`，然后在 [AUR 网站](https://aur.archlinux.org/account/) 添加公钥 |
| `AUR_EMAIL` | AUR git 提交邮箱 | 你的 AUR 账号邮箱 |

---

## 🧪 完整发布流程示例

```bash
# 1. 开发新功能
# 2. 更新版本号
vi rust/Cargo.toml  # version = "0.2.0"

# 3. 测试编译
git add -A
git commit -m "test: build action"
git push
# ✅ 触发构建，不发布

# 4. 确认无误后，发布到 GitHub Release
git commit --amend -m "feat: 添加新功能 build release"
git push --force
# ✅ 触发构建 + 创建 Release

# 5. 如果需要同步到所有包管理器
git commit --amend -m "feat: 添加新功能 v0.2.0 build publish"
git push --force
# ✅ 完整发布流程：
#    - 构建所有平台二进制
#    - 创建 GitHub Release
#    - 更新 Scoop
#    - 更新 Homebrew
#    - 更新 AUR
#    - 构建 DEB/RPM
```

**发布后验证：**
```bash
# Windows (Scoop)
scoop update
scoop install winload

# macOS (Homebrew)
brew update
brew install vincentzyu/tap/winload

# Linux (AUR)
paru -Sy winload-bin

# Linux (手动)
wget https://github.com/VincentZyu233/winload/releases/download/v0.2.0/winload_0.2.0_amd64.deb
```

---

## 📚 参考资源

- [cargo-deb](https://github.com/kornelski/cargo-deb)
- [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm)
- [Scoop Bucket](https://github.com/ScoopInstaller/Scoop)
- [Homebrew Formula](https://docs.brew.sh/Formula-Cookbook)
- [AUR PKGBUILD](https://wiki.archlinux.org/title/PKGBUILD)
- [winget-releaser](https://github.com/vedantmgoyal2009/winget-releaser)

---

## 💡 最佳实践建议

### 1. 版本管理
- ✅ **Cargo.toml 为单一真实源** - 所有版本号从这里提取
- ✅ 语义化版本：`0.1.0` → `0.2.0` (新功能) → `0.2.1` (bug 修复)
- ⚠️ 发布前必须更新 Cargo.toml 版本号

### 2. Commit Message 规范
```bash
# 仅测试编译
"test: xxx build action"

# 发布到 GitHub Release
"feat: xxx build release"

# 完整发布（同步所有平台）
"chore: 发布 v0.2.0 build publish"
```

### 3. 发布检查清单
- [ ] 更新 `rust/Cargo.toml` 版本号
- [ ] 更新 README.md (如有新功能)
- [ ] 本地测试编译: `cargo build --release`
- [ ] 提交代码: `git commit -m "xxx build publish"`
- [ ] 等待 Actions 完成（约 10-15 分钟）
- [ ] 验证 Release 页面
- [ ] 验证包管理器更新

### 4. 渐进式实施策略

**第一周：核心功能**
1. 实现 `build publish` 触发机制
2. 实现 GitHub Release 自动创建
3. 集成 Scoop (Windows)
4. 集成 Homebrew (macOS)

**第二周：Linux 生态**
5. 添加 DEB/RPM 打包
6. 集成 AUR

**第三周：完善**
7. 添加 Winget (需要首次手动提交)
8. 优化 workflow 性能
9. 完善文档

### 5. 故障排查

**Actions 失败常见原因：**
- ❌ Cargo.toml 版本号未更新
- ❌ Secrets 未配置或权限不足
- ❌ 依赖仓库不存在（scoop-bucket/homebrew-tap）
- ❌ AUR SSH key 未配置或格式错误

**调试技巧：**
```bash
# 查看 Actions 日志
# 本地测试打包
cargo install cargo-deb cargo-generate-rpm
cargo deb --no-build
cargo generate-rpm
```

---

**总结：使用 commit message 触发比 tag 更灵活，可以随时重新发布，避免 tag 管理的复杂性！** 🎉
