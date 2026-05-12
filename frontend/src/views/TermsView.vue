<script setup lang="ts">
import StaticPageView from './StaticPageView.vue'
</script>

<template>
  <StaticPageView eyebrow="Legal" title="Terms of Service">
    <p class="doc-meta">
      <strong>Last updated:</strong> May 12, 2026<br />
      <strong>Effective date:</strong> May 12, 2026
    </p>

    <h2>About these terms</h2>
    <p>
      Cube Practice ("the service", "the app", "we", "us") is a non-commercial hobby/portfolio project that helps users practice puzzle cube algorithms in a focused, distraction-free environment. These Terms of Service ("Terms") govern your use of the hosted application at our website. The source code may be published in a public repository from time to time; if so, your rights to the code are governed by the license posted in that repository (currently the <strong>PolyForm Noncommercial License 1.0.0</strong>), not by these Terms. We make no commitment to keep the code publicly available and may make the repository private at any time without notice. These Terms govern only the hosted service.
    </p>
    <p>
      By creating an account or otherwise using the service, you agree to these Terms and to our <RouterLink to="/privacy">Privacy Policy</RouterLink>. If you don't agree, don't use the service.
    </p>
    <p>
      This is a hobby project run by a single maintainer. It may change, break, lose data, or be shut down at any time, sometimes without warning.
    </p>

    <h2>1. Eligibility</h2>
    <p>
      You must be at least 13 years old to use Cube Practice. By creating an account, you represent that you are at least 13.
    </p>
    <p>
      We do not knowingly allow anyone under 13 to use the service. If we learn that an account belongs to someone under 13, we will delete it.
    </p>

    <h2>2. Your account</h2>

    <h3>Creating an account</h3>
    <p>
      You provide an email address, a display name, and a password. Each account must use a distinct email address. You're responsible for keeping your password secret and for everything that happens under your account.
    </p>

    <h3>Multiple accounts</h3>
    <p>
      You may create multiple accounts as long as each uses a distinct email address. We reserve the right to merge or close duplicate accounts that appear to be circumventing rate limits, re-registration windows, or other abuse-prevention measures.
    </p>

    <h3>Account ownership and access</h3>
    <p>
      Accounts are personal to the registered user. You may change the email address on your account through the verified email-change flow; if you do, you remain responsible for who has access to it. We do not act as an intermediary for transferring accounts between people and do not verify the identity of anyone who gains access to an account through an email change.
    </p>

    <h3>Termination by you</h3>
    <p>
      You may delete your account at any time using the self-serve delete button in account settings. Deletion purges your data immediately, with the small exception described in the Privacy Policy.
    </p>

    <h3>Termination by us</h3>
    <p>
      We may suspend or terminate your account at any time, for any reason or no reason, with or without notice. This is consistent with the hobby-project nature of the service. We will not provide an export of your data prior to termination.
    </p>
    <p>
      Reasons we might terminate include, but are not limited to: violating these Terms, abuse of the service, illegal use, or our decision to shut the service down entirely.
    </p>

    <h3>Dormant accounts</h3>
    <p>
      If you do not log in for 12 consecutive months, we may delete your account after sending an email warning to the address on file. If we shut down the service entirely, no warning emails will be sent.
    </p>

    <h2>3. Acceptable use</h2>
    <p>
      You agree not to do any of the following:
    </p>
    <ul>
      <li>Scrape, crawl, or harvest data from the service</li>
      <li>Access the service through automated means without our written permission</li>
      <li>Reverse engineer the service beyond what applicable law permits</li>
      <li>Attempt to access any account other than your own</li>
      <li>Overwhelm the service with traffic, including denial-of-service attacks</li>
      <li>Use the service for any illegal purpose or in violation of any applicable law</li>
      <li>Harass, threaten, or abuse other users (limited application — there is no user-to-user content sharing in the service today)</li>
      <li>Circumvent rate limits, anti-bot challenges, or other access controls</li>
      <li>Use the service or any data obtained from it to train machine learning models</li>
      <li>Reskin, rehost, or otherwise create a competing hosted version of the service for resale (using the service yourself, including for teaching or in a classroom, is fine)</li>
    </ul>
    <p>
      We may take whatever action we think is appropriate in response to a violation, including terminating your account without notice.
    </p>

    <h2>4. Your content</h2>
    <p>
      You retain ownership of any content you create in the service — your custom algorithms, custom case nicknames, custom tags, learning state, and study history. We store this content on your behalf to operate the service for you. We claim no ownership of it.
    </p>
    <p>
      You grant us a narrow, non-exclusive license to store, process, and display your content back to you as necessary to provide the service. This license ends when you delete your account.
    </p>
    <p>
      There is no user-to-user content sharing in the service today, so your content is not displayed to anyone else.
    </p>
    <p>
      When you delete your account, your content is purged. See the <RouterLink to="/privacy">Privacy Policy</RouterLink> for the specifics of what is retained and for how long.
    </p>

    <h2>5. Data export</h2>
    <p>
      There is no self-serve data export feature today. If you want a copy of the data tied to your account, email <strong>me@nebulouscode.com</strong> and we will send you a JSON dump. Self-serve export is on the post-MVP wishlist but is not committed to a timeline.
    </p>

    <h2>6. The service is provided as-is</h2>

    <h3>No warranty</h3>
    <p>
      THE SERVICE IS PROVIDED "AS IS" AND "AS AVAILABLE", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING WITHOUT LIMITATION ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, OR ANY WARRANTY ARISING OUT OF COURSE OF DEALING OR USAGE OF TRADE.
    </p>
    <p>
      We make no representation that the service will be uninterrupted, error-free, secure, or free of viruses or other harmful components. We make no claims about the effectiveness of any spaced-repetition schedule, practice algorithm, or learning method offered by the service.
    </p>

    <h3>No uptime guarantee</h3>
    <p>
      The service runs on shared hosting infrastructure on a free or low-cost tier. It may go down for cold-starts, deploys, infrastructure issues, or indefinitely if we stop running it. There is no uptime SLA.
    </p>

    <h3>No data-loss warranty</h3>
    <p>
      We make no guarantee about data preservation. Database backups exist only as part of our hosting provider's point-in-time restore window (see the Privacy Policy for specifics). Beyond that window, data may be unrecoverable. If your data is irreplaceable, keep your own copy.
    </p>

    <h3>No support SLA</h3>
    <p>
      Support is best-effort by a single maintainer over email at <strong>me@nebulouscode.com</strong>. There is no ticketing system, no on-call rotation, and no guaranteed response time.
    </p>

    <h2>7. Limitation of liability</h2>
    <p>
      TO THE FULLEST EXTENT PERMITTED BY LAW, WE WILL NOT BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR PUNITIVE DAMAGES, OR ANY LOSS OF PROFITS, REVENUES, DATA, USE, GOODWILL, OR OTHER INTANGIBLE LOSSES, ARISING OUT OF OR RELATING TO YOUR USE OF (OR INABILITY TO USE) THE SERVICE — WHETHER BASED ON WARRANTY, CONTRACT, TORT (INCLUDING NEGLIGENCE), STATUTE, OR ANY OTHER LEGAL THEORY, AND WHETHER OR NOT WE HAVE BEEN INFORMED OF THE POSSIBILITY OF SUCH DAMAGE.
    </p>
    <p>
      The service is provided to you free of charge. To the maximum extent allowed by applicable law, our total liability to you for any claim arising out of or relating to these Terms or the service will not exceed ten U.S. dollars ($10).
    </p>

    <h2>8. Changes to the service</h2>
    <p>
      We may change, suspend, or discontinue any part of the service at any time. For significant changes, we will try to give reasonable notice in the app or by email. For project shutdown, we will try to give notice but cannot guarantee it.
    </p>

    <h2>9. Changes to these Terms</h2>
    <p>
      We may update these Terms from time to time.
    </p>
    <ul>
      <li>For <strong>material changes</strong> — anything affecting your rights, obligations, or how your data is handled — we will email the address on file, update the "Last updated" date, and surface a notice on your next login.</li>
      <li>For <strong>non-material changes</strong> — typo fixes, link updates, formatting — we will only update the "Last updated" date.</li>
    </ul>
    <p>
      Continued use of the service 30 days after a material change takes effect constitutes acceptance of the updated Terms. If you don't agree, delete your account before the 30 days elapse.
    </p>

    <h2>10. Intellectual property</h2>

    <h3>Our content</h3>
    <p>
      The Cube Practice website, including its UI, source code, landing page copy, logo (a stylized algorithm case), and other original content, is owned by the maintainer (nebulous-code) and protected by copyright and other intellectual property laws.
    </p>
    <p>
      The source code may, at our discretion, be published in a public repository. If it is, your rights to the code are governed by the license posted in that repository, not by these Terms. As of this version of the Terms, that license is the <strong>PolyForm Noncommercial License 1.0.0</strong> — anyone may read, run, modify, and redistribute the code for any noncommercial purpose; commercial use of the code itself, including standing up a competing hosted version of the service for resale, is reserved to the maintainer. We may change the license posted in the repository or make the repository private at any time without notice; doing so does not change these Terms or your rights to the hosted service.
    </p>

    <h3>Cube algorithms</h3>
    <p>
      Algorithms themselves — sequences of cube moves — are functional and not copyrightable. The presentation, organization, naming, and tagging of algorithms within the service is part of our content.
    </p>

    <h3>Third-party fonts</h3>
    <p>
      The service uses Google Fonts (Newsreader, Inter Tight, JetBrains Mono) licensed under the SIL Open Font License 1.1. No further attribution is required at the Terms level.
    </p>

    <h3>Trademarks</h3>
    <p>
      We hold no registered trademarks for "Cube Practice" or the logo. This is not an invitation to register them yourself.
    </p>

    <h2>11. Governing law and disputes</h2>
    <p>
      These Terms are governed by the laws of the State of Texas, without regard to its conflict-of-laws principles.
    </p>

    <h3>Informal resolution first</h3>
    <p>
      Before filing any legal action, please email <strong>me@nebulouscode.com</strong> with a description of the dispute. We will try in good faith to resolve it within 30 days. Most disagreements can be sorted out this way.
    </p>

    <h3>Venue</h3>
    <p>
      If informal resolution doesn't work, any lawsuit must be filed in the state or federal courts located in Collin County, Texas. You and we consent to the personal jurisdiction of those courts.
    </p>
    <p>
      There is no arbitration clause and no class-action waiver in these Terms.
    </p>

    <h2>12. DMCA / copyright</h2>
    <p>
      The service does not host user-uploaded content visible to other users. There is no mechanism for users to publish copyrighted material to or through the service. Accordingly, we do not maintain a DMCA designated agent and do not process DMCA takedown requests.
    </p>
    <p>
      If you believe content within the service (for example, landing page copy or original algorithms presentation) infringes your copyright, email <strong>me@nebulouscode.com</strong> and we will look into it.
    </p>

    <h2>13. General</h2>

    <h3>Severability</h3>
    <p>
      If any part of these Terms is held to be unenforceable, the rest remains in effect.
    </p>

    <h3>Entire agreement</h3>
    <p>
      These Terms, together with the <RouterLink to="/privacy">Privacy Policy</RouterLink>, make up the entire agreement between you and us regarding the service. They replace any prior agreements on the same subject.
    </p>

    <h3>No waiver</h3>
    <p>
      If we don't enforce a part of these Terms in one situation, that doesn't mean we've waived our right to enforce it later or in other situations.
    </p>

    <h3>Assignment</h3>
    <p>
      We may assign or transfer these Terms (for example, if the project is handed off to another maintainer). You may not assign your account or your rights under these Terms to anyone else.
    </p>

    <h3>Headings</h3>
    <p>
      Section headings are for convenience and have no legal effect.
    </p>

    <h2>14. Contact</h2>
    <p>
      <strong>nebulous-code</strong> — <strong>me@nebulouscode.com</strong>
    </p>
  </StaticPageView>
</template>
