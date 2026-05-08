# TODO — On Your Plate

Things that need a human decision, content, design, or external action — items I can't complete from the terminal/code.

---

## Hard blockers for public launch

- [ ] **Terms of Service content.** Static page at `/terms` ships with placeholder text. Linked from the registration legal footer and Settings → About.
- [ ] **Privacy Policy content.** Static page at `/privacy` ships with placeholder text. Same link surfaces as Terms.
- [ ] **OLL case numbering universality.** Confirm the 1–57 numbering used in `data.jsx` (and seeded into `cases.case_number`) matches the cubing-community convention. ~30-min check; renumbering after launch breaks any URL or progress data already keyed on case number.

## Cosmetic but visible to first-time users

- [ ] **Onboarding screen real copy + design.** `OnboardingView.vue` ships placeholder copy for the two-step screen ("Practice OLL with intention" / "Weakest cases come first"). Designer was going to take a pass once the product was further along.
- [ ] **Landing page copy.** `LandingView.vue` ships placeholder marketing copy (hero, feature bullets, how-it-works steps, CTAs). Replace before public launch.
- [ ] **Splash screen final treatment.** Currently a placeholder; intent is for it to also cover backend cold-start. Confirm whether the placeholder visual is final or whether a polished version is coming.

## Optional for MVP (placeholder is fine)

- [ ] **Acknowledgements page real content.** Currently placeholder. Post-MVP can auto-generate from `package.json` / `Cargo.toml` license metadata.

## Known bugs

- [ ] **Streak shows non-zero after delete + re-register with same email.** Spotted during M7 dogfooding. Backend tests confirm fresh row gets `streak_count=0`; suspected to be a frontend stale-store issue along an as-yet-unidentified path. To investigate.

---

When something is done, just delete the line. When something new comes up, drop it in the right section.
