# portt

A ping-like tool for TCP/UDP ports. Tests port reachability and measures latency.

## Features

- **TCP** port probing with connect-timeout latency measurement
- **UDP** port probing with response/timeout detection
- Customizable timeout, interval, and probe count
- Real-time per-probe latency output
- Summary statistics (min/avg/max RTT, loss rate)
- Graceful Ctrl-C handling with summary display

## Usage

```
portt <HOST> <PORT> [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `-p, --protocol` | `tcp` | Protocol: `tcp` or `udp` |
| `-t, --timeout` | `1` | Timeout per probe (seconds) |
| `-i, --interval` | `1` | Interval between probes (seconds) |
| `-c, --count` | `4` | Number of probes (`0` = infinite) |
| `-h, --help` | | Print help |
| `-V, --version` | | Print version |

### Examples

```bash
# Test TCP port 22, 3 probes, 0.2s interval
portt 192.168.1.1 22 -c 3 -i 0.2

# Test UDP DNS port 53, 0.5s timeout
portt 8.8.8.8 53 -p udp -t 0.5

# Infinite mode (Ctrl-C to stop with summary)
portt 10.0.0.1 80 -c 0 -i 2

# All defaults: TCP, 4 probes, 1s timeout, 1s interval
portt example.com 80
```

### Output Example

```
portt 192.168.1.1:22 using TCP protocol, timeout 1.0s, interval 1.0s

from 192.168.1.1:22: seq=1 time=1.234 ms
from 192.168.1.1:22: seq=2 time=1.156 ms
from 192.168.1.1:22: seq=3 time=1.201 ms
from 192.168.1.1:22: seq=4 time=1.189 ms

--- 192.168.1.1 portt statistics ---
4 probes sent, 4 success, 0 failed, 0.0% loss
rtt min/avg/max = 1.156 ms/1.195 ms/1.234 ms
```

## Install

### From source

```bash
git clone https://github.com/raydoom/portt.git
cd portt
cargo build --release
# binary at target/release/portt
```

### Cargo install (if published)

```bash
cargo install portt
```

### Download pre-built binary

Download from [Releases](https://github.com/raydoom/portt/releases) for Linux, macOS, and Windows.

## How UDP probing works

UDP is connectionless, so portt sends a 1-byte probe and waits for a response within the timeout:

- **Response received** → port is open and the remote service replied
- **Timeout** → port may be open (filtered or non-responsive) or closed
- **Connection refused** → port is closed (ICMP unreachable received)

## License

MIT

---

## portt

类似 ping 的端口检测工具。测试 TCP/UDP 端口可达性并测量延迟。

## 功能

- **TCP** 端口探测，测量连接延迟
- **UDP** 端口探测，检测响应/超时
- 可自定义超时、间隔和探测次数
- 实时显示每次探测延迟
- 汇总统计（最小/平均/最大 RTT、丢包率）
- Ctrl-C 优雅退出并显示统计

## 用法

```
portt <HOST> <PORT> [OPTIONS]
```

### 选项

| 选项 | 默认值 | 说明 |
|------|--------|------|
| `-p, --protocol` | `tcp` | 协议: `tcp` 或 `udp` |
| `-t, --timeout` | `1` | 每次探测超时（秒） |
| `-i, --interval` | `1` | 探测间隔（秒） |
| `-c, --count` | `4` | 探测次数（`0` = 无限） |
| `-h, --help` | | 显示帮助 |
| `-V, --version` | | 显示版本 |

### 示例

```bash
# 测试 TCP 22 端口，3 次，间隔 0.2 秒
portt 192.168.1.1 22 -c 3 -i 0.2

# 测试 UDP DNS 53 端口，超时 0.5 秒
portt 8.8.8.8 53 -p udp -t 0.5

# 无限模式（Ctrl-C 停止并显示统计）
portt 10.0.0.1 80 -c 0 -i 2

# 全部默认：TCP，4 次，超时 1 秒，间隔 1 秒
portt example.com 80
```

### 输出示例

```
portt 192.168.1.1:22 using TCP protocol, timeout 1.0s, interval 1.0s

from 192.168.1.1:22: seq=1 time=1.234 ms
from 192.168.1.1:22: seq=2 time=1.156 ms
from 192.168.1.1:22: seq=3 time=1.201 ms
from 192.168.1.1:22: seq=4 time=1.189 ms

--- 192.168.1.1 portt statistics ---
4 probes sent, 4 success, 0 failed, 0.0% loss
rtt min/avg/max = 1.156 ms/1.195 ms/1.234 ms
```

## 安装

### 从源码编译

```bash
git clone https://github.com/raydoom/portt.git
cd portt
cargo build --release
# 二进制文件位于 target/release/portt
```

### Cargo 安装（如已发布）

```bash
cargo install portt
```

### 下载预编译二进制

从 [Releases](https://github.com/raydoom/portt/releases) 下载 Linux、macOS、Windows 版本。

## UDP 探测原理

UDP 是无连接的，portt 发送 1 字节探测包并在超时内等待响应：

- **收到响应** → 端口开放，远程服务有应答
- **超时** → 端口可能开放（被过滤或无响应）或关闭
- **连接被拒绝** → 端口关闭（收到 ICMP 不可达）

## 许可证

MIT
