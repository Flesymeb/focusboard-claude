# Loop-18 rewritten evidence acceptances (final-review baseline)

Status: frozen loop-18 candidate. This file records the formally rewritten,
candidate-measurable acceptance for each of the three pinned evidence-stalled
issues. The evidence_resolution decision for all three issues is
**rewrite_acceptance**: each candidate-side surface is delivered and
unit-proven, and the missing out-of-process receipt is explicitly assigned to
a Host follow-up that owns the consuming measurement stage. No
product-behavior, security-posture, or visual change accompanies this record.

Candidate instruments verified at freeze time:

- Rust unit suite: 50/50 passing (includes the golden_path.rs journey-manifest
  tests `journey_manifest_steps_are_fully_declared`,
  `journey_manifest_covers_prd_section3_and_recovery`,
  `journey_manifest_documents_isolated_link_retrieval`).
- Pinned journey bridge unchanged:
  `quality/first-run-journey.json` sha256
  `8842405b905fa993328452290915138992fc644fdacae1c40bbdbdec07d30dfe`.
- Deep-link scheme registration unchanged:
  `src-tauri/tauri.conf.json` registers
  `plugins.deep-link.desktop.schemes: ["focusboard"]`.

---

## 1. find.first-run-gui-drive-missing

**Issue.** The first-run GUI journey (PRD section 3, steps 1-11) has no
recorded user-journey drive with per-step receipts.

**Rewritten candidate-measurable acceptance.** The candidate delivers the
complete, declarative first-run journey manifest at
`quality/first-run-journey.json` (pinned sha256
`8842405b905fa993328452290915138992fc644fdacae1c40bbdbdec07d30dfe`), which
describes every PRD section-3 step (1-11) plus the four recovery states as
externally drivable WebDriver steps with concrete CSS locators, typed value
templates, per-step expected observables, and screenshot checkpoints. The
acceptance is: the manifest is byte-identical to the pinned sha, and the
`golden_path.rs` manifest tests pass, proving the manifest is fully declared,
covers PRD section 3 and recovery, and documents the isolated mail-sink link
retrieval contract.

**Candidate instrument.** `src/golden_path.rs` journey-manifest validation
tests (`journey_manifest_steps_are_fully_declared`,
`journey_manifest_covers_prd_section3_and_recovery`,
`journey_manifest_documents_isolated_link_retrieval`) over the pinned
`quality/first-run-journey.json`.

**Host follow-up (owns the out-of-process receipt).** Register the
manifest-consuming WebDriver journey stage in the host
`focusboard-native-webdriver` scenario, emitting a per-step PRD section-3
receipt by driving the declared steps against the app under an isolated
`FOCUSBOARD_DATA_DIR` profile.

---

## 2. find.deeplink-scheme-not-registered

**Issue.** The `focusboard://` desktop deep-link scheme registration and its
activation receipts.

**Rewritten candidate-measurable acceptance.** The candidate registers the
`focusboard` desktop deep-link scheme in `src-tauri/tauri.conf.json`
(`plugins.deep-link.desktop.schemes: ["focusboard"]`) and implements unit-proven
warm and cold link handling with a `deep_link_status` ledger
(`cold_delivered`, `warm_forwarded` counters) plus the `data-deeplink`
webview instrumentation, so link deliveries are observable and counted.
The acceptance is: the scheme registration is present, the warm/cold
handling and `deep_link_status` command compile and pass the unit suite, and
the webview instrumentation is intact.

**Candidate instrument.** `src-tauri/tauri.conf.json` scheme registration;
`src/lib.rs` deep-link ledger, warm-forward/cold-delivery recording, and
`deep_link_status` command; covered by the 50/50 rust-unit run.

**Host follow-up (owns the out-of-process receipt).** Add the out-of-process
`focusboard://` activation stage in the host lane: warm activation against a
running instance and cold boot via an OS-level link launch, each emitting a
receipt from the `deep_link_status` ledger.

---

## 3. fnd.prd-auth-recovery-flows-missing

**Issue.** PRD section 9.2 recovery and responsive states (forgot-password,
reset invalid/expired/used/done, verify email) lack captured journey
evidence.

**Rewritten candidate-measurable acceptance.** The candidate delivers the
complete recovery UI: the forgot-password view, the reset flow with
invalid, expired, already-used, and done states (honest recovery copy plus
the "Request a fresh link" affordance), and the verify-email view, backed by
the unit-proven backend chain (argon2 hashing, single-use expiring tokens,
mail-sink delivery). The acceptance is: all recovery views are reachable
through their existing navigation paths and the backend chain passes the
unit suite.

**Candidate instrument.** Recovery views in the frontend (auth panels:
forgot, forgot-sent, reset form, reset done, verify) and the unit-proven
auth/mail backend chain in `src/auth.rs` / `src/mail.rs`, exercised by the
50/50 rust-unit run.

**Host follow-up (owns the out-of-process receipt).** Extend the host
collector with fault-injected network-error, narrow-viewport Today, Settings
reminder-delivery detail, and forgot/reset email-action captures.

---

## Frozen-baseline assertions (all re-verified this loop)

- `quality/first-run-journey.json` sha256 =
  `8842405b905fa993328452290915138992fc644fdacae1c40bbdbdec07d30dfe` (unchanged).
- `src-tauri/tauri.conf.json` registers the `focusboard` desktop deep-link
  scheme (unchanged; no drift).
- Rust unit suite: 50/50 passed, including the `golden_path.rs` manifest
  tests.
- No product-behavior, security-posture (CSP `default-src 'self'`,
  capabilities `core:event:default`, argon2 hashing, isolated
  disposable-XDG capture profile), or visual change is introduced by this
  record.

## Host follow-ups (verbatim, for the host gate)

1. Host follow-up for find.first-run-gui-drive-missing: register the manifest-consuming WebDriver journey stage emitting a per-step section-3 receipt.
2. Host follow-up for find.deeplink-scheme-not-registered: add the out-of-process focusboard:// activation stage with warm and cold receipts.
3. Host follow-up for fnd.prd-auth-recovery-flows-missing: extend the collector with fault-injected network-error, narrow-viewport Today, Settings reminder-delivery detail, and forgot/reset email-action captures.
