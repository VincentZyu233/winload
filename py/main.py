"""
winload - Windows Network Load Monitor
仿 Linux nload 的终端网络流量监控工具

用法:
    python main.py              # 监控所有活跃网卡
    python main.py -t 200       # 设置刷新间隔 200ms
    python main.py -d "Wi-Fi"   # 指定默认设备

快捷键:
    ←/→  或 ↑/↓   切换网卡
    q              退出
"""

import argparse
import curses
import sys
import time
from importlib.metadata import version as get_pkg_version

from collector import Collector
from ui import UI


def get_version() -> str:
    try:
        return get_pkg_version("winload")
    except Exception:
        return "unknown"


def parse_max_value(s: str) -> float:
    """解析人类可读的流量值，如 '100M' → 100*1024*1024"""
    s = s.strip()
    multipliers = {
        "G": 1024**3,
        "g": 1024**3,
        "M": 1024**2,
        "m": 1024**2,
        "K": 1024,
        "k": 1024,
    }
    for suffix, mul in multipliers.items():
        if s.endswith(suffix):
            return float(s[:-1]) * mul
    return float(s)


def parse_hex_color(s: str):
    """解析十六进制颜色码，如 '0x00d7ff' → (0, 215, 255)"""
    s = s.strip()
    if s.startswith(("0x", "0X")):
        s = s[2:]
    if len(s) != 6:
        raise argparse.ArgumentTypeError(
            f"expected 6 hex digits (e.g. 0x3399ff), got: {s}"
        )
    try:
        r = int(s[0:2], 16)
        g = int(s[2:4], 16)
        b = int(s[4:6], 16)
    except ValueError as e:
        raise argparse.ArgumentTypeError(f"invalid hex color: {e}")
    return (r, g, b)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="winload",
        description="Network Load Monitor — nload-like TUI tool for Windows/Linux/macOS",
        formatter_class=argparse.RawTextHelpFormatter,
    )
    parser.add_argument(
        "-t",
        "--interval",
        type=int,
        default=500,
        metavar="INTERVAL",
        help="Refresh interval in milliseconds\n刷新间隔（毫秒）\n\n[default: 500]",
    )
    parser.add_argument(
        "-a",
        "--average",
        type=int,
        default=300,
        metavar="SEC",
        help="Average window in seconds\n平均值计算窗口（秒）\n\n[default: 300]",
    )
    parser.add_argument(
        "-d",
        "--device",
        type=str,
        default=None,
        metavar="NAME",
        help="Default device name (partial match)\n默认网卡名称（支持部分匹配）",
    )
    parser.add_argument(
        "-e",
        "--emoji",
        action="store_true",
        default=False,
        help="Enable emoji decorations in TUI 🎉\n启用 emoji 装饰模式 🎉",
    )
    parser.add_argument(
        "-u",
        "--unit",
        type=str,
        choices=["bit", "byte"],
        default="bit",
        help="Display unit: bit (default) or byte\n显示单位：bit（默认）或 byte",
    )
    parser.add_argument(
        "-m",
        "--max",
        type=str,
        default=None,
        metavar="VALUE",
        help="Fixed graph Y-axis max (e.g. 100M, 1G, 500K)\n"
        "固定图形 Y 轴最大值（如 100M、1G、500K）\n\n"
        "[default: auto-scale]",
    )
    parser.add_argument(
        "-n",
        "--no-graph",
        action="store_true",
        default=False,
        help="Hide traffic graphs, show only statistics\n隐藏流量图形，仅显示统计信息",
    )
    parser.add_argument(
        "-U",
        "--unicode",
        action="store_true",
        default=False,
        help="Use Unicode block characters for graph (█▓░· instead of #|..)\n"
        "使用 Unicode 块字符绘制图形（█▓░· 代替 #|..）",
    )
    parser.add_argument(
        "-b",
        "--bar-style",
        type=str,
        choices=["fill", "color", "plain"],
        default="fill",
        help="Bar style: fill (default), color, plain\n"
        "状态栏样式：fill（默认），color，plain",
    )
    parser.add_argument(
        "--in-color",
        type=parse_hex_color,
        default=None,
        metavar="HEX",
        help="Incoming (download) graph color, hex RGB (e.g. 0x00d7ff)\n"
        "入站（下载）图形颜色，十六进制 RGB（如 0x00d7ff）\n\n"
        "[default: cyan]",
    )
    parser.add_argument(
        "--out-color",
        type=parse_hex_color,
        default=None,
        metavar="HEX",
        help="Outgoing (upload) graph color, hex RGB (e.g. 0xffaf00)\n"
        "出站（上传）图形颜色，十六进制 RGB（如 0xffaf00）\n\n"
        "[default: gold]",
    )
    parser.add_argument(
        "--hide-separator",
        action="store_true",
        default=False,
        help="Hide separator line (the row of equals signs between header and panels)\n"
        "隐藏分隔线（标题和面板之间的等号行）",
    )
    parser.add_argument(
        "-V",
        "--version",
        action="version",
        version=f"winload {get_version()} (Python edition)",
        help="Print version\n打印版本号",
    )
    parser.add_argument(
        "--no-color",
        action="store_true",
        default=False,
        help="Disable all TUI colors (monochrome mode), press 'c' to toggle at runtime\n"
        "禁用所有 TUI 颜色（单色模式），运行时按 c 可切换",
    )
    return parser.parse_args()


def main_loop(stdscr: "curses.window", args: argparse.Namespace) -> None:
    """curses 主循环"""
    collector = Collector()

    # 解析 --max 参数
    fixed_max = None
    if args.max:
        try:
            fixed_max = parse_max_value(args.max)
        except (ValueError, IndexError):
            pass

    ui = UI(
        stdscr,
        collector,
        emoji=args.emoji,
        unit=args.unit,
        fixed_max=fixed_max,
        no_graph=args.no_graph,
        unicode=args.unicode,
        bar_style=args.bar_style,
        in_color=args.in_color,
        out_color=args.out_color,
        hide_separator=args.hide_separator,
        no_color=args.no_color,
    )

    # 如果指定了默认设备，切换到对应索引
    if args.device:
        for i, v in enumerate(ui.views):
            if args.device.lower() in v.name.lower():
                ui.current_device_idx = i
                break

    # 设置 stdin 非阻塞
    stdscr.nodelay(True)
    stdscr.timeout(100)  # getch 超时 100ms

    refresh_interval_sec = args.interval / 1000.0
    last_update = 0.0

    while True:
        now = time.time()

        # 处理键盘输入
        try:
            key = stdscr.getch()
            if key != -1:
                if not ui.handle_key(key):
                    break
        except curses.error:
            pass

        # 按刷新间隔采样 + 重绘
        if now - last_update >= refresh_interval_sec:
            ui.update()
            ui.draw()
            curses.doupdate()
            last_update = now


def main() -> None:
    args = parse_args()

    # Windows 需要 windows-curses
    try:
        import curses as _curses  # noqa: F401
    except ImportError:
        print("错误: 请先安装 windows-curses")
        print("  pip install windows-curses")
        sys.exit(1)

    try:
        curses.wrapper(lambda stdscr: main_loop(stdscr, args))
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
