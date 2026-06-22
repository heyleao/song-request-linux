use axum::response::Html;

pub async fn page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Song Request Linux Overlay</title>
  <style>
    :root {
      color-scheme: dark;
      font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    body {
      margin: 0;
      background: transparent;
      color: white;
      text-shadow: 0 2px 8px rgb(0 0 0 / 65%);
    }
    main {
      display: inline-grid;
      gap: 4px;
      padding: 16px 18px;
      min-width: 320px;
    }
    .label {
      font-size: 14px;
      opacity: 0.78;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    .song {
      font-size: 28px;
      font-weight: 700;
      line-height: 1.1;
    }
    .meta {
      font-size: 18px;
      opacity: 0.9;
    }
  </style>
</head>
<body>
  <main>
    <div class="label">Tocando agora</div>
    <div class="song" id="song">Aguardando pedido</div>
    <div class="meta" id="meta">Song Request Linux</div>
  </main>
  <script>
    async function refresh() {
      const response = await fetch('/api/status', { cache: 'no-store' });
      const data = await response.json();
      const song = data.current_song;
      document.getElementById('song').textContent = song ? song.title : 'Aguardando pedido';
      document.getElementById('meta').textContent = song
        ? `${song.artist} - pedido por ${song.requester}`
        : `${data.app} v${data.version}`;
    }
    refresh();
    setInterval(refresh, 3000);
  </script>
</body>
</html>"#,
    )
}
