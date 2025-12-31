# NetPath - High-performance Multi-WAN Manager

<div align="center">

[![License: CC BY-NC](https://img.shields.io/badge/License-CC%20BY--NC-lightgrey.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![OpenWrt](https://img.shields.io/badge/OpenWrt-00B5E2?style=flat&logo=openwrt&logoColor=white)](https://openwrt.org/)

**Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.**  
**Non-commercial use only. Attribution required.**

A complete rewrite of mwan3 using modern technologies for superior performance and efficiency.

</div>

## ✨ Features

- 🚀 **High Performance**: Written in Rust for maximum speed and memory safety
- 🔥 **Modern Stack**: Uses nftables instead of legacy iptables
- ⚡ **Efficient**: Async I/O and optimized algorithms
- 🎯 **Flexible Policies**: Multiple load balancing modes (balanced, failover, round-robin, hash-based, weighted)
- 🔍 **Health Tracking**: ICMP ping, HTTP ping, DNS, ARP monitoring
- 📊 **Real-time Metrics**: Live interface statistics and traffic visualization
- 🌐 **Web Interface**: Modern LuCI-based management panel
- 🔄 **Hot Reload**: Change configuration without service interruption
- 📡 **IPC Control**: Unix socket API for external control

## 🏗️ Architecture

```
NetPath
├── core/           # Rust daemon
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── config.rs      # Configuration management
│   │   ├── interface.rs   # Interface state tracking
│   │   ├── nft.rs         # nftables integration
│   │   ├── policy.rs      # Load balancing policies
│   │   ├── route.rs       # Routing table management
│   │   ├── track.rs       # Health monitoring
│   │   ├── ipc.rs         # IPC server
│   │   └── metrics.rs     # Metrics collection
│   └── Cargo.toml
└── luci/           # Web interface
    ├── luasrc/
    │   ├── controller/    # Page controllers
    │   ├── model/cbi/     # Configuration forms
    │   └── view/          # Templates
    └── htdocs/            # Static assets
```

## 📋 Requirements

### Runtime
- **OpenWrt**: 22.03+ (fully tested), 23.05+ (recommended), 24.x/25.x (full support with JavaScript UI)
- **nftables**: 1.0.6+ (kernel 5.4+), 1.0.9+ recommended for OpenWrt 25.x
- **Kernel Modules**: nft_core, nft_nat, nft_fib (25.x), nf_conntrack
- **System**: ip-full, ipset, tc, curl, jsonfilter, bind-dig

### Build
- Rust 1.70+ with musl/cross-compile target support
- OpenWrt SDK (for package building)
- Standard build tools (make, gcc)

## 🚀 Installation

### From Package
```bash
opkg update
opkg install netpath luci-app-netpath
/etc/init.d/netpath enable
/etc/init.d/netpath start
```

### From Source
```bash
# Build core daemon
cd core
make RUST_TARGET=x86_64-unknown-linux-musl
make install DESTDIR=/tmp/install

# The binary and config files will be in /tmp/install
```

## ⚙️ Configuration

### UCI Configuration (OpenWrt)
Edit `/etc/config/netpath`:

```uci
config globals 'globals'
    option enabled '1'
    option nft_table 'netpath'
    
config interface 'wan'
    option enabled '1'
    option ifname 'eth0'
    option track_method 'ping'
    list track_targets '8.8.8.8'
    
config member 'wan_m1_w1'
    option interface 'wan'
    option metric '1'
    option weight '1'
    
config policy 'balanced'
    list members 'wan_m1_w1'
    option mode 'balanced'
    
config rule 'default_rule'
    option policy 'balanced'
```

### TOML Configuration
Edit `/etc/netpath/config.toml`:

```toml
[global]
enabled = true
nft_table = "netpath"

[interfaces.wan]
ifname = "eth0"
enabled = true

[interfaces.wan.track]
method = "ping"
targets = ["8.8.8.8", "1.1.1.1"]
```

See `examples/config.toml` for full configuration options.

## 🎮 Usage

### Via Web Interface
Navigate to **Network → NetPath** in LuCI.

### Via Command Line

```bash
# Start service
/etc/init.d/netpath start

# Reload configuration
/etc/init.d/netpath reload

# Check status (via IPC)
echo '{"cmd":"status"}' | nc -U /var/run/netpath.sock

# Get interface details
echo '{"cmd":"interfaces"}' | nc -U /var/run/netpath.sock

# Enable/disable interface
echo '{"cmd":"enable","interface":"wan"}' | nc -U /var/run/netpath.sock
echo '{"cmd":"disable","interface":"wan"}' | nc -U /var/run/netpath.sock
```

## 🔧 Load Balancing Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **balanced** | Weighted distribution across all interfaces | General multi-WAN |
| **failover** | Use interfaces in metric order | High availability |
| **round-robin** | Rotate through interfaces evenly | Equal distribution |
| **hash** | Source IP hash-based selection | Session persistence |
| **weighted** | Random selection with weights | Unequal bandwidth |
| **random** | Pure random selection | Testing |

## 📊 Monitoring

### Health Check Methods

- **ping**: ICMP echo request (default)
- **httping**: HTTP/HTTPS connectivity check
- **dns**: DNS query resolution
- **arping**: ARP-level connectivity
- **none**: No health checking

### Metrics

- Interface status (online/offline)
- Latency (ms)
- Packet loss (%)
- Traffic counters (TX/RX bytes)
- Connection count

## 🔍 Debugging

```bash
# Check nftables rules
nft list table inet netpath

# View routing tables
ip route show table 1000
ip route show table 1001

# Check routing rules
ip rule show

# View daemon logs
logread | grep netpath

# Test interface tracking
ping -I eth0 8.8.8.8
```

## 🆚 Comparison with mwan3

| Feature | mwan3 | NetPath |
|---------|-------|---------|
| Language | Shell scripts | Rust |
| Firewall | iptables | nftables |
| Performance | Moderate | High |
| Memory | Higher | Lower |
| CPU Usage | Higher | Lower |
| Hot Reload | Limited | Full support |
| Async I/O | No | Yes |
| Type Safety | No | Yes |
| Load Balancing Modes | 2 | 6 |
| Health Check Methods | 2 | 4 |
| OpenWrt 25.x Support | Limited | Full (JavaScript UI) |
| Real-time Metrics | No | Yes |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit pull requests.

## 📄 License

GPL-2.0 - see LICENSE file for details.

## 🙏 Acknowledgments

- Inspired by mwan3
- Built on nftables
- Powered by Rust
- Integrated with OpenWrt/LuCI

## 📞 Support

- Issues: https://github.com/hikaruchang/netpath/issues
- Discussions: https://github.com/hikaruchang/netpath/discussions
- Donate: 0xdb61B2aD59bdF2A066B7fC9F00f86c3EBc4856B4

---

**Note**: NetPath is under active development. Production use should be carefully tested in your environment.
