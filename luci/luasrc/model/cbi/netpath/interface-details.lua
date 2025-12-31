-- Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
-- Non-commercial use only. Attribution required.
local uci = require"luci.model.uci".cursor()
local util = require "luci.util"

arg[1] = arg[1] or ""

m = Map("netpath", translate("Interface") .. ": " .. arg[1])
m.redirect = luci.dispatcher.build_url("admin/network/netpath/interfaces")

if m.uci:get("netpath", arg[1]) ~= "interface" then
    luci.http.redirect(m.redirect)
    return
end

-- Basic settings
s = m:section(NamedSection, arg[1], "interface", translate("Basic Settings"))
s.anonymous = true
s.addremove = false

o = s:option(Flag, "enabled", translate("Enabled"))
o.rmempty = false

o = s:option(Value, "ifname", translate("Interface Name"))
o.placeholder = "eth0"
o.rmempty = false

o = s:option(ListValue, "initial_state", translate("Initial State"))
o:value("online", translate("Online"))
o:value("offline", translate("Offline"))
o.default = "online"

o = s:option(ListValue, "family", translate("Protocol"))
o:value("ipv4", translate("IPv4"))
o:value("ipv6", translate("IPv6"))
o:value("both", translate("IPv4 and IPv6"))
o.default = "ipv4"

-- Tracking settings
s = m:section(NamedSection, arg[1], "interface", translate("Health Tracking"))

o = s:option(ListValue, "track_method", translate("Method"))
o:value("ping", translate("ICMP Ping"))
o:value("httping", translate("HTTP Ping"))
o:value("dns", translate("DNS Query"))
o:value("arping", translate("ARP Ping"))
o:value("none", translate("Disabled"))
o.default = "ping"

o = s:option(DynamicList, "track_targets", translate("Tracking Targets"))
o.placeholder = "8.8.8.8"
o.datatype = "or(ip4addr,hostname)"
o:depends("track_method", "ping")
o:depends("track_method", "httping")
o:depends("track_method", "dns")
o:depends("track_method", "arping")

o = s:option(Value, "track_reliability", translate("Reliability"))
o.placeholder = "1"
o.datatype = "uinteger"
o.default = "1"
o:depends("track_method", "ping")
o:depends("track_method", "httping")
o:depends("track_method", "dns")
o:depends("track_method", "arping")

o = s:option(Value, "track_count", translate("Ping Count"))
o.placeholder = "1"
o.datatype = "range(1,10)"
o.default = "1"
o:depends("track_method", "ping")
o:depends("track_method", "arping")

o = s:option(Value, "track_timeout", translate("Timeout (seconds)"))
o.placeholder = "4"
o.datatype = "range(1,60)"
o.default = "4"

o = s:option(Value, "track_interval", translate("Check Interval (seconds)"))
o.placeholder = "10"
o.datatype = "range(1,600)"
o.default = "10"

o = s:option(Value, "track_down", translate("Failures Before Down"))
o.placeholder = "5"
o.datatype = "range(1,100)"
o.default = "5"

o = s:option(Value, "track_up", translate("Successes Before Up"))
o.placeholder = "5"
o.datatype = "range(1,100)"
o.default = "5"

-- Advanced
s = m:section(NamedSection, arg[1], "interface", translate("Advanced"))

o = s:option(Flag, "flush_conntrack",
             translate("Flush Connection Tracking on Failover"))
o.default = "1"

return m
