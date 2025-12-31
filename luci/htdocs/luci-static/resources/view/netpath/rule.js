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

    m = new form.Map('netpath', _('NetPath - Rules'),
      _('Rules define which traffic uses which policy.') + '<br />' +
      _('Rules are evaluated by priority (lower number = higher priority). First match wins.'));

    s = m.section(form.GridSection, 'rule');
    s.addremove = true;
    s.anonymous = false;
    s.sortable = false;  // Named sections don't support drag-sort, use priority field instead
    s.nodescriptions = true;

    // Sort sections by priority when rendering
    s.cfgsections = function () {
      var sections = uci.sections('netpath', 'rule');
      sections.sort(function (a, b) {
        var pa = parseInt(a.priority) || 100;
        var pb = parseInt(b.priority) || 100;
        return pa - pb;
      });
      return sections.map(function (s) { return s['.name']; });
    };

    // Show priority in section title
    s.sectiontitle = function (section_id) {
      var priority = uci.get('netpath', section_id, 'priority') || '100';
      return section_id + ' [' + priority + ']';
    };

    o = s.option(form.Flag, 'enabled', _('Enabled'));
    o.default = '1';
    o.editable = true;

    o = s.option(form.Value, 'priority', _('Priority'));
    o.datatype = 'uinteger';
    o.default = '100';
    o.placeholder = '100';
    o.description = _('Lower number = higher priority (evaluated first)');
    o.rmempty = false;

    o = s.option(form.Value, 'src_ip', _('Source IP'));
    o.datatype = 'ipaddr';
    o.placeholder = '0.0.0.0/0';

    o = s.option(form.Value, 'src_port', _('Source Port'));
    o.datatype = 'portrange';
    o.placeholder = '0-65535';
    o.modalonly = true;

    o = s.option(form.Value, 'dest_ip', _('Dest IP'));
    o.datatype = 'ipaddr';
    o.placeholder = '0.0.0.0/0';

    o = s.option(form.Value, 'dest_port', _('Dest Port'));
    o.datatype = 'portrange';
    o.placeholder = '0-65535';

    o = s.option(form.ListValue, 'proto', _('Protocol'));
    o.default = 'all';
    o.value('all', _('All'));
    o.value('tcp', 'TCP');
    o.value('udp', 'UDP');
    o.value('icmp', 'ICMP');

    o = s.option(form.ListValue, 'policy', _('Policy'));
    o.rmempty = false;
    o.load = function (section_id) {
      uci.sections('netpath', 'policy').forEach(function (section) {
        this.value(section['.name']);
      }.bind(this));
      return uci.get('netpath', section_id, 'policy');
    };

    o = s.option(form.Flag, 'sticky', _('Sticky'));
    o.default = '0';
    o.description = _('Enable sticky connections for this rule');
    o.modalonly = true;

    o = s.option(form.Value, 'timeout', _('Sticky Timeout'));
    o.datatype = 'uinteger';
    o.default = '600';
    o.placeholder = '600';
    o.description = _('Timeout in seconds for sticky connections');
    o.modalonly = true;
    o.depends('sticky', '1');

    o = s.option(form.Value, 'ipset', _('IPSet'));
    o.placeholder = 'myset';
    o.description = _('Match traffic using this IPSet name');
    o.modalonly = true;

    return m.render();
  }
});
