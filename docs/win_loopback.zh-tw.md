# Windows Loopback 流量監控：為什麼 ETW 不行，Npcap 可以？

> **[📖 English](win_loopback.md)**
> **[📖 简体中文(大陆)](win_loopback.zh-cn.md)**
> **[📖 繁體中文(台灣)](win_loopback.zh-tw.md)**

## TL;DR

Windows 的 loopback (`127.0.0.1`) 流量在 `tcpip.sys` 內部直接短路，**完全繞過了 NDIS 網路驅動層**，
導致 `GetIfEntry` / `GetIfTable` 返回的計數始終為 0。

這是 **Windows 網絡棧的功能缺失**——loopback 接口從未被賦予一個完整的 NDIS miniport 驅動，
標準的監控 API 對它根本不起作用。

Npcap 通過 WFP (Windows Filtering Platform) callout driver 在短路之前攔截數據包，因此能抓到。

---

## Windows 網絡棧架構

```
應用層  →  Winsock  →  AFD.sys  →  tcpip.sys  →  NDIS  →  網卡驅動  →  硬件
```

### 正常網卡流量路程

```
tcpip.sys  →  NDIS (计数器 +1)  →  miniport driver  →  物理網卡
```

NDIS 驅動在這裡更新 `MIB_IFROW.dwInOctets` / `dwOutOctets`，
所以 `GetIfEntry()` 能拿到正確的值。

### Loopback 的流量路徑

```
tcpip.sys  →  直接環回（短路）  →  接收路徑
               ↑
          NDIS 層被完全跳過
          計數器不會更新
```

當數據發往 `127.0.0.1` 時，`tcpip.sys` 在**傳輸層就直接短路**了——
數據包從發送路徑直接複製到接收路徑，**根本不經過 NDIS 層**。

Windows 的 "Loopback Pseudo-Interface 1" 其實是一個**虛擬佔位符**，
它出現在 `GetIfTable` 的接口列表裡（`dwType = IF_TYPE_SOFTWARE_LOOPBACK = 24`），
但背後沒有真正的 NDIS miniport 驅動在做計數，所以計數器永遠是 0。

**这不是什么"性能優化"，而是一個不完整的實現。** Linux 和 macOS 的 loopback 都走完整的裝置路徑，
額外開銷微乎其微。Microsoft只是從未投入精力把 loopback 接口做成一個真正的 NDIS 裝置，
導致沒有標準 API 能報告 loopback 流量，只能靠第三方工具（Npcap）來填坑。

---

## 為什麼 Npcap 能抓到？

Npcap 註冊了一個 **WFP (Windows Filtering Platform) callout driver**，
在 `tcpip.sys` 做環回**之前**攔截數據包，然後複製一份到用戶態：

```
tcpip.sys 發送路徑
    ↓
  WFP callout (Npcap 在這裡攔截並復製)
    ↓
  短路環回到接收路徑
```

这不是通过這不是通過 NDIS 層實現的，而是在更上層——WFP 是 `tcpip.sys` 內部的 hook 點。

所以 Npcap 安裝時需要選中 **"Support loopback traffic capture"**，
這會啟用 WFP callout 驅動，並創建 `NPF_Loopback` 虛擬適配器供 `pcap` 库使用。

---

## 與 Linux / macOS 的對比

### Linux

Linux 的 `lo` 介面是一個**真正的網路裝置**，有完整的驅動實現（`drivers/net/loopback.c`）：

```
應用層 → socket → TCP/IP 協議棧 → dev_queue_xmit() → lo 驅動 → netif_rx() → 接受路徑
                                        ↑
                                  lo 驅動在這裡正常更新 stats
```

`lo` 驅動的 `loopback_xmit()` 函式會正常調用 `dev->stats` 更新計數器，
所以 `/proc/net/dev` 裡的 `lo` 的 `RX bytes` / `TX bytes` 是完全準確的。

```c
// Linux 內核 loopback_xmit() 簡化版
static netdev_tx_t loopback_xmit(struct sk_buff *skb, struct net_device *dev) {
    len = skb->len;
    dev->stats.tx_bytes += len;   // ← 真的會更新！
    dev->stats.tx_packets++;
    dev->stats.rx_bytes += len;
    dev->stats.rx_packets++;
    netif_rx(skb);                // 送回接收路徑
    return NETDEV_TX_OK;
}
```

### macOS

macOS 的 `lo0` 介面與 Linux 類似，也是一個**真正具備 BPF 能力的網路接口**。
loopback 流量會經過 `if_loop.c` 驅動程式，BPF tap 可以在該處擷取封包，
`netstat -I lo0` 可以看到真實的計數器。

macOS 繼承了 BSD 的設計哲學——loopback 就是一個一般的網路接口，只是沒有實體硬體。

---

## 設計哲學總結

|                     | Windows                              | Linux                          | macOS                      |
| ------------------- | ------------------------------------ | ------------------------------ | -------------------------- |
| Loopback 實作層     | `tcpip.sys` 內部短路                 | `lo` 裝置驅動程式              | `lo0` 裝置驅動程式         |
| 經過網路裝置層？    | ❌ 不經過 NDIS                       | ✅ 完整走 `dev_queue_xmit`     | ✅ 完整走 `if_output`      |
| 計數器正確？        | ❌ 永遠為 0                          | ✅ 正確                        | ✅ 正確                    |
| BPF/pcap 原生抓包？ | ❌ 需要 Npcap WFP callout 補救       | ✅ 原生支援                    | ✅ 原生支援                |
| 設計理念            | 功能缺失，實作不完整                 | 一切皆裝置，統一抽象           | BSD 傳統，一切皆介面       |

**根本原因是 Windows 的 loopback 實作不完整**，並非刻意的設計取捨。
Linux 的 `lo` 驅動程式在 loopback 路徑上只多了幾個函式呼叫——開銷微乎其微——卻換來與所有標準監控與抓包工具的完全相容。
微軟只是沒有把這項工作完成：loopback 虛擬介面有出現在介面清單中，但背後沒有真正的驅動程式支援。

---

## winload 的解決方案

winload 提供兩種 Windows loopback 擷取方式：

- **`--npcap` (建議)**: 透過 Npcap 的 WFP callout 擷取真實的 loopback 封包，資料精準。
  需要安裝 [Npcap](https://npcap.com/#download)，安裝時勾選 "Support loopback traffic capture"。

- **`--etw` (實驗性)**: 透過 `GetIfEntry` API 輪詢計數器。
  由於上述 Windows 核心的功能缺失，多數版本會回傳 0，**基本不可用**。僅為相容保留。

在 Linux / macOS 上，loopback 流量透過 `sysinfo` crate 直接取得，不需額外參數。
