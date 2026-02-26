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

AUR 支持两个包：
- **`winload-rust`** — 源码包，从 GitHub 编译（适合信任源码的用户）
- **`winload-rust-bin`** — 预编译二进制包（适合不想编译的用户）

---

#### 方案 A: winload-rust-bin（预编译二进制）

这是一个 **预编译二进制包**（`-bin` 后缀），用户不需要在本地编译 Rust。
支持 **x86_64 + aarch64** 双架构（都用 musl 零依赖）。

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
git clone ssh://aur@aur.archlinux.org/winload-rust-bin.git
cd winload-rust-bin

# ============================================================
# Step 2: 获取二进制的 sha256（从已发布的 GitHub Release）
# ============================================================
VERSION="0.1.5"  # 替换为实际版本号
# VERSION="0.1.6-beta.2"

# 下载 x86_64 版本并计算哈希
wget "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-linux-x86_64-v${VERSION}"
# wget "https://github.com/VincentZyuApps/winload/releases/download/v0.1.6-beta.2/winload-linux-x86_64-v0.1.6-beta.2"
SHA256_X86=$(sha256sum "winload-linux-x86_64-v${VERSION}" | awk '{print $1}')
rm "winload-linux-x86_64-v${VERSION}"

# 下载 aarch64 版本并计算哈希
wget "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-linux-aarch64-v${VERSION}"
SHA256_AARCH64=$(sha256sum "winload-linux-aarch64-v${VERSION}" | awk '{print $1}')
rm "winload-linux-aarch64-v${VERSION}"

echo "x86_64 SHA256: $SHA256_X86"
echo "aarch64 SHA256: $SHA256_AARCH64"

# ============================================================
# Step 3: 创建 PKGBUILD（双架构支持）
# ============================================================
cat > PKGBUILD << EOF
# Maintainer: VincentZyu <vincentzyu233@gmail.com>
pkgname=winload-rust-bin
pkgver=${VERSION}
pkgrel=1
pkgdesc="A lightweight, real-time CLI tool for monitoring network bandwidth and traffic"
arch=('x86_64' 'aarch64')
url="https://github.com/VincentZyuApps/winload"
license=('MIT')
provides=('winload')
conflicts=('winload' 'winload-rust')

source_x86_64=("winload-linux-x86_64-v\${pkgver}::https://github.com/VincentZyuApps/winload/releases/download/v\${pkgver}/winload-linux-x86_64-v\${pkgver}")
source_aarch64=("winload-linux-aarch64-v\${pkgver}::https://github.com/VincentZyuApps/winload/releases/download/v\${pkgver}/winload-linux-aarch64-v\${pkgver}")

noextract=()

sha256sums_x86_64=('${SHA256_X86}')
sha256sums_aarch64=('${SHA256_AARCH64}')

package() {
    if [ "\$CARCH" = "x86_64" ]; then
        install -Dm755 "\$srcdir/winload-linux-x86_64-v\${pkgver}" "\$pkgdir/usr/bin/winload"
    elif [ "\$CARCH" = "aarch64" ]; then
        install -Dm755 "\$srcdir/winload-linux-aarch64-v\${pkgver}" "\$pkgdir/usr/bin/winload"
    fi
}
EOF

echo "✅ PKGBUILD created:"
batcat PKGBUILD

# ============================================================
# Step 4: 生成 .SRCINFO（AUR 必需）
# ============================================================
su builduser
makepkg --printsrcinfo > .SRCINFO

echo "✅ .SRCINFO generated:"
bat .SRCINFO
cat .SRCINFO

# ============================================================
# Step 5: 本地测试构建（可选但推荐）
# ============================================================
docker cp /mnt/d/aaaStuffsaaa/from_git/github/winload/tmp/. arch-container:/tmp/winload-build
docker start -i arch-container
pacman -Syu --noconfirm base-devel
useradd -m builduser
echo "builduser ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
# 切换到构建目录并修改权限
chown -R builduser:builduser /tmp/winload-build
cd /tmp/winload-build
# 1. 先把 pkgver 改成合规的格式 (0.1.6.beta2)
sed -i 's/pkgver=0.1.6-beta.2/pkgver=0.1.6.beta.2/' PKGBUILD
# 2. 在其下方插入原始版本变量 _tagver
sed -i '/pkgver=/a _tagver=0.1.6-beta.2' PKGBUILD
# 3. 把后面所有引用链接和文件的地方从 ${pkgver} 改成 ${_tagver}
sed -i 's/${pkgver}/${_tagver}/g' PKGBUILD
su builduser
makepkg -si
winload --version
sudo pacman -R winload-rust-bin

# ============================================================
# Step 6: 提交到 AUR
# ============================================================
root@DESKTOP-28AGCCU:/mnt/d/aaaStuffsaaa/from_git/github/winload/tmp# mv ./* /mnt/d/aaaStuffsaaa/from_git/aur/winload-rust-bin
mv ./.SRCINFO /mnt/d/aaaStuffsaaa/from_git/aur/winload-rust-bin/
root@DESKTOP-28AGCCU:/mnt/d/aaaStuffsaaa/from_git/github/winload/tmp# ls
root@DESKTOP-28AGCCU:/mnt/d/aaaStuffsaaa/from_git/github/winload/tmp# pwd
/mnt/d/aaaStuffsaaa/from_git/github/winload/tmp
root@DESKTOP-28AGCCU:/mnt/d/aaaStuffsaaa/from_git/github/winload/tmp#


root@DESKTOP-28AGCCU:/mnt/d/aaaStuffsaaa/from_git/aur/winload-rust-bin# ls
PKGBUILD  winload-linux-aarch64-v0.1.6-beta.2  winload-linux-x86_64-v0.1.6-beta.2  winload-rust-bin
root@DESKTOP-28AGCCU:/mnt/d/aaaStuffsaaa/from_git/aur/winload-rust-bin# pwd
/mnt/d/aaaStuffsaaa/from_git/aur/winload-rust-bin
root@DESKTOP-28AGCCU:/mnt/d/aaaStuffsaaa/from_git/aur/winload-rust-bin#

git add PKGBUILD .SRCINFO
git commit -m "Initial upload: winload-rust-bin ${VERSION}"
git push
```

> 📌 **关键知识点**：
> - `arch=('x86_64' 'aarch64')` — 支持双架构
> - `source_x86_64=()` / `source_aarch64=()` — 分别指定不同架构的下载源
> - `sha256sums_x86_64=()` / `sha256sums_aarch64=()` — 分别指定不同架构的哈希
> - `noextract=()` — 裸二进制不需要解压
> - `$CARCH` — makepkg 变量，值为 `x86_64` 或 `aarch64`
> - `.SRCINFO` — AUR 用它来显示包信息，每次改 PKGBUILD 后**必须**重新生成

#### 后续版本更新

```bash
cd winload-rust-bin  # 之前 clone 的 AUR 仓库

# 1. 更新版本号和哈希
NEW_VERSION="0.2.0"

# x86_64
wget "https://github.com/VincentZyu233/winload/releases/download/v${NEW_VERSION}/winload-linux-x86_64-v${NEW_VERSION}"
NEW_SHA256_X86=$(sha256sum "winload-linux-x86_64-v${NEW_VERSION}" | awk '{print $1}')
rm "winload-linux-x86_64-v${NEW_VERSION}"

# aarch64
wget "https://github.com/VincentZyu233/winload/releases/download/v${NEW_VERSION}/winload-linux-aarch64-v${NEW_VERSION}"
NEW_SHA256_AARCH64=$(sha256sum "winload-linux-aarch64-v${NEW_VERSION}" | awk '{print $1}')
rm "winload-linux-aarch64-v${NEW_VERSION}"

# 更新 PKGBUILD
sed -i "s/^pkgver=.*/pkgver=${NEW_VERSION}/" PKGBUILD
sed -i "s/^sha256sums_x86_64=.*/sha256sums_x86_64=('${NEW_SHA256_X86}')/" PKGBUILD
sed -i "s/^sha256sums_aarch64=.*/sha256sums_aarch64=('${NEW_SHA256_AARCH64}')/" PKGBUILD

# 2. 重新生成 .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# 3. 提交推送
# 确保在 /mnt/d/.../aur/winload-rust-bin 目录
# 1. 修正 pkgver
sed -i 's/pkgver=0.1.6-beta.2/pkgver=0.1.6.beta.2/' PKGBUILD
# 2. 插入 _tagver 变量（用于下载链接）
sed -i '/pkgver=/a _tagver=0.1.6-beta.2' PKGBUILD
# 3. 将所有引用改为变量
sed -i 's/${pkgver}/${_tagver}/g' PKGBUILD
git add PKGBUILD .SRCINFO
git commit -m "Update to ${NEW_VERSION}"
git push
```

#### 装个paru测试一下
```shell
# arch
wsl
docker start -i arch-container
pacman -Syu --needed base-devel git
pacman -Syu proxychains proxychains-ng
# 1. 切换到普通用户（假设你之前的 builduser 还在）
su builduser
# 1. 删掉刚才那个没用的目录
cd /tmp
rm -rf paru-bin

# 2. 安装 Rust 编译器（编译 paru 需要它）
sudo pacman -S --needed rust cargo

# 3. 克隆源码仓库（注意这次没加 -bin）
git clone https://aur.archlinux.org/paru.git
cd paru

# 4. 编译并安装
makepkg -si
proxychains4 makepkg -si
paru -S winload-rust-bin
paru -Syu winload-rust-bin
proxychains paru -Syu winload-rust-bin
pacman -Ql winload-rust-bin
```

#### ⚠️ 常见坑

| 问题 | 原因 | 解决 |
|------|------|------|
| `makepkg` 解压失败 | 裸二进制被当作压缩包 | 加 `noextract=()` |
| `source=()` 文件名冲突 | 不同版本下载到同名文件 | 用 `filename::url` 语法重命名 |
| `.SRCINFO` 忘记更新 | AUR 用 `.SRCINFO` 显示包信息 | 每次改 PKGBUILD 后必须重新生成 |
| SSH 权限拒绝 | 公钥未添加到 AUR 账号 | 检查 `~/.ssh/config` 和 AUR 设置 |
| WSL 上 `makepkg` 报错 | WSL 不自带 `makepkg` | 需要 `sudo apt install makepkg` 或用 Arch Docker 测试 |
| 双架构打包失败 | 缺少某个架构的哈希 | 确保 `sha256sums_x86_64` 和 `sha256sums_aarch64` 都填了 |

#### 用户安装方式

```bash
# 使用 AUR helper（自动检测架构）
paru -S winload-rust-bin
# 或
yay -S winload-rust-bin

# 手动安装
git clone https://aur.archlinux.org/winload-rust-bin.git
cd winload-rust-bin
makepkg -si
```

---

#### 方案 B: winload-rust（源码包，从 GitHub 编译）

这是一个 **源码包**，从 GitHub 下载源码并在本地编译。
支持 **x86_64 + aarch64** 双架构。

##### 前期准备（首次）

同 `winload-rust-bin`，需要先配置好 AUR SSH 访问（见上文 Step 1-5）。

##### 创建并发布 PKGBUILD

```bash
# ============================================================
# Step 1: 在 AUR 上创建包（首次）
# ============================================================
git clone ssh://aur@aur.archlinux.org/winload-rust.git
cd winload-rust

# ============================================================
# Step 2: 创建 PKGBUILD
# ============================================================
VERSION="0.1.6-beta.2"  # 替换为实际版本号

cat > PKGBUILD << EOF
# Maintainer: VincentZyu <vincentzyu233@gmail.com>
pkgname=winload-rust
pkgver=${VERSION}
pkgrel=1
pkgdesc="Network Load Monitor - nload for Windows/Linux/macOS (compiled from source)"
arch=('x86_64' 'aarch64')
url="https://github.com/VincentZyuApps/winload"
license=('MIT')
provides=('winload')
conflicts=('winload' 'winload-rust-bin')

depends=('gcc-libs' 'musl')

source=("https://github.com/VincentZyuApps/winload/archive/refs/tags/v\${pkgver}.tar.gz")

sha256sums=('SKIP')  # 用 SKIP，makepkg 会自动验证

build() {
    cd winload-\${pkgver}/rust
    cargo build --release --target \${CARCH}-unknown-linux-musl
}

package() {
    cd winload-\${pkgver}/rust
    install -Dm755 "target/\${CARCH}-unknown-linux-musl/release/winload" "\$pkgdir/usr/bin/winload"
}
EOF

# ============================================================
# Step 3: 生成 .SRCINFO
# ============================================================
makepkg --printsrcinfo > .SRCINFO

# ============================================================
# Step 4: 本地测试构建（可选但推荐）
# ============================================================
makepkg -si
winload --version
sudo pacman -R winload-rust

# ============================================================
# Step 5: 提交到 AUR
# ============================================================
git add PKGBUILD .SRCINFO
git commit -m "Initial upload: winload-rust ${VERSION}"
git push
```

> 📌 **关键知识点**：
> - `source=()` — 从 GitHub 下载源码 tarball
> - `depends=('gcc-libs' 'musl')` — Rust musl 目标依赖
> - `build()` — 调用 cargo 编译，使用 musl 目标
> - `$CARCH` — makepkg 变量，值为 `x86_64` 或 `aarch64`

##### 后续版本更新

```bash
cd winload-rust

# 1. 更新版本号
NEW_VERSION="0.2.0"
sed -i "s/^pkgver=.*/pkgver=${NEW_VERSION}/" PKGBUILD

# 2. 重新生成 .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# 3. 提交推送
git add PKGBUILD .SRCINFO
git commit -m "Update to ${NEW_VERSION}"
git push
```

##### 用户安装方式

```bash
# 使用 AUR helper
paru -S winload-rust
# 或
yay -S winload-rust

# 手动安装
git clone https://aur.archlinux.org/winload-rust.git
cd winload-rust
makepkg -si
```

##### ⚠️ 常见坑

| 问题 | 原因 | 解决 |
|------|------|------|
| 编译失败 | 缺少 musl 目标 | `rustup target add x86_64-unknown-linux-musl` |
| 依赖缺失 | aarch64 交叉编译 | 确保安装 aarch64 工具链 |
| `.SRCINFO` 忘记更新 | AUR 用它显示包信息 | 每次改 PKGBUILD 后必须重新生成 |

---

### 4. Alpine APK ⭐

Alpine Linux 使用 `apk` 包管理器，包来源于 **aports** 仓库（托管在 GitLab: `gitlab.alpinelinux.org/alpine/aports`）。

> 💡 **你的 musl 静态链接二进制天然兼容 Alpine**（Alpine 就是基于 musl 的），所以这是个低成本高回报的平台。

#### Alpine 仓库层级

| 仓库 | 门槛 | 类比 |
|------|------|------|
| **testing** | 任何人提交 MR 即可，审核较宽松 | ≈ AUR |
| **community** | 需要 Alpine 开发者 sponsor + 维护承诺 | ≈ Arch 官方 community |
| **main** | 核心包，严格审核 | ≈ Arch 官方 core/extra |

---

#### 方案 A: 提交到 testing（推荐先做这个）

##### 前期准备（首次）

1. **注册 GitLab 账号**：https://gitlab.alpinelinux.org/users/sign_up

2. **Fork aports 仓库**：
   - 访问 https://gitlab.alpinelinux.org/alpine/aports
   - 点击右上角 **Fork**

3. **Clone 你 fork 的仓库**：
```bash
git clone https://gitlab.alpinelinux.org/<你的用户名>/aports.git
cd aports
git remote add upstream https://gitlab.alpinelinux.org/alpine/aports.git
```

4. **安装 Alpine 打包工具**（在 Alpine 容器中测试时需要）：
```bash
# 方式一：使用 Docker
docker run -it alpine:latest sh
apk add alpine-sdk sudo
adduser -D builder
echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
addgroup builder abuild
su builder
abuild-keygen -a -i  # 生成签名密钥

# 方式二：WSL 中不能直接运行 abuild，建议用 Docker
```

##### 创建 APKBUILD（源码编译版）

```bash
cd aports
git checkout -b winload-new-aport
mkdir -p testing/winload
cd testing/winload
```

创建 `APKBUILD` 文件：

```bash
VERSION="0.1.6_beta4"  # Alpine 用下划线替代连字符！
                        # 0.1.6-beta.4 → 0.1.6_beta4

cat > APKBUILD << 'EOF'
# Contributor: VincentZyu <vincentzyu233@gmail.com>
# Maintainer: VincentZyu <vincentzyu233@gmail.com>
pkgname=winload
pkgver=0.1.6_beta4
pkgrel=0
pkgdesc="Network Load Monitor - nload-like TUI tool for Windows/Linux/macOS"
url="https://github.com/VincentZyuApps/winload"
arch="x86_64 aarch64"
license="MIT"
makedepends="cargo"
source="$pkgname-$pkgver.tar.gz::https://github.com/VincentZyuApps/winload/archive/refs/tags/v${pkgver/_beta/.beta.}.tar.gz"
# ↑ 注意版本号转换：Alpine pkgver 0.1.6_beta4 → GitHub tag v0.1.6-beta.4
# 如果是正式版 (如 0.2.0)，直接用 v$pkgver 即可

# 指定解压后的目录名（GitHub tarball 的顶层目录名）
builddir="$srcdir/winload-${pkgver/_beta/-beta.}"

prepare() {
	default_prepare
	cargo fetch --target="$CTARGET" --manifest-path="$builddir/rust/Cargo.toml"
}

build() {
	cd "$builddir/rust"
	cargo build --release --frozen
}

check() {
	cd "$builddir/rust"
	cargo test --release --frozen
}

package() {
	install -Dm755 "$builddir/rust/target/release/winload" \
		"$pkgdir/usr/bin/winload"

	# 安装 LICENSE
	install -Dm644 "$builddir/LICENSE" \
		"$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}

sha512sums="
SKIP
"
EOF
```

> 📌 **Alpine 版本号规则**：
> - 不允许连字符 `-`，用下划线 `_` 替代：`0.1.6-beta.4` → `0.1.6_beta4`
> - `_beta` / `_rc` / `_alpha` 是 Alpine 约定的预发布后缀
> - 正式版直接用数字：`0.2.0`

##### 本地测试构建（使用 Docker）

```bash
# ============================================================
# 在 Alpine Docker 容器中测试
# ============================================================

# 1. 启动 Alpine 容器并挂载 aports 目录
docker run -it --name alpine-build \
    -v $(pwd)/aports:/home/builder/aports \
    alpine:latest sh

# 2. 容器内设置
apk add alpine-sdk sudo cargo rust
adduser -D builder
echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
addgroup builder abuild
su builder
cd ~

# 3. 生成签名密钥
abuild-keygen -a -i

# 4. 构建测试
cd ~/aports/testing/winload
abuild checksum    # 下载源码并计算 sha512
abuild -r          # 构建包

# 5. 查看产物
ls ~/packages/testing/x86_64/
# → winload-0.1.6_beta4-r0.apk

# 6. 测试安装
sudo apk add --allow-untrusted ~/packages/testing/x86_64/winload-*.apk
winload --version
sudo apk del winload
```

##### 计算校验和（替换 SKIP）

```bash
# 在 Alpine 容器中
cd ~/aports/testing/winload
abuild checksum
# 这会自动下载源码并更新 APKBUILD 中的 sha512sums
cat APKBUILD | grep -A2 "sha512sums="
```

##### 提交 MR 到 aports/testing

```bash
# 回到宿主机
cd aports

# 1. 同步上游
git fetch upstream
git rebase upstream/master

# 2. 提交
git add testing/winload/APKBUILD
git commit -m "testing/winload: new aport

Network Load Monitor - nload-like TUI tool.
https://github.com/VincentZyuApps/winload"

# 3. 推送到你的 fork
git push origin winload-new-aport

# 4. 在 GitLab 上创建 Merge Request
#    - 访问 https://gitlab.alpinelinux.org/<你的用户名>/aports/-/merge_requests/new
#    - Source branch: winload-new-aport
#    - Target branch: master (upstream alpine/aports)
#    - Title: "testing/winload: new aport"
#    - Description 中附上：
#      - 项目简介
#      - 项目 URL
#      - 为什么要加入 Alpine
#      - 你愿意维护这个包
```

##### MR 审核须知

- `testing` 仓库审核较宽松，通常 **1-2 周**内合并
- Reviewer 可能会要求修改 APKBUILD 格式（Alpine 有严格的代码风格）
- 常见审核意见：
  - `sha512sums` 不能用 `SKIP`，必须填实际哈希
  - `check()` 函数要跑测试（如果有的话）
  - 依赖列表要精确
  - 描述不能有营销语言

##### 后续版本更新

包进入 `testing` 后，更新流程和初次提交一样：改 APKBUILD → 提 MR。

```bash
cd aports/testing/winload

# 1. 更新版本号
sed -i 's/pkgver=.*/pkgver=0.2.0/' APKBUILD
sed -i 's/pkgrel=.*/pkgrel=0/' APKBUILD

# 2. 更新校验和
abuild checksum

# 3. 提交 MR
git add APKBUILD
git commit -m "testing/winload: upgrade to 0.2.0"
git push origin winload-update
# 创建 MR...
```

##### 用户安装方式

```bash
# 启用 testing 仓库（默认未启用）
echo "https://dl-cdn.alpinelinux.org/alpine/edge/testing" >> /etc/apk/repositories

# 安装
apk update
apk add winload

# 卸载
apk del winload
```

---

#### 方案 B: 申请进入 community 仓库

> ⚠️ 这是**进阶操作**，建议先在 testing 稳定维护 1-2 个版本再考虑。

##### 什么是 community？

`community` 仓库中的包会在 Alpine 正式发布版中可用（不需要用户手动启用 testing），等同于「官方支持的包」。

##### 进入 community 的要求

1. **包已经在 testing 中稳定运行**（至少 1-2 个发布周期）
2. **有一位 Alpine 开发者愿意 sponsor 你的包**
3. **你承诺持续维护**（及时跟进上游版本、修复安全漏洞）

##### 申请流程

1. **找到一个 sponsor**：
   - 在 Alpine 开发者 IRC/Matrix 频道联系：
     - IRC: `#alpine-devel` on `irc.oftc.net`
     - Matrix: `#alpine-devel:oftc.net`
   - 或者在 MR 中直接 @mention 活跃的 Alpine 开发者
   - 礼貌地介绍你的包和维护意愿

2. **Sponsor 会帮你**：
   - Review 你的 APKBUILD
   - 将包从 `testing` 移动到 `community`
   - 提交 MR: `git mv testing/winload community/winload`

3. **成为 Alpine Contributor**（可选，更高权限）：
   - 持续贡献多个包后，可以申请成为 Alpine Developer
   - 需要签署 CLA（Contributor License Agreement）
   - 访问 https://wiki.alpinelinux.org/wiki/Developer_Handbook 了解详情

##### 从 testing 移到 community 的 MR 示例

```bash
cd aports
git checkout -b winload-to-community
git mv testing/winload community/winload
git commit -m "community/winload: move from testing

Winload has been stable in testing for N releases.
Sponsored-by: <sponsor-name>"
git push origin winload-to-community
# 创建 MR，需要 sponsor 的 approve
```

##### community 的额外要求

| 要求 | 说明 |
|------|------|
| 安全响应 | 上游有 CVE 时需要及时更新 |
| 版本跟进 | Alpine 每个大版本冻结前需要更新到最新 |
| 构建维护 | 确保在所有支持的架构上都能编译 |
| secfixes 注释 | 安全修复需要在 APKBUILD 中标注 CVE 号 |

#### ⚠️ Alpine 常见坑

| 问题 | 原因 | 解决 |
|------|------|------|
| `pkgver` 包含 `-` | Alpine 不允许连字符 | 用 `_` 替代：`0.1.6_beta4` |
| `cargo fetch` 失败 | 容器内无网络 | 确保 Docker 网络正常 |
| `abuild` 权限错误 | 不能以 root 运行 abuild | 切换到普通用户 `su builder` |
| `sha512sums` 校验失败 | 哈希过期 | 重新运行 `abuild checksum` |
| 交叉编译 aarch64 | Alpine CI 会自动在 aarch64 runner 上构建 | 你只需确保 `arch="x86_64 aarch64"` |
| Rust 版本太旧 | Alpine 仓库中的 Rust 可能落后 | 在 APKBUILD 中加 `makedepends="cargo rust>=1.70"` |

---

## 🍎 macOS 平台

### Homebrew ⭐

支持 **x86_64 + ARM64 (Apple Silicon)** 双架构。

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

# ============================================================
# Step 1: 获取各平台的 SHA256 哈希
# ============================================================
VERSION="0.1.5"  # 替换为实际版本号

# macOS x86_64
curl -sL "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-macos-x86_64-v${VERSION}" -o winload-macos-x86_64
SHA256_X86=$(sha256sum winload-macos-x86_64 | awk '{print $1}')
rm winload-macos-x86_64

# macOS ARM64 (aarch64)
curl -sL "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-macos-aarch64-v${VERSION}" -o winload-macos-aarch64
SHA256_AARCH64=$(sha256sum winload-macos-aarch64 | awk '{print $1}')
rm winload-macos-aarch64

# Linux x86_64（可选，Homebrew 也支持 Linux）
curl -sL "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-linux-x86_64-v${VERSION}" -o winload-linux-x86_64
SHA256_LINUX=$(sha256sum winload-linux-x86_64 | awk '{print $1}')
rm winload-linux-x86_64

echo "macOS x86_64 SHA256: $SHA256_X86"
echo "macOS ARM64 SHA256: $SHA256_AARCH64"
echo "Linux x86_64 SHA256: $SHA256_LINUX"

# ============================================================
# Step 2: 创建/更新 Formula
# ============================================================
cat > winload.rb <<'RUBY'
class Winload < Formula
  desc "Network Load Monitor - nload for Windows/Linux/macOS"
  homepage "https://github.com/VincentZyu233/winload"
  version "VERSION"  # 替换为实际版本号，如 0.1.5
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/VincentZyu233/winload/releases/download/vVERSION/winload-macos-aarch64-vVERSION"
      sha256 "SHA256_AARCH64"  # 替换为 ARM64 哈希
    else
      url "https://github.com/VincentZyu233/winload/releases/download/vVERSION/winload-macos-x86_64-vVERSION"
      sha256 "SHA256_X86"  # 替换为 x86_64 哈希
    end
  end

  on_linux do
    url "https://github.com/VincentZyu233/winload/releases/download/vVERSION/winload-linux-x86_64-vVERSION"
    sha256 "SHA256_LINUX"  # 替换为 Linux x86_64 哈希
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

# ============================================================
# Step 3: 实际替换占位符（推荐方式：直接写死版本和哈希）
# ============================================================
# 为简化维护，推荐直接写死版本号和哈希，而不是用变量
cat > winload.rb <<'RUBY'
class Winload < Formula
  desc "Network Load Monitor - nload for Windows/Linux/macOS"
  homepage "https://github.com/VincentZyu233/winload"
  version "0.1.5"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/VincentZyu233/winload/releases/download/v0.1.5/winload-macos-aarch64-v0.1.5"
      sha256 "a1b2c3d4e5f6..."  # 替换为实际 ARM64 哈希
    else
      url "https://github.com/VincentZyu233/winload/releases/download/v0.1.5/winload-macos-x86_64-v0.1.5"
      sha256 "f6e5d4c3b2a1..."  # 替换为实际 x86_64 哈希
    end
  end

  on_linux do
    url "https://github.com/VincentZyu233/winload/releases/download/v0.1.5/winload-linux-x86_64-v0.1.5"
    sha256 "1234567890ab..."  # 替换为实际 Linux 哈希
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
git commit -m "winload: Update to v0.1.5"
git push
```

#### 后续版本更新

```bash
cd homebrew-tap/Formula

VERSION="0.2.0"  # 新版本号

# 重新获取哈希
curl -sL "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-macos-x86_64-v${VERSION}" -o winload-macos-x86_64
SHA256_X86=$(sha256sum winload-macos-x86_64 | awk '{print $1}')
rm winload-macos-x86_64

curl -sL "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-macos-aarch64-v${VERSION}" -o winload-macos-aarch64
SHA256_AARCH64=$(sha256sum winload-macos-aarch64 | awk '{print $1}')
rm winload-macos-aarch64

curl -sL "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-linux-x86_64-v${VERSION}" -o winload-linux-x86_64
SHA256_LINUX=$(sha256sum winload-linux-x86_64 | awk '{print $1}')
rm winload-linux-x86_64

# 更新 Formula（用 sed 替换版本号和哈希）
sed -i 's/version ".*"/version "'${VERSION}'"/' winload.rb

# 重新创建整个文件更简单
cat > winload.rb <<'RUBY'
class Winload < Formula
  desc "Network Load Monitor - nload for Windows/Linux/macOS"
  homepage "https://github.com/VincentZyu233/winload"
  version "${VERSION}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-macos-aarch64-v${VERSION}"
      sha256 "${SHA256_AARCH64}"
    else
      url "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-macos-x86_64-v${VERSION}"
      sha256 "${SHA256_X86}"
    end
  end

  on_linux do
    url "https://github.com/VincentZyu233/winload/releases/download/v${VERSION}/winload-linux-x86_64-v${VERSION}"
    sha256 "${SHA256_LINUX}"
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

git add Formula/winload.rb
git commit -m "winload: Update to v${VERSION}"
git push
```

#### ⚠️ 常见坑

| 问题 | 原因 | 解决 |
|------|------|------|
| `Error: SHA256 mismatch` | 哈希值不正确 | 重新用 `sha256sum` 计算 |
| `No matching binary found` | URL 或文件名错误 | 检查 GitHub Release 中的实际文件名 |
| `on_linux do` 不生效 | Homebrew Linux 版本语法 | 确保是 Homebrew 4.0+ |

#### 用户安装方式
```bash
brew tap VincentZyu233/tap
brew install winload
```

---

## 📱 Termux (Android)

Termux 底层使用 `apt`/`dpkg`（和 Debian 类似），但 **不能直接用现有的 Linux .deb 包**。

> ⚠️ **为什么现有 Linux 二进制不能直接用？**
> 1. **Target triple 不同**：Termux 需要 `aarch64-linux-android` / `x86_64-linux-android`，不是 `aarch64-unknown-linux-gnu`
> 2. **前缀不同**：Termux 的根目录是 `/data/data/com.termux/files/usr/`，不是 `/usr/`
> 3. **libc 不同**：Android 用 Bionic libc，不是 glibc 也不是 musl

#### Termux 包来源

| 方式 | 说明 | 类比 |
|------|------|------|
| **termux-packages 官方仓库** | 提 PR 到 `termux/termux-packages` GitHub 仓库 | ≈ Alpine community（有审核） |
| **termux-user-repository (TUR)** | 社区维护的第三方仓库，门槛更低 | ≈ AUR |

---

### 提交到 TUR（Termux User Repository）⭐

TUR（`termux-user-repository/tur`）是社区驱动的，审核宽松，类似 AUR。

##### 前期准备（首次）

1. **Fork TUR 仓库**：
   - 访问 https://github.com/termux-user-repository/tur
   - 点击右上角 **Fork**

2. **Clone 你 fork 的仓库**：
```bash
git clone https://github.com/<你的用户名>/tur.git
cd tur
git remote add upstream https://github.com/termux-user-repository/tur.git
```

3. **安装 termux-packages 构建环境**（用于本地测试）：
```bash
# 克隆 termux-packages（TUR 的构建系统基于它）
git clone https://github.com/termux/termux-packages.git
cd termux-packages

# 安装 Docker（构建在 Docker 容器中进行）
# 确保你已安装 Docker

# 首次运行：构建 Docker 镜像（需要一些时间）
./scripts/run-docker.sh
# 这会下载并启动一个带完整 Android NDK 的构建容器
```

##### 创建包描述文件

```bash
cd tur
git checkout -b add-winload
mkdir -p tur/winload
cd tur/winload
```

创建 `build.sh` 文件：

```bash
cat > build.sh << 'EOF'
TERMUX_PKG_HOMEPAGE=https://github.com/VincentZyuApps/winload
TERMUX_PKG_DESCRIPTION="Network Load Monitor - nload-like TUI tool"
TERMUX_PKG_LICENSE="MIT"
TERMUX_PKG_MAINTAINER="VincentZyu <vincentzyu233@gmail.com>"
TERMUX_PKG_VERSION="0.1.6-beta.4"
TERMUX_PKG_SRCURL=https://github.com/VincentZyuApps/winload/archive/refs/tags/v${TERMUX_PKG_VERSION}.tar.gz
TERMUX_PKG_SHA256=SKIP_THIS_WILL_BE_FILLED
TERMUX_PKG_AUTO_UPDATE=true
TERMUX_PKG_BUILD_IN_SRC=true

termux_step_make() {
    termux_setup_rust
    cd rust
    cargo build --jobs $TERMUX_PKG_MAKE_PROCESSES --target $CARGO_TARGET_NAME --release
}

termux_step_make_install() {
    install -Dm755 -t $TERMUX_PREFIX/bin rust/target/$CARGO_TARGET_NAME/release/winload
}
EOF
```

> 📌 **TUR build.sh 关键字段**：
> - `TERMUX_PKG_HOMEPAGE` — 项目主页
> - `TERMUX_PKG_DESCRIPTION` — 简短描述
> - `TERMUX_PKG_LICENSE` — 许可证
> - `TERMUX_PKG_VERSION` — 版本号（可以用连字符，不像 Alpine）
> - `TERMUX_PKG_SRCURL` — 源码下载地址
> - `TERMUX_PKG_SHA256` — 源码包 SHA256 校验和
> - `TERMUX_PKG_AUTO_UPDATE=true` — 启用自动版本检测
> - `termux_setup_rust` — Termux 构建系统提供的 Rust 工具链设置函数
> - `$CARGO_TARGET_NAME` — 构建系统自动设置（如 `aarch64-linux-android`）
> - `$TERMUX_PREFIX` — Termux 的安装前缀（`/data/data/com.termux/files/usr`）

##### 计算 SHA256

```bash
# 下载源码包并计算哈希
VERSION="0.1.6-beta.4"
wget "https://github.com/VincentZyuApps/winload/archive/refs/tags/v${VERSION}.tar.gz"
SHA256=$(sha256sum "v${VERSION}.tar.gz" | awk '{print $1}')
echo "SHA256: $SHA256"
rm "v${VERSION}.tar.gz"

# 更新 build.sh 中的哈希
sed -i "s/TERMUX_PKG_SHA256=.*/TERMUX_PKG_SHA256=${SHA256}/" build.sh
```

##### 本地测试构建（使用 Docker）

```bash
# ============================================================
# 方式一：使用 termux-packages 的 Docker 构建系统
# ============================================================

# 1. 克隆 termux-packages（如果还没有）
git clone https://github.com/termux/termux-packages.git
cd termux-packages

# 2. 将你的包复制进去
cp -r /path/to/tur/tur/winload packages/

# 3. 在 Docker 中构建
./scripts/run-docker.sh ./build-package.sh winload

# 4. 构建产物在 output/ 目录
ls output/
# → winload_0.1.6-beta.4_aarch64.deb
# → winload_0.1.6-beta.4_x86_64.deb 等

# ============================================================
# 方式二：直接在 Termux 中测试（如果有 Android 设备）
# ============================================================

# 在 Termux 中
pkg install rust
git clone https://github.com/VincentZyuApps/winload.git
cd winload/rust
cargo build --release
cp target/release/winload $PREFIX/bin/
winload --version
```

##### 提交 PR 到 TUR

```bash
cd tur

# 1. 同步上游
git fetch upstream
git rebase upstream/master

# 2. 提交
git add tur/winload/build.sh
git commit -m "tur/winload: Add new package

Network Load Monitor - nload-like TUI tool.
Homepage: https://github.com/VincentZyuApps/winload"

# 3. 推送到你的 fork
git push origin add-winload

# 4. 在 GitHub 上创建 Pull Request
#    - 访问 https://github.com/<你的用户名>/tur/pulls
#    - 点击 "New pull request"
#    - Base: termux-user-repository/tur master
#    - Compare: 你的 add-winload 分支
#    - Title: "tur/winload: Add new package"
#    - Description 中附上：
#      - 项目简介和 URL
#      - 你在 Termux 中测试过的截图/日志
#      - 支持的架构：aarch64, x86_64
```

##### PR 审核须知

- TUR 审核比较**宽松**，通常 **几天到 1 周**内合并
- Reviewer 可能会要求：
  - 补全 SHA256 校验和（不能用 SKIP）
  - 测试截图/日志
  - `build.sh` 格式调整
- PR 合并后，包会自动构建并发布到 TUR 仓库

##### 后续版本更新

```bash
cd tur/tur/winload

# 1. 更新版本号
NEW_VERSION="0.2.0"
sed -i "s/TERMUX_PKG_VERSION=.*/TERMUX_PKG_VERSION=\"${NEW_VERSION}\"/" build.sh

# 2. 更新 SHA256
wget "https://github.com/VincentZyuApps/winload/archive/refs/tags/v${NEW_VERSION}.tar.gz"
NEW_SHA256=$(sha256sum "v${NEW_VERSION}.tar.gz" | awk '{print $1}')
sed -i "s/TERMUX_PKG_SHA256=.*/TERMUX_PKG_SHA256=${NEW_SHA256}/" build.sh
rm "v${NEW_VERSION}.tar.gz"

# 3. 提交 PR
git add build.sh
git commit -m "tur/winload: Update to ${NEW_VERSION}"
git push origin winload-update
# 创建 PR...
```

##### 用户安装方式

```bash
# 1. 在 Termux 中添加 TUR 仓库（首次）
pkg install tur-repo

# 2. 安装
pkg install winload
# 或
apt install winload

# 3. 使用
winload

# 4. 卸载
pkg uninstall winload
```

##### 关于 Termux 官方仓库（termux-packages）

> 如果 winload 在 TUR 中稳定运行一段时间，可以考虑提 PR 到 `termux/termux-packages` 官方仓库。
> 流程和 TUR 类似，但审核更严格，通常需要：
> - 包在 TUR 中有一定使用量
> - 代码质量和构建配置符合 Termux 标准
> - 维护者响应及时

#### ⚠️ Termux 常见坑

| 问题 | 原因 | 解决 |
|------|------|------|
| 编译失败 `android` 未知 target | 缺少 Android NDK | `termux_setup_rust` 会自动配置 |
| `sysinfo` crate 不工作 | Android 权限限制 | 需要测试，部分网卡信息可能受限 |
| 安装路径错误 | 用了 `/usr/bin` | 必须用 `$TERMUX_PREFIX/bin` |
| Docker 构建内存不足 | Rust 编译吃内存 | 分配 ≥4GB RAM 给 Docker |
| `TERMUX_PKG_SHA256=SKIP` | PR 不会被接受 | 必须填真实哈希 |
| 链接错误 (Bionic) | 使用了 glibc 特有 API | 确保源码兼容 Android Bionic |

---

## 🎯 推荐发布顺序

### 第一批（简单且用户多）
1. ✅ **Scoop** — 已有 CI 自动化 ✨
2. ✅ **DEB** — `cargo-deb` 一条命令出包
3. ✅ **AUR** — 写 PKGBUILD + push 到 AUR

### 第二批
4. ✅ **Homebrew** — 创建 tap 仓库，写 Formula
5. ✅ **RPM** — `cargo-generate-rpm` 出包

### 第三批
6. ⏸️ **Alpine APK (testing)** — 写 APKBUILD + 提 MR 到 aports（musl 二进制天然兼容）
7. ⏸️ **Termux (TUR)** — 写 build.sh + 提 PR 到 TUR（需要 Android target 编译）
8. ⏸️ **Winget** — 首次需要 PR 审核

### 第四批（进阶）
9. ⏸️ **Alpine community** — 需要 sponsor + 维护承诺
10. ⏸️ **Termux 官方** — 从 TUR 毕业到 termux-packages

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
- [ ] 更新 Alpine APKBUILD（更新 pkgver，运行 `abuild checksum`，提 MR）
- [ ] 更新 Termux TUR build.sh（更新 TERMUX_PKG_VERSION + SHA256，提 PR）
- [ ] 测试安装：`scoop install winload`、`brew install winload`、`paru -S winload-rust-bin`、`paru -S winload-rust`
- [ ] 测试安装：Alpine `apk add winload`、Termux `pkg install winload`

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
