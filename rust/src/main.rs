//! winload — Network Load Monitor
//! 仿 Linux nload 的终端网络流量监控工具 (Rust 版)
//!
//! 用法:
//!     winload              # 监控所有活跃网卡
//!     winload -t 200       # 设置刷新间隔 200ms
//!     winload -d "Wi-Fi"   # 指定默认设备
//!
//! 快捷键:
//!     ←/→ 或 ↑/↓   切换网卡
//!     q / Esc       退出

mod collector;
mod graph;
mod i18n;
mod loopback;
mod stats;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use clap::{CommandFactory, FromArgMatches, Parser};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use i18n::{Lang, t, set_lang};

use collector::{Collector, DeviceInfo};
use loopback::{LoopbackCounters, LoopbackMode};
use stats::StatisticsEngine;

// ─── 单位枚举 ─────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Unit {
    /// Display rates in Bit/s (default)
    Bit,
    /// Display rates in Byte/s
    Byte,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum BarStyle {
    /// Background color fills entire line (default)
    Fill,
    /// Background color only on text
    Color,
    /// No background, text color only
    Plain,
}

/// 解析人类可读的流量值，如 "100M" → 100*1024*1024 bytes/s
pub fn parse_max_value(s: &str) -> Result<f64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty value".to_string());
    }
    let (num_str, multiplier) = if let Some(n) = s.strip_suffix('G').or_else(|| s.strip_suffix('g')) {
        (n, 1024.0 * 1024.0 * 1024.0)
    } else if let Some(n) = s.strip_suffix('M').or_else(|| s.strip_suffix('m')) {
        (n, 1024.0 * 1024.0)
    } else if let Some(n) = s.strip_suffix('K').or_else(|| s.strip_suffix('k')) {
        (n, 1024.0)
    } else {
        (s, 1.0)
    };
    let num: f64 = num_str.parse().map_err(|e| format!("invalid number: {e}"))?;
    Ok(num * multiplier)
}

/// 解析十六进制颜色码，支持 0xRRGGBB 或 RRGGBB 格式
pub fn parse_hex_color(s: &str) -> Result<ratatui::style::Color, String> {
    let hex = s.trim().strip_prefix("0x").or_else(|| s.trim().strip_prefix("0X")).unwrap_or(s.trim());
    if hex.len() != 6 {
        return Err(format!("expected 6 hex digits (e.g. 0x3399ff), got: {s}"));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| format!("bad red: {e}"))?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| format!("bad green: {e}"))?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| format!("bad blue: {e}"))?;
    Ok(ratatui::style::Color::Rgb(r, g, b))
}

// ─── CLI 参数 ──────────────────────────────────────────────

/// Network Load Monitor — nload-like TUI tool for Windows/Linux/macOS
#[derive(Parser)]
#[command(name = "winload", version = concat!(env!("CARGO_PKG_VERSION"), " (Rust edition)"))]
struct Args {
    /// Refresh interval in milliseconds
    #[arg(short = 't', long = "interval", default_value = "500")]
    interval: u64,

    /// Average window in seconds
    #[arg(short = 'a', long = "average", default_value = "300")]
    average: u64,

    /// Default device name (partial match)
    #[arg(short = 'd', long = "device")]
    device: Option<String>,

    /// Print debug info about network interfaces and exit
    #[arg(long = "debug-info")]
    debug_info: bool,

    /// Enable emoji decorations in TUI and output
    #[arg(short = 'e', long = "emoji")]
    emoji: bool,

    /// Use Unicode block characters for graph
    #[arg(short = 'U', long = "unicode")]
    unicode: bool,

    /// Display unit: bit (default) or byte
    #[arg(short = 'u', long = "unit", value_enum, default_value = "bit")]
    unit: Unit,

    /// Bar style for header/label/help
    #[arg(short = 'b', long = "bar-style", value_enum, default_value = "fill")]
    bar_style: BarStyle,

    /// Incoming (download) graph color, hex RGB
    #[arg(long = "in-color", value_parser = parse_hex_color)]
    in_color: Option<ratatui::style::Color>,

    /// Outgoing (upload) graph color, hex RGB
    #[arg(long = "out-color", value_parser = parse_hex_color)]
    out_color: Option<ratatui::style::Color>,

    /// Fixed graph Y-axis max (e.g. 100M, 1G, 500K)
    #[arg(short = 'm', long = "max", value_parser = parse_max_value)]
    max: Option<f64>,

    /// Hide traffic graphs, show only statistics
    #[arg(short = 'n', long = "no-graph")]
    no_graph: bool,

    /// Hide separator line
    #[arg(long = "hide-separator")]
    hide_separator: bool,

    /// Disable all TUI colors (monochrome mode)
    #[arg(long = "no-color")]
    no_color: bool,

    /// Use Npcap to capture loopback traffic [Windows only]
    #[arg(long = "npcap")]
    npcap: bool,

    /// Display language
    #[arg(long = "lang", value_enum, default_value = "en-us")]
    lang: Lang,
}

// ─── App 状态 ──────────────────────────────────────────────

/// 单个网卡的视图状态
pub struct DeviceView {
    pub info: DeviceInfo,
    pub engine: StatisticsEngine,
}

/// 应用主状态
pub struct App {
    pub views: Vec<DeviceView>,
    pub current_idx: usize,
    pub emoji: bool,
    pub unicode: bool,
    pub unit: Unit,
    pub bar_style: BarStyle,
    pub in_color: ratatui::style::Color,
    pub out_color: ratatui::style::Color,
    pub fixed_max: Option<f64>,
    pub no_graph: bool,
    pub hide_separator: bool,
    pub no_color: bool,
    pub loopback_mode: LoopbackMode,
    pub loopback_info: Option<String>,
    loopback_counters: Option<LoopbackCounters>,
    collector: Collector,
}

impl App {
    fn new(args: &Args) -> Self {
        let collector = Collector::new();
        let devices = collector.devices();

        let views: Vec<DeviceView> = devices
            .into_iter()
            .map(|info| DeviceView {
                info,
                engine: StatisticsEngine::new(args.interval, args.average),
            })
            .collect();

        // 如果指定了默认设备，定位到对应索引
        let mut current_idx = 0;
        if let Some(ref name) = args.device {
            let lower = name.to_lowercase();
            if let Some(idx) = views
                .iter()
                .position(|v| v.info.name.to_lowercase().contains(&lower))
            {
                current_idx = idx;
            }
        }

        let loopback_mode = if args.npcap {
            LoopbackMode::Npcap
        } else {
            LoopbackMode::None
        };

        Self {
            views,
            current_idx,
            emoji: args.emoji,
            unicode: args.unicode,
            unit: args.unit,
            bar_style: args.bar_style,
            in_color: args.in_color.unwrap_or(ratatui::style::Color::Rgb(0x00, 0xd7, 0xff)),
            out_color: args.out_color.unwrap_or(ratatui::style::Color::Rgb(0xff, 0xaf, 0x00)),
            fixed_max: args.max,
            no_graph: args.no_graph,
            hide_separator: args.hide_separator,
            no_color: args.no_color,
            loopback_mode,
            loopback_info: None,
            loopback_counters: None,
            collector,
        }
    }

    pub fn current_view(&self) -> Option<&DeviceView> {
        self.views.get(self.current_idx)
    }

    fn update(&mut self) {
        let mut snapshots = self.collector.collect();

        // 如果启用了回环捕获，用实时计数器覆盖 loopback 的假数据
        if let Some(ref counters) = self.loopback_counters {
            let elapsed = self.collector.elapsed_secs();
            for (name, snap) in snapshots.iter_mut() {
                if name.to_lowercase().contains("loopback") {
                    snap.bytes_recv = counters.get_recv();
                    snap.bytes_sent = counters.get_sent();
                    snap.elapsed_secs = elapsed;
                }
            }
        }

        for view in &mut self.views {
            if let Some(snap) = snapshots.get(&view.info.name) {
                view.engine.update(snap.clone());
            }
        }
    }

    fn next_device(&mut self) {
        if !self.views.is_empty() {
            self.current_idx = (self.current_idx + 1) % self.views.len();
        }
    }

    fn prev_device(&mut self) {
        if !self.views.is_empty() {
            self.current_idx = (self.current_idx + self.views.len() - 1) % self.views.len();
        }
    }
}

// ─── 主循环 ────────────────────────────────────────────────

fn run(terminal: &mut ratatui::DefaultTerminal, args: Args) -> io::Result<()> {
    let mut app = App::new(&args);

    // 启动回环捕获 (如果指定了 --npcap 或 --etw)
    if app.loopback_mode != LoopbackMode::None {
        let counters = LoopbackCounters::new();
        let result = match app.loopback_mode {
            LoopbackMode::Npcap => loopback::platform::start_npcap(counters.clone()),
            LoopbackMode::None => unreachable!(),
        };
        match result {
            Ok(info_msg) => {
                app.loopback_info = Some(info_msg);
                app.loopback_counters = Some(counters);
            }
            Err(e) => {
                // 恢复终端后打印错误
                ratatui::restore();
                eprintln!("Error: Failed to start loopback capture:\n{e}");
                std::process::exit(1);
            }
        }
    }

    let tick_rate = Duration::from_millis(args.interval);
    let mut last_tick = Instant::now();

    // 初始采集
    app.update();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Windows 下 crossterm 会产生 Press + Release，只处理 Press
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char('c')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            return Ok(());
                        }
                        KeyCode::Char('=') => {
                            app.hide_separator = !app.hide_separator;
                        }
                        KeyCode::Char('c') => {
                            app.no_color = !app.no_color;
                        }
                        KeyCode::Right | KeyCode::Down | KeyCode::Tab | KeyCode::Enter => {
                            app.next_device();
                        }
                        KeyCode::Left | KeyCode::Up => {
                            app.prev_device();
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.update();
            last_tick = Instant::now();
        }
    }
}

// ─── 入口 ──────────────────────────────────────────────────

fn print_system_info() {
    eprintln!("\nSystem: {} | Arch: {} | Target: {}", 
        std::env::consts::OS, 
        std::env::consts::ARCH,
        env!("TARGET"));
}

fn pre_scan_lang() -> Lang {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--lang" {
            if let Some(val) = args.get(i + 1) {
                return match val.as_str() {
                    "zh-cn" => Lang::ZhCn,
                    "zh-tw" => Lang::ZhTw,
                    _ => Lang::EnUs,
                };
            }
        }
        if let Some(rest) = args[i].strip_prefix("--lang=") {
            return match rest {
                "zh-cn" => Lang::ZhCn,
                "zh-tw" => Lang::ZhTw,
                _ => Lang::EnUs,
            };
        }
    }
    Lang::EnUs
}

fn build_translated_command() -> clap::Command {
    let after_help = format!("\nSystem: {} | Arch: {} | Target: {}", 
        std::env::consts::OS, 
        std::env::consts::ARCH,
        env!("TARGET"));

    Args::command()
        .about(t("description"))
        .after_help(after_help)
        .mut_arg("interval", |a| a.help(t("help_interval")))
        .mut_arg("average", |a| a.help(t("help_average")))
        .mut_arg("device", |a| a.help(t("help_device")))
        .mut_arg("debug_info", |a| a.help(t("help_debug_info")))
        .mut_arg("emoji", |a| a.help(t("help_emoji")))
        .mut_arg("unicode", |a| a.help(t("help_unicode")))
        .mut_arg("unit", |a| a.help(t("help_unit")))
        .mut_arg("bar_style", |a| a.help(t("help_bar_style")))
        .mut_arg("in_color", |a| a.help(t("help_in_color")))
        .mut_arg("out_color", |a| a.help(t("help_out_color")))
        .mut_arg("max", |a| a.help(t("help_max")))
        .mut_arg("no_graph", |a| a.help(t("help_no_graph")))
        .mut_arg("hide_separator", |a| a.help(t("help_hide_separator")))
        .mut_arg("no_color", |a| a.help(t("help_no_color")))
        .mut_arg("npcap", |a| a.help(t("help_npcap")))
        .mut_arg("lang", |a| a.help(t("help_lang")))
}

fn main() -> io::Result<()> {
    // Pre-scan --lang before clap parses (for translated --help)
    let lang = pre_scan_lang();
    set_lang(lang);

    let cmd = build_translated_command();
    let matches = cmd.get_matches();
    let args = Args::from_arg_matches(&matches)
        .unwrap_or_else(|e| e.exit());

    // 如果传入 --debug-info，打印接口信息后退出
    if args.debug_info {
        let collector = Collector::new();
        if args.emoji {
            println!("\n🔍🌐 Network Interfaces Debug Info 🖧✨");
        }
        collector.print_debug_info();
        if args.emoji {
            println!("🏁 Done! Happy debugging! 🎉🐛");
        }
        return Ok(());
    }
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, args);
    ratatui::restore();
    print_system_info();
    result
}
