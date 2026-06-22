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
    #player { width: 100vw; height: calc(100vh - 86px); background: #05070a; }
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
    <div id="player"></div>
    <section class="status">
      <div class="label">YouTube Player</div>
      <div class="title" id="title">Aguardando video do YouTube</div>
      <div class="meta" id="meta">Adicione esta pagina como Browser Source no OBS.</div>
    </section>
  </main>

  <script src="https://www.youtube.com/iframe_api"></script>
  <script>
    let player;
    let ready = false;
    let activeVideoId = null;
    let loading = false;

    function setStatus(title, meta) {
      document.getElementById('title').textContent = title;
      document.getElementById('meta').textContent = meta;
    }

    function onYouTubeIframeAPIReady() {
      player = new YT.Player('player', {
        width: '100%',
        height: '100%',
        playerVars: {
          autoplay: 1,
          controls: 1,
          playsinline: 1,
          rel: 0,
          modestbranding: 1
        },
        events: {
          onReady: () => {
            ready = true;
            refresh();
          },
          onStateChange: event => {
            if (event.data === YT.PlayerState.ENDED) {
              finishCurrent();
            }
          },
          onError: () => {
            finishCurrent();
          }
        }
      });
    }

    async function api(path, options) {
      const response = await fetch(path, { cache: 'no-store', ...options });
      const text = await response.text();
      const data = text ? JSON.parse(text) : null;
      if (!response.ok) throw new Error(data?.error || text || response.statusText);
      return data;
    }

    async function refresh() {
      if (!ready || loading) return;
      loading = true;
      try {
        const data = await api('/api/player/youtube');
        const song = data.current_song;
        if (!song) {
          activeVideoId = null;
          setStatus('Aguardando video do YouTube', 'Nenhum pedido YouTube ativo na fila local.');
          loading = false;
          return;
        }

        if (song.video_id !== activeVideoId) {
          activeVideoId = song.video_id;
          setStatus(song.title, `${song.artist} - pedido por ${song.requester}`);
          player.loadVideoById(song.video_id);
        }
      } catch (error) {
        setStatus('Erro no player', error.message);
      } finally {
        loading = false;
      }
    }

    async function finishCurrent() {
      try {
        await api('/api/skip', { method: 'POST' });
      } catch (error) {
        setStatus('Erro ao avancar fila', error.message);
      } finally {
        activeVideoId = null;
        setTimeout(refresh, 750);
      }
    }

    setInterval(refresh, 3000);
  </script>
</body>
</html>"#,
    )
}
