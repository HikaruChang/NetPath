'use strict';
'require view';
'require ui';
'require fs';
'require poll';

return view.extend({
  pollInterval: null,

  load: function () {
    return Promise.resolve();
  },

  renderToolSection: function (title, description, formContent, resultId) {
    var section = E('div', { 'class': 'cbi-section' }, [
      E('h3', title),
      description ? E('p', { 'class': 'cbi-section-descr' }, description) : '',
      E('div', { 'class': 'cbi-section-node' }, [
        E('div', { 'class': 'cbi-value' }, [
          E('div', { 'class': 'cbi-value-field', 'style': 'display:flex; flex-wrap:wrap; gap:10px; align-items:center;' }, formContent)
        ]),
        E('div', { 'class': 'cbi-value' }, [
          E('pre', {
            'id': resultId,
            'class': 'command-output',
            'style': 'background:#1e1e1e; color:#d4d4d4; padding:15px; border-radius:4px; overflow-x:auto; min-height:100px; max-height:400px; white-space:pre; font-family:monospace; font-size:13px; margin:10px 0;'
          }, _('Click button to execute...'))
        ])
      ])
    ]);
    return section;
  },

  runCommand: function (cmd, resultId) {
    var resultEl = document.getElementById(resultId);
    if (resultEl) {
      resultEl.textContent = _('Executing...');
      resultEl.style.color = '#d4d4d4';
    }

    return fs.exec('/bin/sh', ['-c', cmd]).then(function (res) {
      var output = '';
      if (res.stdout) output += res.stdout;
      if (res.stderr) output += res.stderr;
      if (!output) output = _('(No output)');

      if (resultEl) {
        resultEl.textContent = output;
      }
    }).catch(function (err) {
      if (resultEl) {
        resultEl.textContent = _('Error: ') + err.message;
        resultEl.style.color = '#f48771';
      }
    });
  },

  render: function () {
    var self = this;

    var view = E('div', { 'class': 'cbi-map' }, [
      E('h2', _('NetPath - Diagnostics')),
      E('p', { 'class': 'cbi-map-descr' }, _('Network diagnostic tools for troubleshooting WAN connections.'))
    ]);

    // Ping Tool
    var pingInput = E('input', {
      'type': 'text',
      'class': 'cbi-input-text',
      'id': 'ping-target',
      'placeholder': '8.8.8.8',
      'style': 'width:200px;'
    });
    var pingCount = E('select', { 'class': 'cbi-input-select', 'id': 'ping-count', 'style': 'width:80px;' }, [
      E('option', { 'value': '3' }, '3'),
      E('option', { 'value': '5', 'selected': 'selected' }, '5'),
      E('option', { 'value': '10' }, '10')
    ]);
    var pingBtn = E('button', {
      'class': 'btn cbi-button cbi-button-action',
      'click': function () {
        var target = document.getElementById('ping-target').value || '8.8.8.8';
        var count = document.getElementById('ping-count').value;
        self.runCommand('ping -c ' + count + ' ' + target, 'ping-result');
      }
    }, _('Ping'));

    view.appendChild(this.renderToolSection(
      _('Ping Test'),
      _('Test connectivity to a remote host'),
      [
        E('label', { 'style': 'margin-right:5px;' }, _('Target:')),
        pingInput,
        E('label', { 'style': 'margin:0 5px;' }, _('Count:')),
        pingCount,
        pingBtn
      ],
      'ping-result'
    ));

    // Traceroute Tool
    var traceInput = E('input', {
      'type': 'text',
      'class': 'cbi-input-text',
      'id': 'trace-target',
      'placeholder': '8.8.8.8',
      'style': 'width:200px;'
    });
    var traceBtn = E('button', {
      'class': 'btn cbi-button cbi-button-action',
      'click': function () {
        var target = document.getElementById('trace-target').value || '8.8.8.8';
        self.runCommand('traceroute -n -m 15 ' + target, 'trace-result');
      }
    }, _('Traceroute'));

    view.appendChild(this.renderToolSection(
      _('Traceroute'),
      _('Trace the route to a remote host'),
      [
        E('label', { 'style': 'margin-right:5px;' }, _('Target:')),
        traceInput,
        traceBtn
      ],
      'trace-result'
    ));

    // DNS Lookup Tool
    var dnsInput = E('input', {
      'type': 'text',
      'class': 'cbi-input-text',
      'id': 'dns-target',
      'placeholder': 'google.com',
      'style': 'width:200px;'
    });
    var dnsServer = E('input', {
      'type': 'text',
      'class': 'cbi-input-text',
      'id': 'dns-server',
      'placeholder': '8.8.8.8',
      'style': 'width:120px;'
    });
    var dnsBtn = E('button', {
      'class': 'btn cbi-button cbi-button-action',
      'click': function () {
        var target = document.getElementById('dns-target').value || 'google.com';
        var server = document.getElementById('dns-server').value;
        var cmd = server ? ('nslookup ' + target + ' ' + server) : ('nslookup ' + target);
        self.runCommand(cmd, 'dns-result');
      }
    }, _('Lookup'));

    view.appendChild(this.renderToolSection(
      _('DNS Lookup'),
      _('Query DNS records for a domain'),
      [
        E('label', { 'style': 'margin-right:5px;' }, _('Domain:')),
        dnsInput,
        E('label', { 'style': 'margin:0 5px;' }, _('Server (optional):')),
        dnsServer,
        dnsBtn
      ],
      'dns-result'
    ));

    // System Info Section
    var sysSection = E('div', { 'class': 'cbi-section' }, [
      E('h3', _('System Information')),
      E('p', { 'class': 'cbi-section-descr' }, _('View current network configuration and routing information.')),
      E('div', { 'class': 'cbi-section-node' }, [
        E('div', { 'style': 'display:flex; flex-wrap:wrap; gap:10px; margin-bottom:15px;' }, [
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('ip route show table all | head -100', 'sys-result'); }
          }, _('Routing Tables')),
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('ip rule show', 'sys-result'); }
          }, _('IP Rules')),
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('ip addr show', 'sys-result'); }
          }, _('IP Addresses')),
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('cat /etc/netpath.json 2>/dev/null || echo "Config file not found"', 'sys-result'); }
          }, _('NetPath Config')),
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('nft list ruleset 2>/dev/null | head -200 || iptables-save 2>/dev/null | head -200', 'sys-result'); }
          }, _('Firewall Rules')),
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('conntrack -L 2>/dev/null | head -50 || cat /proc/net/nf_conntrack 2>/dev/null | head -50', 'sys-result'); }
          }, _('Conntrack'))
        ]),
        E('pre', {
          'id': 'sys-result',
          'class': 'command-output',
          'style': 'background:#1e1e1e; color:#d4d4d4; padding:15px; border-radius:4px; overflow-x:auto; min-height:150px; max-height:500px; white-space:pre; font-family:monospace; font-size:13px;'
        }, _('Click a button above to view information...'))
      ])
    ]);
    view.appendChild(sysSection);

    // NetPath Status Section
    var statusSection = E('div', { 'class': 'cbi-section' }, [
      E('h3', _('NetPath Service Status')),
      E('div', { 'class': 'cbi-section-node' }, [
        E('div', { 'style': 'display:flex; flex-wrap:wrap; gap:10px; margin-bottom:15px;' }, [
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('pidof netpath >/dev/null && echo "NetPath is running (PID: $(pidof netpath))" || echo "NetPath is not running"', 'status-result'); }
          }, _('Check Status')),
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('/etc/init.d/netpath restart 2>&1 && echo "Service restarted"', 'status-result'); }
          }, _('Restart Service')),
          E('button', {
            'class': 'btn cbi-button cbi-button-action',
            'click': function () { self.runCommand('logread -e netpath 2>/dev/null | tail -50 || journalctl -u netpath --no-pager -n 50 2>/dev/null || echo "No logs found"', 'status-result'); }
          }, _('View Logs'))
        ]),
        E('pre', {
          'id': 'status-result',
          'class': 'command-output',
          'style': 'background:#1e1e1e; color:#d4d4d4; padding:15px; border-radius:4px; overflow-x:auto; min-height:100px; max-height:400px; white-space:pre; font-family:monospace; font-size:13px;'
        }, _('Click a button above to check service status...'))
      ])
    ]);
    view.appendChild(statusSection);

    return view;
  },

  handleSaveApply: null,
  handleSave: null,
  handleReset: null
});
