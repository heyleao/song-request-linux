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
      padding: var(--overlay-padding, 16px 18px);
      min-width: min(320px, 100vw);
      width: min(var(--overlay-width, 620px), calc(100vw - 36px));
      max-width: calc(100vw - 36px);
      text-align: var(--overlay-align, left);
    }
    .label {
      font-size: 14px;
      opacity: 0.78;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    .song {
      font-size: var(--song-size, 28px);
      font-weight: 700;
      line-height: 1.12;
      max-width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .song.multiline {
      display: -webkit-box;
      -webkit-line-clamp: var(--song-lines, 2);
      -webkit-box-orient: vertical;
      white-space: normal;
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
    const params = new URLSearchParams(window.location.search);
    const overlayOptions = {
      maxChars: clampNumber(params.get('max'), 24, 140, 56),
      width: clampNumber(params.get('width'), 240, 1200, 620),
      size: clampNumber(params.get('size'), 14, 64, 28),
      lines: clampNumber(params.get('lines'), 1, 3, 1),
      align: ['left', 'center', 'right'].includes(params.get('align')) ? params.get('align') : 'left'
    };

    document.documentElement.style.setProperty('--overlay-width', `${overlayOptions.width}px`);
    document.documentElement.style.setProperty('--song-size', `${overlayOptions.size}px`);
    document.documentElement.style.setProperty('--song-lines', overlayOptions.lines);
    document.documentElement.style.setProperty('--overlay-align', overlayOptions.align);
    document.getElementById('song').classList.toggle('multiline', overlayOptions.lines > 1);

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

    function clampNumber(value, min, max, fallback) {
      const number = Number(value);
      if (!Number.isFinite(number)) return fallback;
      return Math.min(max, Math.max(min, Math.floor(number)));
    }

    function trimDisplay(value, maxChars = overlayOptions.maxChars) {
      const text = String(value || '').trim();
      if (text.length <= maxChars) return text;
      return `${text.slice(0, Math.max(0, maxChars - 3)).trim()}...`;
    }

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
      return trimDisplay(display);
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
