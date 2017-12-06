pub static DEBUG_TEMPLATE_NAME: &'static str = "debug_html_template";
pub static DEBUG_HTML_TEMPLATE: &'static str = "
<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <title>IRC Server State</title>
</head>
<body>
<h2>Server</h2>
<pre>
{{configuration}}
</pre>
<table>
  <tr>
    <th>Known Nicks</th>
  </tr>
  {{#each nick_to_connections}}
  <tr>
    <td><a href=\"#{{this}}\">{{@key}}</a></td>
  </tr>
  {{/each}}
</table>
<table>
  <tr>
    <th>Socket Pair</th>
    <th>Connection Data</th>
  </tr>
{{#each connections}}
  <tr{{#if this.0.0}} id=\"{{this.0.1}}\"{{/if}}>
    <td>{{@key}}</td>
    <td>
    <pre>
{{this.1}}
    </pre>
    </td>
  </tr>
{{/each}}
</table>

</body>
</html>";

pub static RPL_WELCOME_TEMPLATE_NAME: &'static str = "rpl_welcome_template_name";
pub static RPL_WELCOME_TEMPLATE: &'static str = "Welcome to the {{network_name}} Network, {{nick}}";

pub static RPL_YOURHOST_TEMPLATE_NAME: &'static str = "rpl_yourhost_template_name";
pub static RPL_YOURHOST_TEMPLATE: &'static str = "Your host is {{hostname}}, running version {{version}}";

pub static RPL_CREATED_TEMPLATE_NAME: &'static str = "rpl_created_template_name";
pub static RPL_CREATED_TEMPLATE: &'static str = "This server was created {{created}}";