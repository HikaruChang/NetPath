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

    m = new form.Map('netpath', _('NetPath - Globals'),
      _('Global settings for NetPath.'));

    s = m.section(form.NamedSection, 'globals', 'globals');

    o = s.option(form.Flag, 'enabled', _('Enable NetPath'));
    o.default = '1';
    o.rmempty = false;

    o = s.option(form.Value, 'rtable_offset', _('Routing Table Offset'));
    o.datatype = 'uinteger';
    o.default = '1000';
    o.placeholder = '1000';
    o.description = _('Starting number for custom routing tables');

    o = s.option(form.Value, 'fwmark_offset', _('Firewall Mark Offset'));
    o.default = '0x100';
    o.placeholder = '0x100';
    o.description = _('Offset for firewall marks (hex format)');

    o = s.option(form.Value, 'fwmark_mask', _('Firewall Mark Mask'));
    o.default = '0xff00';
    o.placeholder = '0xff00';
    o.description = _('Mask for firewall marks (hex format)');

    o = s.option(form.Value, 'nft_table', _('NFTables Table Name'));
    o.default = 'netpath';
    o.placeholder = 'netpath';

    o = s.option(form.Flag, 'local_source', _('Local Source Routing'));
    o.default = '1';
    o.description = _('Enable routing for locally originated traffic');

    o = s.option(form.Flag, 'ipv6_enabled', _('Enable IPv6'));
    o.default = '0';

    o = s.option(form.ListValue, 'log_level', _('Log Level'));
    o.default = 'info';
    o.value('error', _('Error'));
    o.value('warn', _('Warning'));
    o.value('info', _('Info'));
    o.value('debug', _('Debug'));
    o.value('trace', _('Trace'));

    o = s.option(form.Value, 'socket_path', _('IPC Socket Path'));
    o.default = '/var/run/netpath.sock';
    o.placeholder = '/var/run/netpath.sock';
    o.description = _('Unix socket path for IPC communication');

    return m.render();
  }
});
