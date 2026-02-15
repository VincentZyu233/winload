![winload](https://socialify.git.ci/VincentZyu233/winload/image?custom_language=Rust&description=1&forks=1&issues=1&language=1&logo=https%3A%2F%2Favatars.githubusercontent.com%2Fu%2F250448479%3Fs%3D200%26v%3D4&name=1&owner=1&pulls=1&stargazers=1&theme=Auto)


# Winload <img src="https://github.com/user-attachments/assets/62fec846-0442-47f6-bbba-78acdc8803ef" height="32px">

> 轻量级实时终端网络流量监控工具，灵感来自 Linux 的 nload。

> **[📖 English](readme.md)**

[![Windows x64 | ARM64](https://img.shields.io/badge/Windows-x64_|_ARM64-0078D4?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/VincentZyu233/winload/releases)
[![Linux x64 | ARM64](https://img.shields.io/badge/Linux-x64_|_ARM64-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/VincentZyu233/winload/releases)
[![macOS x64 | ARM64](https://img.shields.io/badge/macOS-x64_|_ARM64-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/VincentZyu233/winload/releases)

## 🚀 简介
Winload 是一个直观的终端网络流量监控工具。最初为 Windows 打造，弥补 nload 在 Windows 上的空白，现已支持 Linux 和 macOS。

## 🙏 致谢
Winload 的灵感来自 Roland Riegel 的经典 nload 项目，感谢原作者的创意与体验。
https://github.com/rolandriegel/nload

## ✨ 主要特性
- **双实现版本**
	- **Rust 版**: 快速、内存安全、单静态二进制文件，适合日常监控。
	- **Python 版**: 易于修改和扩展，适合原型开发或集成。
- **跨平台**: Windows、Linux、macOS（x64 & ARM64）。
- **实时可视化**: 实时上行/下行流量图和吞吐量统计。
- **简洁界面**: 干净的 TUI，沿袭 nload 的人体工程学设计。

## 📟 用法

```bash
winload              # 监控所有活跃网络接口
winload -t 200       # 设置刷新间隔为 200ms
winload -d "Wi-Fi"   # 启动时定位到 Wi-Fi 网卡
winload -e           # 启用 emoji 装饰 🎉
winload --npcap      # 捕获 127.0.0.1 回环流量 (Windows，需安装 Npcap)
```

### 参数选项

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-t`, `--interval <MS>` | 刷新间隔（毫秒） | `500` |
| `-a`, `--average <SEC>` | 平均值计算窗口（秒） | `300` |
| `-d`, `--device <NAME>` | 默认设备名（模糊匹配） | — |
| `-e`, `--emoji` | 启用 emoji 装饰 🎉 | 关闭 |
| `-U`, `--unicode` | 使用 Unicode 方块字符绘图（█▓░·） | 关闭 |
| `-u`, `--unit <UNIT>` | 显示单位：`bit` 或 `byte` | `bit` |
| `-b`, `--bar-style <STYLE>` | 状态栏样式：`fill`、`color` 或 `plain` | `fill` |
| `--in-color <HEX>` | 下行图形颜色，十六进制 RGB（如 `0x00d7ff`） | 青色 |
| `--out-color <HEX>` | 上行图形颜色，十六进制 RGB（如 `0xffaf00`） | 金色 |
| `-m`, `--max <VALUE>` | 固定 Y 轴最大值（如 `10M`、`1G`、`500K`） | 自动 |
| `-n`, `--no-graph` | 隐藏图形，仅显示统计信息 | 关闭 |
| `--hide-separator` | 隐藏分隔线（等于号一行） | 关闭 |
| `--npcap` | **[Windows Only]** 通过 Npcap 捕获回环流量（推荐） | 关闭 |
| `--etw` | **[Windows Only]** 通过 GetIfEntry API 轮询回环计数器（实验性） | 关闭 |
| `--debug-info` | **[Rust Only]** 打印网络接口调试信息后退出 | — |
| `-h`, `--help` | 打印帮助（`--help --emoji` 可查看 emoji 版！） | — |
| `-V`, `--version` | **[Rust Only]** 打印版本号 | — |

### 快捷键

| 按键 | 功能 |
|------|------|
| `←` / `→` 或 `↑` / `↓` | 切换网络设备 |
| `=` | 切换分隔线的显示/隐藏 |
| `q` / `Esc` | 退出 |

## 🪟 Windows 回环流量 (127.0.0.1)

Windows 无法通过标准 API 报告回环流量——这是 [Windows 网络栈的功能缺失](docs/win_loopback.zh-cn.md)。

winload 提供两种解决方案：

| 参数 | 方式 | 状态 |
|------|------|------|
| `--npcap` | Npcap WFP callout 驱动 | ✅ **推荐** — 数据准确，真实数据包捕获 |
| `--etw` | `GetIfEntry` API 轮询 | ⚠️ 实验性 — 大多数 Windows 版本计数器为 0 |

**使用 `--npcap`**: 安装 [Npcap](https://npcap.com/#download)，安装时勾选"Support loopback traffic capture"。

> 📖 深入了解 Windows 回环为何失效，请阅读 [docs/win_loopback.zh-cn.md](docs/win_loopback.zh-cn.md)

在 Linux 和 macOS 上，回环流量开箱即用，无需额外参数。

## 🖼️ 预览
#### Python 版预览
![docs/preview-py.png](docs/preview-py.png)

#### Rust 版预览
![docs/preview-rust.png](docs/preview-rust.png)
