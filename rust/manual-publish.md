# Winload 手动发布流程指南

> 各平台包管理器的手动上传步骤详解

---

## 📋 发布前准备

### 1. 构建所有平台二进制
```bash
# 本地构建 (WSL)
cd rust
python3 build.py

# 或等待 GitHub Actions 构建完成
# commit message 包含 "build action" 或 "build release"
```

### 2. 验证构建产物
```bash
# 检查 dist 目录或 GitHub Release
ls rust/dist/
# 应该看到：
# - winload-linux-x86_64-v0.1.0
# - winload-windows-x86_64-v0.1.0.exe
# - winload-macos-x86_64-v0.1.0
# - winload-macos-aarch64-v0.1.0
```

### 3. 计算文件哈希（用于包管理器）
```bash
# Linux/macOS/WSL
sha256sum ./winload-*-v0.1.0*

# Windows PowerShell
Get-FileHash ./winload-*.exe -Algorithm SHA256
```
#### for example:
```powershell
PS D:\Downloads> Get-FileHash ./winload-*.exe -Algorithm SHA256

Algorithm       Hash                                                                   Path
---------       ----                                                                   ----
SHA256          B836262FFDEE8F6930F4A57DE0E9644F174D47D98C78896B145A3FC0010FBE03       D:\Downloads\winload-windows-x86_64.exe

```


---

## 🪟 Windows 平台

### 1. Scoop ⭐ (最推荐)

#### 前期准备
1. 创建 Scoop bucket 仓库（首次）
```bash
scoop install gh
# 创建仓库 scoop-bucket
gh repo create scoop-bucket --public

# 克隆到本地
git clone https://github.com/VincentZyu233/scoop-bucket.git
cd scoop-bucket
mkdir -p bucket
```

#### 发布步骤
2. 创建/更新 manifest
```bash
cd scoop-bucket/bucket

# 创建 winload.json
cat > winload.json <<'EOF'
{
    "version": "0.1.0",
    "description": "Network Load Monitor - nload for Windows/Linux/macOS",
    "homepage": "https://github.com/VincentZyu233/winload",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-windows-x86_64-v0.1.0.exe",
            "hash": "sha256:填入上面计算的哈希值"
        }
    },
    "bin": [["winload-windows-x86_64-v0.1.0.exe", "win-nload"]],
    "checkver": {
        "github": "https://github.com/VincentZyu233/winload"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/VincentZyu233/winload/releases/download/v$version/winload-windows-x86_64-v$version.exe"
            }
        }
    }
}
EOF
```

3. 提交并推送
```bash
git add bucket/winload.json
git commit -m "winload: Update to v0.1.0"
git push
```

#### 用户安装方式
```powershell
scoop bucket add vincentzyu https://github.com/VincentZyu233/scoop-bucket
scoop install winload
win-nload
```

---

### 2. Winget

#### 前期准备
1. Fork `microsoft/winget-pkgs` 仓库
2. 安装 winget 工具（Windows 11 自带）

#### 发布步骤（首次需要手动提交 PR）
```bash
# 1. 克隆你 fork 的仓库
git clone https://github.com/VincentZyu233/winget-pkgs.git
cd winget-pkgs

# 2. 创建新分支
git checkout -b winload-0.1.0

# 3. 创建 manifest 目录
mkdir -p manifests/v/VincentZyu/winload/0.1.0

# 4. 创建三个 manifest 文件
```

**VincentZyu.winload.yaml** (主清单)
```yaml
PackageIdentifier: VincentZyu.winload
PackageVersion: 0.1.0
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
```

**VincentZyu.winload.installer.yaml**
```yaml
PackageIdentifier: VincentZyu.winload
PackageVersion: 0.1.0
Installers:
  - Architecture: x64
    InstallerType: portable
    InstallerUrl: https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-windows-x86_64-v0.1.0.exe
    InstallerSha256: 填入哈希值
    Commands:
      - win-nload
ManifestType: installer
ManifestVersion: 1.6.0
```

**VincentZyu.winload.locale.en-US.yaml**
```yaml
PackageIdentifier: VincentZyu.winload
PackageVersion: 0.1.0
PackageLocale: en-US
Publisher: VincentZyu
PackageName: winload
License: MIT
ShortDescription: Network Load Monitor - nload for Windows/Linux/macOS
PackageUrl: https://github.com/VincentZyu233/winload
Tags:
  - network
  - monitor
  - bandwidth
  - cli
ManifestType: defaultLocale
ManifestVersion: 1.6.0
```

```bash
# 5. 提交并推送
git add manifests/v/VincentZyu/winload/
git commit -m "Add: VincentZyu.winload version 0.1.0"
git push origin winload-0.1.0

# 6. 在 GitHub 创建 PR 到 microsoft/winget-pkgs
```

⚠️ **注意：首次提交需要审核（可能需要几天），通过后后续版本可以考虑自动化**

#### 用户安装方式
```powershell
winget install VincentZyu.winload
```

---

### 3. Chocolatey ⚠️ (不推荐，审核周期长)

略，建议优先使用 Scoop。

---

## 🐧 Linux 平台

### 1. DEB (Debian/Ubuntu)

#### 前期准备
```bash
# 安装 cargo-deb
cargo install cargo-deb
```

#### 配置 Cargo.toml
在 `rust/Cargo.toml` 添加：
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
```

#### 构建 DEB 包
```bash
cd rust
cargo deb --target x86_64-unknown-linux-gnu

# 输出在 target/debian/winload_0.1.0_amd64.deb
```

#### 发布到 GitHub Release
```bash
# 手动上传到 GitHub Release
# 或使用 gh cli
gh release upload v0.1.0 target/debian/winload_0.1.0_amd64.deb
```

#### 用户安装方式
```bash
# 下载并安装
wget https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload_0.1.0_amd64.deb
sudo dpkg -i winload_0.1.0_amd64.deb
```

---

### 2. RPM (Fedora/RHEL/CentOS)

#### 前期准备
```bash
# 安装 cargo-generate-rpm
cargo install cargo-generate-rpm
```

#### 配置 Cargo.toml
```toml
[package.metadata.generate-rpm]
assets = [
    { source = "target/release/winload", dest = "/usr/bin/win-nload", mode = "755" },
]
```

#### 构建 RPM 包
```bash
cd rust
cargo build --release --target x86_64-unknown-linux-gnu
cargo generate-rpm --target x86_64-unknown-linux-gnu

# 输出在 target/generate-rpm/winload-0.1.0-1.x86_64.rpm
```

#### 发布到 GitHub Release
```bash
gh release upload v0.1.0 target/generate-rpm/winload-0.1.0-1.x86_64.rpm
```

#### 用户安装方式
```bash
# Fedora/RHEL
sudo dnf install https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-0.1.0-1.x86_64.rpm

# 或手动下载安装
wget https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-0.1.0-1.x86_64.rpm
sudo rpm -i winload-0.1.0-1.x86_64.rpm
```

---

### 3. AUR (Arch Linux) ⭐

#### 前期准备
1. 注册 AUR 账号：https://aur.archlinux.org/register
2. 配置 SSH key：
```bash
ssh-keygen -t ed25519 -C "your@email.com"
# 在 https://aur.archlinux.org/account/ 添加公钥
```

#### 创建 PKGBUILD
```bash
# 创建工作目录
mkdir -p ~/aur-packages/winload-bin
cd ~/aur-packages/winload-bin

# 创建 PKGBUILD
cat > PKGBUILD <<'EOF'
# Maintainer: VincentZyu <your@email.com>
pkgname=winload-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Network Load Monitor - nload for Windows/Linux/macOS"
arch=('x86_64')
url="https://github.com/VincentZyu233/winload"
license=('MIT')
provides=('win-nload')
conflicts=('win-nload')
source=("https://github.com/VincentZyu233/winload/releases/download/v${pkgver}/winload-linux-x86_64-v${pkgver}")
sha256sums=('填入哈希值')

package() {
    install -Dm755 "$srcdir/winload-linux-x86_64-v${pkgver}" "$pkgdir/usr/bin/win-nload"
}
EOF
```

#### 生成 .SRCINFO
```bash
makepkg --printsrcinfo > .SRCINFO
```

#### 发布到 AUR
```bash
# 首次发布
git clone ssh://aur@aur.archlinux.org/winload-bin.git
cd winload-bin
cp ../PKGBUILD ../.SRCINFO .
git add PKGBUILD .SRCINFO
git commit -m "Initial upload: winload-bin 0.1.0"
git push

# 后续更新
# 修改 PKGBUILD 中的 pkgver 和 sha256sums
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to 0.2.0"
git push
```

#### 用户安装方式
```bash
paru -S winload-bin
# 或
yay -S winload-bin
```

---

### 4. Alpine APK

#### 构建 musl 版本
```bash
# 安装 musl target
rustup target add x86_64-unknown-linux-musl

# 构建
cd rust
cargo build --release --target x86_64-unknown-linux-musl
```

#### 创建 APKBUILD
```bash
mkdir -p ~/alpine-packages/winload
cd ~/alpine-packages/winload

cat > APKBUILD <<'EOF'
# Maintainer: VincentZyu <your@email.com>
pkgname=winload
pkgver=0.1.0
pkgrel=0
pkgdesc="Network Load Monitor"
url="https://github.com/VincentZyu233/winload"
arch="x86_64"
license="MIT"
source="https://github.com/VincentZyu233/winload/releases/download/v$pkgver/winload-linux-x86_64-v$pkgver"

package() {
    install -Dm755 "$srcdir/winload-linux-x86_64-v$pkgver" "$pkgdir/usr/bin/win-nload"
}

sha512sums="填入 sha512 哈希"
EOF
```

⚠️ **Alpine APK 需要提交到 Alpine 官方仓库，流程较复杂，建议先覆盖主流平台**

---

## 🍎 macOS 平台

### Homebrew ⭐

#### 前期准备
1. 创建 Homebrew Tap 仓库（首次）
```bash
gh repo create homebrew-tap --public
git clone https://github.com/VincentZyu233/homebrew-tap.git
cd homebrew-tap
mkdir -p Formula
```

#### 创建 Formula
```bash
cd Formula

cat > winload.rb <<'EOF'
class Winload < Formula
  desc "Network Load Monitor - nload for Windows/Linux/macOS"
  homepage "https://github.com/VincentZyu233/winload"
  version "0.1.0"
  license "MIT"
  
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-macos-aarch64-v0.1.0"
      sha256 "填入 ARM64 二进制的哈希"
    else
      url "https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-macos-x86_64-v0.1.0"
      sha256 "填入 x86_64 二进制的哈希"
    end
  end

  def install
    if Hardware::CPU.arm?
      bin.install "winload-macos-aarch64-v#{version}" => "win-nload"
    else
      bin.install "winload-macos-x86_64-v#{version}" => "win-nload"
    end
  end

  test do
    system "#{bin}/win-nload", "--version"
  end
end
EOF
```

#### 提交并推送
```bash
git add Formula/winload.rb
git commit -m "winload: Add formula for version 0.1.0"
git push
```

#### 用户安装方式
```bash
brew tap VincentZyu233/tap
brew install winload
```

---

## 📱 Termux (Android)

### 构建 Android 版本
```bash
# 安装 Android target
rustup target add aarch64-linux-android

# 配置 NDK（需要先安装 Android NDK）
export NDK_HOME=/path/to/android-ndk

# 构建
cd rust
cargo build --release --target aarch64-linux-android
```

### 发布到 Termux Packages
需要提交 PR 到 `termux/termux-packages` 仓库，流程复杂，建议暂缓。

或者提供直接下载方式：
```bash
# 用户安装（Termux 中）
pkg install wget
wget https://github.com/VincentZyu233/winload/releases/download/v0.1.0/winload-android-aarch64-v0.1.0
chmod +x winload-android-aarch64-v0.1.0
mv winload-android-aarch64-v0.1.0 $PREFIX/bin/win-nload
```

---

## 🎯 推荐发布顺序

### 第一批（简单且用户多）
1. ✅ **Scoop** - 创建 bucket 仓库，更新 JSON
2. ✅ **Homebrew** - 创建 tap 仓库，写 Formula
3. ✅ **DEB** - 配置 Cargo.toml，cargo-deb 构建

### 第二批（主流 Linux）
4. ✅ **RPM** - cargo-generate-rpm 构建
5. ✅ **AUR** - 创建 PKGBUILD，推送到 AUR

### 第三批（可选）
6. ⏸️ **Winget** - 首次需要 PR 审核
7. ⏸️ **Alpine APK** - 较复杂
8. ⏸️ **Termux** - 独立维护

---

## 📝 发布检查清单

每次发布新版本时：
- [ ] 更新 `rust/Cargo.toml` 中的版本号
- [ ] 构建所有平台二进制（本地或 GitHub Actions）
- [ ] 计算所有二进制的 SHA256 哈希
- [ ] 创建 GitHub Release 并上传二进制
- [ ] 更新 Scoop manifest（更新 version 和 hash）
- [ ] 更新 Homebrew Formula（更新 version 和 sha256）
- [ ] 构建并上传 DEB 包
- [ ] 构建并上传 RPM 包
- [ ] 更新 AUR PKGBUILD（更新 pkgver 和 sha256sums）
- [ ] 测试安装：`scoop install winload`、`brew install winload`、`paru -S winload-bin`

---

## 🔧 工具脚本

### 计算所有文件哈希
```bash
#!/bin/bash
# hash-all.sh - 计算所有二进制的哈希值

for file in rust/dist/winload-*-v*; do
    if [ -f "$file" ]; then
        echo "=== $(basename $file) ==="
        sha256sum "$file"
        echo
    fi
done
```

### 批量上传到 GitHub Release
```bash
#!/bin/bash
# upload-release.sh - 上传所有文件到 GitHub Release

VERSION="v0.1.0"  # 修改为你的版本号

gh release create "$VERSION" --title "winload $VERSION" --generate-notes

gh release upload "$VERSION" \
    rust/dist/winload-linux-x86_64-$VERSION \
    rust/dist/winload-windows-x86_64-$VERSION.exe \
    rust/dist/winload-macos-x86_64-$VERSION \
    rust/dist/winload-macos-aarch64-$VERSION \
    rust/target/debian/winload_*.deb \
    rust/target/generate-rpm/winload-*.rpm
```

---

## 💡 最佳实践

1. **版本号统一** - Cargo.toml 为单一真实源
2. **先 GitHub Release** - 其他平台都依赖 GitHub Release 的下载链接
3. **测试安装** - 发布后在各平台测试安装
4. **文档同步** - 更新 README.md 的安装说明
5. **社区反馈** - 关注各平台的 issue/PR

---

**总结：优先完成 Scoop + Homebrew + DEB + AUR，即可覆盖 90% 用户！** 🚀
