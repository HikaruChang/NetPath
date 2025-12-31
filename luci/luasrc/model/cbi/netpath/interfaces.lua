-- Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
-- Non-commercial use only. Attribution required.
local uci = require"luci.model.uci".cursor()
local sys = require "luci.sys"

m = Map("netpath", translate("Interfaces"),
        translate("Configure WAN interfaces for load balancing and failover"))

-- Interface list
s = m:section(TypedSection, "interface", translate("Interfaces"))
s.anonymous = false
s.addremove = true
s.template = "cbi/tblsection"

-- Enabled
o = s:option(Flag, "enabled", translate("Enabled"))
o.default = "1"
o.rmempty = false

-- Interface name
o = s:option(Value, "ifname", translate("Interface"))
o.placeholder = "eth0"
o.rmempty = false

function o.validate(self, value)
    if not value or value == "" then
        return nil, translate("Interface name is required")
    end
    return value
end

-- Initial state
o = s:option(ListValue, "initial_state", translate("Initial State"))
o:value("online", translate("Online"))
o:value("offline", translate("Offline"))
o.default = "online"

-- Family
o = s:option(ListValue, "family", translate("Protocol"))
o:value("ipv4", translate("IPv4"))
o:value("ipv6", translate("IPv6"))
o:value("both", translate("IPv4 and IPv6"))
o.default = "ipv4"

-- Tracking method
o = s:option(ListValue, "track_method", translate("Tracking Method"))
o:value("ping", translate("ICMP Ping"))
o:value("httping", translate("HTTP Ping"))
o:value("dns", translate("DNS Query"))
o:value("arping", translate("ARP Ping"))
o:value("none", translate("Disabled"))
o.default = "ping"

-- Flush conntrack on failover
o =
    s:option(Flag, "flush_conntrack", translate("Flush Connections on Failover"))
o.default = "1"

return m
