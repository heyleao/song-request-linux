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
    :root { color-scheme: dark; background: transparent; }
    * { box-sizing: border-box; }
    body { margin: 0; width: 1px; height: 1px; overflow: hidden; background: transparent; }
    audio { width: 1px; height: 1px; opacity: 0; }
  </style>
</head>
<body>
  <audio id="audio" autoplay preload="none"></audio>

  <script>
    let activeVideoId = null;
    let activeSongId = null;
    let loading = false;
    const audio = document.getElementById('audio');

    function setStatus(title, meta) {
      document.title = `${title} - ${meta}`;
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
        if (data.waiting_for_spotify) {
          setStatus('Aguardando Spotify', data.waiting_for_spotify);
          loading = false;
          return;
        }
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
    setInterval(refresh, 1000);
  </script>
</body>
</html>"#,
    )
}
