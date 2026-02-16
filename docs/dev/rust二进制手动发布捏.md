# Winload 手动发布流程指南

> 各平台包管理器的手动上传步骤详解
>
> ⚠️ 本文档中 `$VERSION` / `${VERSION}` 代表当前要发布的版本号（如 `0.1.5`），
> 对应的 tag 为 `v${VERSION}`（如 `v0.1.5`）。
> 版本号以 `rust/Cargo.toml` 中的 `version` 字段为唯一真实源。

---

## 📋 发布前准备

### 1. 确认版本号
```bash
# 从 Cargo.toml 读取版本
grep '^version' rust/Cargo.toml
# 例: version = "0.1.5"
```
> 🔔 以下所有操作都用这个版本号替换 `${VERSION}`。

### 2. 构建所有平台二进制
```bash
# 方式 A: 本地构建 (WSL)
cd rust
python3 build.py

# 方式 B: GitHub Actions 构建
# commit message 包含 "build action"（仅构建） 或 "build release"（构建+发布Release）
```

### 3. 验证构建产物
```bash
# 检查 dist 目录（本地）或 GitHub Release（CI）
ls rust/dist/
# 应该看到：
# - winload-linux-x86_64-v${VERSION}
# - winload-windows-x86_64-v${VERSION}.exe
# - winload-windows-aarch64-v${VERSION}.exe
# - winload-macos-x86_64-v${VERSION}
# - winload-macos-aarch64-v${VERSION}
# （CI 还会额外产出 winload-linux-aarch64-v${VERSION}）
```

### 4. 计算文件哈希（用于包管理器）
```bash
# Linux/macOS/WSL
sha256sum ./winload-*-v*

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

### 5. 命名约定说明
| 安装后的命令名 | 说明 |
|---|---|
| `winload` | 所有平台统一命令名（与 Cargo.toml `[[bin]]` 一致） |

> 📌 以前文档和 Scoop 中用过 `win-nload`，现已统一为 `winload`。

---

## 🪟 Windows 平台

### 1. Scoop ⭐ (最推荐)

> ✅ 已有 GitHub Actions 自动化（commit message 含 `build publish` 即可）。
> 以下为手动流程参考。

#### 前期准备（首次）
```bash
scoop install gh
gh repo create scoop-bucket --public
git clone https://github.com/VincentZyu233/scoop-bucket.git
cd scoop-bucket
mkdir -p bucket
```

#### 发布步骤
```bash
cd scoop-bucket/bucket
# 编辑 winload.json，更新 version / hash / url
git add bucket/winload.json
git commit -m "winload: Update to v${VERSION}"
git push
```

#### 用户安装方式
```powershell
scoop bucket add vincentzyu https://github.com/VincentZyuApps/scoop-bucket
scoop install winload
winload
```

---

### 2. Winget（可选，首次需 PR 审核）

<details>
<summary>展开 Winget 发布步骤</summary>

#### 前期准备
1. Fork `microsoft/winget-pkgs` 仓库
2. 安装 winget 工具（Windows 11 自带）

#### 发布步骤
```bash
git clone https://github.com/VincentZyu233/winget-pkgs.git
cd winget-pkgs
git checkout -b winload-${VERSION}
mkdir -p manifests/v/VincentZyu/winload/${VERSION}
```

在该目录创建三个文件：

**VincentZyu.winload.yaml**
```yaml
PackageIdentifier: VincentZyu.winload
PackageVersion: "${VERSION}"
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
```

**VincentZyu.winload.installer.yaml**
```yaml
PackageIdentifier: VincentZyu.winload
PackageVersion: "${VERSION}"
Installers:
  - Architecture: x64
    InstallerType: portable
    InstallerUrl: https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-windows-x86_64-v${VERSION}.exe
    InstallerSha256: <填入哈希值>
    Commands:
      - winload
ManifestType: installer
ManifestVersion: 1.6.0
```

**VincentZyu.winload.locale.en-US.yaml**
```yaml
PackageIdentifier: VincentZyu.winload
PackageVersion: "${VERSION}"
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
git add manifests/v/VincentZyu/winload/
git commit -m "Add: VincentZyu.winload version ${VERSION}"
git push origin winload-${VERSION}
# 然后在 GitHub 创建 PR 到 microsoft/winget-pkgs
```

⚠️ **首次提交需要审核（可能需要几天到几周）**

</details>

---

## 🐧 Linux 平台

> 📌 **手动发布只涉及 x86_64 (amd64)**，使用 musl 静态链接 = 零依赖。
> ARM64 (aarch64) 的包后续通过 GitHub Actions 自动构建和发布（同样使用 musl，零依赖）。

### 🏗️ WSL 编译环境准备（首次，一次性）

> 以下所有 DEB / RPM / AUR 操作都在 **WSL** 中完成。

```bash
# ============================================================
# 1. 安装 Rust target
# ============================================================
rustup target add x86_64-unknown-linux-musl

# ============================================================
# 2. 安装编译工具链
# ============================================================
sudo apt-get update
sudo apt-get install -y musl-tools

# ============================================================
# 3. 安装打包工具
# ============================================================
cargo install cargo-deb
cargo install cargo-generate-rpm
```

> 💡 **为什么用 musl？**
> musl = 静态链接 = 零运行时依赖，生成的二进制在**任何** Linux 发行版上都能直接跑，
> 不需要担心 glibc 版本问题。

---

### 🔨 构建 x86_64 二进制

```bash
cd rust

cargo build --release --target x86_64-unknown-linux-musl

# 验证产物
file target/x86_64-unknown-linux-musl/release/winload
# → ... statically linked ...
```

---

### 1. DEB 包 (Debian/Ubuntu) ⭐

#### 配置 Cargo.toml（已完成 ✅）

`rust/Cargo.toml` 中已有 `[package.metadata.deb]` 配置。
`cargo deb --target <TARGET>` 会**自动**将 `target/release/` 路径重定向到 `target/<TARGET>/release/`。

#### 构建 DEB 包

```bash
wsl
cd rust

cargo deb --target x86_64-unknown-linux-musl --no-build
# → target/debian/winload_${VERSION}~beta.N-1_amd64.deb

ls -lh target/debian/*.deb
```

> 💡 `--no-build` 跳过编译（上一步已经 build 过了），只做打包。

#### 验证 DEB 包

```bash
# 查看包信息
dpkg-deb -I target/x86_64-unknown-linux-musl/debian/winload_*.deb
# 应该看到 Architecture: amd64

# 查看包内容（确认 /usr/bin/winload 在里面）
dpkg-deb -c target/x86_64-unknown-linux-musl/debian/winload_*.deb

# 测试安装（可选）
sudo dpkg -i target/x86_64-unknown-linux-musl/debian/winload_*.deb
winload --version
sudo dpkg -r winload  # 卸载
```

#### 发布到 GitHub Release

```bash
VERSION="0.1.5"  # 替换为实际版本

gh release upload "v${VERSION}" \
    target/x86_64-unknown-linux-musl/debian/winload_${VERSION}_amd64.deb
```

#### 用户安装方式

```bash
wget https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload_${VERSION}_amd64.deb
sudo dpkg -i winload_${VERSION}_amd64.deb
```

---

### 2. RPM 包 (Fedora/RHEL/CentOS)

<details>
<summary>展开 RPM 发布步骤（x86_64）</summary>

#### 配置 Cargo.toml（已完成 ✅）

同 DEB，`cargo generate-rpm --target <TARGET>` 也会自动重定向路径。

#### 构建 RPM 包

```bash
cd rust

cargo generate-rpm --target x86_64-unknown-linux-musl
# → target/x86_64-unknown-linux-musl/generate-rpm/winload-${VERSION}-1.x86_64.rpm

ls -lh target/x86_64-unknown-linux-musl/generate-rpm/*.rpm
```

#### 发布到 GitHub Release

```bash
gh release upload "v${VERSION}" \
    target/x86_64-unknown-linux-musl/generate-rpm/winload-${VERSION}-1.x86_64.rpm
```

#### 用户安装方式

```bash
sudo dnf install https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-${VERSION}-1.x86_64.rpm
```

</details>

---

### 3. AUR 包 (Arch Linux) ⭐

这是一个 **预编译二进制包**（`-bin` 后缀），用户不需要在本地编译 Rust。
手动发布先只发 x86_64，后续 CI 自动化时再加 aarch64 双架构支持。

#### 前期准备（首次）

1. **注册 AUR 账号**：https://aur.archlinux.org/register

2. **配置 SSH key**（在 WSL 或任意 Linux 上）：
```bash
ssh-keygen -t ed25519 -C "vincentzyu233@gmail.com" -f ~/.ssh/aur
```

3. **配置 SSH**：
```bash
cat >> ~/.ssh/config << 'EOF'
Host aur.archlinux.org
    IdentityFile ~/.ssh/aur
    User aur
EOF
```

4. **添加公钥到 AUR**：
   - 复制 `~/.ssh/aur.pub` 的内容
   - 访问 https://aur.archlinux.org/account/ → SSH Public Key → 粘贴 → 保存

5. **测试连接**：
```bash
ssh -T aur@aur.archlinux.org
# 应该看到: "Interactive shell is disabled."
```

#### 创建并发布 PKGBUILD（详细步骤）

```bash
# ============================================================
# Step 1: 在 AUR 上创建包（首次）
# ============================================================
git clone ssh://aur@aur.archlinux.org/winload-bin.git
cd winload-bin

# ============================================================
# Step 2: 获取二进制的 sha256（从已发布的 GitHub Release）
# ============================================================
VERSION="0.1.5"  # 替换为实际版本号

wget "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-linux-x86_64-v${VERSION}"
SHA256_X86=$(sha256sum "winload-linux-x86_64-v${VERSION}" | awk '{print $1}')
echo "x86_64 SHA256: $SHA256_X86"
rm "winload-linux-x86_64-v${VERSION}"

# ============================================================
# Step 3: 创建 PKGBUILD（先只支持 x86_64）
# ============================================================
# 💡 后续 CI 自动化会升级为 x86_64 + aarch64 双架构（都用 musl 零依赖）
cat > PKGBUILD << EOF
# Maintainer: VincentZyu <vincentzyu233@gmail.com>
pkgname=winload-bin
pkgver=${VERSION}
pkgrel=1
pkgdesc="A lightweight, real-time CLI tool for monitoring network bandwidth and traffic"
arch=('x86_64')
url="https://github.com/VincentZyu233/winload"
license=('MIT')
provides=('winload')
conflicts=('winload')

source=("winload-linux-x86_64-v\${pkgver}::https://github.com/VincentZyu233/winload/releases/download/v\${pkgver}/winload-linux-x86_64-v\${pkgver}")
noextract=("winload-linux-x86_64-v\${pkgver}")
sha256sums=('${SHA256_X86}')

package() {
    install -Dm755 "\$srcdir/winload-linux-x86_64-v\${pkgver}" "\$pkgdir/usr/bin/winload"
}
EOF

echo "✅ PKGBUILD created:"
cat PKGBUILD

# ============================================================
# Step 4: 生成 .SRCINFO（AUR 必需）
# ============================================================
makepkg --printsrcinfo > .SRCINFO

echo "✅ .SRCINFO generated:"
cat .SRCINFO

# ============================================================
# Step 5: 本地测试构建（可选但推荐）
# ============================================================
makepkg -si
winload --version
sudo pacman -R winload-bin

# ============================================================
# Step 6: 提交到 AUR
# ============================================================
git add PKGBUILD .SRCINFO
git commit -m "Initial upload: winload-bin ${VERSION}"
git push
```

> 📌 **关键知识点**：
> - `noextract=()` — 阻止 `makepkg` 尝试解压裸二进制（不是 tar.gz，不能解压！）
> - `source=("filename::url")` — 用 `filename::url` 语法给下载文件重命名，避免版本间冲突
> - `.SRCINFO` — AUR 用它来显示包信息，每次改 PKGBUILD 后**必须**重新生成

#### 后续版本更新

```bash
cd winload-bin  # 之前 clone 的 AUR 仓库

# 1. 更新版本号和哈希
NEW_VERSION="0.2.0"
wget "https://github.com/VincentZyu233/winload/releases/download/v${NEW_VERSION}/winload-linux-x86_64-v${NEW_VERSION}"
NEW_HASH=$(sha256sum "winload-linux-x86_64-v${NEW_VERSION}" | awk '{print $1}')
rm "winload-linux-x86_64-v${NEW_VERSION}"

sed -i "s/^pkgver=.*/pkgver=${NEW_VERSION}/" PKGBUILD
sed -i "s/^sha256sums=.*/sha256sums=('${NEW_HASH}')/" PKGBUILD

# 2. 重新生成 .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# 3. 提交推送
git add PKGBUILD .SRCINFO
git commit -m "Update to ${NEW_VERSION}"
git push
```

#### ⚠️ 常见坑

| 问题 | 原因 | 解决 |
|------|------|------|
| `makepkg` 解压失败 | 裸二进制被当作压缩包 | 加 `noextract=()` |
| `source=()` 文件名冲突 | 不同版本下载到同名文件 | 用 `filename::url` 语法重命名 |
| `.SRCINFO` 忘记更新 | AUR 用 `.SRCINFO` 显示包信息 | 每次改 PKGBUILD 后必须重新生成 |
| SSH 权限拒绝 | 公钥未添加到 AUR 账号 | 检查 `~/.ssh/config` 和 AUR 设置 |
| WSL 上 `makepkg` 报错 | WSL 不自带 `makepkg` | 需要 `sudo apt install makepkg` 或用 Arch Docker 测试 |

#### 用户安装方式

```bash
# 使用 AUR helper（自动检测架构）
paru -S winload-bin
# 或
yay -S winload-bin

# 手动安装
git clone https://aur.archlinux.org/winload-bin.git
cd winload-bin
makepkg -si
```

---

### 4. Alpine APK

<details>
<summary>展开 Alpine APK 步骤（可选）</summary>

#### 构建 musl 版本
```bash
rustup target add x86_64-unknown-linux-musl
cd rust
cargo build --release --target x86_64-unknown-linux-musl
```

⚠️ **Alpine APK 需要提交到 Alpine 官方仓库，流程较复杂，建议先覆盖主流平台**

</details>

---

## 🍎 macOS 平台

### Homebrew ⭐

#### 前期准备（首次）
```bash
gh repo create homebrew-tap --public
git clone https://github.com/VincentZyu233/homebrew-tap.git
cd homebrew-tap
mkdir -p Formula
```

#### 创建/更新 Formula
```bash
cd Formula
cat > winload.rb <<'RUBY'
class Winload < Formula
  desc "Network Load Monitor - nload for Windows/Linux/macOS"
  homepage "https://github.com/VincentZyu233/winload"
  version "${VERSION}"  # 替换为实际版本
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-macos-aarch64-v${VERSION}"
      sha256 "<ARM64 哈希>"
    else
      url "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-macos-x86_64-v${VERSION}"
      sha256 "<x86_64 哈希>"
    end
  end

  on_linux do
    url "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-linux-x86_64-v${VERSION}"
    sha256 "<Linux x86_64 哈希>"
  end

  def install
    binary_name = Dir.glob("winload-*").first
    bin.install binary_name => "winload"
  end

  test do
    system "#{bin}/winload", "--version"
  end
end
RUBY
```

```bash
git add Formula/winload.rb
git commit -m "winload: Update to v${VERSION}"
git push
```

#### 用户安装方式
```bash
brew tap VincentZyu233/tap
brew install winload
```

---

## 📱 Termux (Android)

<details>
<summary>展开 Termux 步骤（可选）</summary>

需要提交 PR 到 `termux/termux-packages` 仓库，流程复杂，建议暂缓。

或者提供直接下载方式：
```bash
# 用户安装（Termux 中）
pkg install wget
wget https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-linux-aarch64-v${VERSION}
chmod +x winload-linux-aarch64-v${VERSION}
mv winload-linux-aarch64-v${VERSION} $PREFIX/bin/winload
```

</details>

---

## 🎯 推荐发布顺序

### 第一批（简单且用户多）
1. ✅ **Scoop** — 已有 CI 自动化 ✨
2. ✅ **DEB** — `cargo-deb` 一条命令出包
3. ✅ **AUR** — 写 PKGBUILD + push 到 AUR

### 第二批
4. ✅ **Homebrew** — 创建 tap 仓库，写 Formula
5. ✅ **RPM** — `cargo-generate-rpm` 出包

### 第三批（可选）
6. ⏸️ **Winget** — 首次需要 PR 审核
7. ⏸️ **Alpine APK** — 较复杂
8. ⏸️ **Termux** — 独立维护

---

## 📝 发布检查清单

每次发布新版本时（手动发布 = x86_64 only）：
- [ ] 更新 `rust/Cargo.toml` 中的版本号
- [ ] 构建所有平台二进制（本地或 GitHub Actions）
- [ ] 计算所有二进制的 SHA256 哈希
- [ ] 创建 GitHub Release 并上传二进制
- [ ] 构建并上传 DEB 包（amd64）
- [ ] 构建并上传 RPM 包（x86_64）
- [ ] 更新 Scoop manifest（CI 自动化 / 手动更新 version+hash）
- [ ] 更新 Homebrew Formula（更新 version 和 sha256）
- [ ] 更新 AUR PKGBUILD（更新 pkgver、sha256sums，重新生成 .SRCINFO）
- [ ] 测试安装：`scoop install winload`、`brew install winload`、`paru -S winload-bin`

> 🤖 后续 CI 自动化后，DEB/RPM/AUR 会自动发布 x86_64 + aarch64 双架构（都用 musl 零依赖）。

---

## 🔧 工具脚本

### 一键计算所有文件哈希
```bash
#!/bin/bash
# hash-all.sh - 计算所有二进制的哈希值
VERSION=$(grep '^version' rust/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "📦 Version: v${VERSION}"
echo ""

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
VERSION="v$(grep '^version' rust/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
echo "📦 Uploading version: ${VERSION}"

gh release create "$VERSION" --title "winload $VERSION" --generate-notes

gh release upload "$VERSION" \
    rust/dist/winload-linux-x86_64-${VERSION} \
    rust/dist/winload-windows-x86_64-${VERSION}.exe \
    rust/dist/winload-windows-aarch64-${VERSION}.exe \
    rust/dist/winload-macos-x86_64-${VERSION} \
    rust/dist/winload-macos-aarch64-${VERSION} \
    rust/target/x86_64-unknown-linux-musl/debian/winload_*.deb \
    rust/target/x86_64-unknown-linux-musl/generate-rpm/winload-*.rpm
```

### 一键构建所有 Linux 包（DEB + RPM，x86_64）
```bash
#!/bin/bash
# build-linux-packages.sh - 在 WSL 中运行
set -e
cd rust

echo "🔨 Building x86_64 (musl, static)..."
cargo build --release --target x86_64-unknown-linux-musl

echo "📦 Building DEB package..."
cargo deb --target x86_64-unknown-linux-musl --no-build

echo "📦 Building RPM package..."
cargo generate-rpm --target x86_64-unknown-linux-musl

echo ""
echo "✅ All packages built:"
ls -lh target/x86_64-unknown-linux-musl/debian/*.deb
ls -lh target/x86_64-unknown-linux-musl/generate-rpm/*.rpm
```

---

## 💡 最佳实践

1. **版本号统一** — `Cargo.toml` 为单一真实源，所有脚本从中提取
2. **先 GitHub Release** — 其他平台都依赖 Release 的下载链接
3. **用 musl 构建** — 静态链接，兼容所有 Linux 发行版
4. **测试安装** — 发布后在各平台测试安装
5. **文档同步** — 更新 README.md 的安装说明
6. **社区反馈** — 关注各平台的 issue/PR

---

## 🤖 GitHub Actions 自动化

已有的 CI（`build.yml`）已支持 `build publish` 触发 Scoop 自动更新。
若要扩展自动化 DEB 构建和 AUR 推送，参见 `build.yml` 中的
`publish-deb` 和 `publish-aur` job。

> 📌 **CI 自动化计划**：
> - CI 会同时构建 **x86_64 + aarch64**，且**都用 musl** = 两个架构都零依赖
> - aarch64 musl 交叉编译在 CI 上用 `cross` 工具（Docker 容器内完成，无需手动装工具链）
> - CI 的 AUR PKGBUILD 会升级为双架构（`source_x86_64` / `source_aarch64` + `$CARCH` 判断）
> - 本文档的手动步骤只需覆盖 x86_64，CI 补全 aarch64

---

**总结：优先完成 Scoop（已自动化）+ DEB + AUR，即可覆盖 90% 用户！** 🚀
