-- Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
-- Non-commercial use only. Attribution required.
m = Map("netpath", translate("Members"),
        translate("Configure interface members with metrics and weights"))

s = m:section(TypedSection, "member", translate("Members"))
s.anonymous = false
s.addremove = true
s.template = "cbi/tblsection"

o = s:option(Flag, "enabled", translate("Enabled"))
o.default = "1"
o.rmempty = false

o = s:option(Value, "interface", translate("Interface"))
o.placeholder = "wan1"
o.rmempty = false

o = s:option(Value, "metric", translate("Metric"))
o.placeholder = "1"
o.default = "1"
o.datatype = "uinteger"

o = s:option(Value, "weight", translate("Weight"))
o.placeholder = "1"
o.default = "1"
o.datatype = "range(1,100)"

return m
