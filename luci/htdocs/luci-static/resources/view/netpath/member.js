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

    m = new form.Map('netpath', _('NetPath - Members'),
      _('Members are used to define interface properties within policies.'));

    s = m.section(form.GridSection, 'member');
    s.addremove = true;
    s.anonymous = false;
    s.nodescriptions = true;

    o = s.option(form.ListValue, 'interface', _('Interface'));
    o.load = function (section_id) {
      uci.sections('netpath', 'interface').forEach(function (section) {
        this.value(section['.name']);
      }.bind(this));
      return uci.get('netpath', section_id, 'interface');
    };

    o = s.option(form.Value, 'metric', _('Metric'));
    o.datatype = 'uinteger';
    o.default = '1';
    o.placeholder = '1';

    o = s.option(form.Value, 'weight', _('Weight'));
    o.datatype = 'range(1, 100)';
    o.default = '1';
    o.placeholder = '1';

    return m.render();
  }
});
