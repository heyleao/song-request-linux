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
	    let retryAttempts = 0;
	    let retryTimer = null;
	    let lastProgressAt = Date.now();
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

	    async function recordPlayerEvent(message) {
	      try {
	        await api('/api/player/youtube/event', {
	          method: 'POST',
	          headers: { 'content-type': 'application/json' },
	          body: JSON.stringify({ message })
	        });
	      } catch (_) {}
	    }

	    function resetAudioState() {
	      activeVideoId = null;
	      activeSongId = null;
	      retryAttempts = 0;
	      if (retryTimer) clearTimeout(retryTimer);
	      retryTimer = null;
	      audio.removeAttribute('src');
	      audio.load();
	    }

	    async function loadSong(song, reason) {
	      if (activeSongId !== song.id) {
	        retryAttempts = 0;
	      }
	      activeVideoId = song.video_id;
	      activeSongId = song.id;
	      lastProgressAt = Date.now();
	      setStatus(song.title, `${reason} - pedido por ${song.requester}`);
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
	      retryAttempts = 0;
	      setStatus(song.title, `${song.artist} - pedido por ${song.requester}`);
	      await recordPlayerEvent(`tocando YouTube: ${song.title}`);
	    }

	    function scheduleRetry(reason) {
	      if (!activeSongId || retryTimer) return;
	      retryAttempts += 1;
	      const delay = Math.min(30000, 1000 * Math.max(2, retryAttempts * 2));
	      setStatus('Recuperando audio do YouTube', `${reason}. Nova tentativa em ${Math.round(delay / 1000)}s.`);
	      recordPlayerEvent(`retry YouTube ${retryAttempts}: ${reason}`);
	      retryTimer = setTimeout(async () => {
	        retryTimer = null;
	        activeVideoId = null;
	        try {
	          await refresh();
	        } catch (_) {}
	      }, delay);
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
	          resetAudioState();
	          setStatus('Aguardando video do YouTube', 'Nenhum pedido YouTube ativo na fila local.');
	          loading = false;
	          return;
	        }
	
	        if (song.id !== activeSongId || song.video_id !== activeVideoId) {
	          await loadSong(song, 'Resolvendo audio');
	        } else if (audio.paused && !audio.ended) {
	          try {
	            await audio.play();
	          } catch (error) {
	            scheduleRetry(error.message || 'audio pausado pelo navegador');
	          }
	        }
	      } catch (error) {
	        setStatus('Erro no player', error.message);
	        scheduleRetry(error.message);
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
	        resetAudioState();
	        setTimeout(refresh, 750);
	      }
	    }
	
	    audio.addEventListener('timeupdate', () => {
	      lastProgressAt = Date.now();
	    });
	    audio.addEventListener('ended', () => {
	      const knownDuration = Number.isFinite(audio.duration) ? audio.duration : 0;
	      if (knownDuration > 30 && audio.currentTime < knownDuration - 10) {
	        scheduleRetry('audio terminou antes do tempo esperado');
	        return;
	      }
	      finishCurrent();
	    });
	    audio.addEventListener('error', () => {
	      if (activeSongId) {
	        const code = audio.error ? audio.error.code : 'desconhecido';
	        scheduleRetry(`erro no audio codigo ${code}`);
	      }
	    });
	    audio.addEventListener('stalled', () => {
	      if (activeSongId) scheduleRetry('download do audio travou');
	    });
	
	    refresh();
	    setInterval(refresh, 1000);
	    setInterval(() => {
	      if (!activeSongId || loading || retryTimer) return;
	      if (!audio.paused && Date.now() - lastProgressAt > 20000) {
	        scheduleRetry('audio sem progresso');
	      }
	    }, 5000);
	  </script>
</body>
</html>"#,
    )
}
