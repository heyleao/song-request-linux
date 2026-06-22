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
      --warn: #ffd166;
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
    input {
      min-height: 38px;
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      color: var(--text);
      padding: 8px 10px;
      min-width: min(100%, 420px);
    }
    input[type="checkbox"] {
      width: auto;
      min-width: auto;
      min-height: auto;
    }
    label {
      display: grid;
      gap: 6px;
      color: var(--muted);
      font-size: 13px;
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
    .message.ok { color: var(--ok); }
    .status-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 8px; }
    .status-card {
      border: 1px solid var(--line);
      border-radius: 6px;
      background: var(--panel-2);
      padding: 10px;
      display: grid;
      gap: 4px;
    }
    .status-card strong { font-size: 13px; }
    .status-card span { color: var(--muted); font-size: 13px; }
    .status-card.ok strong { color: var(--ok); }
    .status-card.warn strong { color: var(--warn); }
    .status-card.bad strong { color: var(--bad); }
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
      <h2>Configuracao</h2>
      <label>
        Provider padrao
        <select id="default-provider">
          <option value="spotify">Spotify</option>
          <option value="youtube">YouTube</option>
        </select>
      </label>
      <label>
        Spotify Client ID
        <input id="spotify-client-id" autocomplete="off" placeholder="Client ID do app Spotify">
      </label>
      <label>
        Twitch Client ID
        <input id="twitch-client-id" autocomplete="off" placeholder="Client ID do app Twitch">
      </label>
      <label>
        Twitch Bot Username
        <input id="twitch-bot-username" autocomplete="off" placeholder="preenchido apos OAuth do bot">
      </label>
      <label>
        Twitch Channel
        <input id="twitch-channel" autocomplete="off" placeholder="canal_do_streamer">
      </label>
      <label>
        YouTube API Key
        <input id="youtube-api-key" autocomplete="off" placeholder="salva como segredo; deixe vazio para manter">
      </label>
      <label>
        Maximo YouTube (segundos)
        <input id="youtube-max-duration" type="number" min="30" max="86400" step="30" value="360">
      </label>
      <label>
        <span><input id="youtube-allow-non-music" type="checkbox"> Aceitar YouTube nao marcado como musica</span>
      </label>
      <div class="row">
        <button id="save-config">Salvar configuracao</button>
      </div>
      <div class="message" id="config-message"></div>
    </section>

    <section>
      <h2>Spotify</h2>
      <div class="status-grid">
        <div class="status-card" id="spotify-client-card"><strong>Client ID</strong><span>Carregando...</span></div>
        <div class="status-card" id="spotify-login-card"><strong>Login OAuth</strong><span>Carregando...</span></div>
        <div class="status-card" id="spotify-device-card"><strong>Device ativo</strong><span>Carregue os devices.</span></div>
      </div>
      <div class="muted" id="spotify-status">Carregando...</div>
      <div class="row">
        <button id="spotify-start">Gerar link de login</button>
        <button id="load-devices">Carregar devices</button>
      </div>
      <code id="spotify-link">O link aparece aqui.</code>
      <div class="message" id="device-message"></div>
      <div class="muted">
        Para a fila funcionar, deixe o Spotify aberto no PC/celular e de play ou pause em uma musica.
      </div>
    </section>

    <section>
      <h2>Twitch Bot</h2>
      <div class="status-grid">
        <div class="status-card" id="twitch-client-card"><strong>Client ID</strong><span>Carregando...</span></div>
        <div class="status-card" id="twitch-token-card"><strong>OAuth do bot</strong><span>Carregando...</span></div>
        <div class="status-card" id="twitch-channel-card"><strong>Canal</strong><span>Carregando...</span></div>
      </div>
      <div class="muted">
        Conecte a conta bot em uma janela privada para nao reaproveitar a sessao do streamer.
        Redirect Twitch: https://localhost:7443/auth/twitch/callback.
      </div>
      <div class="row">
        <button id="twitch-start">Conectar bot</button>
      </div>
      <code id="twitch-link">O link aparece aqui.</code>
      <div class="message" id="twitch-message"></div>
    </section>

    <section>
      <h2>Playlist fallback</h2>
      <div class="muted">
        Playlist usada quando nao houver pedidos. Escolha uma playlist do Spotify para manter a live tocando.
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
  </main>
  <script>
    const statusEl = document.getElementById('spotify-status');
    const linkEl = document.getElementById('spotify-link');
    const deviceMessage = document.getElementById('device-message');
    const playlistSelect = document.getElementById('playlist-select');
    const playlistMessage = document.getElementById('playlist-message');
    const configMessage = document.getElementById('config-message');
    const twitchLink = document.getElementById('twitch-link');
    const twitchMessage = document.getElementById('twitch-message');
    let playlists = [];
    function setCard(id, state, text) {
      const card = document.getElementById(id);
      card.className = `status-card ${state}`;
      card.querySelector('span').textContent = text;
    }
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
      const [status, config] = await Promise.all([
        api('/api/connections/status'),
        api('/api/config')
      ]);
      document.getElementById('default-provider').value = config.default_provider;
      document.getElementById('spotify-client-id').value = config.spotify_client_id || '';
      document.getElementById('twitch-client-id').value = config.twitch_client_id || '';
      document.getElementById('twitch-bot-username').value = config.twitch_bot_username || '';
      document.getElementById('twitch-channel').value = config.twitch_channel || '';
      document.getElementById('youtube-max-duration').value = config.youtube_max_duration_seconds || 360;
      document.getElementById('youtube-allow-non-music').checked = Boolean(config.youtube_allow_non_music);
      setCard(
        'spotify-client-card',
        status.spotify.client_id_configured ? 'ok' : 'bad',
        status.spotify.client_id_configured ? 'Configurado' : 'Pendente'
      );
      setCard(
        'spotify-login-card',
        status.spotify.token_configured ? 'ok' : 'warn',
        status.spotify.token_configured ? 'Conectado' : 'Falta conectar'
      );
      setCard(
        'twitch-client-card',
        config.twitch_client_id ? 'ok' : 'bad',
        config.twitch_client_id ? 'Configurado' : 'Pendente'
      );
      setCard(
        'twitch-token-card',
        config.twitch_bot_token_configured ? 'ok' : 'warn',
        config.twitch_bot_token_configured ? `Token salvo${config.twitch_bot_username ? `: ${config.twitch_bot_username}` : ''}` : 'Falta OAuth do bot'
      );
      setCard(
        'twitch-channel-card',
        config.twitch_channel ? 'ok' : 'bad',
        config.twitch_channel ? `#${config.twitch_channel}` : 'Pendente'
      );
      twitchMessage.textContent = config.twitch_bot_token_configured
        ? 'Token do bot salvo. Se o app estiver aberto, o bot conecta automaticamente; teste no chat com !sr nome da musica.'
        : 'Bot ainda nao conectado.';
      statusEl.textContent = status.spotify.token_configured
        ? 'Spotify conectado. Se aparecer NO_ACTIVE_DEVICE, carregue os devices ou abra o Spotify e de play/pause.'
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
    document.getElementById('load-devices').addEventListener('click', async () => {
      try {
        const devices = await api('/api/spotify/devices');
        const active = devices.find((device) => device.is_active);
        const usable = devices.filter((device) => !device.is_restricted && device.id);
        if (active) {
          setCard('spotify-device-card', 'ok', `${active.name} (${active.device_type})`);
        } else if (usable.length) {
          setCard('spotify-device-card', 'warn', `${usable[0].name} disponivel, sem playback ativo`);
        } else {
          setCard('spotify-device-card', 'bad', 'Nenhum device disponivel');
        }
        deviceMessage.textContent = devices.length
          ? devices.map((device) => `${device.is_active ? 'ativo' : 'inativo'}: ${device.name} (${device.device_type})`).join(' | ')
          : 'Nenhum device encontrado. Abra o Spotify no PC/celular e de play ou pause em uma musica.';
        deviceMessage.className = devices.length ? 'message ok' : 'message error';
      } catch (error) {
        setCard('spotify-device-card', 'bad', 'Falha ao carregar');
        deviceMessage.textContent = error.message;
        deviceMessage.className = 'message error';
      }
    });
    document.getElementById('save-config').addEventListener('click', async () => {
      try {
        await api('/api/config', {
          method: 'POST',
          body: JSON.stringify({
            default_provider: document.getElementById('default-provider').value,
            spotify_client_id: document.getElementById('spotify-client-id').value,
            twitch_client_id: document.getElementById('twitch-client-id').value,
            twitch_bot_username: document.getElementById('twitch-bot-username').value,
            twitch_channel: document.getElementById('twitch-channel').value,
            twitch_bot_oauth_token: null,
            youtube_api_key: document.getElementById('youtube-api-key').value,
            youtube_max_duration_seconds: Number(document.getElementById('youtube-max-duration').value || 360),
            youtube_allow_non_music: document.getElementById('youtube-allow-non-music').checked
          })
        });
        configMessage.textContent = 'Configuracao salva. Para o Twitch chat conectar com o token novo, reinicie o app.';
        configMessage.className = 'message ok';
        await refresh();
      } catch (error) {
        configMessage.textContent = error.message;
        configMessage.className = 'message error';
      }
    });
    document.getElementById('twitch-start').addEventListener('click', async () => {
      try {
        const result = await api('/api/connections/twitch/start', { method: 'POST' });
        twitchLink.textContent = result.auth_url;
        window.open(result.auth_url, '_blank', 'noopener,noreferrer');
      } catch (error) {
        twitchLink.textContent = error.message;
        twitchMessage.textContent = error.message;
        twitchMessage.className = 'message error';
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
