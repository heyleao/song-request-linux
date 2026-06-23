use axum::response::Html;

pub async fn page() -> Html<&'static str> {
    Html(
        r##"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Song Request Linux</title>
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
      background: var(--bg);
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
      border-bottom: 1px solid var(--line);
      background: rgba(16, 23, 32, .96);
      backdrop-filter: blur(12px);
    }
    h1, h2, h3, p { margin: 0; }
    h1 { font-size: 20px; line-height: 1.2; font-weight: 900; }
    h2 { font-size: 16px; line-height: 1.25; font-weight: 900; }
    h3 { font-size: 12px; color: var(--muted); text-transform: uppercase; letter-spacing: .04em; }
    main { padding: 18px; }
    .app-shell {
      display: grid;
      grid-template-columns: 248px minmax(0, 1fr);
      min-height: 100dvh;
      width: 100%;
      overflow-x: hidden;
    }
    .brand {
      display: flex;
      align-items: center;
      gap: 10px;
      min-width: 0;
    }
    .brand-mark {
      display: grid;
      place-items: center;
      width: 38px;
      height: 38px;
      border-radius: 8px;
      background: linear-gradient(135deg, #22c55e 0%, #5aa9ff 100%);
      color: #061016;
      font-weight: 950;
    }
    .brand span { display: block; color: var(--muted); font-size: 12px; margin-top: 2px; }
    .sidebar {
      position: sticky;
      top: 0;
      align-self: start;
      height: 100dvh;
      padding: 18px 14px;
      border-right: 1px solid var(--line);
      background: #0e141c;
      display: grid;
      grid-template-rows: auto 1fr auto;
      gap: 18px;
    }
    .nav-section { display: grid; gap: 7px; }
    .side-note {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface);
      padding: 10px;
      color: var(--muted);
      line-height: 1.45;
      font-size: 12px;
    }
    .top-status, .tabs, .actions, .toolbar, .inline-status, .provider-options {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 8px;
    }
    .top-status { justify-content: flex-end; }
    .pill {
      display: inline-flex;
      align-items: center;
      gap: 7px;
      min-height: 30px;
      padding: 5px 9px;
      border: 1px solid var(--line);
      border-radius: 999px;
      background: var(--surface-2);
      color: var(--muted);
      font-size: 13px;
      white-space: nowrap;
      min-width: 0;
    }
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
      border-color: var(--line);
      background: var(--surface-2);
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
      grid-template-columns: repeat(3, minmax(280px, 1fr));
      gap: 14px;
      align-items: start;
    }
    .grid-logs {
      display: grid;
      grid-template-columns: minmax(360px, 1fr) minmax(320px, 420px);
      gap: 14px;
      align-items: start;
    }
    section {
      background: var(--surface);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 14px;
      box-shadow: var(--shadow);
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
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface-2);
      padding: 10px;
    }
    .status-card { display: grid; gap: 6px; min-height: 76px; }
    .status-card span { color: var(--muted); font-size: 12px; }
    .status-card strong { font-size: 15px; overflow-wrap: anywhere; }
    .current {
      display: grid;
      gap: 6px;
      min-height: 112px;
      background: linear-gradient(180deg, #202a38 0%, #151d28 100%);
    }
    .song-title { font-size: 20px; font-weight: 900; overflow-wrap: anywhere; line-height: 1.25; }
    .song-meta { color: var(--muted); overflow-wrap: anywhere; }
    .provider-card {
      display: grid;
      gap: 12px;
      background: #111922;
    }
    .provider-options {
      align-items: stretch;
      display: grid;
      grid-template-columns: 1fr 1fr;
    }
    .provider-option {
      min-height: 74px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--surface-2);
      padding: 10px;
      display: grid;
      align-content: center;
      gap: 4px;
    }
    .provider-option.active {
      border-color: rgba(34, 197, 94, .75);
      background: linear-gradient(180deg, rgba(34, 197, 94, .14), rgba(29, 37, 48, 1));
    }
    .provider-option strong { font-size: 15px; }
    .provider-option span { color: var(--muted); font-size: 12px; line-height: 1.35; }
    form { display: grid; gap: 10px; }
    label {
      display: grid;
      gap: 6px;
      color: var(--muted);
      font-size: 13px;
      font-weight: 700;
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
    input:focus, select:focus, button:focus-visible, a:focus-visible {
      outline: 2px solid var(--focus);
      outline-offset: 2px;
    }
    button, a.button {
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
    button:hover, a.button:hover { background: #83c4ff; }
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
    .queue, .events, .diagnostics, .endpoints { display: grid; gap: 8px; }
    .queue-item, .event-row { display: grid; gap: 4px; overflow-wrap: anywhere; }
    .queue-item {
      grid-template-columns: minmax(0, 1fr) auto;
      align-items: center;
    }
    .queue-item .queue-meta { grid-column: 1 / -1; color: var(--muted); }
    .queue-item strong { font-size: 15px; }
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
      .app-shell { grid-template-columns: 1fr; }
      .sidebar {
        position: static;
        height: auto;
        width: 100%;
        border-right: 0;
        border-bottom: 1px solid var(--line);
      }
      .tabs { grid-template-columns: repeat(3, minmax(0, 1fr)); }
      .side-note { display: none; }
      .grid-config { grid-template-columns: 1fr 1fr; }
      .status-board { grid-template-columns: 1fr 1fr; }
    }
    @media (max-width: 880px) {
      header, .grid-main, .grid-config, .grid-logs { grid-template-columns: 1fr; }
      header { align-items: flex-start; }
      .page-title, .top-status { width: 100%; min-width: 0; }
      .top-status { justify-content: flex-start; }
      main { padding: 10px; }
      .toolbar { align-items: flex-start; }
    }
    @media (max-width: 520px) {
      .tabs { grid-template-columns: 1fr; }
      .provider-options { grid-template-columns: 1fr; }
      .status-board { grid-template-columns: 1fr; }
      .toolbar { display: grid; grid-template-columns: 1fr; }
      .diagnostic-row, .endpoint-row { grid-template-columns: 1fr; }
      .diagnostic-row code, .endpoint-row code { text-align: left; }
      .top-status { display: grid; grid-template-columns: 1fr; }
      .pill { width: 100%; white-space: normal; justify-content: flex-start; }
      button, a.button { width: 100%; }
      .actions { align-items: stretch; }
    }
  </style>
</head>
<body>
  <a class="skip-link" href="#main">Ir para o painel</a>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">SR</div>
        <div>
          <h1>Song Request Linux</h1>
          <span>Controle de músicas para live</span>
        </div>
      </div>
      <nav class="tabs" aria-label="Seções">
        <button class="tab-button active" data-tab="operation-tab" type="button"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="M4 13h5l2-7 4 14 2-7h3"/></svg></span>Operação</button>
        <button class="tab-button" data-tab="setup-tab" type="button"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z"/><path d="M19.4 15a1.8 1.8 0 0 0 .36 1.98l.03.03a2 2 0 1 1-2.83 2.83l-.03-.03A1.8 1.8 0 0 0 15 19.4a1.8 1.8 0 0 0-1 .6l-.02.02a2 2 0 1 1-3.96 0L10 20a1.8 1.8 0 0 0-1-.6 1.8 1.8 0 0 0-1.98.36l-.03.03a2 2 0 1 1-2.83-2.83l.03-.03A1.8 1.8 0 0 0 4.6 15a1.8 1.8 0 0 0-.6-1l-.02-.02a2 2 0 1 1 0-3.96L4 10a1.8 1.8 0 0 0 .6-1 1.8 1.8 0 0 0-.36-1.98l-.03-.03a2 2 0 1 1 2.83-2.83l.03.03A1.8 1.8 0 0 0 9 4.6a1.8 1.8 0 0 0 1-.6l.02-.02a2 2 0 1 1 3.96 0L14 4a1.8 1.8 0 0 0 1 .6 1.8 1.8 0 0 0 1.98-.36l.03-.03a2 2 0 1 1 2.83 2.83l-.03.03A1.8 1.8 0 0 0 19.4 9c.22.38.43.61.6 1l.02.02a2 2 0 1 1 0 3.96L20 14a1.8 1.8 0 0 0-.6 1Z"/></svg></span>Configuração</button>
        <button class="tab-button" data-tab="logs-tab" type="button"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="M4 6h16M4 12h16M4 18h10"/></svg></span>Logs</button>
        <button class="tab-button" data-tab="guide-tab" type="button"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M4 4.5A2.5 2.5 0 0 1 6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15Z"/></svg></span>Guia</button>
        <a class="tab-button" href="/overlay" target="_blank" rel="noreferrer"><span class="nav-icon"><svg viewBox="0 0 24 24"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="M7 9h5M7 13h10"/></svg></span>Overlay</a>
        <a class="tab-button" href="/player" target="_blank" rel="noreferrer"><span class="nav-icon"><svg viewBox="0 0 24 24"><path d="m8 5 11 7-11 7V5Z"/></svg></span>Player OBS</a>
      </nav>
      <div class="side-note">Use um provider por vez. Links do YouTube entram direto no YouTube/Pear; texto segue o provider ativo.</div>
    </aside>

    <div class="content">
      <header>
        <div class="page-title">
          <h1>Operação da live</h1>
          <p>Fila, player, eventos e conexões em uma tela.</p>
        </div>
        <div class="top-status">
          <span class="pill"><span class="dot" id="twitch-dot"></span>Twitch <strong id="twitch-state">...</strong></span>
          <span class="pill"><span class="dot" id="spotify-dot"></span>Spotify <strong id="spotify-state">...</strong></span>
          <span class="pill"><span class="dot" id="youtube-dot"></span>YouTube <strong id="youtube-state">...</strong></span>
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
              <span class="pill compact"><span class="dot ok"></span>Modo <strong id="provider-mode">...</strong></span>
            </div>
            <div class="provider-options">
              <div class="provider-option" id="provider-spotify"><strong>Spotify</strong><span>Busca e fila pelo app Spotify.</span></div>
              <div class="provider-option" id="provider-youtube"><strong>YouTube/Pear</strong><span>Pedidos via YouTube Music ou Browser Source.</span></div>
            </div>
            <div class="hint" id="provider-detail">Carregando modo atual...</div>
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
                <button class="secondary" id="song" type="button">Ver atual</button>
                <button class="secondary" id="queue-command" type="button">Ver fila</button>
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
              <button class="secondary" id="volume-command" type="button">Volume</button>
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
      <form id="setup-form">
        <div class="grid-config">
          <section>
            <h2>Twitch</h2>
            <label>Client ID
              <input id="setup-twitch-client-id" autocomplete="off" placeholder="Client ID do app Twitch">
            </label>
            <label>Conta do bot
              <input id="setup-twitch-bot-username" autocomplete="off" placeholder="conta_bot">
            </label>
            <label>Canal
              <input id="setup-twitch-channel" autocomplete="off" placeholder="canal_do_streamer">
            </label>
            <div class="actions">
              <button class="secondary" id="setup-twitch-login" type="button">Conectar bot</button>
            </div>
          </section>

          <section>
            <h2>Spotify</h2>
            <label>Provider padrão
              <select id="setup-provider">
                <option value="spotify">Spotify</option>
                <option value="youtube">YouTube</option>
              </select>
            </label>
            <label>Client ID
              <input id="setup-spotify-client-id" autocomplete="off" placeholder="Client ID do app Spotify">
            </label>
            <div class="actions">
              <button class="secondary" id="setup-spotify-login" type="button">Login Spotify</button>
            </div>
          </section>

          <section>
            <h2>YouTube</h2>
            <label>Player
              <select id="setup-youtube-playback">
                <option value="pear">Pear Desktop</option>
                <option value="browser">Browser Source OBS</option>
              </select>
            </label>
            <label>Pear API
              <input id="setup-pear-base-url" autocomplete="off" placeholder="http://127.0.0.1:26538/api/v1">
            </label>
            <label>API Key
              <input id="setup-youtube-api-key" autocomplete="off" placeholder="deixe vazio para manter a chave atual">
            </label>
            <label>Máximo em segundos
              <input id="setup-youtube-max-duration" type="number" inputmode="numeric" min="30" max="86400" step="30" value="360">
            </label>
            <label><span><input id="setup-youtube-allow-non-music" type="checkbox"> Aceitar resultados fora da categoria Música</span></label>
          </section>
        </div>

        <section>
          <div class="toolbar">
            <h2>Salvar configuração</h2>
            <div class="actions">
              <button type="submit">Salvar</button>
            </div>
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
          <h2>Links de configuração</h2>
          <div class="endpoints">
            <div class="endpoint-row"><span>Spotify Developer Dashboard</span><a href="https://developer.spotify.com/dashboard" target="_blank" rel="noreferrer">Abrir</a></div>
            <div class="endpoint-row"><span>Spotify Redirect URI</span><code>http://127.0.0.1:7384/auth/spotify/callback</code></div>
            <div class="endpoint-row"><span>Twitch Developer Console</span><a href="https://dev.twitch.tv/console/apps" target="_blank" rel="noreferrer">Abrir</a></div>
            <div class="endpoint-row"><span>Twitch Redirect URI</span><code>https://localhost:7443/auth/twitch/callback</code></div>
            <div class="endpoint-row"><span>Google Cloud Credentials</span><a href="https://console.cloud.google.com/apis/credentials" target="_blank" rel="noreferrer">Abrir</a></div>
            <div class="endpoint-row"><span>Guia completo</span><a href="https://github.com/heyleao/song-request-linux/blob/main/docs/SETUP.md" target="_blank" rel="noreferrer">docs/SETUP.md</a></div>
          </div>
        </section>
        <section>
          <h2>URLs locais</h2>
          <div class="endpoints">
            <div class="endpoint-row"><span>Dashboard</span><code>http://127.0.0.1:7384/</code></div>
            <div class="endpoint-row"><span>Overlay OBS</span><code>http://127.0.0.1:7384/overlay</code></div>
            <div class="endpoint-row"><span>Player OBS</span><code>http://127.0.0.1:7384/player</code></div>
            <div class="endpoint-row"><span>Pear API</span><code>http://127.0.0.1:26538/api/v1</code></div>
          </div>
        </section>
      </div>
    </div>
      </main>
    </div>
  </div>

  <script>
    const $ = (id) => document.getElementById(id);

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

    function sourceLabel(source) {
      if (!source) return '-';
      if (source.type === 'youtube') return 'YouTube';
      if (source.type === 'spotify') return 'Spotify';
      if (source.type === 'search') return source.provider === 'spotify' ? 'Spotify' : 'YouTube';
      return source.type;
    }

    function stateClass(ok, pending = false) {
      if (ok) return 'dot ok';
      return pending ? 'dot' : 'dot bad';
    }

    function setMessage(id, text, isError = false) {
      const element = $(id);
      element.textContent = text;
      element.classList.toggle('error', isError);
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

    function renderDiagnostics(diagnostics, connections, pear, config) {
      const twitchReady = diagnostics.integrations.twitch.configured;
      const spotifyReady = connections.spotify.token_configured;
      const spotifyConfigured = connections.spotify.client_id_configured;
      const youtubeReady = config.youtube_playback === 'pear' ? pear.reachable : config.youtube_api_key_configured;

      $('twitch-state').textContent = twitchReady ? 'configurado' : 'pendente';
      $('twitch-dot').className = stateClass(twitchReady);
      $('spotify-state').textContent = spotifyReady ? 'conectado' : spotifyConfigured ? 'login' : 'pendente';
      $('spotify-dot').className = stateClass(spotifyReady, spotifyConfigured);
      $('youtube-state').textContent = config.youtube_playback === 'pear'
        ? pear.reachable ? 'Pear ok' : 'Pear pendente'
        : config.youtube_api_key_configured ? 'API ok' : 'API pendente';
      $('youtube-dot').className = stateClass(youtubeReady, config.youtube_api_key_configured || config.youtube_playback === 'pear');
      $('provider-mode').textContent = config.default_provider === 'spotify' ? 'Spotify' : 'YouTube';
      $('provider-spotify').classList.toggle('active', config.default_provider === 'spotify');
      $('provider-youtube').classList.toggle('active', config.default_provider === 'youtube');
      $('provider-detail').textContent = config.default_provider === 'spotify'
        ? 'Texto do !sr busca no Spotify. Links do YouTube continuam entrando direto no YouTube/Pear.'
        : 'Texto do !sr busca no YouTube. Use Spotify apenas quando trocar o provider para Spotify.';

      const rows = [
        ['Bot Twitch', twitchReady ? 'configurado' : 'não configurado'],
        ['Spotify', spotifyReady ? 'conectado' : spotifyConfigured ? 'login pendente' : 'client id pendente'],
        ['YouTube', `${config.youtube_playback === 'pear' ? 'Pear Desktop' : 'Browser Source'} - ${config.youtube_api_key_configured ? 'api key configurada' : 'api key pendente'}`],
        ['Pear Desktop', pear.configured ? pear.reachable ? 'conectado' : 'não encontrado' : 'desativado'],
        ['Pear atual', pear.now_playing || '-'],
        ['Logs', diagnostics.storage.log_dir.exists ? 'ok' : 'pendente']
      ];
      const html = rows.map(([label, value]) => `
        <div class="diagnostic-row"><span>${escapeHtml(label)}</span><code>${escapeHtml(value)}</code></div>
      `).join('');
      $('setup-diagnostics').innerHTML = html;
      $('setup-summary').innerHTML = html;
    }

    async function refresh() {
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

        $('queue-count').textContent = `${queue.queue_length} pedido(s)`;
        $('refresh-state').textContent = 'OK';
        $('playback-mode').textContent = config.youtube_playback === 'pear' ? 'Pear Desktop' : 'Browser Source';
        renderDiagnostics(diagnostics, connections, pear, config);

        if (!$('setup-form').contains(document.activeElement)) {
          $('setup-provider').value = config.default_provider;
          $('setup-spotify-client-id').value = config.spotify_client_id || '';
          $('setup-twitch-client-id').value = config.twitch_client_id || '';
          $('setup-twitch-bot-username').value = config.twitch_bot_username || '';
          $('setup-twitch-channel').value = config.twitch_channel || '';
          $('setup-youtube-playback').value = config.youtube_playback || 'pear';
          $('setup-pear-base-url').value = config.pear_base_url || 'http://127.0.0.1:26538/api/v1';
          $('setup-youtube-max-duration').value = config.youtube_max_duration_seconds || 360;
          $('setup-youtube-allow-non-music').checked = Boolean(config.youtube_allow_non_music);
        }

        const current = queue.current_song;
        $('current-title').textContent = current ? current.title : 'Aguardando pedido';
        $('current-meta').textContent = current
          ? `${current.artist} - pedido por ${current.requester}`
          : 'Nenhuma música tocando';
        $('current-source').textContent = sourceLabel(current?.source);
        $('queue').innerHTML = queue.queue.length
          ? queue.queue.map((item, index) => `
              <div class="queue-item">
                <strong>${index + 1}. ${escapeHtml(item.title)}</strong>
                <span class="pill compact">${escapeHtml(sourceLabel(item.source))}</span>
                <span class="queue-meta">${escapeHtml(item.artist)} - pedido por ${escapeHtml(item.requester)}</span>
              </div>
            `).join('')
          : '<div class="queue-item muted">Fila vazia</div>';
        renderEvents(events);
      } catch (error) {
        $('refresh-state').textContent = 'ERRO';
      }
    }

    async function sendCommand(message, isModerator = false) {
      if (!message.trim()) throw new Error('Digite um comando.');
      return api('/api/chat-command', {
        method: 'POST',
        body: JSON.stringify({
          requester: $('requester').value || 'viewer',
          message,
          is_moderator: isModerator
        })
      });
    }

    document.querySelectorAll('.tab-button[data-tab]').forEach((button) => {
      button.addEventListener('click', () => {
        document.querySelectorAll('.tab-button[data-tab]').forEach((item) => item.classList.remove('active'));
        document.querySelectorAll('.tab').forEach((item) => item.classList.remove('active'));
        button.classList.add('active');
        document.getElementById(button.dataset.tab).classList.add('active');
      });
    });

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

    $('song').addEventListener('click', async () => {
      try {
        const result = await sendCommand('!song');
        const title = result.current_song ? result.current_song.title : 'fila vazia';
        setMessage('request-message', `Atual: ${title}`);
        await refresh();
      } catch (error) {
        setMessage('request-message', error.message, true);
      }
    });

    $('queue-command').addEventListener('click', async () => {
      try {
        const result = await sendCommand('!fila');
        setMessage('request-message', `Fila: ${result.queue.queue_length} pedido(s)`);
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
    $('volume-command').addEventListener('click', async () => {
      try {
        const result = await sendCommand('!vol');
        setMessage('player-message', result.message || 'Volume consultado');
        await refresh();
      } catch (error) {
        setMessage('player-message', error.message, true);
      }
    });

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

    $('setup-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      try {
        await api('/api/config', {
          method: 'POST',
          body: JSON.stringify({
            default_provider: $('setup-provider').value,
            youtube_playback: $('setup-youtube-playback').value,
            pear_base_url: $('setup-pear-base-url').value,
            spotify_client_id: $('setup-spotify-client-id').value,
            twitch_client_id: $('setup-twitch-client-id').value,
            twitch_bot_username: $('setup-twitch-bot-username').value,
            twitch_channel: $('setup-twitch-channel').value,
            twitch_bot_oauth_token: null,
            youtube_api_key: $('setup-youtube-api-key').value,
            youtube_max_duration_seconds: Number($('setup-youtube-max-duration').value || 360),
            youtube_allow_non_music: $('setup-youtube-allow-non-music').checked
          })
        });
        $('setup-youtube-api-key').value = '';
        setMessage('setup-message', 'Configuração salva.');
        await refresh();
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

    $('setup-twitch-login').addEventListener('click', async () => {
      try {
        const result = await api('/api/connections/twitch/start', { method: 'POST' });
        setMessage('setup-message', 'Abrindo login Twitch Bot.');
        window.open(result.auth_url, '_blank', 'noopener,noreferrer');
      } catch (error) {
        setMessage('setup-message', error.message, true);
      }
    });

    $('shutdown-app').addEventListener('click', () => {
      $('refresh-state').textContent = 'SAINDO';
      setMessage('player-message', 'App encerrando. Esta aba pode ser fechada.');
      $('shutdown-app').disabled = true;

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

    refresh();
    setInterval(refresh, 2500);
  </script>
</body>
</html>"##,
    )
}
