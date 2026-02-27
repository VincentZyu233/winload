![winload](https://socialify.git.ci/VincentZyu233/winload/image?custom_language=Rust&description=1&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)

# Winload <img src="https://github.com/user-attachments/assets/62fec846-0442-47f6-bbba-78acdc8803ef" height="32px">

> A lightweight, real-time CLI tool for monitoring network bandwidth and traffic, inspired by Linux's nload.

> **[📖 English](readme.md)**
> **[📖 简体中文(大陆)](readme.zh-cn.md)**
> **[📖 繁體中文(台灣)](readme.zh-tw.md)**
> **[📖 日本語](readme.jp.md)**
> **[📖 한국어](readme.ko.md)**

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/VincentZyuApps/winload)
[![Gitee](https://img.shields.io/badge/Gitee-C71D23?style=for-the-badge&logo=gitee&logoColor=white)](https://gitee.com/vincent-zyu/winload)

[![Windows x64 | ARM64](https://img.shields.io/badge/Windows-x64_|_ARM64-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)
[![Linux x64 | ARM64](https://img.shields.io/badge/Linux-x64_|_ARM64-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/VincentZyuApps/winload/releases)
[![macOS x64 | ARM64](https://img.shields.io/badge/macOS-x64_|_ARM64-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)
[![Android x64 | ARM64](https://img.shields.io/badge/Android-x64_|_ARM64-3DDC84?style=for-the-badge&logo=android&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)

[![PyPI](https://img.shields.io/badge/PyPI-3776AB?style=for-the-badge&logo=pypi&logoColor=white)](https://pypi.org/project/winload/)
[![npm](https://img.shields.io/badge/npm-CB3837?style=for-the-badge&logo=npm&logoColor=white)](https://www.npmjs.com/package/winload-rust-bin)

[![Scoop](https://img.shields.io/badge/Scoop-7B4AE2?style=for-the-badge&logo=scoop&logoColor=white)](https://scoop.sh/#/apps?q=%22https%3A%2F%2Fgithub.com%2FVincentZyuApps%2Fscoop-bucket%22&o=false)
[![AUR](https://img.shields.io/badge/AUR-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)](https://aur.archlinux.org/packages/winload-rust-bin)
[![APT](https://img.shields.io/badge/APT-E95420?style=for-the-badge&logo=debian&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)
[![RPM](https://img.shields.io/badge/RPM-CB1626?style=for-the-badge&logo=redhat&logoColor=white)](https://github.com/VincentZyuApps/winload/releases)

> **[📖 Build Docs](.github/workflows/build.md)**

## 🚀 Introduction
Winload brings an intuitive, visual network monitor to the modern terminal. It started as a Windows-focused tool to fill the nload gap, and now targets Linux and macOS as well.

## 🙏 Acknowledgements
Winload is inspired by the classic nload project by Roland Riegel. Many thanks for the original idea and experience.
https://github.com/rolandriegel/nload

## ✨ Key Features
- **Dual implementations**
	- **Rust edition**: fast, memory-safe, single static binary—great for everyday monitoring.
	- **Python edition**: easy to hack and extend for prototyping or integrations.
- **Cross-platform**: Windows, Linux, and macOS (x64 & ARM64).
- **Real-time visualization**: live incoming/outgoing graphs and throughput stats.
- **Minimal UI**: clean TUI that mirrors nload's ergonomics.

## 📥 Python Edition Installation

### Python (pip)
```bash
pip install winload
```

## 📥 Rust Edition Installation (recommended)

### Windows (Scoop)
```powershell
scoop bucket add vincentzyu https://github.com/VincentZyuApps/scoop-bucket
scoop install winload
```

### Arch Linux (AUR):
```bash
paru -S winload-rust-bin
```

### Linux (one-liner)
> Supports Debian/Ubuntu and derivatives — Linux Mint, Pop!_OS, Deepin, UOS, etc. (apt)

> Supports Fedora/RHEL and derivatives — Rocky Linux, AlmaLinux, CentOS Stream, etc. (dnf)
```bash
curl -fsSL https://raw.githubusercontent.com/VincentZyuApps/winload/main/docs/install_scripts/install.sh | bash
```
> 📄 [View install script source](https://github.com/VincentZyuApps/winload/blob/main/docs/install_scripts/install.sh)

<details>
<summary>Manual install</summary>

**DEB (Debian/Ubuntu):**
```bash
# Download the latest .deb from GitHub Releases
sudo dpkg -i ./winload_*_amd64.deb
# or use apt (auto-resolves dependencies)
sudo apt install ./winload_*_amd64.deb
```

**RPM (Fedora/RHEL):**
```bash
sudo dnf install ./winload-*-1.x86_64.rpm
```

**Or download binaries directly from [GitHub Releases](https://github.com/VincentZyuApps/winload/releases).**

</details>

## ⌨️ Usage

```bash
winload              # Monitor all active network interfaces
winload -t 200       # Set refresh interval to 200ms
winload -d "Wi-Fi"   # Start with a specific device
winload -e           # Enable emoji decorations 🎉
winload --npcap      # Capture 127.0.0.1 loopback traffic (Windows, requires Npcap)
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t`, `--interval <MS>` | Refresh interval in milliseconds | `500` |
| `-a`, `--average <SEC>` | Average calculation window in seconds | `300` |
| `-d`, `--device <NAME>` | Default device name (partial match) | — |
| `-e`, `--emoji` | Enable emoji decorations in TUI 🎉 | off |
| `-U`, `--unicode` | Use Unicode block characters for graph (█▓░·) | off |
| `-u`, `--unit <UNIT>` | Display unit: `bit` or `byte` | `bit` |
| `-b`, `--bar-style <STYLE>` | Bar style: `fill`, `color`, or `plain` | `fill` |
| `--in-color <HEX>` | Incoming graph color, hex RGB (e.g. `0x00d7ff`) | cyan |
| `--out-color <HEX>` | Outgoing graph color, hex RGB (e.g. `0xffaf00`) | gold |
| `-m`, `--max <VALUE>` | Fixed Y-axis max (e.g. `10M`, `1G`, `500K`) | auto |
| `-n`, `--no-graph` | Hide graph, show stats only | off |
| `--hide-separator` | Hide the separator line (row of equals signs) | off |
| `--no-color` | Disable all TUI colors (monochrome mode) | off |
| `--npcap` | **[Windows Rust Only]** Capture loopback traffic via Npcap (recommended) | off |
| `--debug-info` | **[Rust Only]** Print network interface debug info and exit | — |
| `-h`, `--help` | Print help (`--help --emoji` for emoji version!) | — |
| `-V`, `--version` | **[Rust Only]** Print version | — |

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `←` / `→` or `↑` / `↓` | Switch network device |
| `=` | Toggle separator line visibility |
| `c` | Toggle color on/off |
| `q` / `Esc` | Quit |

## 🪟 Windows Loopback (127.0.0.1)

Windows cannot report loopback traffic through standard APIs — this is a [functional deficiency in Windows' network stack](docs/win_loopback.md).

**To capture loopback traffic on Windows**, use the `--npcap` flag:

```bash
winload --npcap
```

This requires [Npcap](https://npcap.com/#download) installed with "Support loopback traffic capture" enabled during setup.

> I previously tried polling Windows' own `GetIfEntry` API directly, but the counters are always 0 for loopback — there is simply no NDIS driver behind the loopback pseudo-interface to count anything. That code path has been removed.

> 📖 For a deep dive into why Windows loopback is broken, see [docs/win_loopback.md](docs/win_loopback.md)

On Linux and macOS, loopback traffic works out of the box — no extra flags needed.

## 🖼️ Previews
#### Python Edition Preview
![docs/preview-py.png](docs/preview-py.png)

#### Rust Edition Preview
![docs/preview-rust.png](docs/preview-rust.png)
