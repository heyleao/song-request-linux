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
      max-width: min(760px, calc(100vw - 36px));
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
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .meta {
      font-size: 18px;
      opacity: 0.9;
      min-height: 22px;
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
    const GENERIC_ARTISTS = new Set(['spotify', 'youtube', 'spotify search', 'youtube search']);
    const NOISE_PATTERNS = [
      /\[[^\]]*\]/g,
      /\([^)]*\b(?:official|video|audio|lyrics?|lyric video|visualizer|remaster(?:ed)?|hd|4k|live|clip|music video|mv)\b[^)]*\)/gi,
      /\([^)]*\)/g,
      /\b(?:official\s*)?(?:music\s*)?video\b/gi,
      /\bofficial audio\b/gi,
      /\blyrics?\b/gi,
      /\blyric video\b/gi,
      /\bvisualizer\b/gi,
      /\bremaster(?:ed)?\b/gi,
      /\b4k\b/gi,
      /\bhd\b/gi
    ];

    function cleanPart(value) {
      let text = String(value || '').normalize('NFKC');
      for (const pattern of NOISE_PATTERNS) text = text.replace(pattern, ' ');
      return text
        .replace(/\s+(?:-|–|—|\|)\s*$/g, ' ')
        .replace(/\s{2,}/g, ' ')
        .trim();
    }

    function splitTitle(value) {
      const parts = String(value || '')
        .split(/\s+(?:-|–|—|\|)\s+/)
        .map(cleanPart)
        .filter(Boolean);
      return parts.length >= 2 ? [parts[0], parts.slice(1).join(' - ')] : null;
    }

    function compactTrack(song) {
      const title = cleanPart(song?.title);
      const artist = cleanPart(song?.artist);
      const parsed = splitTitle(title);
      const artistIsGeneric = GENERIC_ARTISTS.has(artist.toLowerCase());

      let display = parsed && (artistIsGeneric || parsed[0].length <= 42)
        ? `${parsed[0]} - ${parsed[1]}`
        : artist && !artistIsGeneric
          ? `${artist} - ${title}`
          : title;

      display = cleanPart(display);
      return display.length > 88 ? `${display.slice(0, 85).trim()}...` : display;
    }

    async function refresh() {
      const response = await fetch('/api/status', { cache: 'no-store' });
      const data = await response.json();
      const song = data.current_song;
      document.getElementById('song').textContent = song ? compactTrack(song) : 'Aguardando pedido';
      document.getElementById('meta').textContent = song ? '' : `${data.app} v${data.version}`;
    }
    refresh();
    setInterval(refresh, 3000);
  </script>
</body>
</html>"#,
    )
}
