-- Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
-- Non-commercial use only. Attribution required.
--
-- RPC backend for OpenWrt 23.05+ / 25.x
-- Provides JSON-RPC methods for the JavaScript UI
module("luci.controller.netpath.rpc", package.seeall)

local json = require "luci.jsonc"
local sys = require "luci.sys"
local uci = require"luci.model.uci".cursor()
local fs = require "nixio.fs"

-- IPC socket path
local IPC_SOCKET = "/var/run/netpath.sock"

-- Helper function to communicate with NetPath daemon via Unix socket
local function netpath_ipc(cmd)
    local socket = require "nixio.socket"
    local sock = socket.unix()

    if not sock then
        return {success = false, error = "Failed to create socket"}
    end

    local ok, err = sock:connect(IPC_SOCKET)
    if not ok then
        sock:close()
        return {
            success = false,
            error = "Daemon not running: " .. (err or "unknown error")
        }
    end

    -- Set timeout
    sock:setopt("socket", "rcvtimeo", 5)
    sock:setopt("socket", "sndtimeo", 5)

    -- Send command
    local cmd_json = json.stringify(cmd)
    local sent, send_err = sock:send(cmd_json .. "\n")
    if not sent then
        sock:close()
        return {
            success = false,
            error = "Failed to send command: " .. (send_err or "unknown")
        }
    end

    -- Receive response
    local response = ""
    while true do
        local chunk, recv_err = sock:recv(4096)
        if not chunk or #chunk == 0 then break end
        response = response .. chunk
        if chunk:match("\n$") then break end
    end

    sock:close()

    if #response == 0 then
        return {success = false, error = "No response from daemon"}
    end

    local result = json.parse(response)
    if not result then
        return {success = false, error = "Invalid JSON response"}
    end

    return result
end

-- RPC method: Get daemon status
function status()
    local result = netpath_ipc({cmd = "status"})

    if not result.success then
        return {
            running = false,
            error = result.error,
            version = "unknown",
            uptime = "N/A"
        }
    end

    return {
        running = true,
        version = result.version or "1.0.0",
        uptime = result.uptime or "N/A",
        interfaces = result.interfaces or {},
        policies = result.policies or {}
    }
end

-- RPC method: Get interface status
function interfaces()
    local result = netpath_ipc({cmd = "interfaces"})

    if not result.success then return {} end

    return result.interfaces or {}
end

-- RPC method: Get policy status
function policies()
    local result = netpath_ipc({cmd = "policies"})

    if not result.success then return {} end

    return result.policies or {}
end

-- RPC method: Get performance metrics
function metrics()
    local result = netpath_ipc({cmd = "metrics"})

    if not result.success then return {} end

    return result.metrics or {}
end

-- RPC method: Reload configuration
function reload()
    local result = netpath_ipc({cmd = "reload"})

    if not result.success then return {success = false, error = result.error} end

    return {success = true, message = "Configuration reloaded successfully"}
end

-- RPC method: Get nftables rules
function nft_rules()
    local result = sys.exec("nft list table inet netpath 2>&1")

    if not result or #result == 0 then
        return {success = false, error = "Failed to get nftables rules"}
    end

    return {success = true, rules = result}
end

-- RPC method: Get routing tables
function routing_tables()
    local tables = {}

    -- Get main routing table
    local main_routes = sys.exec("ip route show 2>&1")
    if main_routes and #main_routes > 0 then tables["main"] = main_routes end

    -- Get custom routing tables from UCI
    uci:foreach("netpath", "interface", function(s)
        if s.table_id then
            local routes = sys.exec("ip route show table " .. s.table_id ..
                                        " 2>&1")
            if routes and #routes > 0 then
                tables[s[".name"]] = routes
            end
        end
    end)

    return {success = true, tables = tables}
end

-- RPC method: Get interface statistics
function interface_stats(params)
    local ifname = params.interface
    if not ifname then
        return {success = false, error = "Interface name required"}
    end

    local stats = {}

    -- Read from /sys/class/net
    local rx_bytes = fs.readfile("/sys/class/net/" .. ifname ..
                                     "/statistics/rx_bytes")
    local tx_bytes = fs.readfile("/sys/class/net/" .. ifname ..
                                     "/statistics/tx_bytes")
    local rx_packets = fs.readfile("/sys/class/net/" .. ifname ..
                                       "/statistics/rx_packets")
    local tx_packets = fs.readfile("/sys/class/net/" .. ifname ..
                                       "/statistics/tx_packets")
    local rx_errors = fs.readfile("/sys/class/net/" .. ifname ..
                                      "/statistics/rx_errors")
    local tx_errors = fs.readfile("/sys/class/net/" .. ifname ..
                                      "/statistics/tx_errors")

    if rx_bytes then stats.rx_bytes = tonumber(rx_bytes:match("%d+")) or 0 end
    if tx_bytes then stats.tx_bytes = tonumber(tx_bytes:match("%d+")) or 0 end
    if rx_packets then
        stats.rx_packets = tonumber(rx_packets:match("%d+")) or 0
    end
    if tx_packets then
        stats.tx_packets = tonumber(tx_packets:match("%d+")) or 0
    end
    if rx_errors then stats.rx_errors = tonumber(rx_errors:match("%d+")) or 0 end
    if tx_errors then stats.tx_errors = tonumber(tx_errors:match("%d+")) or 0 end

    return {success = true, stats = stats}
end

-- RPC method: Run diagnostic ping test
function diagnostic_ping(params)
    local target = params.target or "8.8.8.8"
    local interface = params.interface
    local count = params.count or 4

    local cmd = "ping -c " .. count .. " -W 2"
    if interface and #interface > 0 then cmd = cmd .. " -I " .. interface end
    cmd = cmd .. " " .. target .. " 2>&1"

    local result = sys.exec(cmd)

    return {success = true, output = result}
end

-- RPC method: Run diagnostic traceroute
function diagnostic_traceroute(params)
    local target = params.target or "8.8.8.8"
    local interface = params.interface

    local cmd = "traceroute -n -q 1 -w 2 -m 15"
    if interface and #interface > 0 then cmd = cmd .. " -i " .. interface end
    cmd = cmd .. " " .. target .. " 2>&1"

    local result = sys.exec(cmd)

    return {success = true, output = result}
end

-- RPC method: Get system information
function system_info()
    local info = {}

    -- Get OpenWrt version
    local version_file = fs.readfile("/etc/openwrt_release")
    if version_file then
        for line in version_file:gmatch("[^\r\n]+") do
            local key, value = line:match("^([^=]+)='?([^']*)'?$")
            if key and value then info[key:lower()] = value end
        end
    end

    -- Get kernel version
    info.kernel = sys.exec("uname -r"):gsub("\n", "")

    -- Get nftables version
    info.nft_version = sys.exec(
                           "nft --version 2>&1 | grep -oE '[0-9]+\\.[0-9]+\\.[0-9]+'"):gsub(
                           "\n", "")

    -- Get memory info
    local meminfo = fs.readfile("/proc/meminfo")
    if meminfo then
        info.mem_total = meminfo:match("MemTotal:%s+(%d+)") or "0"
        info.mem_free = meminfo:match("MemFree:%s+(%d+)") or "0"
        info.mem_available = meminfo:match("MemAvailable:%s+(%d+)") or "0"
    end

    -- Get load average
    local loadavg = fs.readfile("/proc/loadavg")
    if loadavg then
        info.loadavg = loadavg:match("^([%d%.]+%s+[%d%.]+%s+[%d%.]+)") or
                           "0.00 0.00 0.00"
    end

    return {success = true, info = info}
end
