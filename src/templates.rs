pub static DEBUG_TEMPLATE_NAME: &'static str = "debug_html_template";
pub static DEBUG_HTML_TEMPLATE: &'static str = "
<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <title>IRC Server Debug Portal</title>
</head>
<body>

<div id=\"TableOfContents\">
<h2>Table Of Contents</h2>
<ul>
  <li><a href=\"#Server\">Server</a></li>
  <li><a href=\"#Channels\">Channels</a></li>
  <li><a href=\"#Users\">Users</a></li>
</ul>
</div>

<div id=\"Server\">
<h2>Server Wide State</h2>
{{#if configuration.0}}<b style=\"color:green\">Default Configuration</b>{{/if}}
<pre>
{{configuration.1}}
</pre>
</div>

<div id=\"Channels\">
<h2>Channels</h2>
<table>
  <tr>
    <th>Name</th>
    <th>Members</th>
  </tr>
  {{#each channels_to_nicks}}
  <tr>
    <td>{{@key}}</td>
    <td>
      <ul>
        {{#each this}}
        <li><a href=\"#{{this.1}}\">{{this.0}}</a></li>
        {{/each}}
      </ul>
    </td>
  </tr>
  {{/each}}
</table>
</div>

<div id=\"Users\">
<h2>Users</h2>
<table>
  <tr>
    <th>Nick</th>
    <th>User Data</th>
    <th>Channels</th>
  </tr>
  {{#each user_to_channels}}
  <tr>
    <td><a href=\"#{{this.1}}\">{{@key}}</a></td>
    <td><pre>{{this.0}}</pre></td>
    <td>
      <ul>
      {{#each this.2}}
        <li><a href=\"#{{this.1}}\">{{this.0}}</a></li>
      {{/each}}
      </ul>
    </td>
  </tr>
  {{/each}}
</table>
</div>

<div id=\"Connections\">
<h2>Connections</h2>
<table>
  <tr>
    <th>Socket</th>
    <th>Nick</th>
  </tr>
  {{#each connections}}
  <tr>
    <td>{{@key}}</td>
    <td>{{#if this.0}}<a href=\"#{{this.1.1}}\">{{this.1.0}}</a>
    {{else}}<b style=\"color:red\">Not registered.</b>{{/if}}</td>
  </tr>
  {{/each}}
</table>
</div>

</body>
</html>";


pub static RPL_WELCOME_TEMPLATE_NAME: &'static str = "rpl_welcome_template_name";
pub static RPL_WELCOME_TEMPLATE: &'static str = "Welcome to the {{network_name}} Network, {{nick}}";
#[derive(Serialize)]
pub struct Welcome<'a> {
    pub network_name: &'a str,
    pub nick: &'a str,
}

pub static RPL_YOURHOST_TEMPLATE_NAME: &'static str = "rpl_yourhost_template_name";
pub static RPL_YOURHOST_TEMPLATE: &'static str = "Your host is {{hostname}}, running version {{version}}";
#[derive(Serialize)]
pub struct YourHost<'a> {
    pub hostname: &'a str,
    pub version: &'a str,
}

pub static RPL_CREATED_TEMPLATE_NAME: &'static str = "rpl_created_template_name";
pub static RPL_CREATED_TEMPLATE: &'static str = "This server was created {{created}}";
#[derive(Serialize)]
pub struct Created<'a> {
    pub created: &'a str,
}
