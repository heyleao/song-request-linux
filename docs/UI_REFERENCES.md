# UI References

These resources are design references for improving the Song Request Linux
dashboard. They are not runtime dependencies.

## 21st.dev

URL:

```text
https://21st.dev/home
```

Use for:

- UI component inspiration.
- Operational dashboard patterns.
- Setup and status components.
- Queue, player and log layout ideas.
- MCP-assisted component search when available.

Current fit:

- Good for visual direction and interaction patterns.
- Do not paste React components directly into the Rust-served HTML dashboard.
- Translate useful patterns into the existing HTML/CSS unless we intentionally
  add a frontend build step.

## UI UX Pro Max Skill

URL:

```text
https://github.com/nextlevelbuilder/ui-ux-pro-max-skill
```

The project describes itself as an AI skill for design intelligence across
multiple platforms and frameworks. The README highlights design system
generation, UI styles, color palettes, typography pairings, chart
recommendations, UX guidelines and Codex CLI setup through `uipro`.

Use for:

- Reviewing dashboard design decisions.
- Choosing an operational dashboard style.
- Accessibility and pre-delivery UI checks.
- Improving setup, diagnostics, logs and queue screens.

Current fit:

- Treat it as an assistant-side design skill/reference.
- Do not make the app depend on `uipro` at runtime.
- If installed locally, generated recommendations still need human review and
  must match this project's UI/UX direction.

## Project-Specific UI Direction

Song Request Linux is an operational tool for streamers, not a marketing site.
Prefer:

- Dense but readable dashboard layouts.
- Clear connected/error/waiting states.
- Direct controls with low visual noise.
- Accessible contrast and keyboard focus states.
- Stable layouts for OBS/Twitch/Spotify/Pear status while streaming.

Avoid:

- Landing-page hero sections.
- Decorative gradients and oversized cards.
- Heavy animation.
- UI patterns that require React, Tailwind or shadcn unless we intentionally
  migrate the dashboard stack.
