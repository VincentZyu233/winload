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
import platform
import sys
import time
from importlib.metadata import version as get_pkg_version

from collector import Collector
from i18n import t, set_lang, get_lang
from ui import UI


def get_version() -> str:
    try:
        return get_pkg_version("winload")
    except Exception:
        return "unknown"


def get_system_info() -> str:
    """Get system information string"""
    return f"System: {platform.system()} | Arch: {platform.machine()}"


def print_system_info() -> None:
    """Print system information to stderr"""
    print(f"\n{get_system_info()}", file=sys.stderr)


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
    # First pass: extract --lang early so we can set language before building help texts
    pre_parser = argparse.ArgumentParser(add_help=False)
    pre_parser.add_argument("--lang", type=str, default="en-us")
    pre_args, _ = pre_parser.parse_known_args()
    set_lang(pre_args.lang)

    parser = argparse.ArgumentParser(
        prog="winload",
        description=f"winload {get_version()} (Python edition)\n{t('description')}",
        epilog=f"\n{get_system_info()}",
        formatter_class=argparse.RawTextHelpFormatter,
    )
    parser.add_argument(
        "-t",
        "--interval",
        type=int,
        default=500,
        metavar="INTERVAL",
        help=t("help_interval"),
    )
    parser.add_argument(
        "-a",
        "--average",
        type=int,
        default=300,
        metavar="SEC",
        help=t("help_average"),
    )
    parser.add_argument(
        "-d",
        "--device",
        type=str,
        default=None,
        metavar="NAME",
        help=t("help_device"),
    )
    parser.add_argument(
        "-e",
        "--emoji",
        action="store_true",
        default=False,
        help=t("help_emoji"),
    )
    parser.add_argument(
        "-u",
        "--unit",
        type=str,
        choices=["bit", "byte"],
        default="bit",
        help=t("help_unit"),
    )
    parser.add_argument(
        "-m",
        "--max",
        type=str,
        default=None,
        metavar="VALUE",
        help=t("help_max"),
    )
    parser.add_argument(
        "-n",
        "--no-graph",
        action="store_true",
        default=False,
        help=t("help_no_graph"),
    )
    parser.add_argument(
        "-U",
        "--unicode",
        action="store_true",
        default=False,
        help=t("help_unicode"),
    )
    parser.add_argument(
        "-b",
        "--bar-style",
        type=str,
        choices=["fill", "color", "plain"],
        default="fill",
        help=t("help_bar_style"),
    )
    parser.add_argument(
        "--in-color",
        type=parse_hex_color,
        default=None,
        metavar="HEX",
        help=t("help_in_color"),
    )
    parser.add_argument(
        "--out-color",
        type=parse_hex_color,
        default=None,
        metavar="HEX",
        help=t("help_out_color"),
    )
    parser.add_argument(
        "--hide-separator",
        action="store_true",
        default=False,
        help=t("help_hide_separator"),
    )
    parser.add_argument(
        "-V",
        "--version",
        action="version",
        version=f"winload {get_version()} (Python edition)",
        help=t("help_version"),
    )
    parser.add_argument(
        "--no-color",
        action="store_true",
        default=False,
        help=t("help_no_color"),
    )
    parser.add_argument(
        "--lang",
        type=str,
        choices=["en-us", "zh-cn", "zh-tw"],
        default="en-us",
        metavar="LANG",
        help=t("help_lang"),
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
        print(t("error_no_curses"))
        print("  pip install windows-curses")
        sys.exit(1)

    try:
        curses.wrapper(lambda stdscr: main_loop(stdscr, args))
    except KeyboardInterrupt:
        pass
    finally:
        print_system_info()


if __name__ == "__main__":
    main()
