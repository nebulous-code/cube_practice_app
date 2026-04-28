# TODO — On Your Plate

Things that need a human decision, content, design, or external action — items I can't complete from the terminal/code. Roughly ordered by when they block work.

---

## Content / Legal

- [ ] **Terms of Service content.** Static page at `/terms`; linked from registration footer ("By creating an account you agree to our Terms…") and the Settings → About section. Required pre-launch.
- [ ] **Privacy Policy content.** Static page at `/privacy`; linked alongside Terms. Required pre-launch.
- [ ] **Acknowledgements page content.** Optional for MVP — placeholder is fine until launch. Post-MVP can auto-generate from `package.json` / `Cargo.toml` license metadata.

## Design

- [ ] **Onboarding flow.** Two-step screen ("Practice OLL with intention" / "Weakest cases come first") shown once after first verification. You said the designer will build this out once we're further along — non-blocker for backend / data work. Update `outstanding_decisions_auth.md` item 9 when designs land so we can review for new conflicts.
- [ ] **Splash screen final treatment.** Currently a placeholder; intent is for it to also cover backend cold-start. Confirm whether the placeholder visual is final or whether a polished version is coming.

## Verification / external research

- [ ] **OLL case numbering universality.** From the original outstanding decisions §4: confirm whether the 1–57 case numbering used in `data.jsx` matches the cubing-community convention. Worth a quick check before seed migrations are written, since renumbering after launch is painful.
- [ ] **Resend domain verification.** Per `Cube_Practice_Design_Doc.md` §10: "Resend requires domain verification which can take 1–2 business days. Set up and verify the sending domain before beginning auth implementation." Don't let this become the critical path.
- [ ] **reCAPTCHA v3 site key + secret key.** Need both registered in Google's admin console, with the public key wired into `VITE_RECAPTCHA_SITE_KEY` and the secret into `RECAPTCHA_SECRET_KEY` on the backend.

## Decisions still open in design docs

- [ ] **`outstanding_decisions_auth.md` items A–E** (new questions raised while updating the spec for auth — at the top of that file).
- [ ] **`guest_mode_design_doc.md` §8 open questions 1–7** (will need answers before guest mode is implemented as the final MVP step).

## Infrastructure / accounts

- [ ] **Neon Postgres database** provisioned, connection string captured in `DATABASE_URL`.
- [ ] **Render frontend + backend services** created, env vars wired up per `Cube_Practice_Design_Doc.md` §10.
- [ ] **Custom domains on Render** — `cube.nebulouscode.com` and `api.cube.nebulouscode.com`. Walkthrough in `docs/render_custom_domains.md`. Required before the verify-email / login slice ships, because the `SameSite=Strict` session cookie won't travel across `*.onrender.com` (different registrable domains per the public suffix list).

---

When something is done, just delete the line. When something new comes up, drop it in the right section.
