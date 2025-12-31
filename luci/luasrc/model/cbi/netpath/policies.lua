-- Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
-- Non-commercial use only. Attribution required.
m = Map("netpath", translate("Policies"),
        translate("Configure load balancing and failover policies"))

s = m:section(TypedSection, "policy", translate("Policies"))
s.anonymous = false
s.addremove = true
s.template = "cbi/tblsection"

-- Members
o = s:option(DynamicList, "members", translate("Members"))
o.placeholder = "wan_m1_w1"
o.rmempty = false

-- Mode
o = s:option(ListValue, "mode", translate("Load Balance Mode"))
o:value("balanced", translate("Balanced"))
o:value("failover", translate("Failover"))
o:value("roundrobin", translate("Round Robin"))
o:value("random", translate("Random"))
o:value("weighted", translate("Weighted"))
o:value("hash", translate("Hash-based"))
o.default = "balanced"

-- Sticky
o = s:option(Flag, "sticky", translate("Sticky Connections"))
o.default = "0"

-- Last resort
o = s:option(ListValue, "last_resort", translate("Last Resort"))
o:value("unreachable", translate("Unreachable"))
o:value("blackhole", translate("Blackhole"))
o:value("default", translate("Use Default Route"))
o.default = "unreachable"

return m
