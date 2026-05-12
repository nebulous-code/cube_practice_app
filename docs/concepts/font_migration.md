# Font migration — self-hosting

Today Quiet Cube pulls its three typefaces from the Google Fonts CDN via an `@import` at the top of `frontend/src/assets/tokens.css`. This means every visitor makes a third-party request to `fonts.googleapis.com` + `fonts.gstatic.com` before the page can paint. We want to:

- Stop sending visitor IPs to Google on every page load.
- Avoid a flash of unstyled text when the CDN is slow.
- Make local dev work offline.

The fix is to bundle the three families ourselves as `woff2` files served from `frontend/public/fonts/`.

---

## Fonts to pull

Only the weights/styles the app actually uses. Tallied from `font-weight:` and `font-style:` occurrences across `frontend/src` on 2026-05-12 — keep this list in sync if a new weight gets introduced.

- **Newsreader** (serif — titles, eyebrows, italic tags)
  - 400 normal
  - 400 italic
  - 500 normal
  - 500 italic
- **Inter Tight** (sans — body, buttons, labels)
  - 400 normal
  - 500 normal
  - 600 normal
- **JetBrains Mono** (mono — OTP code on `VerifyEmailView`)
  - 400 normal
  - 500 normal

That's 9 font files total.

---

## Where to get them

Use **google-webfonts-helper** — it repackages Google Fonts as `woff2` with the `latin` subset already trimmed.

- <https://gwfh.mranftl.com/fonts/newsreader?subsets=latin>
- <https://gwfh.mranftl.com/fonts/inter-tight?subsets=latin>
- <https://gwfh.mranftl.com/fonts/jetbrains-mono?subsets=latin>

For each family:

- Subset: **latin** only (we don't ship non-latin copy).
- Styles: tick exactly the weights/styles listed above. Leave everything else unchecked.
- Charsets: leave at the default (latin).
- Format: **Modern Browsers** (woff2 only — no woff fallback, no eot).
- Click **Download files** and unzip.

---

## Where to put them

Drop every `.woff2` into `frontend/public/fonts/`, flat (no per-family subdirectory). Keep the filenames google-webfonts-helper hands you — `tokens.css` references them verbatim. The expected layout:

- `frontend/public/fonts/newsreader-v26-latin-regular.woff2`
- `frontend/public/fonts/newsreader-v26-latin-italic.woff2`
- `frontend/public/fonts/newsreader-v26-latin-500.woff2`
- `frontend/public/fonts/newsreader-v26-latin-500italic.woff2`
- `frontend/public/fonts/inter-tight-v9-latin-regular.woff2`
- `frontend/public/fonts/inter-tight-v9-latin-500.woff2`
- `frontend/public/fonts/inter-tight-v9-latin-600.woff2`
- `frontend/public/fonts/jetbrains-mono-v24-latin-regular.woff2`
- `frontend/public/fonts/jetbrains-mono-v24-latin-500.woff2`

The `v<N>` segment is the upstream font version baked in by gwfh. If a future re-download bumps that number (e.g. Newsreader v27), update both the filenames and the `src:` URLs in `tokens.css` together.

---

## After the files are in place

- Run `npm run dev` from `frontend/`. Confirm all three families render (compare against the deployed site).
- Open DevTools → Network → filter "Font". There should be **zero** requests to `fonts.googleapis.com` or `fonts.gstatic.com`. Every font request should be a same-origin `/fonts/*.woff2`.
- Check the OTP screen at `/verify-email` for JetBrains Mono.
- Check `AcknowledgementsView` for Newsreader italic (the `_Speedsolving the Cube_` book title).

---

## Licensing

All three families are SIL OFL 1.1 (already credited in `AcknowledgementsView`). Self-hosting `woff2` copies is allowed; no attribution beyond what's already on the acknowledgements page is required.

---

## Not covered here

- **Variable font version of Newsreader.** Google Fonts ships a variable axis (`opsz` 6..72, `wght` 400..700, `ital` 0..1) that would collapse the four Newsreader files into two. google-webfonts-helper doesn't expose it cleanly; if we ever want it, grab the `.ttf` from <https://github.com/google/fonts/tree/main/ofl/newsreader> and convert with `woff2_compress`. Not worth it until we feel the weight cost.
- **Preloading.** No `<link rel="preload">` tags. Skip until we have a measurable LCP issue — preload for fonts is easy to overdo.
