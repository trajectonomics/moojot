// Request handling for the moojot.trajectonomics.com Worker (entry: worker.js).
//
// Static files in ./public are served by Workers Static Assets. The Worker
// only runs for the paths listed in wrangler.jsonc `run_worker_first`:
//
//   /download     302 to the APK in the latest GitHub release, so a reel can say
//                 "moojot.trajectonomics.com/download" and the link never goes stale.
//   /api/release  JSON describing that release, used by the page for the version,
//                 size and "coming soon" state.

export const REPO = "trajectonomics/moojot";
export const RELEASES_URL = `https://github.com/${REPO}/releases/latest`;
export const NO_RELEASE_PATH = "/#coming-soon";
const CACHE_SECONDS = 300;
const CACHE_KEY = "https://moojot.trajectonomics.com/__cache/latest-release";

/**
 * Reduce a GitHub "latest release" payload to what the page needs.
 * Returns null when the release has no APK to download.
 */
export function pickRelease(release) {
  if (!release || release.draft || !Array.isArray(release.assets)) return null;
  const apks = release.assets.filter((a) => a.name.toLowerCase().endsWith(".apk"));
  if (apks.length === 0) return null;
  const apk =
    apks.find((a) => /universal/i.test(a.name)) ??
    apks.find((a) => /arm64|aarch64/i.test(a.name)) ??
    apks[0];
  const sums = release.assets.find((a) => a.name === "SHA256SUMS");
  return {
    version: release.tag_name,
    publishedAt: release.published_at,
    pageUrl: release.html_url,
    apk: { name: apk.name, size: apk.size, url: apk.browser_download_url },
    checksumsUrl: sums ? sums.browser_download_url : null,
  };
}

/** Latest release (or null), cached at the edge for CACHE_SECONDS. Throws on GitHub errors. */
export async function latestRelease(ctx, deps = {}) {
  const fetchImpl = deps.fetch ?? fetch;
  const cache = deps.cache ?? (typeof caches === "undefined" ? null : caches.default);
  const key = new Request(CACHE_KEY);

  const hit = cache ? await cache.match(key) : undefined;
  if (hit) return hit.json();

  let release;
  try {
    release = await fromApi(fetchImpl);
  } catch (apiError) {
    // Unauthenticated API calls share a 60/hour limit per IP, and Cloudflare's
    // egress IPs are shared, so a 403 here is normal. Read the web pages instead.
    console.warn(`release lookup: ${apiError.message}; trying github.com`);
    release = await fromWeb(fetchImpl); // throws too: never cache an error
  }

  if (cache) {
    const stored = Response.json(release, {
      headers: { "cache-control": `public, max-age=${CACHE_SECONDS}` },
    });
    const put = cache.put(key, stored);
    if (ctx && ctx.waitUntil) ctx.waitUntil(put);
    else await put;
  }
  return release;
}

const UA = { "user-agent": "moojot-site (+https://moojot.trajectonomics.com)" };

async function fromApi(fetchImpl) {
  const res = await fetchImpl(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers: { accept: "application/vnd.github+json", ...UA },
  });
  if (res.status === 200) return pickRelease(await res.json());
  if (res.status === 404) return null; // no published release yet
  throw new Error(`GitHub API answered ${res.status}`);
}

/** Same answer from github.com: /releases/latest redirects to the tag, or to /releases when there is none. */
export async function fromWeb(fetchImpl) {
  const latest = await fetchImpl(`https://github.com/${REPO}/releases/latest`, {
    redirect: "manual",
    headers: UA,
  });
  const location = latest.headers.get("location") ?? "";
  const tagMatch = location.match(/\/releases\/tag\/([^/?#]+)$/);
  if (latest.status >= 300 && latest.status < 400 && !tagMatch) return null;
  if (!tagMatch) throw new Error(`github.com answered ${latest.status}`);

  const tag = decodeURIComponent(tagMatch[1]);
  const list = await fetchImpl(
    `https://github.com/${REPO}/releases/expanded_assets/${encodeURIComponent(tag)}`,
    { headers: UA },
  );
  if (!list.ok) throw new Error(`github.com assets answered ${list.status}`);
  const html = await list.text();
  const prefix = `/${REPO}/releases/download/`;
  const names = new Set();
  for (const m of html.matchAll(/href="([^"]+)"/g)) {
    if (m[1].startsWith(prefix)) names.add(decodeURIComponent(m[1].split("/").pop()));
  }
  const base = `https://github.com/${REPO}/releases`;
  return pickRelease({
    tag_name: tag,
    draft: false,
    published_at: null,
    html_url: `${base}/tag/${encodeURIComponent(tag)}`,
    assets: [...names].map((name) => ({
      name,
      size: null,
      browser_download_url: `${base}/download/${encodeURIComponent(tag)}/${encodeURIComponent(name)}`,
    })),
  });
}

export async function handle(request, env, ctx, deps = {}) {
  const { pathname } = new URL(request.url);

  if (pathname === "/download" || pathname === "/download/") {
    try {
      const release = await latestRelease(ctx, deps);
      const to = release ? release.apk.url : new URL(NO_RELEASE_PATH, request.url).toString();
      return redirect(to);
    } catch {
      return redirect(RELEASES_URL);
    }
  }

  if (pathname === "/api/release") {
    try {
      const release = await latestRelease(ctx, deps);
      return Response.json(
        { release },
        { headers: { "cache-control": `public, max-age=${CACHE_SECONDS}` } },
      );
    } catch {
      return Response.json(
        { release: null, error: "unavailable" },
        { status: 503, headers: { "cache-control": "no-store" } },
      );
    }
  }

  return env.ASSETS.fetch(request);
}

function redirect(location) {
  return new Response(null, {
    status: 302,
    headers: { location, "cache-control": "no-store" },
  });
}
