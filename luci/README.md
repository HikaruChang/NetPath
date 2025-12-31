# LuCI Application for NetPath

This is the web interface for NetPath multi-WAN manager.

## Features

- Interface status monitoring
- Policy configuration
- Traffic rules management
- Real-time statistics and graphs
- Connection tracking
- Failover visualization

## Installation

This package should be installed alongside the netpath core package on OpenWrt.

## Structure

- `luasrc/` - LuCI application code
  - `controller/` - Page controllers
  - `model/cbi/` - Configuration forms
  - `view/` - Custom views
- `htdocs/` - Static resources (CSS, JS, images)
- `po/` - Translations
- `root/` - File system overlay
