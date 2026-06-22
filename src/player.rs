use axum::response::Html;

pub async fn page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Song Request Linux Player</title>
  <style>
    :root {
      color-scheme: dark;
      font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: #0f131a;
      color: #eef2f7;
    }
    * { box-sizing: border-box; }
    body { margin: 0; min-height: 100vh; background: #0f131a; }
    main { display: grid; grid-template-rows: 1fr auto; min-height: 100vh; }
    .visual {
      display: grid;
      place-items: center;
      width: 100vw;
      height: calc(100vh - 86px);
      background: #05070a;
      color: #8d98a8;
      font-size: 14px;
      font-weight: 700;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }
    audio { width: min(720px, calc(100vw - 32px)); }
    .status {
      display: grid;
      gap: 4px;
      padding: 14px 16px;
      border-top: 1px solid #2a3240;
      background: #151a22;
    }
    .label {
      color: #9aa4b2;
      font-size: 12px;
      font-weight: 700;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }
    .title { font-size: 18px; font-weight: 700; line-height: 1.25; }
    .meta { color: #b6bfcb; font-size: 14px; }
  </style>
</head>
<body>
  <main>
    <div class="visual">
      <audio id="audio" controls autoplay preload="none"></audio>
    </div>
    <section class="status">
      <div class="label">YouTube Audio Player</div>
      <div class="title" id="title">Aguardando video do YouTube</div>
      <div class="meta" id="meta">Adicione esta pagina como Browser Source no OBS.</div>
    </section>
  </main>

  <script>
    let activeVideoId = null;
    let activeSongId = null;
    let loading = false;
    const audio = document.getElementById('audio');

    function setStatus(title, meta) {
      document.getElementById('title').textContent = title;
      document.getElementById('meta').textContent = meta;
    }

    async function api(path, options) {
      const response = await fetch(path, { cache: 'no-store', ...options });
      const text = await response.text();
      const data = text ? JSON.parse(text) : null;
      if (!response.ok) throw new Error(data?.error || text || response.statusText);
      return data;
    }

    async function refresh() {
      if (loading) return;
      loading = true;
      try {
        const data = await api('/api/player/youtube');
        const song = data.current_song;
        if (!song) {
          activeVideoId = null;
          activeSongId = null;
          audio.removeAttribute('src');
          audio.load();
          setStatus('Aguardando video do YouTube', 'Nenhum pedido YouTube ativo na fila local.');
          loading = false;
          return;
        }

        if (song.video_id !== activeVideoId) {
          activeVideoId = song.video_id;
          activeSongId = song.id;
          setStatus(song.title, `Resolvendo audio - pedido por ${song.requester}`);
          const audioData = await api('/api/player/youtube/audio', {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ id: song.id })
          });
          await api('/api/player/youtube/start', {
            method: 'POST',
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ id: song.id })
          });
          audio.src = audioData.audio_url;
          audio.load();
          await audio.play();
          setStatus(song.title, `${song.artist} - pedido por ${song.requester}`);
        }
      } catch (error) {
        setStatus('Erro no player', error.message);
      } finally {
        loading = false;
      }
    }

    async function finishCurrent() {
      const finishedSongId = activeSongId;
      if (!finishedSongId) return;
      try {
        await api('/api/player/youtube/finish', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ id: finishedSongId })
        });
      } catch (error) {
        setStatus('Erro ao avancar fila', error.message);
      } finally {
        activeVideoId = null;
        activeSongId = null;
        setTimeout(refresh, 750);
      }
    }

    audio.addEventListener('ended', finishCurrent);
    audio.addEventListener('error', () => {
      if (activeSongId) {
        setStatus('Erro no audio', 'Nao foi possivel tocar esta URL. Pulando para o proximo pedido.');
        finishCurrent();
      }
    });

    refresh();
    setInterval(refresh, 3000);
  </script>
</body>
</html>"#,
    )
}
