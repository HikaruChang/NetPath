-- Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
-- Non-commercial use only. Attribution required.
m = Map("netpath", translate("Rules"),
        translate("Configure traffic routing rules"))

s = m:section(TypedSection, "rule", translate("Routing Rules"))
s.anonymous = false
s.addremove = true
s.template = "cbi/tblsection"

o = s:option(Flag, "enabled", translate("Enabled"))
o.default = "1"
o.rmempty = false

o = s:option(Value, "src_ip", translate("Source IP"))
o.placeholder = "0.0.0.0/0"
o.datatype = "ipaddr"

o = s:option(Value, "dst_ip", translate("Destination IP"))
o.placeholder = "0.0.0.0/0"
o.datatype = "ipaddr"

o = s:option(Value, "src_port", translate("Source Port"))
o.placeholder = "0:65535"

o = s:option(Value, "dst_port", translate("Destination Port"))
o.placeholder = "0:65535"

o = s:option(ListValue, "proto", translate("Protocol"))
o:value("all", translate("All"))
o:value("tcp", "TCP")
o:value("udp", "UDP")
o:value("icmp", "ICMP")
o.default = "all"

o = s:option(Value, "policy", translate("Policy"))
o.placeholder = "balanced"
o.rmempty = false

return m
