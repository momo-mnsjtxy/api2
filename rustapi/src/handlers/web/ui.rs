//! Material Design 3–inspired shared UI for the console and auth surfaces.
//! Color tokens follow M3 tonal surfaces; brand seed is teal (not the default purple demo).

pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn m3_head(title: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} · 梦城API</title>
  <link rel="shortcut icon" href="https://cdn.gqink.cn/blog/favicon.ico">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Figtree:wght@400;500;600;700&family=Noto+Sans+SC:wght@400;500;700&display=swap" rel="stylesheet">
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@24,400,0,0" rel="stylesheet">
  <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.7/dist/chart.umd.min.js"></script>
  <style>{css}</style>
</head>"#,
        title = html_escape(title),
        css = M3_CSS,
    )
}

pub fn icon(name: &str) -> String {
    format!(r#"<span class="msym" aria-hidden="true">{name}</span>"#)
}

/// Authenticated app chrome: navigation rail + top app bar + main.
pub fn app_layout(
    title: &str,
    username: &str,
    email: &str,
    headimg: &str,
    active: &str,
    api_links: &str,
    body: &str,
) -> String {
    format!(
        r#"{head}
<body class="m3-app">
  <div class="shell">
    <aside class="rail" aria-label="主导航">
      <a class="rail-brand" href="/index/index/index">
        <img src="https://cdn.gqink.cn/blog/logo.svg" alt="梦城API" width="36" height="36">
        <span class="rail-brand-text">梦城API</span>
      </a>
      <nav class="rail-nav">
        <a class="rail-item {a_home}" href="/index/index/index">{ih}控制中心</a>
        <a class="rail-item {a_key}" href="/index/index/appkey">{ik}APPKEY</a>
        <a class="rail-item {a_set}" href="/index/index/setting">{is}账号设置</a>
      </nav>
      <div class="rail-apis">
        <div class="rail-apis-label">接口</div>
        <div class="rail-apis-list">{api_links}</div>
      </div>
      <a class="rail-item rail-out" href="/index/index/LoginOut">{io}退出</a>
    </aside>
    <div class="stage">
      <header class="topbar">
        <div class="topbar-text">
          <p class="eyebrow">梦城API · Console</p>
          <h1 class="page-title">{title}</h1>
        </div>
        <div class="user-chip">
          <img class="avatar" src="{headimg}" width="40" height="40" alt="">
          <div class="user-meta">
            <strong>{username}</strong>
            <span>{email}</span>
          </div>
        </div>
      </header>
      <main class="page">{body}</main>
    </div>
  </div>
</body>
</html>"#,
        head = m3_head(title),
        title = html_escape(title),
        headimg = html_escape(headimg),
        username = html_escape(username),
        email = html_escape(email),
        api_links = api_links,
        body = body,
        a_home = if active == "home" { "is-active" } else { "" },
        a_key = if active == "appkey" { "is-active" } else { "" },
        a_set = if active == "setting" {
            "is-active"
        } else {
            ""
        },
        ih = icon("dashboard"),
        ik = icon("key"),
        is = icon("manage_accounts"),
        io = icon("logout"),
    )
}

/// Auth (login / register) full-bleed composition.
pub fn auth_layout(title: &str, panel: &str) -> String {
    format!(
        r#"{head}
<body class="m3-auth">
  <div class="auth-bg" aria-hidden="true"></div>
  <main class="auth-stage">
    <section class="auth-brand">
      <img class="auth-logo" src="https://cdn.gqink.cn/blog/logo.svg" alt="梦城API" width="72" height="72">
      <h1 class="auth-name">梦城API</h1>
      <p class="auth-tag">开放能力控制台 · Material You</p>
    </section>
    <section class="auth-panel surface-xl">{panel}</section>
  </main>
  <footer class="auth-foot">© 梦城 · gqink.cn</footer>
</body>
</html>"#,
        head = m3_head(title),
        panel = panel,
    )
}

pub fn api_nav_link(keyword: &str, name: &str, active_keyword: Option<&str>) -> String {
    let active = active_keyword.map(|k| k == keyword).unwrap_or(false);
    format!(
        r#"<a class="api-link{cls}" href="/index/index/page?api={kw}">{name}</a>"#,
        cls = if active { " is-active" } else { "" },
        kw = urlencoding::encode(keyword),
        name = html_escape(name),
    )
}

pub fn field(label: &str, name: &str, input_type: &str, attrs: &str) -> String {
    format!(
        r#"<label class="field">
  <span class="field-label">{label}</span>
  <input class="field-input" type="{ty}" name="{name}" {attrs}>
</label>"#,
        label = html_escape(label),
        ty = input_type,
        name = html_escape(name),
        attrs = attrs,
    )
}

pub fn filled_button(label: &str, attrs: &str) -> String {
    format!(
        r#"<button class="btn btn-filled" type="submit" {attrs}>{label}</button>"#,
        label = html_escape(label),
        attrs = attrs,
    )
}

pub fn tonal_button(label: &str, attrs: &str) -> String {
    format!(
        r#"<button class="btn btn-tonal" type="submit" {attrs}>{label}</button>"#,
        label = html_escape(label),
        attrs = attrs,
    )
}

pub fn outlined_button_link(label: &str, href: &str) -> String {
    format!(
        r#"<a class="btn btn-outlined" href="{href}">{label}</a>"#,
        href = html_escape(href),
        label = html_escape(label),
    )
}

pub fn trend_chip(pct: f64, up_is_good: bool) -> String {
    let up = pct >= 0.0;
    let good = if up_is_good { up } else { !up };
    let cls = if good { "chip-good" } else { "chip-warn" };
    let arrow = if up { "trending_up" } else { "trending_down" };
    format!(
        r#"<span class="chip {cls}">{icon}{pct:.1}%</span>"#,
        icon = icon(arrow),
        pct = pct.abs(),
        cls = cls,
    )
}

pub fn stat_card(title: &str, value: i64, note: &str, extra: &str) -> String {
    format!(
        r#"<article class="stat-card surface-card">
  <div class="stat-head"><span class="stat-title">{title}</span>{extra}</div>
  <div class="stat-value">{value}</div>
  <p class="stat-note">{note}</p>
</article>"#,
        title = html_escape(title),
        value = value,
        note = html_escape(note),
        extra = extra,
    )
}

pub fn data_table(headers: &[&str], rows_html: &str) -> String {
    let th = headers
        .iter()
        .map(|h| format!("<th>{}</th>", html_escape(h)))
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<div class="table-wrap surface-card">
  <table class="data-table"><thead><tr>{th}</tr></thead><tbody>{rows}</tbody></table>
</div>"#,
        th = th,
        rows = rows_html,
    )
}

pub fn section_card(title: &str, action: &str, inner: &str) -> String {
    format!(
        r#"<section class="section-card surface-card">
  <div class="section-head">
    <h2 class="section-title">{title}</h2>
    <div class="section-actions">{action}</div>
  </div>
  <div class="section-body">{inner}</div>
</section>"#,
        title = html_escape(title),
        action = action,
        inner = inner,
    )
}

pub fn chart_block(canvas_id: &str, labels_json: &str, data_json: &str) -> String {
    format!(
        r#"<div class="chart-box">
  <canvas id="{id}" height="120"></canvas>
</div>
<script>
(() => {{
  const el = document.getElementById('{id}');
  if (!el || !window.Chart) return;
  const ctx = el.getContext('2d');
  const grad = ctx.createLinearGradient(0, 0, 0, el.parentElement.clientHeight || 280);
  grad.addColorStop(0, 'rgba(0, 106, 106, 0.28)');
  grad.addColorStop(1, 'rgba(0, 106, 106, 0.02)');
  new Chart(ctx, {{
    type: 'line',
    data: {{
      labels: {labels},
      datasets: [{{
        label: '调用次数',
        data: {data},
        borderColor: '#006A6A',
        backgroundColor: grad,
        fill: true,
        tension: 0.35,
        pointRadius: 4,
        pointBackgroundColor: '#006A6A',
        borderWidth: 2
      }}]
    }},
    options: {{
      responsive: true,
      maintainAspectRatio: false,
      plugins: {{ legend: {{ display: false }} }},
      scales: {{
        x: {{ grid: {{ display: false }}, ticks: {{ color: '#3F4948', font: {{ family: 'Figtree' }} }} }},
        y: {{ beginAtZero: true, grid: {{ color: 'rgba(190,201,200,0.45)' }}, ticks: {{ color: '#3F4948', precision: 0 }} }}
      }},
      animation: {{ duration: 700, easing: 'easeOutQuart' }}
    }}
  }});
}})();
</script>"#,
        id = html_escape(canvas_id),
        labels = labels_json,
        data = data_json,
    )
}

pub fn error_page(msg: &str) -> String {
    format!(
        r#"{head}
<body class="m3-auth">
  <main class="auth-stage" style="grid-template-columns:1fr;max-width:480px">
    <section class="auth-panel surface-xl">
      <h2 class="panel-title">出错了</h2>
      <p class="panel-sub">{msg}</p>
      <p style="margin-top:1.5rem">{back}</p>
    </section>
  </main>
</body></html>"#,
        head = m3_head("错误"),
        msg = html_escape(msg),
        back = outlined_button_link("返回控制中心", "/index/index/index"),
    )
}

pub fn geetest_hidden() -> &'static str {
    r#"<input type="hidden" name="geetest_challenge" value="rust-fallback">
<input type="hidden" name="geetest_validate" value="d22957a63a507af42dc95a259d8bc8f5">
<input type="hidden" name="geetest_seccode" value="rust-fallback">"#
}

const M3_CSS: &str = r#"
:root {
  --md-sys-color-primary: #006A6A;
  --md-sys-color-on-primary: #FFFFFF;
  --md-sys-color-primary-container: #9CF1F0;
  --md-sys-color-on-primary-container: #002020;
  --md-sys-color-secondary: #4A6362;
  --md-sys-color-secondary-container: #CCE8E7;
  --md-sys-color-on-secondary-container: #051F1F;
  --md-sys-color-tertiary: #4B607C;
  --md-sys-color-tertiary-container: #D3E4FF;
  --md-sys-color-surface: #F4FBFA;
  --md-sys-color-surface-container-lowest: #FFFFFF;
  --md-sys-color-surface-container-low: #EEF5F4;
  --md-sys-color-surface-container: #E8F0EF;
  --md-sys-color-surface-container-high: #E2EAE9;
  --md-sys-color-on-surface: #161D1D;
  --md-sys-color-on-surface-variant: #3F4948;
  --md-sys-color-outline: #6F7978;
  --md-sys-color-outline-variant: #BEC9C8;
  --md-sys-color-error: #BA1A1A;
  --md-sys-color-error-container: #FFDAD6;
  --md-sys-color-success: #146C2E;
  --md-sys-color-success-container: #A6F2A9;
  --md-sys-shape-sm: 8px;
  --md-sys-shape-md: 12px;
  --md-sys-shape-lg: 16px;
  --md-sys-shape-xl: 28px;
  --font: "Figtree", "Noto Sans SC", sans-serif;
  --elev-1: 0 1px 2px rgba(22, 29, 29, 0.06);
}
*, *::before, *::after { box-sizing: border-box; }
html, body { margin: 0; min-height: 100%; }
body {
  font-family: var(--font);
  color: var(--md-sys-color-on-surface);
  background: var(--md-sys-color-surface);
  -webkit-font-smoothing: antialiased;
}
a { color: var(--md-sys-color-primary); text-decoration: none; }
a:hover { text-decoration: underline; }
.msym {
  font-family: "Material Symbols Rounded";
  font-weight: 400;
  font-style: normal;
  font-size: 1.25rem;
  line-height: 1;
  display: inline-block;
  vertical-align: -0.15em;
  margin-right: 0.45rem;
  font-variation-settings: "FILL" 0, "wght" 400, "GRAD" 0, "opsz" 24;
}
.m3-app {
  background:
    radial-gradient(1200px 600px at 0% -10%, rgba(156, 241, 240, 0.55), transparent 55%),
    radial-gradient(900px 500px at 100% 0%, rgba(211, 228, 255, 0.45), transparent 50%),
    linear-gradient(180deg, #EAF6F5 0%, var(--md-sys-color-surface) 42%);
  min-height: 100vh;
}
.shell {
  display: grid;
  grid-template-columns: 248px 1fr;
  min-height: 100vh;
}
.rail {
  position: sticky;
  top: 0;
  height: 100vh;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 1rem 0.75rem 1rem 1rem;
  background: color-mix(in srgb, var(--md-sys-color-surface-container-low) 88%, transparent);
  backdrop-filter: blur(12px);
  border-right: 1px solid var(--md-sys-color-outline-variant);
  animation: rail-in 420ms ease-out both;
}
.rail-brand {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.65rem 0.75rem;
  border-radius: var(--md-sys-shape-lg);
  color: inherit;
  text-decoration: none;
  margin-bottom: 0.5rem;
}
.rail-brand:hover { text-decoration: none; background: var(--md-sys-color-surface-container); }
.rail-brand-text { font-weight: 700; font-size: 1.15rem; letter-spacing: -0.02em; }
.rail-nav { display: flex; flex-direction: column; gap: 0.25rem; }
.rail-item {
  display: flex;
  align-items: center;
  padding: 0.7rem 0.9rem;
  border-radius: 999px;
  color: var(--md-sys-color-on-surface-variant);
  font-weight: 500;
  text-decoration: none;
  transition: background 160ms ease, color 160ms ease, transform 160ms ease;
}
.rail-item:hover {
  background: var(--md-sys-color-surface-container-high);
  text-decoration: none;
  color: var(--md-sys-color-on-surface);
}
.rail-item.is-active {
  background: var(--md-sys-color-secondary-container);
  color: var(--md-sys-color-on-secondary-container);
}
.rail-item.is-active .msym { font-variation-settings: "FILL" 1, "wght" 500, "GRAD" 0, "opsz" 24; }
.rail-apis {
  flex: 1;
  min-height: 0;
  margin-top: 0.75rem;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.rail-apis-label {
  font-size: 0.7rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--md-sys-color-outline);
  padding: 0.35rem 0.9rem;
}
.rail-apis-list {
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  padding-bottom: 0.5rem;
}
.api-link {
  display: block;
  padding: 0.45rem 0.9rem;
  border-radius: var(--md-sys-shape-md);
  color: var(--md-sys-color-on-surface-variant);
  font-size: 0.88rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.api-link:hover, .api-link.is-active {
  background: var(--md-sys-color-primary-container);
  color: var(--md-sys-color-on-primary-container);
  text-decoration: none;
}
.rail-out { margin-top: auto; color: var(--md-sys-color-error); }
.stage { min-width: 0; display: flex; flex-direction: column; }
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1.25rem 1.75rem 0.5rem;
  animation: rise 480ms ease-out both;
}
.eyebrow {
  margin: 0;
  font-size: 0.75rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--md-sys-color-outline);
}
.page-title {
  margin: 0.15rem 0 0;
  font-size: clamp(1.6rem, 2.4vw, 2.1rem);
  font-weight: 700;
  letter-spacing: -0.03em;
}
.user-chip {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.4rem 0.85rem 0.4rem 0.4rem;
  border-radius: 999px;
  background: var(--md-sys-color-surface-container-lowest);
  box-shadow: var(--elev-1);
}
.avatar { border-radius: 50%; object-fit: cover; }
.user-meta { display: flex; flex-direction: column; line-height: 1.2; }
.user-meta strong { font-size: 0.92rem; }
.user-meta span {
  font-size: 0.75rem;
  color: var(--md-sys-color-on-surface-variant);
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.page {
  padding: 0.75rem 1.75rem 2.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  animation: rise 560ms 60ms ease-out both;
}
.surface-card, .surface-xl {
  background: var(--md-sys-color-surface-container-lowest);
  border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 65%, transparent);
  box-shadow: var(--elev-1);
}
.surface-card { border-radius: var(--md-sys-shape-lg); }
.surface-xl { border-radius: var(--md-sys-shape-xl); }
.stat-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.9rem;
}
.stat-card {
  padding: 1.1rem 1.2rem 1rem;
  transition: transform 180ms ease, box-shadow 180ms ease;
}
.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(22, 29, 29, 0.06);
}
.stat-head { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; }
.stat-title { color: var(--md-sys-color-on-surface-variant); font-size: 0.88rem; font-weight: 500; }
.stat-value {
  margin: 0.55rem 0 0.35rem;
  font-size: 2rem;
  font-weight: 700;
  letter-spacing: -0.04em;
  font-variant-numeric: tabular-nums;
}
.stat-note { margin: 0; color: var(--md-sys-color-outline); font-size: 0.8rem; }
.chip {
  display: inline-flex;
  align-items: center;
  gap: 0.1rem;
  padding: 0.2rem 0.55rem;
  border-radius: var(--md-sys-shape-sm);
  font-size: 0.72rem;
  font-weight: 600;
}
.chip .msym { font-size: 0.95rem; margin-right: 0.1rem; }
.chip-good { background: var(--md-sys-color-success-container); color: var(--md-sys-color-success); }
.chip-warn { background: var(--md-sys-color-error-container); color: var(--md-sys-color-error); }
.section-card { padding: 1.15rem 1.25rem 1.25rem; }
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 1rem;
}
.section-title { margin: 0; font-size: 1.05rem; font-weight: 650; letter-spacing: -0.02em; }
.section-actions { display: flex; gap: 0.5rem; flex-wrap: wrap; }
.chart-box { height: 280px; position: relative; }
.table-wrap { overflow: auto; }
.data-table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
.data-table th, .data-table td {
  padding: 0.85rem 1rem;
  text-align: left;
  border-bottom: 1px solid var(--md-sys-color-outline-variant);
}
.data-table th {
  color: var(--md-sys-color-on-surface-variant);
  font-weight: 600;
  font-size: 0.78rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  background: var(--md-sys-color-surface-container-low);
}
.data-table tbody tr { transition: background 120ms ease; }
.data-table tbody tr:hover { background: var(--md-sys-color-surface-container-low); }
.data-table td a { font-weight: 600; }
.panel-title { margin: 0 0 0.35rem; font-size: 1.45rem; font-weight: 700; letter-spacing: -0.03em; }
.panel-sub { margin: 0 0 1.25rem; color: var(--md-sys-color-on-surface-variant); }
.form-stack { display: flex; flex-direction: column; gap: 0.85rem; }
.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
}
.field { display: flex; flex-direction: column; gap: 0.35rem; }
.field-label {
  font-size: 0.78rem;
  font-weight: 600;
  color: var(--md-sys-color-on-surface-variant);
  letter-spacing: 0.02em;
}
.field-input, .field-ro {
  appearance: none;
  width: 100%;
  border: 1px solid var(--md-sys-color-outline);
  background: var(--md-sys-color-surface-container-lowest);
  border-radius: var(--md-sys-shape-sm) var(--md-sys-shape-sm) 0 0;
  border-bottom-width: 2px;
  padding: 0.85rem 0.95rem;
  font: inherit;
  color: inherit;
  transition: border-color 140ms ease, background 140ms ease;
}
.field-input:focus {
  outline: none;
  border-color: var(--md-sys-color-primary);
  background: var(--md-sys-color-surface-container-low);
}
.field-ro {
  background: var(--md-sys-color-surface-container);
  border-style: dashed;
  color: var(--md-sys-color-on-surface-variant);
}
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  min-height: 2.5rem;
  padding: 0.55rem 1.25rem;
  border-radius: 999px;
  border: none;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
  text-decoration: none !important;
  transition: transform 140ms ease, box-shadow 140ms ease, background 140ms ease;
}
.btn:hover { transform: translateY(-1px); }
.btn-filled {
  background: var(--md-sys-color-primary);
  color: var(--md-sys-color-on-primary);
  box-shadow: var(--elev-1);
}
.btn-tonal {
  background: var(--md-sys-color-secondary-container);
  color: var(--md-sys-color-on-secondary-container);
}
.btn-outlined {
  background: transparent;
  color: var(--md-sys-color-primary);
  border: 1px solid var(--md-sys-color-outline);
}
.btn-block { width: 100%; }
.support-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 0.75rem;
}
.support-pill {
  text-align: center;
  padding: 0.9rem;
  border-radius: var(--md-sys-shape-md);
  background: var(--md-sys-color-surface-container);
}
.support-pill strong {
  display: block;
  font-size: 0.8rem;
  color: var(--md-sys-color-outline);
  margin-bottom: 0.25rem;
}
.support-ok { color: var(--md-sys-color-success); font-weight: 700; }
.support-no { color: var(--md-sys-color-outline); }
.lead { margin: 0; color: var(--md-sys-color-on-surface-variant); max-width: 62ch; }
.m3-auth {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}
.auth-bg {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse 80% 60% at 15% 20%, rgba(0, 106, 106, 0.22), transparent 55%),
    radial-gradient(ellipse 70% 50% at 85% 10%, rgba(75, 96, 124, 0.18), transparent 50%),
    linear-gradient(160deg, #D8F3F2 0%, #EEF4FB 45%, #F7FBFA 100%);
  animation: bg-shift 12s ease-in-out infinite alternate;
}
.auth-stage {
  position: relative;
  z-index: 1;
  flex: 1;
  display: grid;
  grid-template-columns: 1.05fr 0.95fr;
  gap: 2rem;
  align-items: center;
  max-width: 1040px;
  width: 100%;
  margin: 0 auto;
  padding: 2.5rem 1.5rem;
}
.auth-brand { animation: rise 520ms ease-out both; }
.auth-logo {
  display: block;
  margin-bottom: 1.25rem;
  filter: drop-shadow(0 8px 18px rgba(0, 106, 106, 0.18));
}
.auth-name {
  margin: 0;
  font-size: clamp(2.6rem, 6vw, 3.6rem);
  font-weight: 700;
  letter-spacing: -0.045em;
  line-height: 1.05;
}
.auth-tag {
  margin: 0.85rem 0 0;
  font-size: 1.05rem;
  color: var(--md-sys-color-on-surface-variant);
  max-width: 28ch;
}
.auth-panel {
  padding: 1.75rem 1.6rem 1.6rem;
  animation: rise 620ms 80ms ease-out both;
}
.auth-foot {
  position: relative;
  z-index: 1;
  text-align: center;
  padding: 1rem;
  color: var(--md-sys-color-outline);
  font-size: 0.85rem;
}
.auth-links {
  margin-top: 1.15rem;
  text-align: center;
  color: var(--md-sys-color-on-surface-variant);
  font-size: 0.92rem;
}
.split-actions { display: flex; gap: 0.6rem; flex-wrap: wrap; align-items: center; }
@keyframes rise {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: none; }
}
@keyframes rail-in {
  from { opacity: 0; transform: translateX(-10px); }
  to { opacity: 1; transform: none; }
}
@keyframes bg-shift {
  from { filter: hue-rotate(0deg) saturate(1); }
  to { filter: hue-rotate(8deg) saturate(1.05); }
}
@media (max-width: 960px) {
  .shell { grid-template-columns: 1fr; }
  .rail {
    position: sticky;
    top: 0;
    height: auto;
    z-index: 20;
    flex-direction: row;
    flex-wrap: wrap;
    align-items: center;
    border-right: none;
    border-bottom: 1px solid var(--md-sys-color-outline-variant);
    padding: 0.65rem 0.75rem;
  }
  .rail-brand-text { display: none; }
  .rail-nav { flex-direction: row; flex-wrap: wrap; }
  .rail-apis { display: none; }
  .rail-out { margin-top: 0; }
  .stat-grid, .form-grid, .support-grid { grid-template-columns: 1fr; }
  .auth-stage { grid-template-columns: 1fr; padding-top: 2rem; }
  .auth-brand { text-align: center; }
  .auth-logo { margin-left: auto; margin-right: auto; }
  .auth-tag { margin-left: auto; margin-right: auto; }
  .user-meta span { max-width: 110px; }
}
"#;
