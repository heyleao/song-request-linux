use axum::response::Html;

pub async fn page() -> Html<&'static str> {
    Html(
        r##"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Song Request Linux</title>
  <link rel="icon" type="image/png" href="/favicon.png">
  <style>
    :root {
      color-scheme: dark;
      --bg: #0b0f14;
      --chrome: #101720;
      --surface: #151b24;
      --surface-2: #1d2530;
      --surface-3: #0f151d;
      --text: #f4f7fb;
      --muted: #9aa6b5;
      --soft: #c7d0dc;
      --line: #2c3442;
      --line-2: #3a4557;
      --ok: #22c55e;
      --warn: #f7b955;
      --bad: #ff7373;
      --action: #5aa9ff;
      --action-2: #7dd3fc;
      --focus: #f7d774;
      --shadow: 0 18px 42px rgba(0, 0, 0, .28);
      font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    * { box-sizing: border-box; }
    html { scroll-behavior: smooth; }
    body {
      margin: 0;
      background: linear-gradient(180deg, #0f151d 0%, var(--bg) 34%, #070b10 100%);
      color: var(--text);
      font-size: 14px;
      letter-spacing: 0;
    }
    a { color: var(--action); }
    a.skip-link {
      position: absolute;
      left: 12px;
      top: -48px;
      z-index: 10;
      padding: 10px 12px;
      background: var(--focus);
      color: #15120a;
      border-radius: 6px;
      font-weight: 800;
    }
    a.skip-link:focus { top: 12px; }
    header {
      position: sticky;
      top: 0;
      z-index: 20;
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 14px;
      min-height: 68px;
      padding: 12px 18px;
      border-bottom: 1px solid rgba(90, 169, 255, .16);
      background: rgba(13, 20, 29, .94);
      backdrop-filter: blur(14px);
      box-shadow: 0 12px 28px rgba(0, 0, 0, .18);
    }
    h1, h2, h3, p { margin: 0; }
    h1 { font-size: 20px; line-height: 1.2; font-weight: 900; }
    h2 { font-size: 16px; line-height: 1.25; font-weight: 900; }
    h3 { font-size: 12px; color: var(--muted); text-transform: uppercase; letter-spacing: .04em; }
    main { padding: 18px; }
    .app-shell {
      display: grid;
      grid-template-columns: 284px minmax(0, 1fr);
      min-height: 100dvh;
      width: 100%;
      overflow-x: hidden;
      background: linear-gradient(to right, #0b1118 0 284px, transparent 284px);
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 12px;
      min-width: 0;
    }
    .brand-mark {
      width: 68px;
      height: 68px;
      border-radius: 8px;
      border: 1px solid rgba(125, 211, 252, .34);
      background: #07111a;
      object-fit: cover;
      flex: 0 0 auto;
      box-shadow: 0 14px 28px rgba(0, 0, 0, .28);
    }
    .brand span { display: block; color: var(--muted); font-size: 12px; margin-top: 2px; }
    .sidebar {
      position: sticky;
      top: 0;
      align-self: stretch;
      min-height: 100dvh;
      padding: 18px 16px;
      border-right: 1px solid rgba(90, 169, 255, .14);
      background: linear-gradient(180deg, #111a24 0%, #0b1118 100%);
      display: grid;
      grid-template-rows: auto 1fr;
      gap: 18px;
    }
    .nav-section { display: grid; gap: 7px; }
    .top-status, .tabs, .actions, .toolbar, .inline-status, .provider-options {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 8px;
    }
    .top-status {
      justify-content: flex-end;
      min-width: 360px;
    }
    .pill {
      display: inline-flex;
      align-items: center;
      gap: 7px;
      min-height: 30px;
      padding: 5px 9px;
      border: 1px solid var(--line);
      border-radius: 999px;
      background: rgba(29, 37, 48, .9);
      color: var(--muted);
      font-size: 13px;
      white-space: nowrap;
      min-width: max-content;
    }
    .top-status .pill { max-width: none; }
    .pill strong { color: var(--text); }
    .pill.compact { min-height: 26px; padding: 4px 8px; font-size: 12px; }
    .dot { width: 8px; height: 8px; border-radius: 999px; background: var(--warn); flex: 0 0 auto; }
    .dot.ok { background: var(--ok); }
    .dot.bad { background: var(--bad); }
    .tabs {
      display: grid;
      align-content: start;
      gap: 7px;
    }
    .tab-button {
      width: 100%;
      min-height: 40px;
      border: 1px solid transparent;
      border-radius: 8px;
      background: transparent;
      color: var(--muted);
      padding: 9px 10px;
      font-weight: 800;
      cursor: pointer;
      text-decoration: none;
      display: flex;
      align-items: center;
      justify-content: flex-start;
      gap: 10px;
      transition: border-color .18s ease, background .18s ease, color .18s ease;
    }
    .tab-button:hover, .tab-button.active {
      border-color: rgba(90, 169, 255, .38);
      background: rgba(29, 37, 48, .95);
      color: var(--text);
    }
    .nav-icon {
      width: 20px;
      height: 20px;
      display: inline-grid;
      place-items: center;
      color: var(--soft);
      flex: 0 0 auto;
    }
    .nav-icon svg { width: 18px; height: 18px; stroke: currentColor; stroke-width: 2; fill: none; stroke-linecap: round; stroke-linejoin: round; }
    .tab { display: none; }
    .tab.active { display: block; }
    .content {
      min-width: 0;
      display: grid;
      grid-template-rows: auto 1fr;
    }
    .page-title {
      display: grid;
      gap: 4px;
      min-width: 220px;
    }
    .page-title p { color: var(--muted); line-height: 1.35; }
    .grid-main {
      display: grid;
      grid-template-columns: minmax(390px, 1.08fr) minmax(360px, .92fr);
      gap: 14px;
      align-items: start;
    }
    .grid-config {
      display: grid;
      grid-template-columns: repeat(12, minmax(0, 1fr));
      gap: 16px;
      align-items: start;
    }
    .setup-card {
      grid-column: span 4;
      display: grid;
      gap: 12px;
      align-content: start;
    }
    .setup-card.wide { grid-column: span 8; }
    .setup-card.full { grid-column: 1 / -1; }
    .setup-card h2 { margin-bottom: 2px; }
    .form-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 12px;
      align-items: start;
    }
    .form-grid.single { grid-template-columns: 1fr; }
    .form-grid.compact { grid-template-columns: repeat(4, minmax(0, 1fr)); }
    .form-grid .full, label.full, .full-row { grid-column: 1 / -1; }
    .card-actions {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
      margin-top: 2px;
    }
    .setup-flow {
      display: grid;
      gap: 16px;
      max-width: 1160px;
    }
    .setup-step {
      display: grid;
      grid-template-columns: 54px minmax(0, 1fr);
      gap: 14px;
      align-items: start;
      background: var(--surface);
    }
    .setup-step.provider-hidden { display: none; }
    .step-number {
      width: 40px;
      height: 40px;
      border-radius: 999px;
      display: inline-grid;
      place-items: center;
      background: var(--action);
      color: #06111d;
      font-weight: 950;
      box-shadow: 0 10px 24px rgba(90, 169, 255, .18);
    }
    .step-body {
      display: grid;
      gap: 12px;
      min-width: 0;
    }
    .step-head {
      display: flex;
      flex-wrap: wrap;
      align-items: start;
      justify-content: space-between;
      gap: 10px;
    }
    .step-copy {
      color: var(--muted);
      line-height: 1.45;
      max-width: 760px;
    }
    .setup-quick-list {
      display: grid;
      gap: 7px;
      color: var(--muted);
      font-size: 13px;
      line-height: 1.45;
    }
    .setup-quick-list span {
      display: block;
      padding-left: 18px;
      position: relative;
    }
    .setup-quick-list span::before {
      content: "";
      position: absolute;
      left: 3px;
      top: .62em;
      width: 6px;
      height: 6px;
      border-radius: 999px;
      background: var(--action-2);
    }
    .setup-inline-link {
      display: inline-flex;
      align-items: center;
      min-height: 34px;
      border: 1px solid var(--line);
      border-radius: 6px;
      padding: 7px 10px;
      background: var(--surface-2);
      color: var(--text);
      font-weight: 800;
      text-decoration: none;
    }
    .setup-inline-link:hover { border-color: var(--action); }
    .advanced-panel {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface);
      padding: 0;
      overflow: hidden;
    }
    .advanced-panel summary {
      cursor: pointer;
      padding: 14px;
      font-weight: 900;
      color: var(--text);
      list-style: none;
    }
    .advanced-panel summary::-webkit-details-marker { display: none; }
    .advanced-panel summary::after {
      content: "+";
      float: right;
      color: var(--action-2);
      font-size: 18px;
      line-height: 1;
    }
    .advanced-panel[open] summary::after { content: "-"; }
    .advanced-content {
      display: grid;
      gap: 14px;
      padding: 0 14px 14px;
    }
    .setup-save-bar {
      position: sticky;
      bottom: 12px;
      z-index: 8;
      display: grid;
      grid-template-columns: minmax(220px, 1fr) auto;
      gap: 10px 14px;
      align-items: center;
      background: rgba(21, 27, 36, .96);
      backdrop-filter: blur(10px);
    }
    .setup-save-bar .diagnostics,
    .setup-save-bar .message { grid-column: 1 / -1; }
    .setup-save-bar .diagnostics {
      max-height: 150px;
      overflow: auto;
    }
    .setup-save-bar {
      border-color: rgba(247, 185, 85, .38);
      box-shadow: 0 18px 42px rgba(0, 0, 0, .28);
    }
    .setup-save-bar h2::after {
      content: " sempre visível";
      color: var(--warn);
      font-size: 12px;
      font-weight: 800;
      margin-left: 8px;
    }
    .unsaved-banner {
      position: fixed;
      left: max(18px, env(safe-area-inset-left));
      right: max(18px, env(safe-area-inset-right));
      bottom: max(18px, env(safe-area-inset-bottom));
      z-index: 70;
      display: grid;
      grid-template-columns: minmax(220px, 1fr) auto;
      gap: 12px;
      align-items: center;
      padding: 12px;
      border: 1px solid rgba(247, 185, 85, .58);
      border-radius: 8px;
      background: rgba(16, 23, 32, .98);
      box-shadow: 0 20px 46px rgba(0, 0, 0, .42);
      backdrop-filter: blur(14px);
    }
    .unsaved-banner[hidden] { display: none; }
    .unsaved-banner strong { color: var(--warn); }
    .unsaved-banner p { color: var(--muted); line-height: 1.35; margin-top: 2px; }
    .unsaved-actions {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      justify-content: flex-end;
    }
    .tab-button.dirty::after {
      content: "";
      width: 8px;
      height: 8px;
      border-radius: 999px;
      background: var(--warn);
      margin-left: auto;
      box-shadow: 0 0 0 3px rgba(247, 185, 85, .16);
    }
    .grid-logs {
      display: grid;
      grid-template-columns: minmax(360px, 1fr) minmax(320px, 420px);
      gap: 14px;
      align-items: start;
    }
    section {
      background: rgba(21, 27, 36, .94);
      border: 1px solid rgba(90, 169, 255, .16);
      border-radius: 8px;
      padding: 14px;
      box-shadow: 0 16px 34px rgba(0, 0, 0, .2);
      min-width: 0;
    }
    .stack { display: grid; gap: 14px; }
    .substack { display: grid; gap: 10px; }
    .toolbar {
      justify-content: space-between;
      margin-bottom: 10px;
    }
    .toolbar h2 { margin-right: auto; }
    .status-board {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 8px;
    }
    .status-card, .current, .queue-item, .event-row, .diagnostic-row, .endpoint-row {
      border: 1px solid rgba(154, 166, 181, .18);
      border-radius: 8px;
      background: rgba(29, 37, 48, .82);
      padding: 10px;
    }
    .status-card { display: grid; gap: 6px; min-height: 76px; }
    .status-card span { color: var(--muted); font-size: 12px; }
    .status-card strong { font-size: 15px; overflow-wrap: anywhere; }
    .current {
      display: grid;
      gap: 6px;
      min-height: 112px;
      border-color: rgba(90, 169, 255, .26);
      background: linear-gradient(180deg, #233044 0%, #151d28 100%);
    }
    .song-title { font-size: 20px; font-weight: 900; overflow-wrap: anywhere; line-height: 1.25; }
    .song-meta { color: var(--muted); overflow-wrap: anywhere; }
    .provider-card {
      display: grid;
      gap: 12px;
      border-color: rgba(125, 211, 252, .18);
      background: linear-gradient(180deg, #121d29 0%, #101820 100%);
    }
    .provider-options {
      align-items: stretch;
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
    .provider-option {
      min-height: 58px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface-2);
      padding: 12px;
      display: grid;
      align-content: center;
      cursor: pointer;
    }
    .provider-option.active {
      border-color: rgba(34, 197, 94, .75);
      background: linear-gradient(180deg, rgba(34, 197, 94, .14), rgba(29, 37, 48, 1));
    }
    .provider-option strong { font-size: 15px; }
    .provider-option span {
      color: var(--muted);
      font-size: 12px;
      line-height: 1.35;
    }
    .player-config-card {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface-2);
      padding: 12px;
      display: grid;
      gap: 10px;
    }
    .player-config-card h3 {
      margin: 0;
      font-size: 15px;
    }
    .player-config-card p {
      margin: 0;
      color: var(--muted);
      font-size: 13px;
      line-height: 1.45;
    }
    .player-config-card.provider-hidden { display: none; }
    .provider-exclusive-note {
      color: var(--muted);
      font-size: 13px;
    }
    form { display: grid; gap: 14px; }
    label {
      display: grid;
      gap: 6px;
      color: var(--muted);
      font-size: 13px;
      font-weight: 700;
      min-width: 0;
    }
    input, select {
      width: 100%;
      min-height: 40px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--surface-3);
      color: var(--text);
      padding: 8px 10px;
      font: inherit;
    }
    input[type="checkbox"] {
      width: auto;
      min-height: auto;
      accent-color: var(--action);
    }
    .language-select {
      width: 72px;
      min-width: 72px;
      flex: 0 0 72px;
      min-height: 34px;
      padding: 5px 28px 5px 10px;
      font-size: 13px;
      font-weight: 800;
    }
    input:focus, select:focus, button:focus-visible, a:focus-visible {
      outline: 2px solid var(--focus);
      outline-offset: 2px;
    }
    button, a.button, a.secondary {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 38px;
      border: 1px solid #4b92d8;
      border-radius: 6px;
      background: var(--action);
      color: #07111f;
      padding: 8px 12px;
      font-weight: 900;
      text-decoration: none;
      cursor: pointer;
      transition: border-color .18s ease, background .18s ease, color .18s ease;
    }
    button:hover, a.button:hover, a.secondary:hover { background: #83c4ff; }
    button.secondary, a.secondary {
      border-color: var(--line);
      background: var(--surface-2);
      color: var(--text);
    }
    button.secondary:hover, a.secondary:hover { border-color: var(--action); }
    button.danger {
      border-color: #be4b4b;
      background: #ff7b7b;
      color: #210606;
    }
    button:disabled {
      opacity: .55;
      cursor: not-allowed;
    }
    body.stale-instance .content {
      filter: saturate(.65);
    }
    body.stale-instance main,
    body.stale-instance header .top-status {
      pointer-events: none;
    }
    .instance-notice {
      position: fixed;
      inset: 0;
      z-index: 80;
      display: grid;
      place-items: center;
      padding: 18px;
      background: rgba(3, 7, 12, .72);
      backdrop-filter: blur(8px);
    }
    .instance-notice[hidden] { display: none; }
    .instance-dialog {
      width: min(520px, 100%);
      border: 1px solid var(--line-2);
      border-radius: 8px;
      background: var(--surface);
      box-shadow: var(--shadow);
      padding: 18px;
      display: grid;
      gap: 12px;
    }
    .instance-dialog p {
      color: var(--muted);
      line-height: 1.45;
    }
    .volume-control {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      min-height: 38px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--surface-2);
      padding: 4px;
    }
    .volume-readout {
      min-width: 108px;
      text-align: center;
      color: var(--text);
      font-weight: 900;
      white-space: nowrap;
    }
    .icon-button {
      width: 34px;
      min-width: 34px;
      padding: 0;
      font-size: 18px;
      line-height: 1;
    }
    .setup-callout {
      display: grid;
      grid-template-columns: minmax(260px, .8fr) minmax(320px, 1.2fr);
      gap: 14px;
      align-items: start;
      background: linear-gradient(180deg, #182130 0%, #111922 100%);
    }
    .setup-callout h2 span { color: var(--action-2); }
    .setup-callout p { color: var(--muted); line-height: 1.45; margin-top: 6px; }
    .requirement-list { display: grid; gap: 8px; }
    .requirement-row {
      display: grid;
      grid-template-columns: 92px minmax(0, 1fr);
      gap: 10px;
      align-items: start;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface-2);
      padding: 10px;
    }
    .requirement-row strong { font-size: 12px; text-transform: uppercase; letter-spacing: .04em; }
    .requirement-row span { color: var(--muted); line-height: 1.4; overflow-wrap: anywhere; }
    .requirement-row.ok strong { color: var(--ok); }
    .requirement-row.warn strong { color: var(--warn); }
    .requirement-row.bad strong { color: var(--bad); }
    .field-note {
      color: var(--muted);
      font-size: 12px;
      line-height: 1.4;
      margin-top: -2px;
    }
    .command-access-grid, .command-card-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
      gap: 10px;
    }
    .command-card {
      display: grid;
      gap: 10px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface-2);
      padding: 12px;
    }
    .command-card h3 {
      color: var(--text);
      font-size: 14px;
      text-transform: none;
      letter-spacing: 0;
    }
    .command-card p { color: var(--muted); font-size: 12px; line-height: 1.35; }
    .setup-actions {
      padding-top: 4px;
      align-items: stretch;
    }
    .queue, .events, .diagnostics, .endpoints { display: grid; gap: 8px; }
    .queue-item, .event-row { display: grid; gap: 4px; overflow-wrap: anywhere; }
    .queue-item {
      grid-template-columns: minmax(0, 1fr) auto;
      align-items: center;
    }
    .queue-item .queue-meta { grid-column: 1 / -1; color: var(--muted); }
    .queue-item strong { font-size: 15px; }
    .remove-queue-item {
      width: 30px;
      min-width: 30px;
      min-height: 30px;
      padding: 0;
      color: #ffd7d7;
    }
    .queue-persistence {
      display: grid;
      gap: 3px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface-2);
      padding: 8px 10px;
      margin-bottom: 8px;
      color: var(--muted);
      font-size: 12px;
      line-height: 1.35;
    }
    .queue-persistence strong { color: var(--ok); }
    .queue-persistence code {
      color: var(--soft);
      overflow-wrap: anywhere;
      white-space: normal;
    }
    .event-row strong { font-size: 12px; color: var(--ok); text-transform: uppercase; }
    .event-row.error strong { color: var(--bad); }
    .event-row.player strong { color: var(--action); }
    .event-row.volume strong { color: var(--warn); }
    .diagnostic-row, .endpoint-row {
      display: grid;
      grid-template-columns: minmax(150px, 1fr) minmax(140px, auto);
      gap: 10px;
      align-items: center;
      color: var(--muted);
      font-size: 13px;
    }
    .diagnostic-row code, .endpoint-row code { text-align: right; }
    .endpoint-row a { font-weight: 800; text-decoration: none; }
    .endpoint-row.with-action { grid-template-columns: minmax(110px, .45fr) minmax(0, 1fr) 36px auto; }
    .endpoint-row.with-action a.secondary { min-width: 58px; min-height: 34px; padding: 6px 8px; }
    .copy-button { min-height: 34px; }
    .copy-button svg { width: 17px; height: 17px; stroke: currentColor; stroke-width: 2; fill: none; }
    .copy-button.copied { border-color: var(--ok); color: var(--ok); }
    .obs-size-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 8px;
    }
    .obs-size-card {
      border: 1px solid rgba(154, 166, 181, .18);
      border-radius: 8px;
      background: rgba(10, 16, 23, .42);
      padding: 10px;
      min-width: 0;
    }
    .obs-size-card strong {
      display: block;
      color: var(--text);
      font-size: 18px;
      line-height: 1.15;
    }
    .obs-size-card span {
      display: block;
      color: var(--muted);
      font-size: 12px;
      margin-top: 4px;
    }
    .endpoint-description {
      color: var(--muted);
      font-size: 12px;
      line-height: 1.4;
      margin-top: -4px;
      padding: 0 2px 4px;
    }
    code { color: var(--ok); overflow-wrap: anywhere; }
    .message {
      min-height: 20px;
      color: var(--muted);
      font-size: 13px;
      overflow-wrap: anywhere;
    }
    .message.error { color: var(--bad); }
    .hint { color: var(--muted); font-size: 13px; line-height: 1.45; }
    .muted { color: var(--muted); }
    .divider {
      height: 1px;
      background: var(--line);
      margin: 2px 0;
    }
    @media (prefers-reduced-motion: reduce) {
      * { transition: none !important; }
    }
    @media (max-width: 1100px) {
      .app-shell { grid-template-columns: 1fr; background: transparent; }
      .sidebar {
        position: static;
        min-height: auto;
        height: auto;
        width: 100%;
        border-right: 0;
        border-bottom: 1px solid var(--line);
      }
      .tabs { grid-template-columns: repeat(3, minmax(0, 1fr)); }
      .setup-card, .setup-card.wide { grid-column: span 6; }
      .form-grid.compact { grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .status-board { grid-template-columns: 1fr 1fr; }
    }
    @media (max-width: 880px) {
      header, .grid-main, .grid-logs, .setup-callout { grid-template-columns: 1fr; }
      .setup-card, .setup-card.wide { grid-column: 1 / -1; }
      .form-grid { grid-template-columns: 1fr; }
      .setup-step { grid-template-columns: 1fr; }
      .step-number { width: 34px; height: 34px; }
      .setup-save-bar { grid-template-columns: 1fr; position: static; }
      .unsaved-banner { grid-template-columns: 1fr; left: 10px; right: 10px; bottom: 10px; }
      .unsaved-actions { justify-content: stretch; }
      header { align-items: flex-start; }
      .page-title, .top-status { width: 100%; min-width: 0; }
      .top-status { justify-content: flex-start; }
      .top-status .pill { max-width: 100%; }
      main { padding: 10px; }
      .toolbar { align-items: flex-start; }
    }
    @media (max-width: 520px) {
      .tabs { grid-template-columns: 1fr; }
      .provider-options { grid-template-columns: 1fr; }
      .status-board { grid-template-columns: 1fr; }
      .toolbar { display: grid; grid-template-columns: 1fr; }
      .obs-size-grid { grid-template-columns: 1fr; }
      .diagnostic-row, .endpoint-row, .endpoint-row.with-action { grid-template-columns: 1fr; }
      .form-grid.compact { grid-template-columns: 1fr; }
      .diagnostic-row code, .endpoint-row code { text-align: left; }
      .top-status { display: flex; align-items: center; }
      .pill { width: 100%; white-space: normal; justify-content: flex-start; }
      .language-select { width: 72px; min-width: 72px; flex-basis: 72px; }
      .top-status > button { width: auto; }
      button, a.button, a.secondary { width: 100%; }
      .actions { align-items: stretch; }
    }
  </style>
</head>
<body>
  <a class="skip-link" href="#main">Ir para o painel</a>
  <div class="instance-notice" id="instance-notice" hidden role="dialog" aria-modal="true" aria-labelledby="instance-title">
    <div class="instance-dialog">
      <h2 id="instance-title">Outra aba assumiu o painel</h2>
      <p>Uma nova aba do Song Request Linux foi aberta e virou a aba ativa. Para evitar comandos duplicados, esta aba ficou em espera.</p>
      <div class="actions">
        <button id="instance-takeover" type="button">Usar esta aba</button>
        <button class="secondary" id="instance-close" type="button">Fechar esta aba</button>
      </div>
      <p class="hint" id="instance-message"></p>
    </div>
  </div>
  <div class="unsaved-banner" id="unsaved-banner" hidden role="status" aria-live="polite">
    <div>
      <strong>Alterações não salvas</strong>
      <p>Salve a configuração antes de trocar de tela, fechar o app ou iniciar a live.</p>
    </div>
    <div class="unsaved-actions">
      <button id="global-save-setup" type="button">Salvar configuração</button>
      <button class="secondary" id="discard-setup" type="button">Descartar</button>
    </div>
  </div>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <img class="brand-mark" src="/assets/logo-srl.png" alt="SRL">
        <div>
          <h1>Song Request Linux</h1>
          <span>Controle de músicas para live</span>
        </div>
      </div>
      <nav class="tabs" aria-label="Seções">
        <button class="tab-button active" data-tab="operation-tab" type="button"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="M4 13h5l2-7 4 14 2-7h3"/></svg></span>Operação</button>
        <button class="tab-button" data-tab="setup-tab" id="setup-tab-button" type="button"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z"/><path d="M19.4 15a1.8 1.8 0 0 0 .36 1.98l.03.03a2 2 0 1 1-2.83 2.83l-.03-.03A1.8 1.8 0 0 0 15 19.4a1.8 1.8 0 0 0-1 .6l-.02.02a2 2 0 1 1-3.96 0L10 20a1.8 1.8 0 0 0-1-.6 1.8 1.8 0 0 0-1.98.36l-.03.03a2 2 0 1 1-2.83-2.83l.03-.03A1.8 1.8 0 0 0 4.6 15a1.8 1.8 0 0 0-.6-1l-.02-.02a2 2 0 1 1 0-3.96L4 10a1.8 1.8 0 0 0 .6-1 1.8 1.8 0 0 0-.36-1.98l-.03-.03a2 2 0 1 1 2.83-2.83l.03.03A1.8 1.8 0 0 0 9 4.6a1.8 1.8 0 0 0 1-.6l.02-.02a2 2 0 1 1 3.96 0L14 4a1.8 1.8 0 0 0 1 .6 1.8 1.8 0 0 0 1.98-.36l.03-.03a2 2 0 1 1 2.83 2.83l-.03.03A1.8 1.8 0 0 0 19.4 9c.22.38.43.61.6 1l.02.02a2 2 0 1 1 0 3.96L20 14a1.8 1.8 0 0 0-.6 1Z"/></svg></span>Configuração</button>
        <button class="tab-button" data-tab="logs-tab" type="button"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="M4 6h16M4 12h16M4 18h10"/></svg></span>Logs</button>
        <button class="tab-button" data-tab="guide-tab" type="button"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M4 4.5A2.5 2.5 0 0 1 6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15Z"/></svg></span>Guia</button>
      </nav>
    </aside>

    <div class="content">
      <header>
        <div class="page-title">
          <h1>Operação da live</h1>
          <p>Fila, player, eventos e conexões em uma tela.</p>
        </div>
        <div class="top-status">
          <span class="pill"><span class="dot" id="twitch-dot"></span>Twitch <strong id="twitch-state">verificando</strong></span>
          <span class="pill"><span class="dot" id="spotify-dot"></span>Spotify <strong id="spotify-state">verificando</strong></span>
          <span class="pill"><span class="dot" id="youtube-dot"></span>YouTube <strong id="youtube-state">verificando</strong></span>
          <select class="language-select" id="language-select" aria-label="Idioma">
            <option value="pt">PT</option>
            <option value="en">EN</option>
          </select>
          <button class="secondary" id="shutdown-app" type="button">Encerrar</button>
        </div>
      </header>

      <main id="main">
        <div class="tab active" id="operation-tab">
      <div class="grid-main">
        <div class="stack">
          <section class="provider-card">
            <div class="toolbar">
              <h2>Provider ativo</h2>
              <span class="pill compact"><span class="dot ok"></span>Modo <strong id="provider-mode">verificando</strong></span>
            </div>
            <div class="provider-options">
              <div class="provider-option" id="provider-spotify"><strong>Spotify</strong><span>Busca, fila, volume e fallback pelo Spotify.</span></div>
              <div class="provider-option" id="provider-youtube-pear"><strong>YouTube/Pear</strong><span>Busca pelo SRL e playback no Pear Desktop.</span></div>
              <div class="provider-option" id="provider-youtube-browser"><strong>YouTube/OBS</strong><span>Busca pelo SRL e playback no Browser Source.</span></div>
            </div>
            <div class="provider-exclusive-note">Escolha um modo por live: Spotify, YouTube/Pear ou YouTube/OBS Browser.</div>
          </section>

          <section>
            <div class="toolbar">
              <h2>Ao vivo</h2>
              <span class="pill"><span class="dot ok"></span>Dashboard <strong id="refresh-state">OK</strong></span>
            </div>
            <div class="status-board">
              <div class="status-card"><span>Fila</span><strong id="queue-count">0 pedido(s)</strong></div>
              <div class="status-card"><span>Origem</span><strong id="current-source">-</strong></div>
              <div class="status-card"><span>Player YouTube</span><strong id="playback-mode">-</strong></div>
              <div class="status-card"><span>Eventos</span><strong id="event-count">0</strong></div>
            </div>
            <div class="current">
              <h3>Tocando agora</h3>
              <div class="song-title" id="current-title">Aguardando pedido</div>
              <div class="song-meta" id="current-meta">Nenhuma música tocando</div>
            </div>
          </section>

          <section>
            <div class="toolbar">
              <h2>Novo pedido</h2>
              <span class="hint">Chat ou teste manual</span>
            </div>
            <form id="request-form">
              <label>Solicitante
                <input id="requester" autocomplete="off" value="heyleao">
              </label>
              <label>Música, artista ou link
                <input id="query" autocomplete="off" placeholder="system of a down spiders ou https://youtu.be/...">
              </label>
              <div class="actions">
                <button type="submit">Adicionar pedido</button>
              </div>
            </form>
            <div class="message" id="request-message"></div>
          </section>
        </div>

        <div class="stack">
          <section>
            <div class="toolbar">
              <h2>Controles</h2>
              <span class="hint">Moderador/broadcaster</span>
            </div>
            <div class="actions">
              <button class="secondary" id="play-command" type="button">Play</button>
              <button class="secondary" id="pause-command" type="button">Pause</button>
              <button class="danger" id="skip" type="button">Skip</button>
              <div class="volume-control" aria-label="Volume">
                <button class="secondary icon-button" id="volume-down" type="button" aria-label="Diminuir volume">-</button>
                <span class="volume-readout" id="volume-level">Volume --</span>
                <button class="secondary icon-button" id="volume-up" type="button" aria-label="Aumentar volume">+</button>
              </div>
            </div>
            <div class="message" id="player-message"></div>
          </section>

          <section>
            <div class="toolbar">
              <h2>Fila de pedidos</h2>
              <div class="actions">
                <button class="secondary" id="refresh-queue" type="button">Atualizar</button>
                <button class="danger" id="clear-queue" type="button">Zerar</button>
              </div>
            </div>
            <div class="queue-persistence" id="queue-persistence">Persistência: verificando...</div>
            <div class="queue" id="queue"></div>
            <div class="message" id="queue-message"></div>
          </section>

          <section>
            <div class="toolbar">
              <h2>Últimos eventos</h2>
              <button class="secondary" id="refresh-events-preview" type="button">Atualizar</button>
            </div>
            <div class="events" id="events-preview"></div>
          </section>
        </div>
      </div>
    </div>

    <div class="tab" id="setup-tab">
      <form id="setup-form" class="setup-flow">
        <section class="setup-callout">
          <div>
            <h2>Configuração para iniciar a live</h2>
            <p id="setup-provider-help">Siga os passos em ordem. Preencha, conecte as contas e clique em Salvar.</p>
            <p class="field-note"><strong>Escolha um modo por live:</strong> Spotify, YouTube via Pear ou YouTube via OBS Browser.</p>
            <p class="field-note">Provider atual: <strong id="setup-active-provider">verificando</strong></p>
          </div>
          <div class="requirement-list" id="setup-provider-requirements"></div>
        </section>

        <section class="setup-step">
          <div class="step-number">1</div>
          <div class="step-body">
            <div class="step-head">
              <div>
                <h2>Twitch: ligar o bot ao chat</h2>
                <p class="step-copy">Use a conta do bot aqui. Ela vai ler o chat e responder aos comandos.</p>
              </div>
              <a class="setup-inline-link" href="https://dev.twitch.tv/console/apps" target="_blank" rel="noreferrer">Criar app Twitch</a>
            </div>
            <div class="setup-quick-list">
              <span>Redirect no app Twitch: <code>https://localhost:7443/auth/twitch/callback</code></span>
              <span>Cole o Client ID, nome do bot e canal.</span>
              <span>Clique em Conectar bot e entre com a conta do bot.</span>
            </div>
            <div class="form-grid">
              <label>Client ID Twitch
                <input id="setup-twitch-client-id" autocomplete="off" placeholder="Client ID do app Twitch">
              </label>
              <label>Conta do bot
                <input id="setup-twitch-bot-username" autocomplete="off" placeholder="lelos_bot">
              </label>
              <label>Canal da live
                <input id="setup-twitch-channel" autocomplete="off" placeholder="hey_leao">
              </label>
              <label>Modo de operação
                <select id="setup-operation-mode">
                  <option value="spotify">Spotify</option>
                  <option value="youtube_pear">YouTube via Pear Desktop</option>
                  <option value="youtube_browser">YouTube via OBS Browser Source</option>
                </select>
              </label>
              <input id="setup-provider" type="hidden" value="spotify">
              <input id="setup-youtube-playback" type="hidden" value="pear">
            </div>
            <div class="card-actions">
              <button class="secondary" id="setup-twitch-login" type="button">Conectar bot</button>
            </div>
          </div>
        </section>

        <section class="setup-step" data-provider-step="spotify">
          <div class="step-number">2</div>
          <div class="step-body">
            <div class="step-head">
              <div>
                <h2>Spotify: tocar músicas e fallback</h2>
                <p class="step-copy">Obrigatório se o provider padrão for Spotify. Precisa de Premium e Spotify aberto no PC da live.</p>
              </div>
              <a class="setup-inline-link" href="https://developer.spotify.com/dashboard" target="_blank" rel="noreferrer">Criar app Spotify</a>
            </div>
            <div class="setup-quick-list">
              <span>Redirect no app Spotify: <code>http://127.0.0.1:7384/auth/spotify/callback</code></span>
              <span>Cole o Client ID e clique em Login Spotify.</span>
              <span>Se quiser música quando a fila estiver vazia, ative a playlist fallback.</span>
            </div>
            <div class="form-grid">
              <label class="full">Client ID Spotify
                <input id="setup-spotify-client-id" autocomplete="off" placeholder="Client ID do app Spotify">
              </label>
              <label class="full"><span><input id="setup-spotify-fallback-enabled" type="checkbox"> Tocar playlist fallback quando não houver pedidos</span></label>
              <label class="full">Playlist fallback
                <select id="setup-spotify-fallback-playlist">
                  <option value="">Nenhuma playlist selecionada</option>
                </select>
              </label>
            </div>
            <div class="card-actions">
              <button class="secondary" id="setup-spotify-login" type="button">Login Spotify</button>
              <button class="secondary" id="setup-spotify-load-playlists" type="button">Carregar playlists</button>
              <button class="secondary" id="setup-spotify-save-playlist" type="button">Salvar fallback</button>
            </div>
          </div>
        </section>

        <section class="setup-step" data-provider-step="youtube">
          <div class="step-number">3</div>
          <div class="step-body">
            <div class="step-head">
              <div>
                <h2>YouTube: busca por texto e links</h2>
                <p class="step-copy">A API Key é usada pelo Song Request Linux para pesquisar e validar pedidos. O Pear toca com a conta logada no próprio Pear/YouTube; ele não usa sua API Key como login.</p>
              </div>
              <a class="setup-inline-link" href="https://console.cloud.google.com/apis/credentials" target="_blank" rel="noreferrer">Criar API Key</a>
            </div>
            <div class="setup-quick-list">
              <span>Escolha um modo de operação: Spotify, YouTube via Pear ou YouTube via OBS Browser.</span>
              <span>Texto como <code>system of a down spiders</code> usa YouTube Data API v3 para buscar o vídeo.</span>
              <span>Link direto do YouTube vai direto para o player escolhido e usa menos API.</span>
            </div>
            <div class="form-grid">
              <label class="full">YouTube API Keys
                <textarea id="setup-youtube-api-key" rows="4" autocomplete="off" placeholder="uma chave por linha; deixe vazio para manter as chaves atuais"></textarea>
                <span class="field-note">Vale para YouTube/Pear e YouTube/OBS Browser quando o pedido vem por texto. Para mais margem de quota, crie várias API Keys no Google Cloud e cole uma por linha.</span>
              </label>
              <label>Máximo do vídeo em segundos
                <input id="setup-youtube-max-duration" type="number" inputmode="numeric" min="30" max="86400" step="30" value="360">
              </label>
              <label><span><input id="setup-youtube-allow-non-music" type="checkbox"> Aceitar vídeo fora da categoria Música</span></label>
            </div>

            <div class="player-config-card" data-youtube-mode="pear">
              <h3>YouTube via Pear Desktop</h3>
              <p>Use quando quiser tocar pelo Pear. Abra o Pear, entre com sua conta Google/YouTube se necessário e ative o plugin API Server.</p>
              <label>Pear API
                <input id="setup-pear-base-url" autocomplete="off" placeholder="http://127.0.0.1:26538/api/v1">
              </label>
            </div>

            <div class="player-config-card" data-youtube-mode="browser">
              <h3>YouTube via OBS Browser Source</h3>
              <p>Use quando quiser tocar dentro do OBS. Adicione a fonte <code>http://127.0.0.1:7384/player</code>, marque <code>Controlar audio via OBS</code> e monitore o áudio pelo mixer do OBS.</p>
              <div class="endpoint-row with-action"><span>Player OBS</span><code>http://127.0.0.1:7384/player</code><button class="secondary icon-button copy-button" type="button" data-copy-value="http://127.0.0.1:7384/player" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button><a class="secondary" href="/player" target="_blank" rel="noreferrer">Abrir</a></div>
            </div>
          </div>
        </section>

        <section class="setup-step">
          <div class="step-number">4</div>
          <div class="step-body">
            <div class="step-head">
              <div>
                <h2>Live: comportamento da fila</h2>
                <p class="step-copy">Escolha se a fila deve continuar depois que a live acabar e o app abrir de novo.</p>
              </div>
            </div>
            <div class="form-grid">
              <label class="full"><span><input id="setup-queue-persistence-enabled" type="checkbox"> Continuar com a fila salva quando o app abrir de novo</span></label>
              <label>Texto do overlay
                <input id="setup-overlay-label" autocomplete="off" maxlength="40" placeholder="Tocando agora">
              </label>
              <label>Linhas do nome no overlay
                <select id="setup-overlay-lines">
                  <option value="1">1 linha</option>
                  <option value="2">2 linhas</option>
                  <option value="3">3 linhas</option>
                </select>
              </label>
            </div>
            <div class="setup-quick-list">
              <span>Marcado: pedidos pendentes voltam na próxima abertura.</span>
              <span>Desmarcado: a próxima live começa com fila vazia.</span>
              <span>No OBS, use o overlay: <code id="setup-overlay-url">http://127.0.0.1:7384/overlay?max=48&width=520&size=24&lines=1</code></span>
              <span>Tamanho da Browser Source: largura <code>620</code>, altura <code id="setup-overlay-height">120</code>. Depois posicione e redimensione na cena se precisar.</span>
            </div>
          </div>
        </section>

        <details class="advanced-panel">
          <summary>Avançado: comandos, permissões e limites</summary>
          <div class="advanced-content">
            <section class="setup-card full">
              <h2>Comandos e permissões do chat</h2>
              <p class="field-note">Cada caixa configura um comando específico. Use vírgula para aliases: <code>!fila, !queue, !q</code>.</p>
              <div class="command-card-grid">
                <div class="command-card">
                  <h3>Pedido de música</h3>
                  <p>Adiciona uma música na fila.</p>
                  <label>Comandos
                    <input id="setup-cmd-song-request" autocomplete="off" placeholder="!sr, !ssr">
                  </label>
                  <label>Permissão
                    <select id="setup-access-song-request"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Música atual</h3>
                  <p>Mostra o que está tocando agora.</p>
                  <label>Comandos
                    <input id="setup-cmd-current-song" autocomplete="off" placeholder="!song">
                  </label>
                  <label>Permissão
                    <select id="setup-access-current-song"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Fila</h3>
                  <p>Mostra as próximas músicas.</p>
                  <label>Comandos
                    <input id="setup-cmd-queue" autocomplete="off" placeholder="!queue, !fila, !q">
                  </label>
                  <label>Permissão
                    <select id="setup-access-queue"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Remover último pedido</h3>
                  <p>Remove o último pedido do próprio usuário.</p>
                  <label>Comandos
                    <input id="setup-cmd-remove" autocomplete="off" placeholder="!rm, !remove">
                  </label>
                  <label>Permissão
                    <select id="setup-access-remove"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Skip</h3>
                  <p>Pula a música atual.</p>
                  <label>Comandos
                    <input id="setup-cmd-skip" autocomplete="off" placeholder="!skip">
                  </label>
                  <label>Permissão
                    <select id="setup-access-skip"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Play</h3>
                  <p>Retoma o player.</p>
                  <label>Comandos
                    <input id="setup-cmd-play" autocomplete="off" placeholder="!play, !resume">
                  </label>
                  <label>Permissão
                    <select id="setup-access-play"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Pause / Stop</h3>
                  <p>Pausa o player.</p>
                  <label>Comandos
                    <input id="setup-cmd-pause" autocomplete="off" placeholder="!pause, !stop">
                  </label>
                  <label>Permissão
                    <select id="setup-access-pause"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Next / Pular</h3>
                  <p>Avança para a próxima música.</p>
                  <label>Comandos
                    <input id="setup-cmd-next" autocomplete="off" placeholder="!next, !pular">
                  </label>
                  <label>Permissão
                    <select id="setup-access-next"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Volume</h3>
                  <p>Sem número mostra o volume. Com número muda o volume.</p>
                  <label>Comandos
                    <input id="setup-cmd-volume" autocomplete="off" placeholder="!vol, !volume">
                  </label>
                  <label>Quem pode ver
                    <select id="setup-access-volume-read"></select>
                  </label>
                  <label>Quem pode mudar
                    <select id="setup-access-volume-set"></select>
                  </label>
                </div>
                <div class="command-card">
                  <h3>Ajuda</h3>
                  <p>Mostra a lista de comandos.</p>
                  <label>Comandos
                    <input id="setup-cmd-help" autocomplete="off" placeholder="!commands, !comandos, !help">
                  </label>
                  <label>Permissão
                    <select id="setup-access-help"></select>
                  </label>
                </div>
              </div>
            </section>

            <section class="setup-card full">
              <h2>Limites de pedidos por cargo</h2>
              <div class="form-grid compact">
                <label>Limite follower
                  <input id="setup-limit-follower" type="number" inputmode="numeric" min="0" max="100" step="1" value="1">
                </label>
                <label>Limite subscriber
                  <input id="setup-limit-subscriber" type="number" inputmode="numeric" min="0" max="100" step="1" value="2">
                </label>
                <label>Limite VIP
                  <input id="setup-limit-vip" type="number" inputmode="numeric" min="0" max="100" step="1" value="3">
                </label>
                <label>Limite moderador
                  <input id="setup-limit-moderator" type="number" inputmode="numeric" min="0" max="100" step="1" value="10">
                </label>
                <label>Limite streamer
                  <input id="setup-limit-streamer" type="number" inputmode="numeric" min="0" max="100" step="1" value="0">
                </label>
              </div>
              <p class="field-note">Os cargos vêm das badges/tags oficiais da Twitch. Limite 0 significa sem limite.</p>
            </section>
          </div>
        </details>

        <section class="setup-save-bar" id="setup-save-bar">
          <div>
            <h2>Salvar configuração</h2>
            <p class="field-note" id="setup-save-hint">Sem alterações pendentes. Quando mudar algo, salve antes de sair da tela.</p>
          </div>
          <div class="actions">
            <button id="setup-save-button" type="submit">Salvar configuração</button>
          </div>
          <div class="diagnostics" id="setup-summary"></div>
          <div class="message" id="setup-message"></div>
        </section>
      </form>
    </div>

    <div class="tab" id="logs-tab">
      <div class="grid-logs">
        <section>
          <div class="toolbar">
            <h2>Logs em tempo real</h2>
            <button class="secondary" id="refresh-events" type="button">Atualizar</button>
          </div>
          <div class="events" id="events"></div>
        </section>
        <section>
          <h2>Diagnóstico</h2>
          <div class="diagnostics" id="setup-diagnostics"></div>
        </section>
      </div>
    </div>

    <div class="tab" id="guide-tab">
      <div class="grid-main">
        <section>
          <h2>Passo a passo</h2>
          <div class="endpoints">
            <div class="endpoint-row"><span>1. Configure Twitch</span><a href="https://dev.twitch.tv/console/apps" target="_blank" rel="noreferrer">Abrir Twitch</a></div>
            <p class="endpoint-description">Crie um app Public, use o redirect abaixo, cole o Client ID no Setup e clique em Conectar bot.</p>
            <div class="endpoint-row with-action"><span>Redirect Twitch</span><code>https://localhost:7443/auth/twitch/callback</code><button class="secondary icon-button copy-button" type="button" data-copy-value="https://localhost:7443/auth/twitch/callback" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>
            <div class="endpoint-row"><span>2. Configure Spotify</span><a href="https://developer.spotify.com/dashboard" target="_blank" rel="noreferrer">Abrir Spotify</a></div>
            <p class="endpoint-description">Crie um app, use o redirect abaixo, cole o Client ID no Setup e clique em Login Spotify.</p>
            <div class="endpoint-row with-action"><span>Redirect Spotify</span><code>http://127.0.0.1:7384/auth/spotify/callback</code><button class="secondary icon-button copy-button" type="button" data-copy-value="http://127.0.0.1:7384/auth/spotify/callback" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>
            <div class="endpoint-row"><span>3. Configure YouTube</span><a href="https://console.cloud.google.com/apis/credentials" target="_blank" rel="noreferrer">Abrir Google</a></div>
            <p class="endpoint-description">Ative YouTube Data API v3, crie uma API Key e cole no Setup.</p>
            <p class="endpoint-description"><strong>Escolha o player no Setup:</strong> Pear Desktop ou Browser Source OBS. Use apenas um modo por vez.</p>
            <div class="endpoint-row"><span>Guia completo</span><a href="https://github.com/heyleao/song-request-linux/blob/main/docs/SETUP.md" target="_blank" rel="noreferrer">Abrir guia</a></div>
          </div>
        </section>
        <section>
          <h2>OBS</h2>
          <div class="endpoints">
            <div class="endpoint-row with-action"><span>Dashboard</span><code>http://127.0.0.1:7384/</code><button class="secondary icon-button copy-button" type="button" data-copy-value="http://127.0.0.1:7384/" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button><a class="secondary" href="/" target="_blank" rel="noreferrer">Abrir</a></div>
            <div class="endpoint-row with-action"><span>Overlay pronto</span><code id="guide-overlay-url">http://127.0.0.1:7384/overlay?max=48&width=520&size=24&lines=1</code><button class="secondary icon-button copy-button" type="button" data-copy-target="guide-overlay-url" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button><a class="secondary" id="guide-overlay-open" href="/overlay?max=48&width=520&size=24&lines=1" target="_blank" rel="noreferrer">Abrir</a></div>
            <p class="endpoint-description">No OBS, adicione como Browser Source para mostrar a musica atual.</p>
            <div class="obs-size-grid" aria-label="Tamanho recomendado para a Browser Source do overlay">
              <div class="obs-size-card"><strong>620 px</strong><span>Largura da fonte no OBS</span></div>
              <div class="obs-size-card"><strong id="guide-overlay-height">120 px</strong><span>Altura da fonte no OBS</span></div>
            </div>
            <p class="endpoint-description">Use 2 linhas quando o nome da musica for maior. O parametro <code>width=520</code> limita o texto dentro do overlay.</p>
            <div class="endpoint-row with-action"><span>Player YouTube OBS</span><code>http://127.0.0.1:7384/player</code><button class="secondary icon-button copy-button" type="button" data-copy-value="http://127.0.0.1:7384/player" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button><a class="secondary" href="/player" target="_blank" rel="noreferrer">Abrir</a></div>
            <p class="endpoint-description"><strong>Browser Source OBS:</strong> adicione a URL acima como Browser Source, marque <code>Controlar audio via OBS</code> e, nas propriedades avancadas de audio, use <code>Monitorar e enviar saida</code>. O volume fica no mixer do OBS.</p>
            <div class="endpoint-row with-action"><span>Pear API</span><code>http://127.0.0.1:26538/api/v1</code><button class="secondary icon-button copy-button" type="button" data-copy-value="http://127.0.0.1:26538/api/v1" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>
            <p class="endpoint-description"><strong>Pear Desktop:</strong> nao use a fonte <code>/player</code>. Abra o Pear, ative o plugin <code>API Server</code> na porta <code>26538</code> e deixe a Pear API salva no Setup.</p>
          </div>
        </section>
        <section>
          <div class="toolbar">
            <h2>Instalar e atualizar</h2>
            <button class="secondary" id="update-app" type="button">Atualizar pelo GitHub</button>
          </div>
          <div class="endpoints">
            <div class="endpoint-row with-action"><span>Instalar</span><code>./scripts/install-user-friendly --with-pear</code><button class="secondary icon-button copy-button" type="button" data-copy-value="./scripts/install-user-friendly --with-pear" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>
            <div class="endpoint-row with-action"><span>Abrir</span><code>./scripts/song-request-linux-open</code><button class="secondary icon-button copy-button" type="button" data-copy-value="./scripts/song-request-linux-open" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>
            <div class="endpoint-row with-action"><span>Fechar</span><code>./scripts/song-request-linux-stop</code><button class="secondary icon-button copy-button" type="button" data-copy-value="./scripts/song-request-linux-stop" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>
            <div class="endpoint-row with-action"><span>Atualizar manual</span><code>./scripts/update-from-github --restart</code><button class="secondary icon-button copy-button" type="button" data-copy-value="./scripts/update-from-github --restart" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>
            <div class="endpoint-row with-action"><span>Remover app</span><code>./scripts/uninstall-user</code><button class="secondary icon-button copy-button" type="button" data-copy-value="./scripts/uninstall-user" aria-label="Copiar" title="Copiar"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>
          </div>
          <p class="hint">Atualizar preserva configuracao, tokens e logs. A fila so volta se a persistencia da fila estiver ligada.</p>
          <div class="message" id="update-message"></div>
        </section>
      </div>
    </div>
      </main>
    </div>
  </div>

  <script>
    const $ = (id) => document.getElementById(id);
    let setupDirty = false;
    let lastConfig = null;
    let desiredVolume = null;
    let volumeTimer = null;
    let volumeInFlight = false;
    let spotifyPlaylists = [];
    const instanceId = crypto.randomUUID ? crypto.randomUUID() : `${Date.now()}-${Math.random()}`;
    const instanceKey = 'song-request-linux-active-dashboard';
    const instanceChannel = 'BroadcastChannel' in window ? new BroadcastChannel('song-request-linux-dashboard') : null;
    let isActiveInstance = true;
    const I18N_EN = {
      'Ir para o painel': 'Skip to dashboard',
      'Outra aba assumiu o painel': 'Another tab took over the dashboard',
      'Uma nova aba do Song Request Linux foi aberta e virou a aba ativa. Para evitar comandos duplicados, esta aba ficou em espera.': 'A new Song Request Linux tab was opened and became active. To avoid duplicate commands, this tab is on standby.',
      'Usar esta aba': 'Use this tab',
      'Fechar esta aba': 'Close this tab',
      'Alterações não salvas': 'Unsaved changes',
      'Salve a configuração antes de trocar de tela, fechar o app ou iniciar a live.': 'Save settings before changing screens, closing the app, or starting the stream.',
      'Salvar configuração': 'Save settings',
      'Descartar': 'Discard',
      'Controle de músicas para live': 'Music control for livestreams',
      'Operação': 'Live',
      'Configuração': 'Setup',
      'Guia': 'Guide',
      'Operação da live': 'Live operation',
      'Fila, player, eventos e conexões em uma tela.': 'Queue, player, events, and connections on one screen.',
      'verificando': 'checking',
      'Encerrar': 'Quit',
      'Provider ativo': 'Active provider',
      'Modo': 'Mode',
      'Escolha um modo por live.': 'Choose one mode per stream.',
      'Ao vivo': 'Live status',
      'Dashboard': 'Dashboard',
      'Fila': 'Queue',
      'Origem': 'Source',
      'Player YouTube': 'YouTube player',
      'Eventos': 'Events',
      'Tocando agora': 'Now playing',
      'Aguardando pedido': 'Waiting for request',
      'Nenhuma música tocando': 'No song playing',
      'Novo pedido': 'New request',
      'Chat ou teste manual': 'Chat or manual test',
      'Solicitante': 'Requester',
      'Música, artista ou link': 'Song, artist, or link',
      'Adicionar pedido': 'Add request',
      'Controles': 'Controls',
      'Moderador/broadcaster': 'Moderator/broadcaster',
      'Volume --': 'Volume --',
      'Fila de pedidos': 'Request queue',
      'Atualizar': 'Refresh',
      'Zerar': 'Clear',
      'Últimos eventos': 'Latest events',
      'Configuração para iniciar a live': 'Setup to start the stream',
      'Siga os passos em ordem. Preencha, conecte as contas e clique em Salvar.': 'Follow the steps in order. Fill in the fields, connect accounts, and click Save.',
      'Escolha um modo por live:': 'Choose one mode per stream:',
      'Spotify, YouTube via Pear ou YouTube via OBS Browser.': 'Spotify, YouTube via Pear, or YouTube via OBS Browser.',
      'Provider atual:': 'Current provider:',
      'Twitch: ligar o bot ao chat': 'Twitch: connect the bot to chat',
      'Use a conta do bot aqui. Ela vai ler o chat e responder aos comandos.': 'Use the bot account here. It reads chat and responds to commands.',
      'Criar app Twitch': 'Create Twitch app',
      'Client ID Twitch': 'Twitch Client ID',
      'Conta do bot': 'Bot account',
      'Canal da live': 'Stream channel',
      'Modo de operação': 'Operation mode',
      'Conectar bot': 'Connect bot',
      'Spotify: tocar músicas e fallback': 'Spotify: playback and fallback',
      'Obrigatório se o provider padrão for Spotify. Precisa de Premium e Spotify aberto no PC da live.': 'Required when Spotify is the active mode. Requires Premium and Spotify open on the stream PC.',
      'Criar app Spotify': 'Create Spotify app',
      'Client ID Spotify': 'Spotify Client ID',
      'Tocar playlist fallback quando não houver pedidos': 'Play fallback playlist when there are no requests',
      'Playlist fallback': 'Fallback playlist',
      'Nenhuma playlist selecionada': 'No playlist selected',
      'Login Spotify': 'Spotify login',
      'Carregar playlists': 'Load playlists',
      'Salvar fallback': 'Save fallback',
      'YouTube: links e pedidos do YouTube': 'YouTube: links and YouTube requests',
      'Escolha onde o YouTube vai tocar: OBS Browser Source ou Pear Desktop.': 'Choose where YouTube plays: OBS Browser Source or Pear Desktop.',
      'Escolha o player no Setup:': 'Choose the player in Setup:',
      'Pear Desktop ou Browser Source OBS. Use apenas um modo por vez.': 'Pear Desktop or OBS Browser Source. Use only one mode at a time.',
      'Player YouTube OBS': 'OBS YouTube player',
      'Browser Source OBS:': 'OBS Browser Source:',
      'adicione a URL acima como Browser Source, marque': 'add the URL above as a Browser Source, enable',
      'e, nas propriedades avancadas de audio, use': 'and, in Advanced Audio Properties, use',
      'O volume fica no mixer do OBS.': 'Volume is controlled in the OBS mixer.',
      'Pear Desktop:': 'Pear Desktop:',
      'nao use a fonte': 'do not use the',
      'Abra o Pear, ative o plugin': 'Open Pear, enable the',
      'na porta': 'plugin on port',
      'e deixe a Pear API salva no Setup.': 'and keep the Pear API saved in Setup.',
      'Criar API Key': 'Create API key',
      'Player YouTube': 'YouTube player',
      'Pear API': 'Pear API',
      'YouTube API Keys': 'YouTube API keys',
      'Máximo do vídeo em segundos': 'Max video length in seconds',
      'Aceitar vídeo fora da categoria Música': 'Accept videos outside the Music category',
      'Live: comportamento da fila': 'Stream: queue behavior',
      'Escolha se a fila deve continuar depois que a live acabar e o app abrir de novo.': 'Choose whether the queue continues after the stream ends and the app opens again.',
      'Continuar com a fila salva quando o app abrir de novo': 'Restore saved queue when the app opens again',
      'Texto do overlay': 'Overlay label',
      'Linhas do nome no overlay': 'Song title lines',
      '1 linha': '1 line',
      '2 linhas': '2 lines',
      '3 linhas': '3 lines',
      'Avançado: comandos, permissões e limites': 'Advanced: commands, permissions, and limits',
      'Comandos e permissões do chat': 'Chat commands and permissions',
      'Pedido de música': 'Song request',
      'Adiciona uma música na fila.': 'Adds a song to the queue.',
      'Comandos': 'Commands',
      'Permissão': 'Permission',
      'Música atual': 'Current song',
      'Mostra o que está tocando agora.': 'Shows what is playing now.',
      'Mostra as próximas músicas.': 'Shows the next songs.',
      'Remover último pedido': 'Remove last request',
      'Remove o último pedido do próprio usuário.': 'Removes the user\'s last request.',
      'Pula a música atual.': 'Skips the current song.',
      'Retoma o player.': 'Resumes the player.',
      'Pause / Stop': 'Pause / Stop',
      'Pausa o player.': 'Pauses the player.',
      'Next / Pular': 'Next / Skip',
      'Avança para a próxima música.': 'Moves to the next song.',
      'Sem número mostra o volume. Com número muda o volume.': 'Without a number, shows volume. With a number, changes volume.',
      'Quem pode ver': 'Who can view',
      'Quem pode mudar': 'Who can change',
      'Ajuda': 'Help',
      'Mostra a lista de comandos.': 'Shows the command list.',
      'Limites de pedidos por cargo': 'Request limits by role',
      'Limite follower': 'Follower limit',
      'Limite subscriber': 'Subscriber limit',
      'Limite VIP': 'VIP limit',
      'Limite moderador': 'Moderator limit',
      'Limite streamer': 'Streamer limit',
      'Salvar': 'Save',
      'Logs em tempo real': 'Real-time logs',
      'Guia rápido': 'Quick guide',
      'Copiar': 'Copy',
      'Instalar e atualizar': 'Install and update',
      'Atualizar pelo GitHub': 'Update from GitHub',
      'Instalar': 'Install',
      'Abrir': 'Open',
      'Fechar': 'Close',
      'Atualizar manual': 'Manual update',
      'Remover app': 'Uninstall app',
      'Fila vazia': 'Queue empty',
      'Nenhum evento ainda': 'No events yet',
      'Aba em espera': 'Tab on standby',
      'Song Request Linux encerrando': 'Song Request Linux is shutting down'
    };
    const I18N_PLACEHOLDERS_EN = {
      'system of a down spiders ou https://youtu.be/...': 'system of a down spiders or https://youtu.be/...',
      'Client ID do app Twitch': 'Twitch app Client ID',
      'Client ID do app Spotify': 'Spotify app Client ID',
      'deixe vazio para manter a chave atual': 'leave empty to keep the current key',
      'Tocando agora': 'Now playing'
    };
    const I18N_REVERSE_EN = Object.fromEntries(Object.entries(I18N_EN).map(([pt, en]) => [en, pt]));
    const I18N_PLACEHOLDERS_REVERSE_EN = Object.fromEntries(Object.entries(I18N_PLACEHOLDERS_EN).map(([pt, en]) => [en, pt]));

    function currentLanguage() {
      const queryLang = new URLSearchParams(window.location.search).get('lang');
      if (queryLang === 'en' || queryLang === 'pt') {
        localStorage.setItem('song-request-linux-language', queryLang);
        return queryLang;
      }
      return localStorage.getItem('song-request-linux-language') || 'pt';
    }

    function translateText(value, lang) {
      if (lang === 'en') return I18N_EN[value] || value;
      return I18N_REVERSE_EN[value] || value;
    }

    function translateTree(root = document.body) {
      const lang = currentLanguage();
      document.documentElement.lang = lang === 'en' ? 'en' : 'pt-BR';
      const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
        acceptNode(node) {
          if (!node.nodeValue.trim()) return NodeFilter.FILTER_REJECT;
          const parent = node.parentElement;
          if (!parent || ['SCRIPT', 'STYLE', 'CODE', 'PRE'].includes(parent.tagName)) return NodeFilter.FILTER_REJECT;
          return NodeFilter.FILTER_ACCEPT;
        }
      });
      const nodes = [];
      while (walker.nextNode()) nodes.push(walker.currentNode);
      nodes.forEach((node) => {
        const raw = node.nodeValue;
        const trimmed = raw.trim();
        const source = node.__i18nSource || I18N_REVERSE_EN[trimmed] || trimmed;
        node.__i18nSource = source;
        const translated = translateText(source, lang);
        node.nodeValue = raw.replace(trimmed, translated);
      });
      document.querySelectorAll('[placeholder]').forEach((element) => {
        const value = element.getAttribute('placeholder');
        const source = element.dataset.i18nPlaceholder || I18N_PLACEHOLDERS_REVERSE_EN[value] || value;
        element.dataset.i18nPlaceholder = source;
        element.setAttribute('placeholder', lang === 'en' ? (I18N_PLACEHOLDERS_EN[source] || source) : source);
      });
      const select = $('language-select');
      if (select) select.value = lang;
    }


    function markActiveInstance() {
      isActiveInstance = true;
      document.body.classList.remove('stale-instance');
      $('instance-notice').hidden = true;
      localStorage.setItem(instanceKey, JSON.stringify({ id: instanceId, at: Date.now() }));
      instanceChannel?.postMessage({ type: 'activate', id: instanceId });
    }

    function markStaleInstance() {
      if (!isActiveInstance) return;
      isActiveInstance = false;
      document.body.classList.add('stale-instance');
      $('instance-notice').hidden = false;
      $('instance-message').textContent = 'Se quiser continuar por aqui, clique em Usar esta aba. A outra aba entrará em espera.';
      if ($('refresh-state')) $('refresh-state').textContent = 'ABA ANTIGA';
    }

    function closeThisTab() {
      $('instance-message').textContent = 'Tentando fechar a aba. Se o navegador bloquear, pode fechar manualmente.';
      window.close();
      setTimeout(() => {
        document.body.innerHTML = '<main><section><h2>Aba em espera</h2><p class="muted">Esta aba pode ser fechada. O painel ativo está em outra aba.</p></section></main>';
      }, 250);
    }

    instanceChannel?.addEventListener('message', (event) => {
      if (event.data?.type === 'activate' && event.data.id !== instanceId) markStaleInstance();
      if (event.data?.type === 'shutdown' && event.data.id !== instanceId) {
        document.body.innerHTML = '<main><section><h2>Song Request Linux encerrando</h2><p class="muted">Outra aba solicitou o encerramento do app.</p></section></main>';
      }
    });

    window.addEventListener('storage', (event) => {
      if (event.key !== instanceKey || !event.newValue) return;
      try {
        const active = JSON.parse(event.newValue);
        if (active.id && active.id !== instanceId) markStaleInstance();
      } catch (_) {}
    });

    window.addEventListener('beforeunload', (event) => {
      if (setupDirty) {
        event.preventDefault();
        event.returnValue = '';
      }
      const active = localStorage.getItem(instanceKey);
      if (!active) return;
      try {
        if (JSON.parse(active).id === instanceId) localStorage.removeItem(instanceKey);
      } catch (_) {}
    });

    async function api(path, options = {}) {
      const response = await fetch(path, {
        headers: { 'content-type': 'application/json', ...(options.headers || {}) },
        ...options
      });
      const text = await response.text();
      const data = text ? JSON.parse(text) : null;
      if (!response.ok) throw new Error(data?.error || text || 'Falha na requisição');
      return data;
    }

    function escapeHtml(value) {
      return String(value)
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#039;');
    }

    function aliasesToText(values) {
      return (values || []).join(', ');
    }

    function textToAliases(value, fallback) {
      const aliases = String(value || '')
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean)
        .map((item) => item.startsWith('!') ? item : `!${item}`);
      return aliases.length ? aliases : fallback;
    }

    function groupedAliases(value, fallbackGroups) {
      const groups = String(value || '')
        .split('|')
        .map((group, index) => textToAliases(group, fallbackGroups[index] || []));
      return fallbackGroups.map((fallback, index) => groups[index]?.length ? groups[index] : fallback);
    }

    function numberFromInput(id, fallback) {
      const value = Number($(id).value);
      if (!Number.isFinite(value)) return fallback;
      return Math.max(0, Math.min(100, Math.floor(value)));
    }

    function overlayLinesHeight(lines) {
      return lines >= 3 ? 180 : lines === 2 ? 150 : 120;
    }

    function overlayConfigFromForm() {
      return {
        label: $('setup-overlay-label').value || 'Tocando agora',
        lines: Number($('setup-overlay-lines').value || 1)
      };
    }

    function overlayUrl(overlay) {
      const config = overlay || overlayConfigFromForm();
      const params = new URLSearchParams({
        max: '48',
        width: '520',
        size: '24',
        lines: String(Math.max(1, Math.min(3, Number(config.lines || 1))))
      });
      const label = String(config.label || 'Tocando agora').trim();
      if (label && label !== 'Tocando agora') params.set('label', label);
      return `/overlay?${params.toString()}`;
    }

    function renderOverlayGuide(overlay) {
      const config = overlay || overlayConfigFromForm();
      const url = overlayUrl(config);
      const fullUrl = `${window.location.origin}${url}`;
      const height = overlayLinesHeight(Number(config.lines || 1));
      if ($('setup-overlay-url')) $('setup-overlay-url').textContent = fullUrl;
      if ($('guide-overlay-url')) $('guide-overlay-url').textContent = fullUrl;
      if ($('guide-overlay-open')) $('guide-overlay-open').href = url;
      if ($('setup-overlay-height')) $('setup-overlay-height').textContent = String(height);
      if ($('guide-overlay-height')) $('guide-overlay-height').textContent = `${height} px`;
    }

    function fillOverlaySettings(config) {
      const overlay = config.overlay || { label: 'Tocando agora', lines: 1 };
      $('setup-overlay-label').value = overlay.label || 'Tocando agora';
      $('setup-overlay-lines').value = String(overlay.lines || 1);
      renderOverlayGuide(overlay);
    }

    function queueLimitsFromForm() {
      return {
        follower: numberFromInput('setup-limit-follower', 1),
        subscriber: numberFromInput('setup-limit-subscriber', 2),
        vip: numberFromInput('setup-limit-vip', 3),
        moderator: numberFromInput('setup-limit-moderator', 10),
        streamer: numberFromInput('setup-limit-streamer', 0)
      };
    }

    function commandSettingsFromForm() {
      const current = lastConfig?.command_settings || {};
      const aliases = current.aliases || {};
      const access = current.access || {};

      return {
        aliases: {
          song_request: textToAliases($('setup-cmd-song-request').value, aliases.song_request || ['!sr']),
          current_song: textToAliases($('setup-cmd-current-song').value, aliases.current_song || ['!song']),
          queue: textToAliases($('setup-cmd-queue').value, aliases.queue || ['!queue', '!fila', '!q']),
          remove: textToAliases($('setup-cmd-remove').value, aliases.remove || ['!rm', '!remove']),
          skip: textToAliases($('setup-cmd-skip').value, aliases.skip || ['!skip']),
          play: textToAliases($('setup-cmd-play').value, aliases.play || ['!play', '!resume']),
          pause: textToAliases($('setup-cmd-pause').value, aliases.pause || ['!pause', '!stop']),
          next: textToAliases($('setup-cmd-next').value, aliases.next || ['!next', '!pular']),
          volume: textToAliases($('setup-cmd-volume').value, aliases.volume || ['!vol', '!volume']),
          help: textToAliases($('setup-cmd-help').value, aliases.help || ['!commands', '!comandos', '!help'])
        },
        access: {
          song_request: $('setup-access-song-request').value,
          current_song: $('setup-access-current-song').value,
          queue: $('setup-access-queue').value,
          remove: $('setup-access-remove').value,
          skip: $('setup-access-skip').value,
          play: $('setup-access-play').value,
          pause: $('setup-access-pause').value,
          next: $('setup-access-next').value,
          playback: $('setup-access-next').value,
          volume_read: $('setup-access-volume-read').value,
          volume_set: $('setup-access-volume-set').value,
          help: $('setup-access-help').value
        }
      };
    }

    function fillCommandSettings(config) {
      fillPermissionSelects();
      const settings = config.command_settings || {};
      const aliases = settings.aliases || {};
      const access = settings.access || {};
      const legacyPlayback = access.playback || 'moderator';
      $('setup-cmd-song-request').value = aliasesToText(aliases.song_request || ['!sr']);
      $('setup-cmd-current-song').value = aliasesToText(aliases.current_song || ['!song']);
      $('setup-cmd-queue').value = aliasesToText(aliases.queue || ['!queue', '!fila', '!q']);
      $('setup-cmd-remove').value = aliasesToText(aliases.remove || ['!rm', '!remove']);
      $('setup-cmd-skip').value = aliasesToText(aliases.skip || ['!skip']);
      $('setup-cmd-play').value = aliasesToText(aliases.play || ['!play', '!resume']);
      $('setup-cmd-pause').value = aliasesToText(aliases.pause || ['!pause', '!stop']);
      $('setup-cmd-next').value = aliasesToText(aliases.next || ['!next', '!pular']);
      $('setup-cmd-volume').value = aliasesToText(aliases.volume || ['!vol', '!volume']);
      $('setup-cmd-help').value = aliasesToText(aliases.help || ['!commands', '!comandos', '!help']);
      $('setup-access-song-request').value = access.song_request || 'follower';
      $('setup-access-current-song').value = access.current_song || 'follower';
      $('setup-access-queue').value = access.queue || 'follower';
      $('setup-access-remove').value = access.remove || 'follower';
      $('setup-access-skip').value = access.skip || 'moderator';
      $('setup-access-play').value = access.play || legacyPlayback;
      $('setup-access-pause').value = access.pause || legacyPlayback;
      $('setup-access-next').value = access.next || legacyPlayback;
      $('setup-access-volume-read').value = access.volume_read || 'follower';
      $('setup-access-volume-set').value = access.volume_set || 'moderator';
      $('setup-access-help').value = access.help || 'follower';
      const limits = config.queue_limits || {};
      $('setup-limit-follower').value = limits.follower ?? 1;
      $('setup-limit-subscriber').value = limits.subscriber ?? 2;
      $('setup-limit-vip').value = limits.vip ?? 3;
      $('setup-limit-moderator').value = limits.moderator ?? 10;
      $('setup-limit-streamer').value = limits.streamer ?? 0;
    }

    function fillPermissionSelects() {
      const labels = [
        ['follower', 'Follower'],
        ['subscriber', 'Subscriber / sub'],
        ['vip', 'VIP'],
        ['moderator', 'Moderador'],
        ['streamer', 'Streamer']
      ];
      document.querySelectorAll('select[id^="setup-access-"]').forEach((select) => {
        if (select.options.length) return;
        select.innerHTML = labels.map(([value, label]) => `<option value="${value}">${label}</option>`).join('');
      });
    }

    function operationModeFromValues(provider, playback) {
      if (provider === 'spotify') return 'spotify';
      return playback === 'browser' ? 'youtube_browser' : 'youtube_pear';
    }

    function operationModeFromConfig(config = {}) {
      return operationModeFromValues(config.default_provider || $('setup-provider').value || 'youtube', config.youtube_playback || $('setup-youtube-playback').value || 'pear');
    }

    function operationModeLabel(mode) {
      if (mode === 'spotify') return 'Spotify';
      if (mode === 'youtube_browser') return 'YouTube/OBS';
      return 'YouTube/Pear';
    }

    function paintOperationMode(mode, pending = false) {
      $('provider-mode').textContent = operationModeLabel(mode);
      $('setup-active-provider').textContent = operationModeLabel(mode);
      $('provider-spotify').classList.toggle('active', mode === 'spotify');
      $('provider-youtube-pear').classList.toggle('active', mode === 'youtube_pear');
      $('provider-youtube-browser').classList.toggle('active', mode === 'youtube_browser');
      if (pending) {
        const suffix = ' selecionado. Clique em Salvar configuração para aplicar na live.';
        $('setup-provider-help').textContent = operationModeLabel(mode) + suffix;
        if (mode === 'spotify') {
          $('setup-provider-requirements').innerHTML = [
            requirementRow('warn', 'Spotify', 'Salve para ativar busca, fila, volume e fallback pelo Spotify.'),
            requirementRow('warn', 'YouTube', 'Ficará inativo enquanto Spotify for o modo da live.')
          ].join('');
        } else if (mode === 'youtube_pear') {
          $('setup-provider-requirements').innerHTML = [
            requirementRow('warn', 'YouTube API para busca', 'Necessária para pedidos por nome. Links diretos do YouTube usam menos API.'),
            requirementRow('warn', 'Playback Pear Desktop', 'Abra o Pear, faça login no Pear/YouTube se necessário e ative o API Server na porta 26538.')
          ].join('');
        } else {
          $('setup-provider-requirements').innerHTML = [
            requirementRow('warn', 'YouTube API para busca', 'Necessária para pedidos por nome. Links diretos do YouTube usam menos API.'),
            requirementRow('warn', 'Playback OBS Browser', 'Adicione http://127.0.0.1:7384/player como Browser Source e monitore o áudio no OBS.')
          ].join('');
        }
      }
    }

    function applyOperationMode(mode, options = {}) {
      const provider = mode === 'spotify' ? 'spotify' : 'youtube';
      const playback = mode === 'youtube_browser' ? 'browser' : 'pear';
      $('setup-operation-mode').value = mode;
      $('setup-provider').value = provider;
      $('setup-youtube-playback').value = playback;
      updateProviderStepVisibility({ default_provider: provider, youtube_playback: playback });
      paintOperationMode(mode, Boolean(options.dirty));
      if (options.dirty) {
        setSetupDirty(true, `Modo alterado para ${operationModeLabel(mode)}. Clique em Salvar configuração.`);
      }
    }

    function updateProviderStepVisibility(config) {
      const provider = config.default_provider || $('setup-provider').value || 'youtube';
      const playback = config.youtube_playback || $('setup-youtube-playback').value || 'pear';
      document.querySelectorAll('[data-provider-step]').forEach((section) => {
        const visible = section.dataset.providerStep === provider;
        section.classList.toggle('provider-hidden', !visible);
        section.querySelectorAll('input, select, textarea, button').forEach((control) => {
          control.disabled = !visible;
        });
      });
      document.querySelectorAll('[data-youtube-mode]').forEach((section) => {
        const visible = provider === 'youtube' && section.dataset.youtubeMode === playback;
        section.classList.toggle('provider-hidden', !visible);
        section.querySelectorAll('input, select, textarea, button').forEach((control) => {
          control.disabled = !visible;
        });
      });
      document.querySelectorAll('.setup-step:not(.provider-hidden) .step-number').forEach((item, index) => {
        item.textContent = String(index + 1);
      });
    }

    function sourceLabel(source) {
      if (!source) return '-';
      if (source.type === 'youtube') return 'YouTube';
      if (source.type === 'spotify') return 'Spotify';
      if (source.type === 'search') return source.provider === 'spotify' ? 'Spotify' : 'YouTube';
      return source.type;
    }

    function fallbackPlaylistName(connections) {
      return connections?.spotify?.fallback_playlist?.name || 'playlist fallback';
    }

    function isSpotifyFallbackSong(song) {
      return song
        && String(song.requester || '').toLowerCase() === 'spotify'
        && String(song.artist || '').toLowerCase() === 'spotify';
    }

    function songMeta(song, connections) {
      if (!song) return 'Nenhuma música tocando';
      if (isSpotifyFallbackSong(song)) return `Playlist - ${fallbackPlaylistName(connections)}`;
      return `${song.artist} - pedido por ${song.requester}`;
    }

    function stateClass(ok, pending = false) {
      if (ok) return 'dot ok';
      return pending ? 'dot' : 'dot bad';
    }

    function requirementRow(state, label, detail) {
      return `
        <div class="requirement-row ${state}">
          <strong>${state === 'ok' ? 'OK' : state === 'warn' ? 'Atenção' : 'Falta'}</strong>
          <span><b>${escapeHtml(label)}</b><br>${escapeHtml(detail)}</span>
        </div>
      `;
    }

    function renderProviderRequirements(config, connections, pear) {
      const mode = operationModeFromConfig(config);
      const provider = operationModeLabel(mode);
      $('setup-active-provider').textContent = provider;
      updateProviderStepVisibility(config);

      if (config.default_provider === 'spotify') {
        const spotifyProduct = connections.spotify.product;
        const premiumState = connections.spotify.premium === true ? 'ok' : connections.spotify.premium === false ? 'bad' : 'warn';
        const premiumDetail = connections.spotify.premium === true
          ? 'Conta Premium confirmada pelo Spotify. Mantenha o app Spotify aberto com um device ativo.'
          : connections.spotify.premium === false
            ? `Conta logada reportou plano ${spotifyProduct || 'nao premium'}. Controle de fila/playback exige Premium.`
            : connections.spotify.product_check_error
              ? 'Nao consegui validar o plano. Clique em Login Spotify para conceder o escopo user-read-private.'
              : 'Plano ainda nao validado. Clique em Login Spotify se esta mensagem continuar aparecendo.';
        $('setup-provider-help').textContent = 'Modo Spotify ativo: pedidos por texto entram no Spotify. YouTube fica parado até você trocar o provider.';
        $('setup-provider-requirements').innerHTML = [
          requirementRow(
            connections.spotify.client_id_configured ? 'ok' : 'bad',
            'Spotify Client ID',
            connections.spotify.client_id_configured ? 'Client ID salvo.' : 'Preencha o Client ID no card Spotify.'
          ),
          requirementRow(
            connections.spotify.token_configured ? 'ok' : 'bad',
            'Login Spotify',
            connections.spotify.token_configured ? 'OAuth conectado.' : 'Clique em Login Spotify depois de salvar o Client ID.'
          ),
          requirementRow(
            premiumState,
            'Premium e device ativo',
            premiumDetail
          )
        ].join('');
        return;
      }

      const pearMode = config.youtube_playback === 'pear';
      $('setup-provider-help').textContent = pearMode
        ? 'Modo YouTube/Pear ativo: o SRL usa YouTube Data API para buscar por texto e manda tocar no Pear. O Pear usa a conta logada nele.'
        : 'Modo YouTube/OBS ativo: o SRL usa YouTube Data API para buscar por texto e toca no Browser Source /player dentro do OBS.';
      $('setup-provider-requirements').innerHTML = [
        requirementRow(
          config.youtube_api_key_configured ? 'ok' : 'bad',
          'YouTube API para busca',
          config.youtube_api_key_configured
            ? `${config.youtube_api_key_count || 1} chave(s) salva(s). Necessária para pedidos por nome; links diretos usam menos API.`
            : 'Preencha ao menos uma API Key para buscar música por nome. Link direto do YouTube ainda é o caminho mais leve.'
        ),
        requirementRow(
          pearMode ? pear.reachable ? 'ok' : 'bad' : 'ok',
          pearMode ? 'Playback Pear Desktop' : 'Playback OBS Browser',
          pearMode
            ? pear.reachable ? 'Pear respondeu na API local. Login Google/YouTube fica dentro do Pear, não na API Key do SRL.' : 'Abra o Pear Desktop e confira o plugin API Server na porta 26538.'
            : 'Adicione http://127.0.0.1:7384/player como fonte de navegador no OBS e monitore o áudio pelo OBS.'
        ),
        requirementRow(
          'warn',
          'Filtro de duração',
          `Limite atual: ${config.youtube_max_duration_seconds || 360}s. Ajuste para evitar videos longos na fila.`
        )
      ].join('');
    }

    function setMessage(id, text, isError = false) {
      const element = $(id);
      element.textContent = text;
      element.classList.toggle('error', isError);
    }

    async function refreshUpdateStatus(showEmpty = false) {
      try {
        const status = await api('/api/update/status');
        if (!showEmpty && (!status || status.status === 'none')) return;
        const isError = status.status === 'failed' || status.status === 'unknown';
        setMessage('update-message', status.message || 'Status de atualizacao indisponivel.', isError);
      } catch (_) {
        if (showEmpty) setMessage('update-message', 'Nao foi possivel ler o status da atualizacao.', true);
      }
    }

    function setSetupDirty(value, reason = '') {
      setupDirty = Boolean(value);
      document.body.classList.toggle('setup-dirty', setupDirty);
      $('unsaved-banner').hidden = !setupDirty;
      $('setup-tab-button')?.classList.toggle('dirty', setupDirty);
      $('setup-save-bar')?.classList.toggle('dirty', setupDirty);
      if ($('setup-save-hint')) {
        $('setup-save-hint').textContent = setupDirty
          ? (reason || 'Existem alterações pendentes. Clique em Salvar configuração.')
          : 'Sem alterações pendentes. Quando mudar algo, salve antes de sair da tela.';
      }
      if (setupDirty && reason) setMessage('setup-message', reason);
    }

    async function discardSetupChanges() {
      setSetupDirty(false);
      setMessage('setup-message', 'Alterações descartadas.');
      await refresh();
    }

    function renderEvents(events) {
      $('event-count').textContent = events.length;
      const html = events.length
        ? events.map((event) => `
            <div class="event-row ${escapeHtml(event.kind)}">
              <strong>${escapeHtml(event.kind)}</strong>
              <span>${escapeHtml(event.message)}</span>
            </div>
          `).join('')
        : '<div class="event-row muted">Nenhum evento ainda</div>';
      $('events').innerHTML = html;
      $('events-preview').innerHTML = events.slice(0, 6).length
        ? events.slice(0, 6).map((event) => `
            <div class="event-row ${escapeHtml(event.kind)}">
              <strong>${escapeHtml(event.kind)}</strong>
              <span>${escapeHtml(event.message)}</span>
            </div>
          `).join('')
        : '<div class="event-row muted">Nenhum evento ainda</div>';
    }

    function renderQueuePersistence(queue) {
      const persistence = queue.persistence;
      if (!persistence) {
        $('queue-persistence').innerHTML = '<span>Persistência: não informada nesta resposta.</span>';
        return;
      }
      if (!persistence.enabled) {
        $('queue-persistence').innerHTML = '<span><strong>Persistência desativada</strong> - a fila atual não será restaurada ao reabrir o app.</span>';
        return;
      }
      const saved = `${persistence.saved_items} item(ns) salvo(s)`;
      const state = persistence.exists ? 'arquivo encontrado' : 'arquivo será criado no próximo pedido';
      $('queue-persistence').innerHTML = `
        <span><strong>Persistência ativa</strong> - ${escapeHtml(saved)} - ${escapeHtml(state)}</span>
        <code>${escapeHtml(persistence.path)}</code>
      `;
    }

    function renderVolume(volume) {
      const label = volume.level === null || volume.level === undefined
        ? 'Volume --'
        : `Volume ${volume.level}%`;
      $('volume-level').textContent = label;
      $('volume-level').title = volume.message || '';
    }

    async function refreshVolume() {
      if (volumeInFlight || desiredVolume !== null) return;
      try {
        renderVolume(await api('/api/volume'));
      } catch (error) {
        $('volume-level').textContent = 'Volume --';
        $('volume-level').title = error.message;
      }
    }

    function accessLabel(access) {
      if (access === 'streamer') return 'streamer';
      if (access === 'moderator') return 'moderador';
      if (access === 'vip') return 'VIP';
      if (access === 'subscriber') return 'subscriber / sub';
      return 'follower';
    }

    function fillSpotifyPlaylistOptions(playlists, selectedId = '') {
      const options = ['<option value="">Nenhuma playlist selecionada</option>']
        .concat(playlists.map((playlist) => {
          const total = playlist.tracks?.total;
          const suffix = total === undefined ? '' : ` (${total})`;
          return `<option value="${escapeHtml(playlist.id)}">${escapeHtml(playlist.name)}${suffix}</option>`;
        }));
      $('setup-spotify-fallback-playlist').innerHTML = options.join('');
      $('setup-spotify-fallback-playlist').value = selectedId || '';
    }

    function setSpotifyFallbackControls(enabled) {
      $('setup-spotify-fallback-playlist').disabled = !enabled;
      $('setup-spotify-load-playlists').disabled = !enabled;
      $('setup-spotify-save-playlist').disabled = !enabled;
      $('setup-spotify-fallback-playlist').title = enabled
        ? ''
        : 'Ative a playlist fallback para escolher uma playlist.';
    }

    function renderSpotifyFallback(connections, config) {
      const selected = connections.spotify.fallback_playlist;
      const selectedId = selected?.id || '';
      const enabled = Boolean(config.spotify_fallback_enabled);
      $('setup-spotify-fallback-enabled').checked = enabled;
      setSpotifyFallbackControls(enabled);
      const hasSelected = selected && !spotifyPlaylists.some((playlist) => playlist.id === selected.id);
      const playlists = hasSelected ? [selected, ...spotifyPlaylists] : spotifyPlaylists;
      fillSpotifyPlaylistOptions(playlists, selectedId);
    }

    function renderDiagnostics(diagnostics, connections, pear, config) {
      const twitchReady = diagnostics.integrations.twitch.configured;
      const spotifyReady = connections.spotify.token_configured;
      const spotifyConfigured = connections.spotify.client_id_configured;
      const youtubeReady = config.default_provider !== 'youtube' || (config.youtube_api_key_configured && (config.youtube_playback !== 'pear' || pear.reachable));

      $('twitch-state').textContent = twitchReady ? 'configurado' : 'pendente';
      $('twitch-dot').className = stateClass(twitchReady);
      $('spotify-state').textContent = spotifyReady ? 'conectado' : spotifyConfigured ? 'login' : 'pendente';
      $('spotify-dot').className = stateClass(spotifyReady, spotifyConfigured);
      $('youtube-state').textContent = config.default_provider !== 'youtube'
        ? 'inativo'
        : config.youtube_playback === 'pear'
          ? youtubeReady ? 'Pear + API ok' : 'Pear/API pendente'
          : config.youtube_api_key_configured ? 'OBS + API ok' : 'API pendente';
      $('youtube-dot').className = stateClass(youtubeReady, config.default_provider !== 'youtube' || config.youtube_api_key_configured || config.youtube_playback === 'pear');
      const mode = operationModeFromConfig(config);
      paintOperationMode(mode);
      renderProviderRequirements(config, connections, pear);
      renderSpotifyFallback(connections, config);

      const rows = [
        ['Bot Twitch', twitchReady ? 'configurado' : 'não configurado'],
        ['Spotify', spotifyReady ? 'conectado' : spotifyConfigured ? 'login pendente' : 'client id pendente'],
        ['YouTube', config.default_provider === 'youtube' ? `${config.youtube_playback === 'pear' ? 'Pear Desktop' : 'OBS Browser'} - ${config.youtube_api_key_configured ? `${config.youtube_api_key_count || 1} api key(s)` : 'api key pendente'}` : 'inativo'],
        ['Pear Desktop', pear.configured ? pear.reachable ? 'conectado' : 'não encontrado' : 'desativado'],
        ['Pear atual', pear.now_playing || '-'],
        ['Logs', diagnostics.storage.log_dir.exists ? 'ok' : 'pendente']
      ];
      const html = rows.map(([label, value]) => `
        <div class="diagnostic-row"><span>${escapeHtml(label)}</span><code>${escapeHtml(value)}</code></div>
      `).join('');
      const commandHtml = (config.commands_summary || []).map((command) => `
        <div class="diagnostic-row">
          <span>${escapeHtml(command.name)} · ${escapeHtml(accessLabel(command.access))}</span>
          <code>${escapeHtml((command.aliases || []).join(', '))}</code>
        </div>
      `).join('');
      const limits = config.queue_limits || {};
      const limitHtml = `
        <div class="diagnostic-row"><span>Limites da fila</span><code>follower ${limits.follower ?? 1} · sub ${limits.subscriber ?? 2} · VIP ${limits.vip ?? 3} · mod ${limits.moderator ?? 10} · streamer ${limits.streamer ?? 0}</code></div>
      `;
      $('setup-diagnostics').innerHTML = html;
      $('setup-summary').innerHTML = `${html}<div class="divider"></div>${limitHtml}${commandHtml}`;
    }

    async function refresh() {
      if (!isActiveInstance) return;
      try {
        const [status, queue, diagnostics, connections, pear, events, config] = await Promise.all([
          api('/api/status'),
          api('/api/queue'),
          api('/api/diagnostics'),
          api('/api/connections/status'),
          api('/api/pear/status'),
          api('/api/events'),
          api('/api/config')
        ]);
        lastConfig = config;

        $('queue-count').textContent = `${queue.queue_length} pedido(s)`;
        $('refresh-state').textContent = 'OK';
        $('playback-mode').textContent = config.default_provider === 'youtube' ? (config.youtube_playback === 'pear' ? 'Pear Desktop' : 'OBS Browser') : 'Inativo';
        await refreshVolume();
        renderDiagnostics(diagnostics, connections, pear, config);

        if (!setupDirty && !$('setup-form').contains(document.activeElement)) {
          applyOperationMode(operationModeFromConfig(config));
          $('setup-spotify-client-id').value = config.spotify_client_id || '';
          $('setup-twitch-client-id').value = config.twitch_client_id || '';
          $('setup-spotify-fallback-enabled').checked = Boolean(config.spotify_fallback_enabled || connections.spotify.fallback_playlist);
          setSpotifyFallbackControls($('setup-spotify-fallback-enabled').checked);
          $('setup-twitch-bot-username').value = config.twitch_bot_username || '';
          $('setup-twitch-channel').value = config.twitch_channel || '';
          $('setup-pear-base-url').value = config.pear_base_url || 'http://127.0.0.1:26538/api/v1';
          $('setup-youtube-max-duration').value = config.youtube_max_duration_seconds || 360;
          $('setup-youtube-allow-non-music').checked = Boolean(config.youtube_allow_non_music);
          $('setup-queue-persistence-enabled').checked = Boolean(config.queue_persistence_enabled);
          fillOverlaySettings(config);
          fillCommandSettings(config);
        }

        const current = queue.current_song;
        $('current-title').textContent = current ? current.title : 'Aguardando pedido';
        $('current-meta').textContent = songMeta(current, connections);
        $('current-source').textContent = sourceLabel(current?.source);
        renderQueuePersistence(queue);
        $('queue').innerHTML = queue.queue.length
          ? queue.queue.map((item, index) => `
              <div class="queue-item">
                <strong>${index + 1}. ${escapeHtml(item.title)}</strong>
                ${item.id > 0 && item.requester !== 'spotify'
                  ? `<button class="secondary remove-queue-item" type="button" data-id="${item.id}" aria-label="Remover ${escapeHtml(item.title)}" title="Remover da fila">x</button>`
                  : `<span class="pill compact">${escapeHtml(sourceLabel(item.source))}</span>`}
                ${item.id > 0 && item.requester !== 'spotify'
                  ? `<span class="pill compact">${escapeHtml(sourceLabel(item.source))}</span>`
                  : ''}
                <span class="queue-meta">${escapeHtml(songMeta(item, connections))}</span>
              </div>
            `).join('')
          : '<div class="queue-item muted">Fila vazia</div>';
        renderEvents(events);
        translateTree();
      } catch (error) {
        $('refresh-state').textContent = 'ERRO';
      }
    }

    async function sendCommand(message, isModerator = false) {
      if (!message.trim()) throw new Error('Digite um comando.');
      return api('/api/chat-command', {
        method: 'POST',
        body: JSON.stringify({
          requester: $('requester').value || 'follower',
          message,
          is_moderator: isModerator,
          role: isModerator ? 'moderator' : 'follower'
        })
      });
    }

    function showTab(tabId, options = {}) {
      const current = document.querySelector('.tab.active')?.id;
      if (!options.force && setupDirty && current === 'setup-tab' && tabId !== 'setup-tab') {
        const leave = window.confirm('Existem alterações não salvas. Sair da configuração sem salvar?');
        if (!leave) return false;
      }
      const button = document.querySelector(`.tab-button[data-tab="${tabId}"]`);
      if (!button) return false;
      document.querySelectorAll('.tab-button[data-tab]').forEach((item) => item.classList.remove('active'));
      document.querySelectorAll('.tab').forEach((item) => item.classList.remove('active'));
      button.classList.add('active');
      document.getElementById(tabId).classList.add('active');
      if (options.updateHash !== false && window.location.hash !== `#${tabId}`) {
        history.replaceState(null, '', `#${tabId}`);
      }
      return true;
    }

    function tabFromHash() {
      const tabId = window.location.hash.replace('#', '');
      return document.querySelector(`.tab-button[data-tab="${tabId}"]`) ? tabId : 'operation-tab';
    }

    document.querySelectorAll('.tab-button[data-tab]').forEach((button) => {
      button.addEventListener('click', () => showTab(button.dataset.tab));
    });

    window.addEventListener('hashchange', () => showTab(tabFromHash(), { force: true, updateHash: false }));
    showTab(tabFromHash(), { force: true, updateHash: false });
    $('language-select').value = currentLanguage();
    $('language-select').addEventListener('change', () => {
      localStorage.setItem('song-request-linux-language', $('language-select').value);
      translateTree();
    });
    translateTree();

    $('request-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      try {
        const request = await api('/api/song-requests', {
          method: 'POST',
          body: JSON.stringify({
            requester: $('requester').value || 'heyleao',
            query: $('query').value
          })
        });
        setMessage('request-message', `Adicionado: ${request.title}`);
        $('query').value = '';
        await refresh();
      } catch (error) {
        setMessage('request-message', error.message, true);
      }
    });

    async function playerCommand(command) {
      try {
        const result = await sendCommand(command, true);
        setMessage('player-message', result.message || `${command} enviado`);
        await refresh();
      } catch (error) {
        setMessage('player-message', error.message, true);
      }
    }

    $('play-command').addEventListener('click', () => playerCommand('!play'));
    $('pause-command').addEventListener('click', () => playerCommand('!pause'));
    $('skip').addEventListener('click', () => playerCommand('!skip'));

    function currentVolumeLevel() {
      return Number(($('volume-level').textContent.match(/\d+/) || [50])[0]);
    }

    function clampVolume(level) {
      return Math.max(1, Math.min(100, level));
    }

    function scheduleVolumeApply() {
      clearTimeout(volumeTimer);
      volumeTimer = setTimeout(applyDesiredVolume, 180);
    }

    function adjustVolume(delta) {
      const base = desiredVolume ?? currentVolumeLevel();
      desiredVolume = clampVolume(base + delta);
      renderVolume({ level: desiredVolume, message: 'Volume desejado; aplicando em segundo plano...' });
      setMessage('player-message', `Volume desejado: ${desiredVolume}%`);
      scheduleVolumeApply();
    }

    async function applyDesiredVolume() {
      if (volumeInFlight || desiredVolume === null) return;
      const target = desiredVolume;
      volumeInFlight = true;
      try {
        const result = await api('/api/volume', {
          method: 'POST',
          body: JSON.stringify({ level: target })
        });
        if (desiredVolume === target) {
          desiredVolume = null;
          renderVolume(result.level === target ? result : { ...result, level: target });
          setMessage('player-message', result.message || `Volume ajustado para ${target}%.`);
        }
      } catch (error) {
        setMessage('player-message', `Volume ${target}% pendente: ${error.message}`, true);
      } finally {
        volumeInFlight = false;
        if (desiredVolume !== null && desiredVolume !== target) {
          scheduleVolumeApply();
        }
      }
    }

    async function copyToClipboard(value) {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(value);
        return;
      }
      const textarea = document.createElement('textarea');
      textarea.value = value;
      textarea.setAttribute('readonly', '');
      textarea.style.position = 'fixed';
      textarea.style.left = '-9999px';
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      textarea.remove();
    }

    document.addEventListener('click', async (event) => {
      const button = event.target.closest('.copy-button');
      if (!button) return;
      const targetId = button.dataset.copyTarget;
      const target = targetId ? document.getElementById(targetId) : null;
      const value = (target ? target.textContent : button.dataset.copyValue || '').trim();
      if (!value) return;
      try {
        await copyToClipboard(value);
        button.classList.add('copied');
        button.title = 'Copiado';
        button.setAttribute('aria-label', 'Copiado');
        setTimeout(() => {
          button.classList.remove('copied');
          button.title = 'Copiar';
          button.setAttribute('aria-label', 'Copiar');
        }, 1200);
      } catch (error) {
        button.title = 'Nao foi possivel copiar';
      }
    });

    $('volume-down').addEventListener('click', () => adjustVolume(-5));
    $('volume-up').addEventListener('click', () => adjustVolume(5));

    $('refresh-queue').addEventListener('click', refresh);
    $('refresh-events').addEventListener('click', refresh);
    $('refresh-events-preview').addEventListener('click', refresh);
    $('clear-queue').addEventListener('click', async () => {
      try {
        await api('/api/queue', { method: 'DELETE' });
        setMessage('queue-message', 'Fila zerada.');
        await refresh();
      } catch (error) {
        setMessage('queue-message', error.message, true);
      }
    });

    $('queue').addEventListener('click', async (event) => {
      const button = event.target.closest('.remove-queue-item');
      if (!button) return;
      const item = button.closest('.queue-item');
      button.disabled = true;
      if (item) {
        item.remove();
        if (!$('queue').querySelector('.queue-item')) {
          $('queue').innerHTML = '<div class="queue-item muted">Fila vazia</div>';
        }
      }
      try {
        await api(`/api/queue/${button.dataset.id}`, { method: 'DELETE' });
        setMessage('queue-message', 'Música removida da fila.');
        await refresh();
      } catch (error) {
        setMessage('queue-message', error.message, true);
        await refresh();
      }
    });

    async function saveSetup(message = 'Configuração salva.') {
      const result = await api('/api/config', {
        method: 'POST',
        body: JSON.stringify({
          default_provider: $('setup-provider').value,
          youtube_playback: $('setup-youtube-playback').value,
          pear_base_url: $('setup-pear-base-url').value,
          spotify_client_id: $('setup-spotify-client-id').value,
          spotify_fallback_enabled: $('setup-spotify-fallback-enabled').checked,
          queue_persistence_enabled: $('setup-queue-persistence-enabled').checked,
          twitch_client_id: $('setup-twitch-client-id').value,
          twitch_bot_username: $('setup-twitch-bot-username').value,
          twitch_channel: $('setup-twitch-channel').value,
          twitch_bot_oauth_token: null,
          youtube_api_key: $('setup-youtube-api-key').value,
          youtube_max_duration_seconds: Number($('setup-youtube-max-duration').value || 360),
          youtube_allow_non_music: $('setup-youtube-allow-non-music').checked,
          command_settings: commandSettingsFromForm(),
          queue_limits: queueLimitsFromForm(),
          overlay: overlayConfigFromForm()
        })
      });
      $('setup-youtube-api-key').value = '';
      setSetupDirty(false);
      setMessage('setup-message', message);
      await refresh();
      return result;
    }

    $('setup-form').addEventListener('input', (event) => {
      if (event.target.matches('input, select, textarea')) {
        setSetupDirty(true, 'Alteração pendente. Clique em Salvar configuração.');
      }
    });

    $('setup-form').addEventListener('change', (event) => {
      if (event.target.matches('input, select, textarea')) {
        setSetupDirty(true, 'Alteração pendente. Clique em Salvar configuração.');
      }
    });

    $('setup-operation-mode').addEventListener('change', () => {
      applyOperationMode($('setup-operation-mode').value, { dirty: true });
    });

    $('setup-spotify-fallback-enabled').addEventListener('change', () => {
      setSpotifyFallbackControls($('setup-spotify-fallback-enabled').checked);
      setSetupDirty(true, $('setup-spotify-fallback-enabled').checked
        ? 'Playlist fallback marcada. Clique em Salvar configuração.'
        : 'Playlist fallback desmarcada. Clique em Salvar configuração.');
    });

    ['setup-overlay-label', 'setup-overlay-lines'].forEach((id) => {
      $(id).addEventListener('input', () => renderOverlayGuide());
      $(id).addEventListener('change', () => renderOverlayGuide());
    });

    $('setup-queue-persistence-enabled').addEventListener('change', () => {
      setSetupDirty(true, $('setup-queue-persistence-enabled').checked
        ? 'Persistência da fila marcada. Clique em Salvar configuração.'
        : 'Persistência da fila desmarcada. Clique em Salvar configuração.');
    });

    $('provider-spotify').addEventListener('click', () => {
      applyOperationMode('spotify', { dirty: true });
      showTab('setup-tab');
    });

    $('provider-youtube-pear').addEventListener('click', () => {
      applyOperationMode('youtube_pear', { dirty: true });
      showTab('setup-tab');
    });

    $('provider-youtube-browser').addEventListener('click', () => {
      applyOperationMode('youtube_browser', { dirty: true });
      showTab('setup-tab');
    });

    $('global-save-setup').addEventListener('click', () => {
      $('setup-form').requestSubmit();
    });

    $('discard-setup').addEventListener('click', async () => {
      await discardSetupChanges();
    });

    $('setup-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      try {
        await saveSetup();
      } catch (error) {
        setMessage('setup-message', error.message, true);
      }
    });

    $('setup-spotify-login').addEventListener('click', async () => {
      try {
        const result = await api('/api/connections/spotify/start', { method: 'POST' });
        setMessage('setup-message', 'Abrindo login Spotify.');
        window.open(result.auth_url, '_blank', 'noopener,noreferrer');
      } catch (error) {
        setMessage('setup-message', error.message, true);
      }
    });

    $('setup-spotify-load-playlists').addEventListener('click', async () => {
      try {
        spotifyPlaylists = await api('/api/spotify/playlists');
        const current = (await api('/api/connections/status')).spotify.fallback_playlist;
        fillSpotifyPlaylistOptions(spotifyPlaylists, current?.id || '');
        setMessage('setup-message', `${spotifyPlaylists.length} playlist(s) carregada(s).`);
      } catch (error) {
        setMessage('setup-message', error.message, true);
      }
    });

    $('setup-spotify-save-playlist').addEventListener('click', async () => {
      try {
        const id = $('setup-spotify-fallback-playlist').value;
        const playlist = spotifyPlaylists.find((item) => item.id === id)
          || (await api('/api/connections/status')).spotify.fallback_playlist;
        if (!playlist) throw new Error('Carregue e selecione uma playlist primeiro.');
        await api('/api/spotify/fallback-playlist', {
          method: 'POST',
          body: JSON.stringify({ id: playlist.id, name: playlist.name, uri: playlist.uri })
        });
        $('setup-spotify-fallback-enabled').checked = true;
        setSpotifyFallbackControls(true);
        await saveSetup(`Playlist fallback salva e ativada: ${playlist.name}.`);
      } catch (error) {
        setMessage('setup-message', error.message, true);
      }
    });

    $('setup-twitch-login').addEventListener('click', async () => {
      try {
        const result = await api('/api/connections/twitch/start', { method: 'POST' });
        setMessage('setup-message', 'Abrindo login Twitch Bot.');
        window.open(result.auth_url, '_blank', 'noopener,noreferrer');
      } catch (error) {
        setMessage('setup-message', error.message, true);
      }
    });

    $('update-app').addEventListener('click', async () => {
      try {
        $('update-app').disabled = true;
        setMessage('update-message', 'Atualização iniciada. O app pode reiniciar e esta página pode cair por alguns segundos.');
        const result = await api('/api/update', {
          method: 'POST',
          headers: { 'x-song-request-action': 'update' }
        });
        setMessage('update-message', result.message || 'Atualização iniciada.');
        localStorage.setItem('song-request-linux-update-pending', '1');
        setTimeout(() => refreshUpdateStatus(false), 8000);
      } catch (error) {
        $('update-app').disabled = false;
        setMessage('update-message', error.message, true);
      }
    });

    $('instance-takeover').addEventListener('click', () => {
      markActiveInstance();
      refresh();
    });
    $('instance-close').addEventListener('click', closeThisTab);

    $('shutdown-app').addEventListener('click', () => {
      if (!confirm('Encerrar o Song Request Linux agora? Isso para o bot, a fila e o player local.')) return;
      $('refresh-state').textContent = 'SAINDO';
      setMessage('player-message', 'App encerrando. Esta aba pode ser fechada.');
      $('shutdown-app').disabled = true;
      instanceChannel?.postMessage({ type: 'shutdown', id: instanceId });

      fetch('/api/shutdown', {
        method: 'POST',
        keepalive: true,
        headers: { 'x-song-request-action': 'shutdown' }
      }).catch(() => {});
      setTimeout(() => {
        document.body.innerHTML = '<main><section><h2>Song Request Linux encerrado</h2><p class="muted">Esta aba pode ser fechada.</p></section></main>';
        window.close();
      }, 600);
    });

    markActiveInstance();
    refreshUpdateStatus(localStorage.getItem('song-request-linux-update-pending') === '1')
      .finally(() => localStorage.removeItem('song-request-linux-update-pending'));
    refresh();
    setInterval(refresh, 2500);
  </script>
</body>
</html>"##,
    )
}
