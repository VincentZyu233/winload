//! 基于 ratatui 的 TUI 界面渲染
//! 仿 nload 的双面板布局：上半 Incoming / 下半 Outgoing

use std::collections::VecDeque;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::graph;
use crate::stats::{self, TrafficStats};
use crate::{App, BarStyle, Unit};
#[cfg(target_os = "windows")]
use crate::loopback::LoopbackMode;

/// 主绘制入口
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.height < 10 || area.width < 40 {
        draw_too_small(frame, area, app.emoji);
        return;
    }

    // 判断当前是否为 Windows 平台的 Loopback 设备且未启用捕获
    let show_loopback_warning = {
        #[cfg(target_os = "windows")]
        {
            app.loopback_mode == LoopbackMode::None
                && app.current_view()
                    .map(|v| v.info.name.to_lowercase().contains("loopback"))
                    .unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        { false }
    };

    // 使用 --etw 时在 loopback 设备上显示计数器为 0 的警告
    let show_etw_warning = {
        #[cfg(target_os = "windows")]
        {
            app.loopback_mode == LoopbackMode::Etw
                && app.current_view()
                    .map(|v| v.info.name.to_lowercase().contains("loopback"))
                    .unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        { false }
    };

    // 使用 --npcap 或 --etw 时，在 loopback 设备上显示捕获信息
    let show_loopback_info = app.loopback_info.is_some()
        && app.current_view()
            .map(|v| v.info.name.to_lowercase().contains("loopback"))
            .unwrap_or(false);

    // Calculate base header height:
    // - 1 line for device always
    // - +1 if there are warnings/info
    // - +1 if separator is not hidden
    let mut header_height = 1; // device line
    if show_loopback_warning || show_etw_warning || show_loopback_info {
        header_height += 1; // warning/info line
    }
    if !app.hide_separator {
        header_height += 1; // separator line
    }

    // 主布局: 头部(动态高度) + 内容 + 帮助栏(1行)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height), // Header + (warning/info) + separator
            Constraint::Min(6),               // Content (Incoming + Outgoing)
            Constraint::Length(1),             // Help bar
        ])
        .split(area);

    draw_header(frame, chunks[0], app, show_loopback_warning, show_etw_warning, show_loopback_info);
    draw_panels(frame, chunks[1], app);
    draw_help(frame, chunks[2], app.emoji, app.bar_style);
}

// ─── Header ────────────────────────────────────────────────

/// 将文本用空格填充到指定宽度
fn pad_to_width(text: &str, width: usize) -> String {
    let text_len = text.chars().count();
    if text_len >= width {
        text.to_string()
    } else {
        format!("{}{}", text, " ".repeat(width - text_len))
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, show_loopback_warning: bool, show_etw_warning: bool, show_loopback_info: bool) {
    if let Some(view) = app.current_view() {
        let addr_str = if !view.info.addrs.is_empty() {
            format!(" [{}]", view.info.addrs[0])
        } else {
            String::new()
        };

        let is_loopback = view.info.name.to_lowercase().contains("loopback");

        // 在 loopback 设备上追加捕获模式标记
        let mode_tag = if is_loopback {
            #[cfg(target_os = "windows")]
            {
                match app.loopback_mode {
                    LoopbackMode::Npcap => " [npcap]",
                    LoopbackMode::Etw => " [etw]",
                    LoopbackMode::None => "",
                }
            }
            #[cfg(not(target_os = "windows"))]
            { "" }
        } else {
            ""
        };

        let header_text = if app.emoji {
            format!(
                "🖧 Device {}{} ({}/{}){} 📡:",
                view.info.name,
                addr_str,
                app.current_idx + 1,
                app.views.len(),
                mode_tag,
            )
        } else {
            format!(
                "Device {}{} ({}/{}){}:",
                view.info.name,
                addr_str,
                app.current_idx + 1,
                app.views.len(),
                mode_tag,
            )
        };

        let width = area.width as usize;

        let header_style = match app.bar_style {
            BarStyle::Fill => Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
            BarStyle::Color => Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
            BarStyle::Plain => Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        };

        let header_display = if app.bar_style == BarStyle::Fill {
            pad_to_width(&header_text, width)
        } else {
            header_text
        };

        let header = Line::from(Span::styled(header_display, header_style));

        let mut lines = vec![header];
        
        if show_loopback_warning {
            let warn_text = " \u{26a0} Loopback: use --npcap (npcap.com) or --etw";
            let warn_style = match app.bar_style {
                BarStyle::Fill => Style::default().bg(Color::Red).fg(Color::White),
                BarStyle::Color => Style::default().bg(Color::Red).fg(Color::White),
                BarStyle::Plain => Style::default().fg(Color::Yellow),
            };
            let warn_display = if app.bar_style == BarStyle::Fill {
                pad_to_width(warn_text, width)
            } else {
                warn_text.to_string()
            };
            lines.push(Line::from(Span::styled(warn_display, warn_style)));
        }

        if show_etw_warning {
            let etw_text = if app.emoji {
                " ⚠️ ETW: 大多数 Windows loopback 计数器为 0，建议使用 --npcap (npcap.com)"
            } else {
                " \u{26a0} ETW: loopback counters are 0 on most Windows, try --npcap (npcap.com)"
            };
            let etw_style = match app.bar_style {
                BarStyle::Fill => Style::default().bg(Color::Yellow).fg(Color::Black),
                BarStyle::Color => Style::default().bg(Color::Yellow).fg(Color::Black),
                BarStyle::Plain => Style::default().fg(Color::Yellow),
            };
            let etw_display = if app.bar_style == BarStyle::Fill {
                pad_to_width(etw_text, width)
            } else {
                etw_text.to_string()
            };
            lines.push(Line::from(Span::styled(etw_display, etw_style)));
        }

        if show_loopback_info {
            if let Some(ref info) = app.loopback_info {
                let info_text = format!(" {info}");
                let info_style = match app.bar_style {
                    BarStyle::Fill => Style::default().bg(Color::Green).fg(Color::Black),
                    BarStyle::Color => Style::default().bg(Color::Green).fg(Color::Black),
                    BarStyle::Plain => Style::default().fg(Color::Green),
                };
                let info_display = if app.bar_style == BarStyle::Fill {
                    pad_to_width(&info_text, width)
                } else {
                    info_text
                };
                lines.push(Line::from(Span::styled(info_display, info_style)));
            }
        }

        // Add separator line as part of lines if not hidden
        if !app.hide_separator {
            let sep_width = (area.width as usize).min(120);
            let separator = Line::from(Span::styled(
                "=".repeat(sep_width),
                Style::default().fg(Color::Cyan),
            ));
            lines.push(separator);
        }

        let text_height = lines.len() as u16;
        frame.render_widget(
            Paragraph::new(lines),
            Rect {
                height: text_height,
                ..area
            },
        );
    }
}

// ─── Panels ────────────────────────────────────────────────

fn draw_panels(frame: &mut Frame, area: Rect, app: &App) {
    let panels = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    if let Some(view) = app.current_view() {
        let (in_label, out_label) = if app.emoji {
            ("⬇️📥 Incoming", "⬆️📤 Outgoing")
        } else {
            ("Incoming", "Outgoing")
        };
        draw_traffic_panel(
            frame,
            panels[0],
            in_label,
            &view.engine.incoming,
            &view.engine.incoming_history,
            app.emoji,
            app.unicode,
            app.unit,
            app.bar_style,
            app.in_color,
            app.fixed_max,
            app.no_graph,
        );
        draw_traffic_panel(
            frame,
            panels[1],
            out_label,
            &view.engine.outgoing,
            &view.engine.outgoing_history,
            app.emoji,
            app.unicode,
            app.unit,
            app.bar_style,
            app.out_color,
            app.fixed_max,
            app.no_graph,
        );
    }
}

fn draw_traffic_panel(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    stats: &TrafficStats,
    history: &VecDeque<f64>,
    emoji: bool,
    unicode: bool,
    unit: Unit,
    bar_style: BarStyle,
    graph_color: Color,
    fixed_max: Option<f64>,
    no_graph: bool,
) {
    if area.height < 2 || area.width < 20 {
        return;
    }

    // 面板内布局: 标签行(1) + 内容区
    let panel_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    // ── 标签行 ──
    let peak = history.iter().cloned().fold(0.0_f64, f64::max);
    let scale_max = if let Some(m) = fixed_max {
        m
    } else {
        graph::next_power_of_2_scaled(peak)
    };
    let scale_label = graph::get_graph_scale_label_unit(scale_max, unit);
    let label_text = format!("{label} ({scale_label}):");
    let width = area.width as usize;

    let label_style = match bar_style {
        BarStyle::Fill => Style::default()
            .bg(graph_color)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
        BarStyle::Color => Style::default()
            .bg(graph_color)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
        BarStyle::Plain => Style::default()
            .fg(graph_color)
            .add_modifier(Modifier::BOLD),
    };
    let label_display = if bar_style == BarStyle::Fill {
        pad_to_width(&label_text, width)
    } else {
        label_text
    };
    let label_line = Line::from(Span::styled(label_display, label_style));
    frame.render_widget(Paragraph::new(vec![label_line]), panel_chunks[0]);

    if no_graph {
        // ── 无图模式: 统计信息占满宽度 ──
        draw_stats(frame, panel_chunks[1], stats, emoji, unit);
    } else {
        // ── 内容区: 左侧图形 + 右侧统计 ──
        let stat_width: u16 = if emoji { 28 } else { 24 };
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(stat_width)])
            .split(panel_chunks[1]);

        draw_graph(frame, content_chunks[0], history, scale_max, unicode, graph_color);
        draw_stats(frame, content_chunks[1], stats, emoji, unit);
    }
}

// ─── Graph ─────────────────────────────────────────────────

fn draw_graph(frame: &mut Frame, area: Rect, history: &VecDeque<f64>, max_value: f64, unicode: bool, graph_color: Color) {
    let width = area.width as usize;
    let height = area.height as usize;

    let lines = graph::render_graph(history, width, height, max_value, unicode);

    // 较暗的颜色用于低密度区域
    let dim_color = Color::DarkGray;

    let styled_lines: Vec<Line> = lines
        .iter()
        .map(|line| {
            let spans: Vec<Span> = line
                .chars()
                .map(|ch| match ch {
                    // Unicode block chars
                    '█' => Span::styled("█", Style::default().fg(graph_color)),
                    '▓' => Span::styled("▓", Style::default().fg(graph_color)),
                    '░' => Span::styled("░", Style::default().fg(dim_color)),
                    '·' => Span::styled("·", Style::default().fg(dim_color)),
                    // ASCII chars
                    '#' => Span::styled("#", Style::default().fg(graph_color)),
                    '|' => Span::styled("|", Style::default().fg(graph_color)),
                    '.' => Span::styled(".", Style::default().fg(dim_color)),
                    _ => Span::raw(" "),
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(styled_lines), area);
}

// ─── Stats ─────────────────────────────────────────────────

fn draw_stats(frame: &mut Frame, area: Rect, stats: &TrafficStats, emoji: bool, unit: Unit) {
    let stat_lines = format_stats_lines(stats, emoji, unit);
    let stat_count = stat_lines.len() as u16;

    // 底部对齐
    if area.height >= stat_count {
        let inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(stat_count)])
            .split(area);
        frame.render_widget(Paragraph::new(stat_lines), inner[1]);
    } else {
        frame.render_widget(Paragraph::new(stat_lines), area);
    }
}

fn format_stats_lines(st: &TrafficStats, emoji: bool, unit: Unit) -> Vec<Line<'static>> {
    let label_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::White);

    if emoji {
        vec![
            Line::from(vec![
                Span::styled("⚡ Curr: ", label_style),
                Span::styled(stats::format_speed_unit(st.current, unit), value_style),
            ]),
            Line::from(vec![
                Span::styled("📊  Avg: ", label_style),
                Span::styled(stats::format_speed_unit(st.average, unit), value_style),
            ]),
            Line::from(vec![
                Span::styled("📏  Min: ", label_style),
                Span::styled(stats::format_speed_unit(st.minimum, unit), value_style),
            ]),
            Line::from(vec![
                Span::styled("🚀  Max: ", label_style),
                Span::styled(stats::format_speed_unit(st.maximum, unit), value_style),
            ]),
            Line::from(vec![
                Span::styled("📦  Ttl: ", label_style),
                Span::styled(stats::format_bytes(st.total), value_style),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Curr: ", label_style),
                Span::styled(stats::format_speed_unit(st.current, unit), value_style),
            ]),
            Line::from(vec![
                Span::styled(" Avg: ", label_style),
                Span::styled(stats::format_speed_unit(st.average, unit), value_style),
            ]),
            Line::from(vec![
                Span::styled(" Min: ", label_style),
                Span::styled(stats::format_speed_unit(st.minimum, unit), value_style),
            ]),
            Line::from(vec![
                Span::styled(" Max: ", label_style),
                Span::styled(stats::format_speed_unit(st.maximum, unit), value_style),
            ]),
            Line::from(vec![
                Span::styled(" Ttl: ", label_style),
                Span::styled(stats::format_bytes(st.total), value_style),
            ]),
        ]
    }
}

// ─── Help / Error ──────────────────────────────────────────

fn draw_help(frame: &mut Frame, area: Rect, emoji: bool, bar_style: BarStyle) {
    let help_text = if emoji {
        #[cfg(target_os = "windows")]
        { " ⬅️/➡️ Switch Device | 🚪 q Quit | 💡 Loopback: --npcap" }
        #[cfg(not(target_os = "windows"))]
        { " ⬅️/➡️ Switch Device | 🚪 q Quit" }
    } else {
        #[cfg(target_os = "windows")]
        { " \u{2190}/\u{2192} Switch Device | q Quit | Loopback: --npcap" }
        #[cfg(not(target_os = "windows"))]
        { " \u{2190}/\u{2192} Switch Device | q Quit" }
    };

    let width = area.width as usize;

    let help_style = match bar_style {
        BarStyle::Fill => Style::default()
            .bg(Color::White)
            .fg(Color::Black),
        BarStyle::Color => Style::default()
            .bg(Color::White)
            .fg(Color::Black),
        BarStyle::Plain => Style::default()
            .fg(Color::Yellow),
    };
    let help_display = if bar_style == BarStyle::Fill {
        pad_to_width(help_text, width)
    } else {
        help_text.to_string()
    };
    let help = Line::from(Span::styled(help_display, help_style));
    frame.render_widget(Paragraph::new(vec![help]), area);
}

fn draw_too_small(frame: &mut Frame, area: Rect, emoji: bool) {
    let msg = if emoji {
        "😭 Terminal too small! 📌"
    } else {
        "Terminal too small!"
    };
    let x = area.width.saturating_sub(msg.len() as u16) / 2;
    let y = area.height / 2;
    let line = Line::from(Span::styled(
        msg,
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(
        Paragraph::new(vec![line]),
        Rect {
            x: area.x + x,
            y: area.y + y,
            width: msg.len() as u16,
            height: 1,
        },
    );
}
