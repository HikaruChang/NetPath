-- Copyright (c) 2024-2026 Hikaru (i@rua.moe). All rights reserved.
-- Non-commercial use only. Attribution required.
module("luci.controller.netpath", package.seeall)

function index()
    if not nixio.fs.access("/etc/config/netpath") then return end

    -- Main entry
    local page = entry({"admin", "network", "netpath"},
                       alias("admin", "network", "netpath", "overview"),
                       _("NetPath"), 50)
    page.dependent = true
    page.acl_depends = {"luci-app-netpath"}

    -- Overview page
    entry({"admin", "network", "netpath", "overview"},
          template("netpath/overview"), _("Overview"), 1)

    -- Interfaces
    entry({"admin", "network", "netpath", "interfaces"}, arcombine(
              cbi("netpath/interfaces"), cbi("netpath/interface-details")),
          _("Interfaces"), 2).leaf = true

    -- Members
    entry({"admin", "network", "netpath", "members"}, cbi("netpath/members"),
          _("Members"), 3)

    -- Policies
    entry({"admin", "network", "netpath", "policies"}, cbi("netpath/policies"),
          _("Policies"), 4)

    -- Rules
    entry({"admin", "network", "netpath", "rules"}, cbi("netpath/rules"),
          _("Rules"), 5)

    -- Advanced
    entry({"admin", "network", "netpath", "advanced"}, cbi("netpath/advanced"),
          _("Advanced"), 6)

    -- Diagnostics
    entry({"admin", "network", "netpath", "diagnostics"},
          template("netpath/diagnostics"), _("Diagnostics"), 7)

    -- API endpoints
    entry({"admin", "network", "netpath", "status"}, call("action_status")).leaf =
        true

    entry({"admin", "network", "netpath", "metrics"}, call("action_metrics")).leaf =
        true

    entry({"admin", "network", "netpath", "action"}, call("action_control")).leaf =
        true

    entry({"admin", "network", "netpath", "diag"}, call("action_diag")).leaf =
        true
end

function action_status()
    local sys = require "luci.sys"
    local uci = require"luci.model.uci".cursor()

    local status = {
        status = "stopped",
        interfaces = 0,
        total_interfaces = 0,
        policies = 0,
        interface_list = {}
    }

    -- Check if service is running
    if sys.call("pgrep netpath > /dev/null") == 0 then
        status.status = "running"
    end

    -- Count interfaces
    uci:foreach("netpath", "interface", function(s)
        status.total_interfaces = status.total_interfaces + 1
        if s.enabled == "1" then
            status.interfaces = status.interfaces + 1
            table.insert(status.interface_list, {
                name = s[".name"],
                enabled = true,
                ipv4 = s.ipv4 or "-",
                gateway = s.gateway or "-",
                metric = s.metric or "-"
            })
        end
    end)

    -- Count policies
    uci:foreach("netpath", "policy", function(s)
        if s.enabled == "1" then status.policies = status.policies + 1 end
    end)

    luci.http.prepare_content("application/json")
    luci.http.write_json(status)
end

function action_metrics()
    local json = require "luci.jsonc"
    local socket_path = "/var/run/netpath.sock"

    local metrics = {}

    if nixio.fs.access(socket_path) then
        local socket = nixio.socket("unix", "stream")
        if socket:connect(socket_path) then
            socket:send('{"cmd":"interfaces"}\n')
            local response = socket:recv(8192)
            socket:close()

            if response then metrics = json.parse(response) end
        end
    end

    luci.http.prepare_content("application/json")
    luci.http.write_json(metrics)
end

function action_control()
    local sys = require "luci.sys"
    local action = luci.http.formvalue("action")
    local result = {success = false}

    if action == "reload" then
        result.success = (sys.call("/etc/init.d/netpath reload") == 0)
    elseif action == "restart" then
        result.success = (sys.call("/etc/init.d/netpath restart") == 0)
    elseif action == "enable" then
        result.success = (sys.call("/etc/init.d/netpath enable") == 0)
    elseif action == "disable" then
        result.success = (sys.call("/etc/init.d/netpath disable") == 0)
    end

    luci.http.prepare_content("application/json")
    luci.http.write_json(result)
end
function action_diag()
    local sys = require "luci.sys"
    local cmd = luci.http.formvalue("cmd")
    local iface = luci.http.formvalue("interface")
    local output = ""

    if cmd == "ping" and iface then
        output = sys.exec("ping -c 4 -I " .. iface .. " 8.8.8.8 2>&1")
    elseif cmd == "traceroute" and iface then
        output = sys.exec("traceroute -i " .. iface .. " 8.8.8.8 2>&1")
    elseif cmd == "routes" then
        output = sys.exec("ip route show table all 2>&1")
    elseif cmd == "nft" then
        output = sys.exec("nft list table inet netpath 2>&1")
    end

    luci.http.prepare_content("application/json")
    luci.http.write_json({output = output})
end
