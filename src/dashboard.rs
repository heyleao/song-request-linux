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
    main {
      display: grid;
      grid-template-columns: minmax(320px, 420px) minmax(0, 1fr);
      gap: 14px;
      padding: 14px;
      max-width: 1320px;
      margin: 0 auto;
    }
    section {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 14px;
    }
    .stack { display: grid; gap: 14px; }
    .toolbar, .actions, .status-grid {
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
    .dot {
      width: 8px;
      height: 8px;
      border-radius: 999px;
      background: var(--warn);
    }
    .dot.ok { background: var(--ok); }
    .dot.bad { background: var(--bad); }
    label {
      display: grid;
      gap: 6px;
      color: var(--muted);
      font-size: 13px;
      margin-top: 12px;
    }
    input {
      width: 100%;
      min-height: 38px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      color: var(--text);
      padding: 8px 10px;
      font: inherit;
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
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 8px;
      margin-top: 12px;
    }
    .metric div, .current, .queue-item, .diagnostic-row {
      border: 1px solid var(--line);
      border-radius: 8px;
      background: var(--panel-2);
      padding: 10px;
    }
    .metric strong { display: block; font-size: 20px; }
    .metric span { color: var(--muted); font-size: 12px; }
    .current {
      display: grid;
      gap: 4px;
      min-height: 82px;
      margin-top: 12px;
    }
    .song-title {
      font-size: 19px;
      font-weight: 800;
      overflow-wrap: anywhere;
    }
    .queue {
      display: grid;
      gap: 8px;
      margin-top: 12px;
    }
    .queue-item {
      display: grid;
      gap: 3px;
      overflow-wrap: anywhere;
    }
    .diagnostics {
      display: grid;
      gap: 8px;
      margin-top: 12px;
    }
    .diagnostic-row {
      display: flex;
      justify-content: space-between;
      gap: 10px;
      color: var(--muted);
      font-size: 13px;
    }
    code { color: var(--ok); overflow-wrap: anywhere; }
    .message { min-height: 20px; color: var(--muted); font-size: 13px; margin-top: 10px; }
    .message.error { color: var(--bad); }
    @media (max-width: 900px) {
      header, main { grid-template-columns: 1fr; }
      .status-grid { justify-content: flex-start; }
      .metric { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
  <header>
    <h1>Song Request Linux</h1>
    <div class="status-grid">
      <span class="pill"><span class="dot" id="twitch-dot"></span>Twitch <strong id="twitch-state">...</strong></span>
      <span class="pill"><span class="dot ok"></span>Provider <strong id="provider">...</strong></span>
      <span class="pill"><span class="dot ok"></span>Local <strong>127.0.0.1</strong></span>
    </div>
  </header>

  <main>
    <div class="stack">
      <section>
        <div class="toolbar">
          <h2>Setup</h2>
          <a class="button secondary" href="/api/diagnostics" target="_blank" rel="noreferrer">Diagnostico</a>
          <a class="button secondary" href="/connections" target="_blank" rel="noreferrer">Conexoes</a>
          <a class="button secondary" href="/overlay" target="_blank" rel="noreferrer">Overlay</a>
        </div>
        <div class="diagnostics" id="setup-diagnostics"></div>
      </section>

      <section>
        <h2>Teste Rapido</h2>
        <form id="command-form">
          <label>Requester <input id="requester" autocomplete="off" value="heyleao"></label>
          <label>Comando <input id="command" autocomplete="off" value="!sr daft punk one more time"></label>
          <div class="actions">
            <button type="submit">Enviar</button>
            <button class="secondary" id="song" type="button">!song</button>
            <button class="secondary" id="queue-command" type="button">!fila</button>
            <button class="secondary" id="volume-command" type="button">!vol</button>
            <button class="secondary" id="help-command" type="button">!comandos</button>
            <button class="danger" id="skip" type="button">!skip</button>
          </div>
        </form>
        <div class="message" id="command-message"></div>
      </section>

      <section>
        <h2>Pedido Manual</h2>
        <form id="request-form">
          <label>Musica ou link <input id="query" autocomplete="off" placeholder="https://youtu.be/... ou nome da musica"></label>
          <button type="submit">Adicionar</button>
        </form>
        <div class="message" id="request-message"></div>
      </section>
    </div>

    <section>
      <div class="toolbar">
        <h2>Operacao</h2>
        <code>http://127.0.0.1:7384/overlay</code>
      </div>
      <div class="metric">
        <div><strong id="queue-count">0</strong><span>na fila</span></div>
        <div><strong id="current-source">-</strong><span>origem atual</span></div>
        <div><strong id="refresh-state">OK</strong><span>dashboard</span></div>
      </div>
      <div class="current">
        <div class="muted">Tocando agora</div>
        <div class="song-title" id="current-title">Aguardando pedido</div>
        <div id="current-meta" class="muted">Nenhuma musica tocando</div>
      </div>
      <div class="queue" id="queue"></div>
    </section>
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

    async function refresh() {
      try {
        const [status, queue, diagnostics, connections] = await Promise.all([
          api('/api/status'),
          api('/api/queue'),
          api('/api/diagnostics'),
          api('/api/connections/status')
        ]);

        $('provider').textContent = status.provider;
        $('queue-count').textContent = queue.queue_length;
        $('refresh-state').textContent = 'OK';

        const twitchReady = diagnostics.integrations.twitch.configured;
        $('twitch-state').textContent = twitchReady ? 'configurado' : 'pendente';
        $('twitch-dot').className = `dot ${twitchReady ? 'ok' : 'bad'}`;

        $('setup-diagnostics').innerHTML = [
          ['Bot Twitch', twitchReady ? 'configurado' : 'nao configurado'],
          ['Spotify', connections.spotify.token_configured ? 'conectado' : connections.spotify.client_id_configured ? 'login pendente' : 'client id pendente'],
          ['Overlay', 'http://127.0.0.1:7384/overlay'],
          ['Logs', diagnostics.storage.log_dir.exists ? 'ok' : 'pendente']
        ].map(([label, value]) => `
          <div class="diagnostic-row"><span>${escapeHtml(label)}</span><code>${escapeHtml(value)}</code></div>
        `).join('');

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
      } catch (error) {
        $('refresh-state').textContent = 'ERRO';
      }
    }

    async function sendCommand(message, isModerator = false) {
      return api('/api/chat-command', {
        method: 'POST',
        body: JSON.stringify({
          requester: $('requester').value || 'viewer',
          message,
          is_moderator: isModerator
        })
      });
    }

    $('command-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      try {
        const result = await sendCommand($('command').value);
        setMessage('command-message', `Resultado: ${result.status}`);
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

    $('skip').addEventListener('click', async () => {
      try {
        const result = await sendCommand('!skip', true);
        setMessage('command-message', `Skip: ${result.current_song ? result.current_song.title : 'fila vazia'}`);
        await refresh();
      } catch (error) {
        setMessage('command-message', error.message, true);
      }
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
    setInterval(refresh, 3000);
  </script>
</body>
</html>"#,
    )
}
