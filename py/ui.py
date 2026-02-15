"""
ui.py - 基于 curses 的 TUI 界面渲染
仿 nload 的双面板布局：上半 Incoming / 下半 Outgoing
"""

import curses
import sys
from typing import List, Optional

from collector import Collector, DeviceInfo
from stats import StatisticsEngine, TrafficStats, format_speed, format_speed_unit, format_bytes
from graph import render_graph, next_power_of_2_scaled, get_graph_scale_label_unit


class DeviceView:
    """单个网卡设备的视图，包含 Incoming 和 Outgoing 两个面板"""

    def __init__(self, name: str, info: Optional[DeviceInfo] = None):
        self.name = name
        self.info = info
        self.engine = StatisticsEngine()

    def get_addr_str(self) -> str:
        if self.info and self.info.addrs:
            return self.info.addrs[0]
        return ""


class UI:
    """curses TUI 主控"""

    # 颜色对
    COLOR_HEADER = 1
    COLOR_SEPARATOR = 2
    COLOR_LABEL = 3
    COLOR_GRAPH_FULL = 4
    COLOR_GRAPH_HIGH = 5
    COLOR_GRAPH_LOW = 6
    COLOR_STAT_LABEL = 7
    COLOR_STAT_VALUE = 8
    COLOR_HELP = 9
    COLOR_ERROR = 10

    def __init__(self, stdscr: "curses.window", collector: Collector,
                 emoji: bool = False, unit: str = "bit",
                 fixed_max: Optional[float] = None, no_graph: bool = False,
                 unicode: bool = False, bar_style: str = "fill",
                 in_color: Optional[tuple] = None, out_color: Optional[tuple] = None,
                 hide_separator: bool = False):
        self.stdscr = stdscr
        self.collector = collector
        self.current_device_idx = 0
        self.views: List[DeviceView] = []
        self.emoji = emoji
        self.unit = unit
        self.fixed_max = fixed_max
        self.no_graph = no_graph
        self.unicode = unicode
        self.bar_style = bar_style
        self.in_color_rgb = in_color
        self.out_color_rgb = out_color
        self.hide_separator = hide_separator

        # 初始化颜色
        curses.start_color()
        curses.use_default_colors()
        try:
            curses.init_pair(self.COLOR_HEADER, curses.COLOR_WHITE, -1)
            curses.init_pair(self.COLOR_SEPARATOR, curses.COLOR_CYAN, -1)
            curses.init_pair(self.COLOR_LABEL, curses.COLOR_GREEN, -1)
            curses.init_pair(self.COLOR_GRAPH_FULL, curses.COLOR_GREEN, -1)
            curses.init_pair(self.COLOR_GRAPH_HIGH, curses.COLOR_GREEN, -1)
            curses.init_pair(self.COLOR_GRAPH_LOW, curses.COLOR_WHITE, -1)
            curses.init_pair(self.COLOR_STAT_LABEL, curses.COLOR_CYAN, -1)
            curses.init_pair(self.COLOR_STAT_VALUE, curses.COLOR_WHITE, -1)
            curses.init_pair(self.COLOR_HELP, curses.COLOR_YELLOW, -1)
            curses.init_pair(self.COLOR_ERROR, curses.COLOR_RED, -1)
        except curses.error:
            pass

        # 自定义颜色 (in/out graph colors)
        # 颜色对 11-16 用于自定义 graph 颜色
        self.COLOR_IN_GRAPH = self.COLOR_GRAPH_FULL   # 默认复用绿色
        self.COLOR_OUT_GRAPH = self.COLOR_GRAPH_FULL
        self.COLOR_IN_LABEL = self.COLOR_LABEL
        self.COLOR_OUT_LABEL = self.COLOR_LABEL
        self._can_change_color = curses.can_change_color()
        try:
            if self._can_change_color and self.in_color_rgb:
                r, g, b = self.in_color_rgb
                curses.init_color(20, r * 1000 // 255, g * 1000 // 255, b * 1000 // 255)
                curses.init_pair(11, 20, -1)
                self.COLOR_IN_GRAPH = 11
                self.COLOR_IN_LABEL = 11
            if self._can_change_color and self.out_color_rgb:
                r, g, b = self.out_color_rgb
                curses.init_color(21, r * 1000 // 255, g * 1000 // 255, b * 1000 // 255)
                curses.init_pair(12, 21, -1)
                self.COLOR_OUT_GRAPH = 12
                self.COLOR_OUT_LABEL = 12
        except curses.error:
            pass

        # 隐藏光标
        try:
            curses.curs_set(0)
        except curses.error:
            pass

        self._init_views()

    def _init_views(self) -> None:
        """根据 collector 的设备列表创建视图"""
        self.views.clear()
        for name in self.collector.device_names:
            info = self.collector.get_device_info(name)
            self.views.append(DeviceView(name, info))
        if not self.views:
            # fallback: 如果没有设备（不太可能），至少显示一个占位
            self.views.append(DeviceView("(no device)"))

    @property
    def current_view(self) -> DeviceView:
        idx = self.current_device_idx % len(self.views)
        return self.views[idx]

    def next_device(self) -> None:
        if self.views:
            self.current_device_idx = (
                (self.current_device_idx + 1) % len(self.views)
            )

    def prev_device(self) -> None:
        if self.views:
            self.current_device_idx = (
                (self.current_device_idx - 1) % len(self.views)
            )

    def update(self) -> None:
        """采集一次数据并更新所有设备的统计"""
        snapshots = self.collector.collect()
        for view in self.views:
            snap = snapshots.get(view.name)
            if snap:
                view.engine.update(snap)

    def draw(self) -> None:
        """绘制整个界面"""
        self.stdscr.erase()
        max_y, max_x = self.stdscr.getmaxyx()

        if max_y < 10 or max_x < 40:
            self._draw_too_small(max_y, max_x)
            self.stdscr.noutrefresh()
            return

        view = self.current_view
        device_idx = self.current_device_idx % len(self.views)

        row = 0

        # ── 头部: Device name [ip] (n/m): ──
        addr = view.get_addr_str()
        addr_str = f" [{addr}]" if addr else ""
        if self.emoji:
            header = (
                f"🖧 Device {view.name}{addr_str} "
                f"({device_idx + 1}/{len(self.views)}) 📡:"
            )
        else:
            header = (
                f"Device {view.name}{addr_str} "
                f"({device_idx + 1}/{len(self.views)}):"
            )
        header_attr = self._get_bar_attr(self.COLOR_HEADER, bold=True)
        if self.bar_style == "fill":
            header = header.ljust(max_x - 1)
        self._safe_addstr(row, 0, header, header_attr)
        row += 1

        # ── Loopback 警告（仅 Windows）──
        if self._is_loopback_on_windows(view):
            warning = " \u26a0 Loopback traffic stats are not available on Windows"
            warn_attr = self._get_bar_attr(self.COLOR_HELP)
            if self.bar_style == "fill":
                warning = warning.ljust(max_x - 1)
            self._safe_addstr(row, 0, warning, warn_attr)
            row += 1

        # ── 分隔线 ──
        if not self.hide_separator:
            sep = "=" * min(max_x - 1, 120)
            self._safe_addstr(row, 0, sep, curses.color_pair(self.COLOR_SEPARATOR))
            row += 1

        # 可用于面板的高度
        usable_height = max_y - row - 1  # 留 1 行给底部帮助
        panel_height = usable_height // 2

        if panel_height < 3:
            self._safe_addstr(row, 0, "Terminal too small")
            self.stdscr.noutrefresh()
            return

        # ── Incoming 面板 ──
        in_label = "⬇️📥 Incoming" if self.emoji else "Incoming"
        self._draw_panel(
            start_row=row,
            max_x=max_x,
            panel_height=panel_height,
            label=in_label,
            stats=view.engine.incoming,
            history=view.engine.incoming_history,
            is_incoming=True,
        )
        row += panel_height

        # ── Outgoing 面板 ──
        out_label = "⬆️📤 Outgoing" if self.emoji else "Outgoing"
        self._draw_panel(
            start_row=row,
            max_x=max_x,
            panel_height=panel_height,
            label=out_label,
            stats=view.engine.outgoing,
            history=view.engine.outgoing_history,
            is_incoming=False,
        )
        row += panel_height

        # ── 底部帮助行 ──
        if self.emoji:
            help_text = " ⬅️/➡️ Switch Device | 🚪 q Quit"
        else:
            help_text = " ←/→ Switch Device | q Quit"
        help_attr = self._get_bar_attr(self.COLOR_HELP)
        if self.bar_style == "fill":
            help_text = help_text.ljust(max_x - 1)
        self._safe_addstr(
            max_y - 1, 0,
            help_text[:max_x - 1],
            help_attr,
        )

        self.stdscr.noutrefresh()

    def _draw_panel(
        self,
        start_row: int,
        max_x: int,
        panel_height: int,
        label: str,
        stats: TrafficStats,
        history,
        is_incoming: bool = True,
    ) -> None:
        """绘制一个流量面板（图形 + 统计）"""
        # 选择颜色
        graph_color = self.COLOR_IN_GRAPH if is_incoming else self.COLOR_OUT_GRAPH
        label_color = self.COLOR_IN_LABEL if is_incoming else self.COLOR_OUT_LABEL

        # 统计信息（5 行）
        stat_lines = self._format_stats(stats)
        stat_width = max(len(s) for s in stat_lines) + 2 if stat_lines else 20

        # 确定缩放上限
        if self.fixed_max is not None:
            scale_max = self.fixed_max
        else:
            peak = max(history) if history else 0.0
            scale_max = next_power_of_2_scaled(peak)

        # 标签行
        scale_label = get_graph_scale_label_unit(scale_max, self.unit)
        label_text = f"{label} ({scale_label}):"
        label_attr = self._get_bar_attr(label_color, bold=True)
        if self.bar_style == "fill":
            label_text = label_text.ljust(max_x - 1)
        self._safe_addstr(
            start_row, 0, label_text, label_attr,
        )

        graph_rows = panel_height - 1  # 去掉标签行

        if graph_rows < 1:
            return

        if self.no_graph:
            # 不绘制图形，只绘制统计信息（左对齐）
            stat_start_row = start_row + 1 + graph_rows - len(stat_lines)
            stat_col = 2
            for i, s in enumerate(stat_lines):
                r = stat_start_row + i
                if r < start_row + 1:
                    continue
                parts = s.split(": ", 1)
                if len(parts) == 2:
                    lbl_part = parts[0] + ": "
                    val_part = parts[1]
                    self._safe_addstr(
                        r, stat_col, lbl_part,
                        curses.color_pair(self.COLOR_STAT_LABEL) | curses.A_BOLD,
                    )
                    self._safe_addstr(
                        r, stat_col + len(lbl_part), val_part,
                        curses.color_pair(self.COLOR_STAT_VALUE),
                    )
                else:
                    self._safe_addstr(r, stat_col, s, curses.color_pair(self.COLOR_STAT_VALUE))
            return

        # 图形区域尺寸
        graph_cols = max(max_x - stat_width - 2, 10)

        # 渲染图形
        lines = render_graph(
            history=history,
            width=graph_cols,
            height=graph_rows,
            max_value=scale_max,
            unicode=self.unicode,
        )

        # Unicode 和 ASCII 字符映射
        if self.unicode:
            full_chars = {"█", "▓"}
            dim_chars = {"░"}
        else:
            full_chars = {"#", "|"}
            dim_chars = {"."}

        # 绘制图形
        for i, line in enumerate(lines):
            row = start_row + 1 + i
            for col_idx, ch in enumerate(line):
                if ch in full_chars:
                    color = curses.color_pair(graph_color)
                elif ch in dim_chars:
                    color = curses.color_pair(self.COLOR_GRAPH_LOW) | curses.A_DIM
                else:
                    continue  # 空格不画
                self._safe_addch(row, col_idx, ch, color)

        # 绘制统计信息（右侧，底部对齐）
        stat_start_row = start_row + 1 + graph_rows - len(stat_lines)
        stat_col = max_x - stat_width
        for i, s in enumerate(stat_lines):
            r = stat_start_row + i
            if r < start_row + 1:
                continue
            # 标签部分 (Cur: / Avg: / ...)
            parts = s.split(": ", 1)
            if len(parts) == 2:
                lbl_part = parts[0] + ": "
                val_part = parts[1]
                self._safe_addstr(
                    r, stat_col, lbl_part,
                    curses.color_pair(self.COLOR_STAT_LABEL) | curses.A_BOLD,
                )
                self._safe_addstr(
                    r, stat_col + len(lbl_part), val_part,
                    curses.color_pair(self.COLOR_STAT_VALUE),
                )
            else:
                self._safe_addstr(r, stat_col, s, curses.color_pair(self.COLOR_STAT_VALUE))

    def _get_bar_attr(self, color_pair_id: int, bold: bool = False) -> int:
        """根据 bar_style 返回对应的 curses 属性"""
        attr = curses.color_pair(color_pair_id)
        if bold:
            attr |= curses.A_BOLD
        if self.bar_style in ("fill", "color"):
            # fill: 背景色铺满整行 (文字会被 ljust 填充)
            # color: 背景色仅在文字上 (不 ljust)
            attr |= curses.A_REVERSE
        # plain: 无背景色，纯前景着色
        return attr

    def _format_stats(self, stats: TrafficStats) -> List[str]:
        """格式化 5 行统计文本"""
        fmt = lambda v: format_speed_unit(v, self.unit)
        if self.emoji:
            return [
                f"⚡ Curr: {fmt(stats.current)}",
                f"📊  Avg: {fmt(stats.average)}",
                f"📏  Min: {fmt(stats.minimum)}",
                f"🚀  Max: {fmt(stats.maximum)}",
                f"📦  Ttl: {format_bytes(stats.total)}",
            ]
        return [
            f"Curr: {fmt(stats.current)}",
            f" Avg: {fmt(stats.average)}",
            f" Min: {fmt(stats.minimum)}",
            f" Max: {fmt(stats.maximum)}",
            f" Ttl: {format_bytes(stats.total)}",
        ]

    def _draw_too_small(self, max_y: int, max_x: int) -> None:
        msg = "😭 Terminal too small! 📌" if self.emoji else "Terminal too small!"
        y = max_y // 2
        x = max(0, (max_x - len(msg)) // 2)
        self._safe_addstr(
            y, x, msg,
            curses.color_pair(self.COLOR_ERROR) | curses.A_BOLD,
        )

    def _safe_addstr(self, y: int, x: int, text: str, attr: int = 0) -> None:
        """安全写入字符串，忽略边界溢出"""
        max_y, max_x = self.stdscr.getmaxyx()
        if y < 0 or y >= max_y or x < 0 or x >= max_x:
            return
        try:
            self.stdscr.addnstr(y, x, text, max_x - x - 1, attr)
        except curses.error:
            pass

    def _safe_addch(self, y: int, x: int, ch: str, attr: int = 0) -> None:
        max_y, max_x = self.stdscr.getmaxyx()
        if y < 0 or y >= max_y or x < 0 or x >= max_x:
            return
        try:
            self.stdscr.addch(y, x, ch, attr)
        except curses.error:
            pass

    def _is_loopback_on_windows(self, view: DeviceView) -> bool:
        """检测当前是否为 Windows 平台的 Loopback 设备"""
        if sys.platform != "win32":
            return False
        # 设备名包含 "loopback"（同 Rust 逻辑）
        if "loopback" in view.name.lower():
            return True
        # 地址为 127.0.0.1
        if view.info and any(a == "127.0.0.1" for a in view.info.addrs):
            return True
        return False

    def handle_key(self, key: int) -> bool:
        """
        处理按键输入。
        返回 False 表示退出。
        """
        if key in (ord("q"), ord("Q")):
            return False
        elif key == ord("="):
            self.hide_separator = not self.hide_separator
        elif key in (curses.KEY_RIGHT, curses.KEY_DOWN, ord("\t"),
                     curses.KEY_NPAGE, 10):  # 10 = Enter
            self.next_device()
        elif key in (curses.KEY_LEFT, curses.KEY_UP, curses.KEY_PPAGE):
            self.prev_device()
        return True
