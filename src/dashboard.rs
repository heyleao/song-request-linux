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
      --bg: #101114;
      --panel: #191b20;
      --panel-2: #20232a;
      --text: #f4f6f8;
      --muted: #aab1bd;
      --line: #343842;
      --accent: #29c184;
      font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    * { box-sizing: border-box; }
    body { margin: 0; background: var(--bg); color: var(--text); }
    header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
      padding: 18px 24px;
      border-bottom: 1px solid var(--line);
      background: #15171c;
    }
    h1 { margin: 0; font-size: 20px; font-weight: 700; }
    main {
      display: grid;
      grid-template-columns: minmax(320px, 420px) minmax(0, 1fr);
      gap: 18px;
      padding: 18px;
      max-width: 1280px;
      margin: 0 auto;
    }
    section {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 16px;
    }
    h2 { margin: 0 0 12px; font-size: 15px; font-weight: 700; }
    label {
      display: grid;
      gap: 6px;
      color: var(--muted);
      font-size: 13px;
      margin-bottom: 12px;
    }
    input {
      width: 100%;
      min-height: 40px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      color: var(--text);
      padding: 9px 10px;
      font: inherit;
    }
    button, a.button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 38px;
      border: 1px solid #21885f;
      border-radius: 6px;
      background: var(--accent);
      color: #06110c;
      padding: 8px 12px;
      font-weight: 700;
      text-decoration: none;
      cursor: pointer;
    }
    button.secondary, a.secondary {
      border-color: var(--line);
      background: var(--panel-2);
      color: var(--text);
    }
    .actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; }
    .status { display: grid; gap: 6px; color: var(--muted); font-size: 14px; }
    .current {
      display: grid;
      gap: 4px;
      min-height: 72px;
      padding: 12px;
      border-radius: 8px;
      background: var(--panel-2);
      border: 1px solid var(--line);
    }
    .song-title { font-size: 18px; font-weight: 700; color: var(--text); }
    .queue { display: grid; gap: 8px; margin-top: 12px; }
    .queue-item {
      display: grid;
      gap: 2px;
      padding: 10px;
      border: 1px solid var(--line);
      border-radius: 8px;
      background: #17191e;
    }
    code { color: var(--accent); overflow-wrap: anywhere; }
    .message { min-height: 20px; color: var(--muted); font-size: 13px; margin-top: 10px; }
    @media (max-width: 840px) {
      main { grid-template-columns: 1fr; }
      header { align-items: flex-start; flex-direction: column; }
    }
  </style>
</head>
<body>
  <header>
    <h1>Song Request Linux</h1>
    <div class="actions">
      <a class="button secondary" href="/overlay" target="_blank" rel="noreferrer">Overlay</a>
      <a class="button secondary" href="/api/diagnostics" target="_blank" rel="noreferrer">Diagnostico</a>
    </div>
  </header>
  <main>
    <div>
      <section>
        <h2>Pedido Manual</h2>
        <form id="request-form">
          <label>Requester <input id="requester" autocomplete="off" value="heyleao"></label>
          <label>Musica ou link <input id="query" autocomplete="off" placeholder="https://youtu.be/... ou nome da musica"></label>
          <button type="submit">Adicionar</button>
        </form>
        <div class="message" id="request-message"></div>
      </section>
      <section style="margin-top: 18px;">
        <h2>Comando Twitch</h2>
        <form id="command-form">
          <label>Mensagem <input id="command" autocomplete="off" value="!sr daft punk one more time"></label>
          <div class="actions">
            <button type="submit">Enviar comando</button>
            <button class="secondary" id="skip" type="button">Skip mod</button>
          </div>
        </form>
        <div class="message" id="command-message"></div>
      </section>
    </div>
    <section>
      <h2>Fila</h2>
      <div class="status">
        <div>Provider: <code id="provider">-</code></div>
        <div>Overlay OBS: <code>http://127.0.0.1:7384/overlay</code></div>
      </div>
      <div style="height: 14px;"></div>
      <div class="current">
        <div class="song-title" id="current-title">Aguardando pedido</div>
        <div id="current-meta">Nenhuma musica tocando</div>
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
    async function refresh() {
      const [status, queue] = await Promise.all([api('/api/status'), api('/api/queue')]);
      $('provider').textContent = status.provider;
      const current = queue.current_song;
      $('current-title').textContent = current ? current.title : 'Aguardando pedido';
      $('current-meta').textContent = current
        ? `${current.artist} - pedido por ${current.requester}`
        : 'Nenhuma musica tocando';
      $('queue').innerHTML = queue.queue.map((item) => `
        <div class="queue-item">
          <strong>${escapeHtml(item.title)}</strong>
          <span>${escapeHtml(item.artist)} - pedido por ${escapeHtml(item.requester)}</span>
        </div>
      `).join('');
    }
    $('request-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      try {
        const request = await api('/api/song-requests', {
          method: 'POST',
          body: JSON.stringify({ requester: $('requester').value, query: $('query').value })
        });
        $('request-message').textContent = `Adicionado: ${request.title}`;
        $('query').value = '';
        await refresh();
      } catch (error) {
        $('request-message').textContent = error.message;
      }
    });
    $('command-form').addEventListener('submit', async (event) => {
      event.preventDefault();
      try {
        const result = await api('/api/chat-command', {
          method: 'POST',
          body: JSON.stringify({ requester: $('requester').value || 'viewer', message: $('command').value })
        });
        $('command-message').textContent = `Resultado: ${result.status}`;
        await refresh();
      } catch (error) {
        $('command-message').textContent = error.message;
      }
    });
    $('skip').addEventListener('click', async () => {
      const result = await api('/api/chat-command', {
        method: 'POST',
        body: JSON.stringify({ requester: 'mod', message: '!skip', is_moderator: true })
      });
      $('command-message').textContent = `Skip: ${result.current_song ? result.current_song.title : 'fila vazia'}`;
      await refresh();
    });
    refresh();
    setInterval(refresh, 3000);
  </script>
</body>
</html>"#,
    )
}
