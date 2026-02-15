![winload](https://socialify.git.ci/VincentZyu233/winload/image?custom_language=Rust&description=1&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)


# Winload <img src="https://github.com/user-attachments/assets/62fec846-0442-47f6-bbba-78acdc8803ef" height="32px">

> A lightweight, real-time CLI tool for monitoring network bandwidth and traffic, inspired by Linux's nload.

> **[📖 中文文档](readme.zh-cn.md)**

[![Windows x64 | ARM64](https://img.shields.io/badge/Windows-x64_|_ARM64-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/VincentZyu233/winload/releases)
[![Linux x64 | ARM64](https://img.shields.io/badge/Linux-x64_|_ARM64-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/VincentZyu233/winload/releases)
[![macOS x64 | ARM64](https://img.shields.io/badge/macOS-x64_|_ARM64-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/VincentZyu233/winload/releases)

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

## 📟 Usage

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
| `--npcap` | **[Windows Rust Only]** Capture loopback traffic via Npcap (recommended) | off |
| `--etw` | **[Windows Rust Only]** Poll loopback counters via GetIfEntry (experimental) | off |
| `--debug-info` | **[Rust Only]** Print network interface debug info and exit | — |
| `-h`, `--help` | Print help (`--help --emoji` for emoji version!) | — |
| `-V`, `--version` | **[Rust Only]** Print version | — |

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `←` / `→` or `↑` / `↓` | Switch network device |
| `=` | Toggle separator line visibility |
| `q` / `Esc` | Quit |

## 🪟 Windows Loopback (127.0.0.1)

Windows cannot report loopback traffic through standard APIs — this is a [functional deficiency in Windows' network stack](docs/win_loopback.md).

winload provides two workarounds:

| Flag | Method | Status |
|------|--------|--------|
| `--npcap` | Npcap WFP callout driver | ✅ **Recommended** — accurate, real packet capture |
| `--etw` | `GetIfEntry` API polling | ⚠️ Experimental — counters are 0 on most Windows versions |

**To use `--npcap`**: Install [Npcap](https://npcap.com/#download) and enable "Support loopback traffic capture" during installation.

> 📖 For a deep dive into why Windows loopback is broken, see [docs/win_loopback.md](docs/win_loopback.md)

On Linux and macOS, loopback traffic works out of the box — no extra flags needed.

## 🖼️ Previews
#### preview of python impl
![docs/preview-py.png](docs/preview-py.png)

#### preview of rust impl
![docs/preview-rust.png](docs/preview-rust.png)
