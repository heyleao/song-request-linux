use axum::response::Html;

pub async fn page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Song Request Linux</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #0f1115;
      --panel: #181b21;
      --panel-2: #20242c;
      --panel-3: #14171d;
      --text: #f3f5f7;
      --muted: #a9b0bb;
      --line: #343945;
      --ok: #2bd180;
      --warn: #f0b84b;
      --bad: #ff6b6b;
      --action: #62a8ff;
      font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    * { box-sizing: border-box; }
    body { margin: 0; background: var(--bg); color: var(--text); }
    header {
      display: grid;
      grid-template-columns: minmax(220px, 1fr) auto;
      gap: 16px;
      align-items: center;
      min-height: 64px;
      padding: 14px 20px;
      border-bottom: 1px solid var(--line);
      background: #151820;
    }
    h1 { margin: 0; font-size: 19px; line-height: 1.2; }
    h2 { margin: 0; font-size: 15px; }
    main { max-width: 1320px; margin: 0 auto; padding: 14px; }
    .status-grid, .tabs, .actions, .toolbar {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
    }
    .status-grid { justify-content: flex-end; }
    .pill {
      display: inline-flex;
      align-items: center;
      gap: 7px;
      min-height: 30px;
      padding: 5px 9px;
      border: 1px solid var(--line);
      border-radius: 999px;
      background: var(--panel-2);
      color: var(--muted);
      font-size: 13px;
      white-space: nowrap;
    }
    .dot { width: 8px; height: 8px; border-radius: 999px; background: var(--warn); }
    .dot.ok { background: var(--ok); }
    .dot.bad { background: var(--bad); }
    .tabs {
      margin-bottom: 12px;
      border-bottom: 1px solid var(--line);
      padding-bottom: 8px;
    }
    .tab-button {
      min-height: 34px;
      border: 1px solid transparent;
      border-radius: 6px;
      background: transparent;
      color: var(--muted);
      padding: 7px 10px;
      font-weight: 700;
      cursor: pointer;
    }
    .tab-button.active {
      border-color: var(--line);
      background: var(--panel-2);
      color: var(--text);
    }
    .tab { display: none; }
    .tab.active { display: block; }
    .layout {
      display: grid;
      grid-template-columns: minmax(320px, 420px) minmax(0, 1fr);
      gap: 14px;
    }
    section {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 14px;
    }
    .stack { display: grid; gap: 14px; }
    label {
      display: grid;
      gap: 6px;
      color: var(--muted);
      font-size: 13px;
      margin-top: 12px;
    }
    input, select {
      width: 100%;
      min-height: 38px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      color: var(--text);
      padding: 8px 10px;
      font: inherit;
    }
    input[type="checkbox"] {
      width: auto;
      min-height: auto;
    }
    button, a.button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 36px;
      border: 1px solid #3d7fc8;
      border-radius: 6px;
      background: var(--action);
      color: #07111f;
      padding: 8px 11px;
      font-weight: 700;
      text-decoration: none;
      cursor: pointer;
    }
    button.secondary, a.secondary {
      border-color: var(--line);
      background: var(--panel-2);
      color: var(--text);
    }
    button.danger {
      border-color: #b84949;
      background: #ff7b7b;
      color: #210606;
    }
    .muted { color: var(--muted); }
    .metric {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 8px;
      margin-top: 12px;
    }
    .metric div, .current, .queue-item, .diagnostic-row, .event-row {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--panel-2);
      padding: 10px;
    }
    .metric strong { display: block; font-size: 20px; }
    .metric span { color: var(--muted); font-size: 12px; }
    .current { display: grid; gap: 4px; min-height: 82px; margin-top: 12px; }
    .song-title { font-size: 19px; font-weight: 800; overflow-wrap: anywhere; }
    .queue, .events, .diagnostics { display: grid; gap: 8px; margin-top: 12px; }
    .queue-item, .event-row { display: grid; gap: 3px; overflow-wrap: anywhere; }
    .event-row strong { font-size: 12px; color: var(--ok); text-transform: uppercase; }
    .event-row.error strong { color: var(--bad); }
    .event-row.player strong { color: var(--action); }
    .event-row.volume strong { color: var(--warn); }
    .diagnostic-row {
      display: flex;
      justify-content: space-between;
      gap: 10px;
      color: var(--muted);
      font-size: 13px;
    }
    .diagnostic-row a { color: var(--action); font-weight: 700; text-decoration: none; }
    .diagnostic-row a:hover { text-decoration: underline; }
    code { color: var(--ok); overflow-wrap: anywhere; }
    .message { min-height: 20px; color: var(--muted); font-size: 13px; margin-top: 10px; }
    .message.error { color: var(--bad); }
    @media (max-width: 900px) {
      header, .layout { grid-template-columns: 1fr; }
      .status-grid { justify-content: flex-start; }
      .metric { grid-template-columns: 1fr 1fr; }
    }
  </style>
</head>
<body>
  <header>
    <h1>Song Request Linux</h1>
    <div class="status-grid">
      <span class="pill"><span class="dot" id="twitch-dot"></span>Twitch <strong id="twitch-state">...</strong></span>
      <span class="pill"><span class="dot ok"></span>Modo <strong id="provider">...</strong></span>
      <span class="pill"><span class="dot ok"></span>Local <strong>127.0.0.1</strong></span>
      <button class="secondary" id="shutdown-app" type="button">Encerrar</button>
    </div>
  </header>

  <main>
    <nav class="tabs">
      <button class="tab-button active" data-tab="overview">Visao geral</button>
      <button class="tab-button" data-tab="queue-tab">Fila</button>
      <a class="tab-button" href="/overlay" target="_blank" rel="noreferrer">Overlay</a>
      <button class="tab-button" data-tab="commands-tab">Comandos</button>
      <button class="tab-button" data-tab="player-tab">Player</button>
      <button class="tab-button" data-tab="logs-tab">Logs</button>
      <button class="tab-button" data-tab="setup-tab">Setup</button>
      <button class="tab-button" data-tab="guide-tab">Guia</button>
    </nav>

    <div class="tab active" id="overview">
      <div class="layout">
        <section>
          <div class="toolbar">
            <h2>Operacao</h2>
            <code>http://127.0.0.1:7384/overlay</code>
          </div>
          <div class="metric">
            <div><strong id="queue-count">0</strong><span>na fila</span></div>
            <div><strong id="current-source">-</strong><span>origem atual</span></div>
            <div><strong id="event-count">0</strong><span>eventos</span></div>
            <div><strong id="refresh-state">OK</strong><span>dashboard</span></div>
          </div>
          <div class="current">
            <div class="muted">Tocando agora</div>
            <div class="song-title" id="current-title">Aguardando pedido</div>
            <div id="current-meta" class="muted">Nenhuma musica tocando</div>
          </div>
        </section>
        <section>
          <h2>Log ao vivo</h2>
          <div class="events" id="events-preview"></div>
        </section>
      </div>
    </div>

    <div class="tab" id="queue-tab">
      <section>
        <div class="toolbar">
          <h2>Fila</h2>
          <button class="secondary" id="refresh-queue" type="button">Atualizar</button>
        </div>
        <div class="queue" id="queue"></div>
      </section>
    </div>

    <div class="tab" id="commands-tab">
      <div class="layout">
        <section>
          <h2>Teste rapido</h2>
          <form id="command-form">
            <label>Requester <input id="requester" autocomplete="off" value="heyleao"></label>
            <label>Comando <input id="command" autocomplete="off" placeholder="!sr nome da musica ou link YouTube"></label>
            <div class="actions">
              <button type="submit">Enviar</button>
              <button class="secondary" id="song" type="button">!song</button>
              <button class="secondary" id="queue-command" type="button">!fila</button>
              <button class="secondary" id="volume-command" type="button">!vol</button>
              <button class="secondary" id="help-command" type="button">!comandos</button>
            </div>
          </form>
          <div class="message" id="command-message"></div>
        </section>
        <section>
          <h2>Pedido manual</h2>
          <form id="request-form">
            <label>Musica ou link <input id="query" autocomplete="off" placeholder="nome da musica no Spotify ou link YouTube"></label>
            <button type="submit">Adicionar</button>
          </form>
          <div class="message" id="request-message"></div>
        </section>
      </div>
    </div>

    <div class="tab" id="player-tab">
      <section>
        <h2>Player</h2>
        <div class="actions">
          <button class="secondary" id="play-command" type="button">!play</button>
          <button class="secondary" id="pause-command" type="button">!pause</button>
          <button class="danger" id="skip" type="button">!skip</button>
        </div>
        <div class="message" id="player-message"></div>
      </section>
    </div>

    <div class="tab" id="logs-tab">
      <div class="layout">
        <section>
          <div class="toolbar">
            <h2>Logs em tempo real</h2>
            <button class="secondary" id="refresh-events" type="button">Atualizar</button>
          </div>
          <div class="events" id="events"></div>
        </section>
        <section>
          <h2>Diagnostico</h2>
          <div class="diagnostics" id="setup-diagnostics"></div>
        </section>
      </div>
    </div>

    <div class="tab" id="setup-tab">
      <section>
        <h2>Setup</h2>
        <form id="setup-form">
          <label>Provider padrao
            <select id="setup-provider">
              <option value="spotify">Spotify</option>
              <option value="youtube">YouTube</option>
            </select>
          </label>
          <label>Spotify Client ID
            <input id="setup-spotify-client-id" autocomplete="off" placeholder="Client ID do app Spotify">
          </label>
          <label>Twitch Client ID
            <input id="setup-twitch-client-id" autocomplete="off" placeholder="Client ID do app Twitch">
          </label>
          <label>Twitch Bot Username
            <input id="setup-twitch-bot-username" autocomplete="off" placeholder="conta_bot">
          </label>
          <label>Twitch Channel
            <input id="setup-twitch-channel" autocomplete="off" placeholder="canal_do_streamer">
          </label>
          <label>YouTube API Key
            <input id="setup-youtube-api-key" autocomplete="off" placeholder="deixe vazio para manter a chave atual">
          </label>
          <label>Maximo YouTube (segundos)
            <input id="setup-youtube-max-duration" type="number" min="30" max="86400" step="30" value="360">
          </label>
          <label><span><input id="setup-youtube-allow-non-music" type="checkbox"> Aceitar YouTube nao marcado como musica</span></label>
          <div class="actions">
            <button type="submit">Salvar setup</button>
            <button class="secondary" id="setup-spotify-login" type="button">Login Spotify</button>
            <button class="secondary" id="setup-twitch-login" type="button">Conectar bot Twitch</button>
          </div>
        </form>
        <div class="message" id="setup-message"></div>
      </section>
    </div>

    <div class="tab" id="guide-tab">
      <section>
        <h2>Guia rapido</h2>
        <div class="diagnostics">
          <div class="diagnostic-row"><span>Spotify Client ID: crie um app e copie o Client ID.</span><a href="https://developer.spotify.com/dashboard" target="_blank" rel="noreferrer">Spotify Dashboard</a></div>
          <div class="diagnostic-row"><span>Spotify Redirect URI: cole no app Spotify.</span><code>http://127.0.0.1:7384/auth/spotify/callback</code></div>
          <div class="diagnostic-row"><span>Twitch Client ID: crie um app publico para o bot.</span><a href="https://dev.twitch.tv/console/apps" target="_blank" rel="noreferrer">Twitch Console</a></div>
          <div class="diagnostic-row"><span>Twitch Redirect URI: cole no app Twitch.</span><code>https://localhost:7443/auth/twitch/callback</code></div>
          <div class="diagnostic-row"><span>YouTube API Key: ative YouTube Data API v3 e crie uma chave.</span><a href="https://console.cloud.google.com/apis/credentials" target="_blank" rel="noreferrer">Google Cloud Credentials</a></div>
          <div class="diagnostic-row"><span>Guia completo de configuracao.</span><a href="https://github.com/heyleao/song-request-linux/blob/main/docs/SETUP.md" target="_blank" rel="noreferrer">docs/SETUP.md</a></div>
        </div>
      </section>
    </div>
  </main>

  <script>
    const $ = (id) => document.getElementById(id);

    async function api(path, options = {}) {
      const response = await fetch(path, {
        headers: { 'content-type': 'application/json', ...(options.headers || {}) },
        ...options
      });
      const data = await response.json();
      if (!response.ok) throw new Error(data.error || 'Falha na requisicao');
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
      if (source.type === 'search') return source.provider;
      return source.type;
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
      $('events-preview').innerHTML = events.slice(0, 8).length
        ? events.slice(0, 8).map((event) => `
            <div class="event-row ${escapeHtml(event.kind)}">
              <strong>${escapeHtml(event.kind)}</strong>
              <span>${escapeHtml(event.message)}</span>
            </div>
          `).join('')
        : '<div class="event-row muted">Nenhum evento ainda</div>';
    }

    async function refresh() {
      try {
        const [status, queue, diagnostics, connections, events, config] = await Promise.all([
          api('/api/status'),
          api('/api/queue'),
          api('/api/diagnostics'),
          api('/api/connections/status'),
          api('/api/events'),
          api('/api/config')
        ]);

        $('provider').textContent = status.provider === 'spotify' ? 'Spotify + YouTube links' : status.provider;
        $('queue-count').textContent = queue.queue_length;
        $('refresh-state').textContent = 'OK';

        const twitchReady = diagnostics.integrations.twitch.configured;
        $('twitch-state').textContent = twitchReady ? 'configurado' : 'pendente';
        $('twitch-dot').className = `dot ${twitchReady ? 'ok' : 'bad'}`;

        $('setup-diagnostics').innerHTML = [
          ['Bot Twitch', twitchReady ? 'configurado' : 'nao configurado'],
          ['Spotify', connections.spotify.token_configured ? 'conectado' : connections.spotify.client_id_configured ? 'login pendente' : 'client id pendente'],
          ['YouTube', config.youtube_api_key_configured ? 'api key configurada' : 'api key pendente'],
          ['Logs', diagnostics.storage.log_dir.exists ? 'ok' : 'pendente']
        ].map(([label, value]) => `
          <div class="diagnostic-row"><span>${escapeHtml(label)}</span><code>${escapeHtml(value)}</code></div>
        `).join('');

        if (!$('setup-form').contains(document.activeElement)) {
          $('setup-provider').value = config.default_provider;
          $('setup-spotify-client-id').value = config.spotify_client_id || '';
          $('setup-twitch-client-id').value = config.twitch_client_id || '';
          $('setup-twitch-bot-username').value = config.twitch_bot_username || '';
          $('setup-twitch-channel').value = config.twitch_channel || '';
          $('setup-youtube-max-duration').value = config.youtube_max_duration_seconds || 360;
          $('setup-youtube-allow-non-music').checked = Boolean(config.youtube_allow_non_music);
        }

        const current = queue.current_song;
        $('current-title').textContent = current ? current.title : 'Aguardando pedido';
        $('current-meta').textContent = current
          ? `${current.artist} - pedido por ${current.requester}`
          : 'Nenhuma musica tocando';
        $('current-source').textContent = sourceLabel(current?.source);
        $('queue').innerHTML = queue.queue.length
          ? queue.queue.map((item) => `
              <div class="queue-item">
                <strong>${escapeHtml(item.title)}</strong>
                <span class="muted">${escapeHtml(item.artist)} - pedido por ${escapeHtml(item.requester)}</span>
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

    $('command-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      try {
        const result = await sendCommand($('command').value);
        setMessage('command-message', result.message || `Resultado: ${result.status}`);
        await refresh();
      } catch (error) {
        setMessage('command-message', error.message, true);
      }
    });

    $('song').addEventListener('click', async () => {
      try {
        const result = await sendCommand('!song');
        const title = result.current_song ? result.current_song.title : 'fila vazia';
        setMessage('command-message', `Atual: ${title}`);
        await refresh();
      } catch (error) {
        setMessage('command-message', error.message, true);
      }
    });

    $('queue-command').addEventListener('click', async () => {
      try {
        const result = await sendCommand('!fila');
        setMessage('command-message', `Fila: ${result.queue.queue_length} pedido(s)`);
        await refresh();
      } catch (error) {
        setMessage('command-message', error.message, true);
      }
    });

    $('volume-command').addEventListener('click', async () => {
      try {
        const result = await sendCommand('!vol');
        setMessage('command-message', result.message || 'Volume consultado');
        await refresh();
      } catch (error) {
        setMessage('command-message', error.message, true);
      }
    });

    $('help-command').addEventListener('click', async () => {
      try {
        const result = await sendCommand('!comandos');
        setMessage('command-message', `Comandos: ${result.commands.join(', ')}`);
      } catch (error) {
        setMessage('command-message', error.message, true);
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
    $('refresh-queue').addEventListener('click', refresh);
    $('refresh-events').addEventListener('click', refresh);
    $('setup-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      try {
        await api('/api/config', {
          method: 'POST',
          body: JSON.stringify({
            default_provider: $('setup-provider').value,
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
        setMessage('setup-message', 'Setup salvo.');
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

    refresh();
    setInterval(refresh, 2500);
  </script>
</body>
</html>"#,
    )
}
