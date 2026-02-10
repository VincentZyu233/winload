![winload](https://socialify.git.ci/VincentZyu233/winload/image?description=1&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)

# Winload <img src="https://github.com/user-attachments/assets/62fec846-0442-47f6-bbba-78acdc8803ef" height="32px">

> A lightweight, real-time CLI tool for monitoring network bandwidth and traffic, inspired by Linux's nload.

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
- **Cross-platform vision**: Windows, Linux, and macOS.
- **Real-time visualization**: live incoming/outgoing graphs and throughput stats.
- **Minimal UI**: clean TUI that mirrors nload’s ergonomics.

## 📟 Usage

```bash
winload              # Monitor all active network interfaces
winload -t 200       # Set refresh interval to 200ms
winload -d "Wi-Fi"   # Start with a specific device
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-t`, `--interval <MS>` | Refresh interval in milliseconds | `500` |
| `-a`, `--average <SEC>` | Average calculation window in seconds | `300` |
| `-d`, `--device <NAME>` | Default device name (partial match) | — |
| `--debug-info` | Print network interface debug info and exit *(Rust only)* | — |
| `-h`, `--help` | Print help | — |
| `-V`, `--version` | Print version *(Rust only)* | — |

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `←` / `→` or `↑` / `↓` | Switch network device |
| `q` / `Esc` | Quit |

##  🖼️Previews
#### preview of python impl
![docs/preview-py.png](docs/preview-py.png)

#### preview of rust impl
![docs/preview-rust.png](docs/preview-rust.png)
