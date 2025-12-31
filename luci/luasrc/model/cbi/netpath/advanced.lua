-- Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
-- Non-commercial use only. Attribution required.
m = Map("netpath", translate("Advanced Settings"),
        translate("Advanced NetPath configuration"))

s = m:section(NamedSection, "globals", "globals", translate("Global Settings"))

o = s:option(Flag, "enabled", translate("Enable NetPath"))
o.default = "1"
o.rmempty = false

o = s:option(Value, "rtable_offset", translate("Routing Table Offset"))
o.placeholder = "1000"
o.default = "1000"
o.datatype = "uinteger"

o = s:option(Value, "fwmark_offset", translate("Firewall Mark Offset"))
o.placeholder = "0x100"
o.default = "0x100"

o = s:option(Value, "nft_table", translate("NFTables Table Name"))
o.placeholder = "netpath"
o.default = "netpath"

o = s:option(Flag, "ipv6_enabled", translate("Enable IPv6"))
o.default = "0"

o = s:option(Value, "log_level", translate("Log Level"))
o:value("error", translate("Error"))
o:value("warn", translate("Warning"))
o:value("info", translate("Info"))
o:value("debug", translate("Debug"))
o.default = "info"

return m
