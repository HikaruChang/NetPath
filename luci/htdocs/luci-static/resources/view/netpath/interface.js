'use strict';
'require form';
'require view';
'require uci';

return view.extend({
  load: function () {
    return uci.load('netpath');
  },

  render: function () {
    var m, s, o;

    m = new form.Map('netpath', _('NetPath - Interfaces'),
      _('Configure WAN interfaces for load balancing and failover.'));

    s = m.section(form.GridSection, 'interface');
    s.addremove = true;
    s.anonymous = false;
    s.sortable = true;

    o = s.option(form.Flag, 'enabled', _('Enabled'));
    o.default = '1';
    o.rmempty = false;

    o = s.option(form.Value, 'ifname', _('Device'));
    o.placeholder = 'eth0';
    o.rmempty = false;

    o = s.option(form.ListValue, 'initial_state', _('Initial State'));
    o.default = 'online';
    o.value('online', _('Online'));
    o.value('offline', _('Offline'));

    o = s.option(form.ListValue, 'family', _('IP Family'));
    o.default = 'ipv4';
    o.value('ipv4', 'IPv4');
    o.value('ipv6', 'IPv6');
    o.value('both', 'IPv4 + IPv6');
    o.modalonly = true;

    o = s.option(form.ListValue, 'track_method', _('Track Method'));
    o.default = 'ping';
    o.value('ping', _('Ping (ICMP)'));
    o.value('arping', _('ARP Ping'));
    o.value('httping', _('HTTP Ping'));
    o.value('dns', _('DNS Query'));
    o.value('none', _('None (Always Online)'));

    o = s.option(form.DynamicList, 'track_targets', _('Track Targets'));
    o.datatype = 'host';
    o.placeholder = '8.8.8.8';
    o.modalonly = true;
    o.depends('track_method', 'ping');
    o.depends('track_method', 'arping');
    o.depends('track_method', 'httping');
    o.depends('track_method', 'dns');

    o = s.option(form.Value, 'track_reliability', _('Reliability'));
    o.datatype = 'range(1,100)';
    o.default = '1';
    o.placeholder = '1';
    o.description = _('Number of targets that must respond for interface to be considered online');
    o.modalonly = true;

    o = s.option(form.Value, 'track_count', _('Ping Count'));
    o.datatype = 'range(1,10)';
    o.default = '1';
    o.placeholder = '1';
    o.modalonly = true;

    o = s.option(form.Value, 'track_timeout', _('Timeout'));
    o.datatype = 'uinteger';
    o.default = '4';
    o.placeholder = '4';
    o.description = _('Seconds');
    o.modalonly = true;

    o = s.option(form.Value, 'track_interval', _('Check Interval'));
    o.datatype = 'uinteger';
    o.default = '10';
    o.placeholder = '10';
    o.description = _('Seconds between checks');
    o.modalonly = true;

    o = s.option(form.Value, 'track_down', _('Down Threshold'));
    o.datatype = 'uinteger';
    o.default = '5';
    o.placeholder = '5';
    o.description = _('Failures before marking interface offline');
    o.modalonly = true;

    o = s.option(form.Value, 'track_up', _('Up Threshold'));
    o.datatype = 'uinteger';
    o.default = '5';
    o.placeholder = '5';
    o.description = _('Successes before marking interface online');
    o.modalonly = true;

    o = s.option(form.Flag, 'flush_conntrack', _('Flush Conntrack'));
    o.default = '1';
    o.description = _('Flush connection tracking on interface state change');
    o.modalonly = true;

    return m.render();
  }
});
