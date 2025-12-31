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

    m = new form.Map('netpath', _('NetPath - Policies'),
      _('Policies define how traffic is distributed across members.'));

    s = m.section(form.GridSection, 'policy');
    s.addremove = true;
    s.anonymous = false;
    s.nodescriptions = true;

    o = s.option(form.DynamicList, 'members', _('Members'));
    o.load = function (section_id) {
      uci.sections('netpath', 'member').forEach(function (section) {
        this.value(section['.name']);
      }.bind(this));
      return uci.get('netpath', section_id, 'members');
    };

    o = s.option(form.ListValue, 'mode', _('Mode'));
    o.default = 'balanced';
    o.value('balanced', _('Load Balance'));
    o.value('failover', _('Failover'));
    o.value('roundrobin', _('Round Robin'));
    o.value('random', _('Random'));
    o.value('weighted', _('Weighted'));
    o.value('hash', _('Source Hash'));

    o = s.option(form.Flag, 'sticky', _('Sticky'));
    o.default = '0';
    o.description = _('Enable sticky connections for this policy');
    o.modalonly = true;

    o = s.option(form.Value, 'sticky_timeout', _('Sticky Timeout'));
    o.datatype = 'uinteger';
    o.default = '600';
    o.placeholder = '600';
    o.description = _('Timeout in seconds for sticky connections');
    o.modalonly = true;
    o.depends('sticky', '1');

    o = s.option(form.ListValue, 'last_resort', _('Last Resort'));
    o.default = 'unreachable';
    o.value('unreachable', _('Unreachable'));
    o.value('blackhole', _('Blackhole'));
    o.value('default', _('Default Route'));
    o.description = _('Action when all members are offline');
    o.modalonly = true;

    return m.render();
  }
});
