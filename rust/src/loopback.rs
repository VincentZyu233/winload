//! Windows 本地回环流量捕获模块
//!
//! 提供两种后端:
//! - Npcap: 通过 pcap crate 捕获 \Device\NPF_Loopback 上的数据包
//! - ETW:   通过 Event Tracing for Windows 获取网络事件 (实验性)
//!
//! 此模块仅在 Windows 平台编译。非 Windows 平台下提供空实现。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Npcap 下载地址 (仅 Windows)
#[cfg(target_os = "windows")]
pub const NPCAP_URL: &str = "https://npcap.com/#download";

/// 回环流量计数器 (线程安全，可在采集线程和主线程之间共享)
#[derive(Clone)]
pub struct LoopbackCounters {
    pub bytes_recv: Arc<AtomicU64>,
    pub bytes_sent: Arc<AtomicU64>,
}

impl LoopbackCounters {
    pub fn new() -> Self {
        Self {
            bytes_recv: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_recv(&self) -> u64 {
        self.bytes_recv.load(Ordering::Relaxed)
    }

    pub fn get_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }
}

/// 回环捕获模式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopbackMode {
    /// 不捕获回环流量 (默认)
    None,
    /// 使用 Npcap (pcap) 捕获
    Npcap,
    /// 使用 ETW 捕获 (实验性)
    Etw,
}

// ═══════════════════════════════════════════════════════════
//  Windows 实现
// ═══════════════════════════════════════════════════════════

#[cfg(target_os = "windows")]
pub mod platform {
    use super::*;
    use std::thread;

    /// 启动 Npcap 回环捕获线程
    ///
    /// 返回 Ok(info_msg) 成功时，后台线程会持续累加计数器。
    /// 返回 Err(msg) 如果 Npcap 不可用或打开设备失败。
    #[cfg(feature = "npcap")]
    pub fn start_npcap(counters: LoopbackCounters) -> Result<String, String> {
        // 尝试查找 Npcap Loopback 适配器
        let devices = pcap::Device::list().map_err(|e| {
            format!(
                "Failed to list pcap devices: {e}\n\n\
                 Npcap is not installed or not working.\n\
                 Please install Npcap from: {NPCAP_URL}\n\
                 (Enable 'Support loopback traffic capture' during installation)\n\n\
                 Or try running without --npcap flag, or use --etw instead."
            )
        })?;

        // Npcap 的回环设备通常包含 "Loopback" 或 "NPF_Loopback"
        let loopback_dev = devices
            .iter()
            .find(|d| {
                let name_lower = d.name.to_lowercase();
                let desc_lower = d.desc.as_deref().unwrap_or("").to_lowercase();
                name_lower.contains("loopback")
                    || name_lower.contains("npf_loopback")
                    || desc_lower.contains("adapter for loopback traffic capture")
                    || desc_lower.contains("npcap loopback")
            })
            .ok_or_else(|| {
                let available: Vec<String> = devices
                    .iter()
                    .map(|d| {
                        format!(
                            "  {} ({})",
                            d.name,
                            d.desc.as_deref().unwrap_or("no description")
                        )
                    })
                    .collect();
                format!(
                    "Npcap loopback adapter not found.\n\n\
                     Make sure Npcap is installed with 'Support loopback traffic' enabled.\n\
                     Download Npcap: {NPCAP_URL}\n\n\
                     Or try running without --npcap flag, or use --etw instead.\n\n\
                     Available devices:\n{}",
                    available.join("\n")
                )
            })?;

        let dev_name = loopback_dev.name.clone();
        let info_msg = format!("[npcap] Found loopback device: {dev_name}");

        // 在后台线程中持续捕获
        thread::Builder::new()
            .name("npcap-loopback".to_string())
            .spawn(move || {
                if let Err(e) = npcap_capture_loop(&dev_name, &counters) {
                    eprintln!("[npcap] Capture error: {e}");
                }
            })
            .map_err(|e| format!("Failed to spawn npcap thread: {e}"))?;

        Ok(info_msg)
    }

    #[cfg(feature = "npcap")]
    fn npcap_capture_loop(device_name: &str, counters: &LoopbackCounters) -> Result<(), String> {
        let mut cap = pcap::Capture::from_device(device_name)
            .map_err(|e| format!("Cannot open device: {e}"))?
            .promisc(false)
            .snaplen(96) // 只需要 IP 头来获取包长度
            .timeout(100) // 100ms 超时，避免阻塞
            .open()
            .map_err(|e| format!("Cannot start capture: {e}"))?;

        loop {
            match cap.next_packet() {
                Ok(packet) => {
                    // Npcap loopback 使用 DLT_NULL 格式:
                    // 前 4 字节是地址族 (AF_INET=2, AF_INET6=23/30)
                    // 之后是完整的 IP 包
                    let data = packet.data;
                    if data.len() < 4 {
                        continue;
                    }

                    // 获取 IP 包长度 (跳过 4 字节的 DLT_NULL 头)
                    let ip_payload = &data[4..];
                    let pkt_len = ip_payload.len() as u64;

                    if ip_payload.is_empty() {
                        continue;
                    }

                    // 判断是 incoming 还是 outgoing:
                    // 对于 loopback，发送和接收的包都会被捕获。
                    // 简单方法: 所有包同时计入 recv 和 sent (因为回环流量
                    // 是自己发给自己的，收发对等)
                    counters.bytes_recv.fetch_add(pkt_len, Ordering::Relaxed);
                    counters.bytes_sent.fetch_add(pkt_len, Ordering::Relaxed);
                }
                Err(pcap::Error::TimeoutExpired) => {
                    // 正常超时，继续循环
                    continue;
                }
                Err(e) => {
                    return Err(format!("Packet capture error: {e}"));
                }
            }
        }
    }

    #[cfg(not(feature = "npcap"))]
    pub fn start_npcap(_counters: LoopbackCounters) -> Result<String, String> {
        Err(format!(
            "winload was compiled without Npcap support (feature 'npcap' disabled).\n\
             Recompile with: cargo build --features npcap\n\n\
             Or download a pre-built release that includes Npcap support.\n\
             Npcap download: {NPCAP_URL}"
        ))
    }

    /// 启动 ETW 回环捕获
    ///
    /// 实验性功能 — 使用 Windows GetIfEntry API 定时轮询 Loopback 接口统计。
    /// 注意: 标准 API 对 loopback 的计数器可能始终为 0，
    /// 但某些 Windows 版本/补丁下可能有效。
    #[cfg(feature = "etw")]
    pub fn start_etw(counters: LoopbackCounters) -> Result<String, String> {
        // 查找 loopback 接口的 dwIndex
        let loopback_idx = find_loopback_interface_index()?;
        let info_msg = format!("[etw] Found loopback interface index: {loopback_idx} (experimental, counters may be 0)");

        thread::Builder::new()
            .name("etw-loopback".to_string())
            .spawn(move || {
                etw_poll_loop(loopback_idx, &counters);
            })
            .map_err(|e| format!("Failed to spawn ETW thread: {e}"))?;

        Ok(info_msg)
    }

    /// 通过 GetIfTable 遍历所有接口，找到 dwType == 24 (SOFTWARE_LOOPBACK) 的索引
    #[cfg(feature = "etw")]
    fn find_loopback_interface_index() -> Result<u32, String> {
        use windows_sys::Win32::NetworkManagement::IpHelper::{GetIfTable, MIB_IFTABLE};

        unsafe {
            // 第一次调用获取所需 buffer 大小
            let mut size: u32 = 0;
            GetIfTable(std::ptr::null_mut(), &mut size, 0);

            if size == 0 {
                return Err("GetIfTable returned size 0".to_string());
            }

            // 分配 buffer 并第二次调用
            let mut buf: Vec<u8> = vec![0u8; size as usize];
            let table_ptr = buf.as_mut_ptr() as *mut MIB_IFTABLE;
            let ret = GetIfTable(table_ptr, &mut size, 0);
            if ret != 0 {
                return Err(format!("GetIfTable failed with error code: {ret}"));
            }

            let num = (*table_ptr).dwNumEntries as usize;
            // table 字段是 [MIB_IFROW; 1]，实际是变长数组
            let entries = std::slice::from_raw_parts((*table_ptr).table.as_ptr(), num);

            for entry in entries {
                // IF_TYPE_SOFTWARE_LOOPBACK = 24
                if entry.dwType == 24 {
                    return Ok(entry.dwIndex);
                }
            }

            Err("No loopback interface found via GetIfTable".to_string())
        }
    }

    /// 定时轮询 loopback 接口的 dwInOctets / dwOutOctets
    #[cfg(feature = "etw")]
    fn etw_poll_loop(if_index: u32, counters: &LoopbackCounters) {
        use windows_sys::Win32::NetworkManagement::IpHelper::{GetIfEntry, MIB_IFROW};

        let poll_interval = std::time::Duration::from_millis(200);

        loop {
            unsafe {
                let mut row: MIB_IFROW = std::mem::zeroed();
                row.dwIndex = if_index;

                let ret = GetIfEntry(&mut row);
                if ret == 0 {
                    // dwInOctets / dwOutOctets 是 u32 累计值
                    // 对于 loopback，大多数 Windows 版本可能报告 0
                    counters.bytes_recv.store(row.dwInOctets as u64, Ordering::Relaxed);
                    counters.bytes_sent.store(row.dwOutOctets as u64, Ordering::Relaxed);
                }
            }

            thread::sleep(poll_interval);
        }
    }

    #[cfg(not(feature = "etw"))]
    pub fn start_etw(_counters: LoopbackCounters) -> Result<String, String> {
        Err("winload was compiled without ETW support (feature 'etw' disabled).\n\
             Recompile with: cargo build --features etw"
            .to_string())
    }
}

// ═══════════════════════════════════════════════════════════
//  非 Windows 平台 — 空实现 (Linux/macOS 不需要特殊处理)
// ═══════════════════════════════════════════════════════════

#[cfg(not(target_os = "windows"))]
pub mod platform {
    use super::*;

    pub fn start_npcap(_counters: LoopbackCounters) -> Result<String, String> {
        Err("--npcap is only supported on Windows. \
             On Linux/macOS, loopback traffic is natively available."
            .to_string())
    }

    pub fn start_etw(_counters: LoopbackCounters) -> Result<String, String> {
        Err("--etw is only supported on Windows. \
             On Linux/macOS, loopback traffic is natively available."
            .to_string())
    }
}
