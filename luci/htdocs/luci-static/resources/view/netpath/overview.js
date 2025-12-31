'use strict';
'require view';
'require uci';
'require rpc';
'require fs';

var callInitList = rpc.declare({
  object: 'luci',
  method: 'getInitList',
  params: ['name'],
  expect: { '': {} }
});

var callInitAction = rpc.declare({
  object: 'luci',
  method: 'setInitAction',
  params: ['name', 'action'],
  expect: { result: false }
});

var callNetPathStatus = rpc.declare({
  object: 'netpath',
  method: 'status',
  expect: {}
});

return view.extend({
  load: function () {
    return Promise.all([
      uci.load('netpath'),
      callInitList('netpath')
    ]);
  },

  getInterfaceStatus: function (ifaceName) {
    return callNetPathStatus().then(function (response) {
      if (response) {
        try {
          // Check for nested data structure (response.data.interfaces or response.interfaces)
          var interfaces = response.interfaces;
          if (!interfaces && response.data && Array.isArray(response.data.interfaces)) {
            interfaces = response.data.interfaces;
          }

          if (Array.isArray(interfaces)) {
            for (var i = 0; i < interfaces.length; i++) {
              if (interfaces[i].name === ifaceName) {
                return interfaces[i];
              }
            }
          }
        } catch (e) {
          console.error('NetPath parse error:', e);
          return { status: 'unknown' };
        }
      }
      return { status: 'unknown' };
    }).catch(function (err) {
      console.error('NetPath RPC error:', err);
      return { status: 'unknown' };
    });
  },

  render: function (data) {
    var self = this;
    var initData = data[1] || {};
    var initInfo = initData.netpath || {};
    var enabled = initInfo.enabled ? true : false;

    var interfaces = uci.sections('netpath', 'interface') || [];
    var members = uci.sections('netpath', 'member') || [];
    var policies = uci.sections('netpath', 'policy') || [];
    var rules = uci.sections('netpath', 'rule') || [];

    var content = E('div', { 'class': 'cbi-map' }, [
      E('h2', _('NetPath - Multi-WAN Manager')),
      E('div', { 'class': 'cbi-map-descr' },
        _('NetPath is a multi-WAN load balancing and failover solution for OpenWrt.')),

      E('div', { 'class': 'cbi-section' }, [
        E('h3', _('Service Status')),
        E('table', { 'class': 'table' }, [
          E('tr', { 'class': 'tr' }, [
            E('td', { 'class': 'td', 'style': 'width:200px' }, E('strong', _('Enabled'))),
            E('td', { 'class': 'td' }, enabled ?
              E('span', { 'style': 'color:green' }, _('Yes')) :
              E('span', { 'style': 'color:red' }, _('No')))
          ]),
          E('tr', { 'class': 'tr' }, [
            E('td', { 'class': 'td' }, E('strong', _('Interfaces'))),
            E('td', { 'class': 'td' }, String(interfaces.length))
          ]),
          E('tr', { 'class': 'tr' }, [
            E('td', { 'class': 'td' }, E('strong', _('Members'))),
            E('td', { 'class': 'td' }, String(members.length))
          ]),
          E('tr', { 'class': 'tr' }, [
            E('td', { 'class': 'td' }, E('strong', _('Policies'))),
            E('td', { 'class': 'td' }, String(policies.length))
          ]),
          E('tr', { 'class': 'tr' }, [
            E('td', { 'class': 'td' }, E('strong', _('Rules'))),
            E('td', { 'class': 'td' }, String(rules.length))
          ])
        ])
      ]),

      E('div', { 'class': 'cbi-section' }, [
        E('h3', _('Configured Interfaces')),
        E('table', { 'class': 'table' },
          interfaces.length > 0 ?
            [E('tr', { 'class': 'tr table-titles' }, [
              E('th', { 'class': 'th' }, _('Name')),
              E('th', { 'class': 'th' }, _('Enabled')),
              E('th', { 'class': 'th' }, _('Track Method')),
              E('th', { 'class': 'th' }, _('Latency')),
              E('th', { 'class': 'th' }, _('Packet Loss')),
              E('th', { 'class': 'th' }, _('Status'))
            ])].concat(interfaces.map(function (iface) {
              var latencyCell = E('td', { 'class': 'td' }, '-');
              var lossCell = E('td', { 'class': 'td' }, '-');
              var statusCell = E('td', { 'class': 'td' }, E('span', { 'style': 'color:orange' }, '? Unknown'));

              self.getInterfaceStatus(iface['.name']).then(function (info) {
                var status = info.status || 'unknown';
                var statusText, statusColor;

                // Update Latency and Loss
                if (info.latency !== undefined) {
                  latencyCell.textContent = info.latency.toFixed(2) + ' ms';
                }
                if (info.loss !== undefined) {
                  lossCell.textContent = info.loss.toFixed(2) + '%';
                }

                switch (status.toLowerCase()) {
                  case 'online':
                    statusText = '● Online';
                    statusColor = 'green';
                    break;
                  case 'offline':
                    statusText = '● Offline';
                    statusColor = 'red';
                    break;
                  case 'disabled':
                    statusText = '○ Disabled';
                    statusColor = 'gray';
                    break;
                  default:
                    statusText = '? Unknown';
                    statusColor = 'orange';
                }
                statusCell.firstChild.textContent = statusText;
                statusCell.firstChild.style.color = statusColor;
              });

              return E('tr', { 'class': 'tr' }, [
                E('td', { 'class': 'td' }, iface['.name']),
                E('td', { 'class': 'td' }, iface.enabled != '0' ? '✓' : '✗'),
                E('td', { 'class': 'td' }, (iface.track_method || 'ping').toUpperCase()),
                latencyCell,
                lossCell,
                statusCell
              ]);
            })) :
            [E('tr', { 'class': 'tr' }, [
              E('td', { 'class': 'td', 'colspan': '6' },
                E('em', _('No interfaces configured')))
            ])]
        )
      ]),

      E('div', { 'class': 'cbi-section' }, [
        E('h3', _('Service Control')),
        E('div', { 'class': 'cbi-page-actions' }, [
          E('button', {
            'class': 'cbi-button cbi-button-action',
            'click': function () {
              callInitAction('netpath', 'restart').then(function () {
                window.setTimeout(function () { location.reload(); }, 2000);
              });
            }
          }, _('Restart Service')),
          '\u00a0',
          E('button', {
            'class': 'cbi-button cbi-button-apply',
            'click': function () {
              callInitAction('netpath', 'reload').then(function () {
                window.setTimeout(function () { location.reload(); }, 2000);
              });
            }
          }, _('Reload Configuration'))
        ])
      ])
    ]);

    return content;
  },

  handleSaveApply: null,
  handleSave: null,
  handleReset: null
});
