use axum::response::Html;

pub async fn page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Conexoes - Song Request Linux</title>
  <style>
    :root {
      color-scheme: dark;
      --bg: #0f1115;
      --panel: #181b21;
      --panel-2: #20242c;
      --text: #f3f5f7;
      --muted: #a9b0bb;
      --line: #343945;
      --action: #62a8ff;
      --ok: #2bd180;
      --bad: #ff6b6b;
      font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    * { box-sizing: border-box; }
    body { margin: 0; background: var(--bg); color: var(--text); }
    header, main { max-width: 980px; margin: 0 auto; padding: 18px; }
    header { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
    h1 { margin: 0; font-size: 20px; }
    section {
      display: grid;
      gap: 12px;
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 16px;
      margin-bottom: 14px;
    }
    button, a.button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 38px;
      border: 1px solid #3d7fc8;
      border-radius: 6px;
      background: var(--action);
      color: #07111f;
      padding: 8px 12px;
      font-weight: 700;
      text-decoration: none;
      cursor: pointer;
    }
    select {
      min-height: 38px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      color: var(--text);
      padding: 8px 10px;
      min-width: min(100%, 420px);
    }
    a.secondary {
      border-color: var(--line);
      background: var(--panel-2);
      color: var(--text);
    }
    .row { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
    .muted { color: var(--muted); }
    .message { min-height: 20px; color: var(--muted); }
    .message.error { color: var(--bad); }
    code {
      display: block;
      padding: 10px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      overflow-wrap: anywhere;
      color: var(--ok);
    }
    .bad { color: var(--bad); }
  </style>
</head>
<body>
  <header>
    <h1>Conexoes</h1>
    <a class="button secondary" href="/">Dashboard</a>
  </header>
  <main>
    <section>
      <h2>Spotify</h2>
      <div class="muted" id="spotify-status">Carregando...</div>
      <div class="row">
        <button id="spotify-start">Gerar link de login</button>
      </div>
      <code id="spotify-link">O link aparece aqui.</code>
      <div class="muted">
        Abra este link em uma janela privada se precisar garantir que a conta conectada seja a conta correta.
      </div>
    </section>

    <section>
      <h2>Playlist fallback</h2>
      <div class="muted">
        Esta playlist sera usada quando nao houver pedidos na fila. Nesta etapa ela fica salva; a reproducao automatica entra no proximo passo.
      </div>
      <div class="row">
        <button id="load-playlists">Carregar playlists</button>
        <select id="playlist-select">
          <option value="">Nenhuma playlist carregada</option>
        </select>
        <button id="save-playlist">Salvar fallback</button>
      </div>
      <div class="message" id="playlist-message"></div>
    </section>

    <section>
      <h2>Twitch Bot</h2>
      <div class="muted">
        MVP atual usa variaveis de ambiente para o bot. A conexao OAuth separada do streamer fica para Channel Points/EventSub.
      </div>
      <code>TWITCH_BOT_USERNAME, TWITCH_BOT_OAUTH_TOKEN, TWITCH_CHANNEL</code>
    </section>
  </main>
  <script>
    const statusEl = document.getElementById('spotify-status');
    const linkEl = document.getElementById('spotify-link');
    const playlistSelect = document.getElementById('playlist-select');
    const playlistMessage = document.getElementById('playlist-message');
    let playlists = [];
    async function api(path, options = {}) {
      const response = await fetch(path, {
        headers: { 'content-type': 'application/json', ...(options.headers || {}) },
        ...options
      });
      const data = await response.json();
      if (!response.ok) throw new Error(data.error || 'Falha na requisicao');
      return data;
    }
    async function refresh() {
      const status = await api('/api/connections/status');
      statusEl.textContent = status.spotify.token_configured
        ? 'Spotify conectado.'
        : status.spotify.client_id_configured
          ? 'Client ID configurado. Falta login.'
          : 'Configure SPOTIFY_CLIENT_ID antes de conectar.';
      statusEl.className = status.spotify.client_id_configured ? 'muted' : 'bad';
      if (status.spotify.fallback_playlist) {
        playlistMessage.textContent = `Fallback atual: ${status.spotify.fallback_playlist.name}`;
        playlistMessage.className = 'message';
      }
    }
    document.getElementById('spotify-start').addEventListener('click', async () => {
      try {
        const result = await api('/api/connections/spotify/start', { method: 'POST' });
        linkEl.textContent = result.auth_url;
        window.open(result.auth_url, '_blank', 'noopener,noreferrer');
      } catch (error) {
        linkEl.textContent = error.message;
      }
    });
    document.getElementById('load-playlists').addEventListener('click', async () => {
      try {
        playlists = await api('/api/spotify/playlists');
        playlistSelect.innerHTML = playlists.length
          ? playlists.map((playlist, index) => `<option value="${index}">${playlist.name} (${playlist.tracks.total})</option>`).join('')
          : '<option value="">Nenhuma playlist encontrada</option>';
        playlistMessage.textContent = playlists.length
          ? `${playlists.length} playlists carregadas.`
          : 'Nenhuma playlist encontrada.';
        playlistMessage.className = 'message';
      } catch (error) {
        playlistMessage.textContent = error.message;
        playlistMessage.className = 'message error';
      }
    });
    document.getElementById('save-playlist').addEventListener('click', async () => {
      try {
        const playlist = playlists[Number(playlistSelect.value)];
        if (!playlist) throw new Error('Carregue e selecione uma playlist primeiro.');
        await api('/api/spotify/fallback-playlist', {
          method: 'POST',
          body: JSON.stringify({ id: playlist.id, name: playlist.name, uri: playlist.uri })
        });
        playlistMessage.textContent = `Fallback salvo: ${playlist.name}`;
        playlistMessage.className = 'message';
        await refresh();
      } catch (error) {
        playlistMessage.textContent = error.message;
        playlistMessage.className = 'message error';
      }
    });
    refresh();
  </script>
</body>
</html>"#,
    )
}
