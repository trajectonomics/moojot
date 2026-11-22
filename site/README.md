# moojot.trajectonomics.com

This is the landing page for MooJot. It's built for people arriving from short-form video (Instagram reels) on an Android phone, so its main job is to get the app installed: a download button, three install steps, and a tip for getting out of in-app browsers.

The page is plain HTML, CSS and JS with no build step and no dependencies. It's served by a Cloudflare Worker with Static Assets.

| Path | What it is |
|---|---|
| `public/` | The static site: `index.html`, `styles.css`, `boot.js` (sets the language before paint), `app.js` (language switch, release info, animated Moo), `_headers`, `404.html`, `img/`, `fonts/` |
| `worker.js` | Worker entry. It only runs for `/download` and `/api/*` (see `run_worker_first`). |
| `release.js` | Looks up the latest GitHub release. It uses the API first and falls back to github.com when Cloudflare's shared IPs are rate limited, then caches the answer for 5 minutes. |
| `release.test.js` | Tests for `release.js` |
| `wrangler.jsonc` | Worker `moojot-site` on the custom domain `moojot.trajectonomics.com` |

## Links that matter

- `https://moojot.trajectonomics.com/download` redirects (302) to the `.apk` in the latest GitHub release. Use this short link in reels and bios. Before the first release it lands on `/#coming-soon`.
- `https://moojot.trajectonomics.com/api/release` returns `{ "release": null | { version, apk: { name, size, url }, checksumsUrl, … } }`.
- `?lang=th` / `?lang=en` forces a language. Otherwise the page uses the visitor's saved choice, then the browser language. Thai is the no-JS default.

## Commands

```bash
node --test site/release.test.js           # unit tests (Node 20+)
cd site && npx wrangler@4 dev --port 8799   # local preview at http://127.0.0.1:8799
cd site && npx wrangler@4 deploy            # deploy (needs a Cloudflare login for the Trajectonomics account)
```

`wrangler` is run with `npx` on purpose, so the repo's `package.json` doesn't gain a dependency.

## Content rules

- Every piece of copy exists in Thai and English as sibling elements marked `lang="th"` and `lang="en"`. CSS hides whichever language isn't active, so add both when you change text.
- Only claim what the app actually does. Check `docs/prd/PRD.md` and `docs/privacy-policy.md`.
- Screenshots in `img/shot-*.webp` were taken from the real Svelte UI using invented sample data. Never use a real ledger.
- The roadmap section links to GitHub issues. Update it when those issues close.
- No analytics, cookies or third-party requests. That's part of the privacy promise on the page.

## Assets

- `img/moo-wave.webp` is `static/mascot/mascot-idle.webp`. `img/moo-still.webp` is its first frame, used until the page has loaded.
- `img/icon-*.png` and `img/apple-touch-icon.png` are resized from `static/app-icon.png`.
- `img/og.jpg` (1200×630) is the link preview card.
- `fonts/` holds Noto Sans Thai from `@fontsource/noto-sans-thai` (SIL OFL 1.1, see `fonts/OFL.txt`).
