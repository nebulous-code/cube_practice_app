# TODO — On Your Plate

Things that need a human decision, content, design, or external action — items I can't complete from the terminal/code.

---

## Hard blockers for public launch

- [x] **Terms of Service content.** Static page at `/terms` ships with placeholder text. Linked from the registration legal footer and Settings → About.
- [x] **Privacy Policy content.** Static page at `/privacy` ships with placeholder text. Same link surfaces as Terms.
- [x] **OLL case numbering universality.** Confirm the 1–57 numbering used in `data.jsx` (and seeded into `cases.case_number`) matches the cubing-community convention. ~30-min check; renumbering after launch breaks any URL or progress data already keyed on case number.
    - speedsolving.com has the best universal case numbering.
- [x] Update Case numbering
- [x] Add Case Presentation Rotation
- [ ] Update fonts to be stored and served locally.
- [ ] Create a me@nebulouscode.com email address

## Cosmetic but visible to first-time users

- [x] **Onboarding screen real copy + design.** `OnboardingView.vue` ships placeholder copy for the two-step screen ("Practice OLL with intention" / "Weakest cases come first"). Designer was going to take a pass once the product was further along.
- [x] **Landing page copy.** `LandingView.vue` ships placeholder marketing copy (hero, feature bullets, how-it-works steps, CTAs). Replace before public launch.
- [x] **README.md** Need to add a readme so that people that land on the github page can get a sense of the project

## Optional for MVP

- [x] **Acknowledgements page real content.** Currently placeholder. Post-MVP can auto-generate from `package.json` / `Cargo.toml` license metadata.
- [ ]  **Remove API URL Redirect** Render charges per url redirect. we're using two api.cube.neb... and cube.neb... we could live without api.cube.neb... and just point to api-cube.onrender.com. 

## Known bugs

- [ ] **Streak shows non-zero after delete + re-register with same email.** Spotted during M7 dogfooding. Backend tests confirm fresh row gets `streak_count=0`; suspected to be a frontend stale-store issue along an as-yet-unidentified path. To investigate.

## Release Tasks

- [ ] **Render Main Branch** Need to merge dev into main and then setup a main branch deployment on render. This is what cube.nebulouscode.com should point to long term.

## Post Release Cleanup

- [ ] **Dev Render Environement** dev doesn't need a custom url I'll hit onrender.com manually for dev long term

---

When something is done, just delete the line. When something new comes up, drop it in the right section.
