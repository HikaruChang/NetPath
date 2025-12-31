'use strict';
'require view';
'require ui';
'require fs';
'require uci';

return view.extend({
  load: function () {
    return Promise.all([
      fs.exec('/usr/bin/netpath-migrate-mwan3', ['check']),
      uci.load('netpath')
    ]);
  },

  render: function (data) {
    var self = this;
    var checkResult = data[0];
    var mwan3Exists = false;

    try {
      var parsed = JSON.parse(checkResult.stdout);
      mwan3Exists = parsed.exists === true;
    } catch (e) {
      mwan3Exists = false;
    }

    var view = E('div', { 'class': 'cbi-map' }, [
      E('h2', _('NetPath - Migrate from mwan3')),
      E('p', { 'class': 'cbi-map-descr' },
        _('Import your existing mwan3 configuration into NetPath for a seamless transition.'))
    ]);

    // Status Section
    var statusSection = E('div', { 'class': 'cbi-section' }, [
      E('h3', _('Migration Status')),
      E('table', { 'class': 'table' }, [
        E('tr', { 'class': 'tr' }, [
          E('td', { 'class': 'td', 'style': 'width:200px' }, E('strong', _('mwan3 Configuration'))),
          E('td', { 'class': 'td' }, mwan3Exists ?
            E('span', { 'style': 'color:green' }, '✓ ' + _('Found')) :
            E('span', { 'style': 'color:red' }, '✗ ' + _('Not found')))
        ])
      ])
    ]);
    view.appendChild(statusSection);

    if (!mwan3Exists) {
      var noConfigSection = E('div', { 'class': 'cbi-section' }, [
        E('div', { 'class': 'alert-message warning' }, [
          E('h4', _('No mwan3 Configuration Found')),
          E('p', _('The mwan3 configuration file (/etc/config/mwan3) was not found on this system.')),
          E('p', _('If you have mwan3 installed, please ensure it is properly configured before attempting migration.'))
        ])
      ]);
      view.appendChild(noConfigSection);
      return view;
    }

    // Preview Section
    var previewSection = E('div', { 'class': 'cbi-section' }, [
      E('h3', _('Configuration Preview')),
      E('p', { 'class': 'cbi-section-descr' },
        _('Click the button below to preview what will be migrated from mwan3.')),
      E('div', { 'style': 'margin-bottom:15px;' }, [
        E('button', {
          'class': 'btn cbi-button cbi-button-action',
          'click': function () {
            self.loadPreview();
          }
        }, _('Load Preview'))
      ]),
      E('div', { 'id': 'migration-preview' }, [
        E('pre', {
          'id': 'preview-output',
          'style': 'background:#1e1e1e; color:#d4d4d4; padding:15px; border-radius:4px; overflow-x:auto; min-height:200px; max-height:500px; white-space:pre; font-family:monospace; font-size:13px;'
        }, _('Click "Load Preview" to see mwan3 configuration...'))
      ])
    ]);
    view.appendChild(previewSection);

    // Migration Actions Section
    var actionsSection = E('div', { 'class': 'cbi-section' }, [
      E('h3', _('Migration Actions')),
      E('div', { 'class': 'alert-message warning', 'style': 'margin-bottom:15px;' }, [
        E('h4', '⚠️ ' + _('Important Notes')),
        E('ul', [
          E('li', _('Your existing NetPath configuration will be backed up before migration.')),
          E('li', _('After migration, you should disable mwan3 to avoid conflicts.')),
          E('li', _('Review the preview above before proceeding with migration.')),
          E('li', _('This operation cannot be automatically undone.'))
        ])
      ]),
      E('div', { 'style': 'display:flex; gap:10px; flex-wrap:wrap;' }, [
        E('button', {
          'class': 'btn cbi-button cbi-button-positive',
          'id': 'btn-migrate',
          'click': function () {
            self.doMigration();
          }
        }, '🔄 ' + _('Start Migration')),
        E('button', {
          'class': 'btn cbi-button cbi-button-action',
          'click': function () {
            self.disableMwan3();
          }
        }, '⏹️ ' + _('Disable mwan3')),
        E('button', {
          'class': 'btn cbi-button cbi-button-action',
          'click': function () {
            self.enableNetpath();
          }
        }, '▶️ ' + _('Enable NetPath'))
      ]),
      E('div', { 'style': 'margin-top:15px;' }, [
        E('pre', {
          'id': 'migration-output',
          'style': 'background:#1e1e1e; color:#d4d4d4; padding:15px; border-radius:4px; overflow-x:auto; min-height:100px; max-height:300px; white-space:pre; font-family:monospace; font-size:13px; display:none;'
        })
      ])
    ]);
    view.appendChild(actionsSection);

    // Post-Migration Checklist
    var checklistSection = E('div', { 'class': 'cbi-section' }, [
      E('h3', _('Post-Migration Checklist')),
      E('div', { 'class': 'cbi-section-node' }, [
        E('ol', { 'style': 'line-height:2;' }, [
          E('li', _('Review the migrated configuration in each tab (Interfaces, Members, Policies, Rules)')),
          E('li', _('Verify interface names match your network configuration')),
          E('li', _('Test connectivity on each WAN interface')),
          E('li', _('Disable mwan3 service to prevent conflicts')),
          E('li', _('Enable and start NetPath service')),
          E('li', _('Monitor the Overview page to verify interfaces are online'))
        ])
      ])
    ]);
    view.appendChild(checklistSection);

    return view;
  },

  loadPreview: function () {
    var outputEl = document.getElementById('preview-output');
    if (outputEl) {
      outputEl.textContent = _('Loading preview...');
      outputEl.style.color = '#d4d4d4';
    }

    return fs.exec('/usr/bin/netpath-migrate-mwan3', ['preview']).then(function (res) {
      var output = '';
      if (res.stdout) output += res.stdout;
      if (res.stderr) output += res.stderr;
      if (!output) output = _('(No output)');

      if (outputEl) {
        outputEl.textContent = output;
      }
    }).catch(function (err) {
      if (outputEl) {
        outputEl.textContent = _('Error: ') + err.message;
        outputEl.style.color = '#f48771';
      }
    });
  },

  doMigration: function () {
    var self = this;
    var outputEl = document.getElementById('migration-output');
    var btnEl = document.getElementById('btn-migrate');

    if (!confirm(_('Are you sure you want to migrate mwan3 configuration to NetPath?\n\nYour existing NetPath configuration will be backed up.'))) {
      return;
    }

    if (outputEl) {
      outputEl.style.display = 'block';
      outputEl.textContent = _('Starting migration...');
      outputEl.style.color = '#d4d4d4';
    }

    if (btnEl) {
      btnEl.disabled = true;
      btnEl.textContent = _('Migrating...');
    }

    return fs.exec('/usr/bin/netpath-migrate-mwan3', ['migrate']).then(function (res) {
      var output = '';
      if (res.stdout) output += res.stdout;
      if (res.stderr) output += res.stderr;
      if (!output) output = _('Migration completed!');

      if (outputEl) {
        outputEl.textContent = output;
        if (res.code === 0) {
          outputEl.style.color = '#89d185';
        } else {
          outputEl.style.color = '#f48771';
        }
      }

      if (btnEl) {
        btnEl.disabled = false;
        btnEl.textContent = '✓ ' + _('Migration Complete');
        btnEl.className = 'btn cbi-button cbi-button-positive';
      }

      // Reload UCI
      return uci.load('netpath');
    }).catch(function (err) {
      if (outputEl) {
        outputEl.textContent = _('Error: ') + err.message;
        outputEl.style.color = '#f48771';
      }
      if (btnEl) {
        btnEl.disabled = false;
        btnEl.textContent = _('Start Migration');
      }
    });
  },

  disableMwan3: function () {
    var outputEl = document.getElementById('migration-output');

    if (!confirm(_('Disable mwan3 service?\n\nThis will stop mwan3 and prevent it from starting on boot.'))) {
      return;
    }

    if (outputEl) {
      outputEl.style.display = 'block';
      outputEl.textContent = _('Disabling mwan3...');
    }

    return fs.exec('/bin/sh', ['-c', '/etc/init.d/mwan3 stop 2>&1; /etc/init.d/mwan3 disable 2>&1; echo "mwan3 has been disabled"']).then(function (res) {
      if (outputEl) {
        outputEl.textContent = (res.stdout || '') + (res.stderr || '') || _('mwan3 disabled');
        outputEl.style.color = '#89d185';
      }
    }).catch(function (err) {
      if (outputEl) {
        outputEl.textContent = _('Error: ') + err.message;
        outputEl.style.color = '#f48771';
      }
    });
  },

  enableNetpath: function () {
    var outputEl = document.getElementById('migration-output');

    if (outputEl) {
      outputEl.style.display = 'block';
      outputEl.textContent = _('Enabling NetPath...');
    }

    return fs.exec('/bin/sh', ['-c', '/etc/init.d/netpath enable 2>&1; /etc/init.d/netpath restart 2>&1; echo "NetPath has been enabled and started"']).then(function (res) {
      if (outputEl) {
        outputEl.textContent = (res.stdout || '') + (res.stderr || '') || _('NetPath enabled');
        outputEl.style.color = '#89d185';
      }
    }).catch(function (err) {
      if (outputEl) {
        outputEl.textContent = _('Error: ') + err.message;
        outputEl.style.color = '#f48771';
      }
    });
  },

  handleSaveApply: null,
  handleSave: null,
  handleReset: null
});
