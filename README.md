# HoH Development Record

<!-- HoH generated development record; do not edit. -->

**Product:** `focusboard`  
**Workflow:** Project Planner → Developer → QA Tester

This README records high-level autonomous development and QA history.

---

## Publication scope

This product history records iteration goals and QA outcomes. Detailed tool, cache, receipt, and staging provenance remains in the private HoH Runtime audit.

This README is the canonical issue ledger for this product: the host-derived persistent ledger table and per-loop QA findings below are the authoritative issue record. GitHub Issues are not used to track this product's issues.

---

## Persistent issue ledger

| Field | Value |
| --- | --- |
| Open | 11 |
| Closed | 6 |
| All | 17 |

---

## Loop 14

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-14-6b853ced8f4f` | `warm_start:loop-13-6b853ced8f4f` | PLANNED |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Close gap:prd_implementation_review this loop by shipping the pinned PRD depth slices — Subtask and ActivityEvent made live (PRD 5), Inbox search with priority, due, and completion filters plus a priority editor (PRD 6.1), and the Projects board with Planned/In progress/Done columns, archive/restore, and keyboard-accessible moves (PRD 6.2) — and by resolving the two evidence-stalled journey issues through measurement infrastructure the registered host replay can actually execute: out-of-process focusboard:// activation cold and warm with counters exported in runtime_state, and a clean-store typed-UI first-run drive. Preserved assets (auth recovery implementation and 28-test suite, deterministic two-run rendering, PRD 9 visual direction) must not regress. |
| Strategy | `vertical-slices-registered-instrument-contract` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NOT STARTED |

### QA Tester

| Field | Value |
| --- | --- |
| Status | UNTESTED |

---

## Loop 13

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-13-6b853ced8f4f` | `development_snapshot:attempt-c630bb882d92d844409cde07` | FAIL |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Deliver the complete PRD 4.2/8 password-recovery journey — forgot-password request with enumeration-safe responses, single-use expiring reset that invalidates existing sessions, and a reset email action page with expired and used-token recovery states — while making the two strategy-exhausted external-journey evidence debts measurable through opt-in isolated diagnostics readable by the host's focusboard-native-webdriver external lanes, keeping cargo fmt clean and every preserved behavior intact. |
| Strategy | `recovery-surface-and-external-evidence-readiness` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 10 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-c630bb882d92d844409cde07` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-13-6b853ced8f4f` |
| Product completion | IN PROGRESS |
| Review scope | `phase` |
| Findings | 12 (5 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `fnd.auth-recovery-implemented-pending-journey` | MAJOR | INTERNAL | HIGH | Forgot/reset password flows are now implemented with host-run unit coverage, but no bound receipt drives forgot -> reset -> sign-in as an external UI journey, so the PRD 4.2/8 recovery capability stays fixed_pending_verify rather than verified. |
| `fnd.rustfmt-regression-librs1246-closed` | NOTE | INTERNAL | LOW | Verified closed: the loop-12 rustfmt regression at src-tauri/src/lib.rs:1246 is gone; cargo fmt --check exits 0 both in the host white-box lane and in an independent Tester re-run over the frozen source. |
| `fnd.prd-subtask-absent-activity-schema-dead` | MAJOR | USER BLOCKING | HIGH | PRD 5 partial progress and remaining gap: an activity_events table was added this loop but nothing writes or reads it, and the Subtask entity is still absent everywhere, so subtasks are impossible and no activity history is ever recorded. |
| `fnd.prd-inbox-search-filters-still-missing` | MAJOR | USER BLOCKING | HIGH | Inbox still has no search and no priority or due filter, and priority remains settable nowhere in the UI even though the tasks table stores a priority column; confirmed in source and in this loop's runtime capture. |
| `fnd.prd-projects-board-archive-still-missing` | MAJOR | USER BLOCKING | HIGH | Projects still offers create and rename only: no Planned/In progress/Done board columns, no archive or restore, and no drag-and-drop with a keyboard alternative; confirmed in source and this loop's runtime capture. |
| `fnd.prd-settings-depth-still-missing` | MAJOR | USER BLOCKING | HIGH | Settings still covers only profile (email, display name, timezone), a global reminder toggle, and reminder deliveries; password change, sessions management, and account-deletion request are absent, confirmed in source and runtime capture. |
| `fnd.state-board-states-still-uncaptured` | MINOR | INTERNAL | MEDIUM | PRD 9.2 recovery and responsive states remain runtime-uncaptured: all 13 bound captures across both runs are 1280x800 success or anonymous-auth states; no network-error retry, narrow-viewport Today, or email-settings detail capture exists. |
| `fnd.deeplink-external-activation-still-unproven` | MAJOR | INTERNAL | HIGH | External focusboard:// activation is still receipt-unproven: external_deeplink is not_run in both loop-13 receipts with counters unread (warm=0, cold=0), while registration prerequisites stay proven by the built-config scheme unit test. |
| `fnd.section3-external-journey-still-unproven` | MAJOR | INTERNAL | HIGH | PRD section-3 remains corroborated but not proven as an external user journey: the only verdict is the host-triggered renderer self-drive (PASS 16/16 in both runs, explicitly untrusted authority); typed-form registration, email-link follow, and UI sign-out/sign-in persistence are still runtime-unproven. |
| `fnd.isolated-store-probe-foreign-path-recurs` | NOTE | INTERNAL | LOW | Host collector's isolated_native_store check stays false on the foreign com.focusboard.app data path while the product identifier is app.focusboard.desktop; product-side identifier-derived data dir is unit-proven. Host follow-up. |
| `fnd.whitebox-advisory-stale-layout-recurs` | NOTE | INTERNAL | LOW | Advisory white-box lane remains miscalibrated to the single-crate layout: source-contract blocks on an absent src-tauri/src/commands.rs and semantic-implementation reports 1/5 while the crate links rusqlite, passes 28 unit tests, and captures prove durable data. Host follow-up. |
| `fnd.hydration-marker-miscalibration-recurs` | NOTE | INTERNAL | LOW | Host hydration calibration is still wrong: all six signed-in surface rows report active_nav='page', matched=false, hydrated=false while every bound capture shows the correct heading and highlighted nav item. Host follow-up. |

---

## Loop 12

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-12-9dad3086a4fc` | `development_snapshot:attempt-137b1337bcb8d20b3bd2792d` | FAIL |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | golden_path |
| Objective | Close gap:prd_golden_path on the loop-12 candidate by making the release-critical journey externally provable: register the focusboard:// activation entry point so out-of-process warm and cold activation actually reaches the app, expose the identifier-derived durable-store location (app.focusboard.desktop) in the runtime receipt, and keep the PRD section-3 first-run path, sign-out persistence, and validated capture lanes unregressed — so validate:prd_golden_path can reach pass on host-recaptured external evidence instead of renderer self-drive authority. |
| Strategy | `external-entrypoint-and-receipt-instrumentation` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 11 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-137b1337bcb8d20b3bd2792d` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-12-9dad3086a4fc` |
| Product completion | IN PROGRESS |
| Review scope | `golden_path` |
| Findings | 12 (8 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `fnd.prd-auth-recovery-flows-missing` | MAJOR | USER BLOCKING | HIGH | PRD 4.2 and 8 require a forgot-password request, single-use password reset, and a reset email action page; the candidate implements only sign-in, create-account, and verify-email surfaces, so password recovery is impossible. |
| `fnd.prd-projects-board-archive-missing` | MAJOR | USER BLOCKING | HIGH | PRD 6.2 requires a compact board view with Planned/In progress/Done columns, project archive/restore, and a keyboard-accessible drag alternative; the candidate renders a single list per project with create/rename only. |
| `fnd.prd-inbox-search-filters-missing` | MAJOR | USER BLOCKING | HIGH | PRD 6.1 requires Inbox search plus priority, due, and completion filters; the candidate has only an open/completed split, and the Task model's priority field has no UI to set or display it. |
| `fnd.prd-settings-depth-missing` | MAJOR | USER BLOCKING | HIGH | PRD 8 requires Settings to cover profile, timezone, notification preferences, password, sessions, and account deletion request; the candidate implements only profile, a global reminder toggle, and reminder deliveries. |
| `fnd.prd-data-model-subtask-activity-missing` | MAJOR | USER BLOCKING | MEDIUM | PRD 5 mandates Subtask (task id, title, completion state, order) and ActivityEvent (entity ref, event type, timestamp, history render) among authoritative entities that must exist; neither exists anywhere in the candidate. |
| `fnd.deeplink-external-activation-still-unproven` | MAJOR | INTERNAL | HIGH | Out-of-process focusboard:// activation remains receipt-unproven: external_deeplink is not_run in both loop-12 receipts with counters unread (warm=0, cold=0), while registration prerequisites are now independently proven by the host-run built-config scheme test. |
| `fnd.section3-external-journey-still-unproven` | MAJOR | INTERNAL | HIGH | PRD section-3 remains corroborated but not proven as an external user journey: the only journey verdict is the host-triggered renderer self-drive (PASS 16/16 twice, explicitly untrusted authority); typed-form registration, email-link follow, and UI sign-out/sign-in persistence are still runtime-unproven. |
| `fnd.isolated-store-probe-still-foreign-path` | MINOR | INTERNAL | LOW | fixed_pending_verify is disproven: the host collector's isolated_native_store check still probes the foreign com.focusboard.app data path while the product identifier is app.focusboard.desktop, so the check stays false and unlabeled for a fourth loop. |
| `fnd.hydration-marker-miscalibration-persists` | MINOR | INTERNAL | LOW | Host receipt hydration calibration is still wrong: all 6 signed-in surface rows report active_nav='page', matched=false, hydrated=false while every bound capture shows the correct heading and highlighted nav item. |
| `fnd.state-board-states-still-uncaptured` | MAJOR | INTERNAL | HIGH | PRD 9.2 recovery and responsive states remain uncaptured at runtime on this loop's bound evidence: every capture is a 1280x800 success state; network-error-with-retry, narrow-viewport Today, and the email-settings detail have no bound live capture. |
| `fnd.whitebox-advisory-stale-layout-persists` | MINOR | INTERNAL | LOW | Advisory white-box lane remains miscalibrated to the single-crate layout: source-contract blocks on an absent src-tauri/src/commands.rs and semantic-implementation reports 1/5 while cargo links rusqlite, 22 unit tests pass, and captures prove durable data. |
| `fnd.rustfmt-regression-librs1246` | MINOR | INTERNAL | LOW | cargo fmt --check regresses on the frozen candidate: a formatting diff at src-tauri/src/lib.rs:1246 in the new built-config scheme assertion reintroduces the style debt previously closed as find.rustfmt-style-debt. |

---

## Loop 11

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-11-8c9f20ade198` | `development_snapshot:attempt-a4f6f943df946142eff7e459` | NEEDS VISUAL QA |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | golden_path |
| Objective | Make the PRD golden path provable by instruments other than the candidate's own counters: verify and harden the out-of-process focusboard:// activation preconditions under disposable-xdg isolation, keep every first-run step drivable through visible UI and OS entry points, and rewrite the three stalled acceptances so host receipts measure external actions — with the host collector and probe repairs routed to Host follow-up. |
| Strategy | `external-lane-acceptance-rewrite` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 1 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-a4f6f943df946142eff7e459` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | UNTESTED |

---

## Loop 10

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-10-543cf61f25fc` | `development_snapshot:attempt-92a714ad03318e4b0d35c6b4` | PHASE PASS / PRODUCT IN PROGRESS |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Produce a new candidate on which the host-extended replay can conclusively pass: one receipt bound to the new hash with data-goldenpath-state=pass at 16/16 as host-observed evidence, product_ui_separation=pass, live captures of Today, Inbox, Projects, Calendar, Focus, and Settings, and readable deeplink counters — while preserving the anonymous smoke, embedded-assets build, clean session lifecycle, green 20/20 unit suite, and rustfmt-clean sources. |
| Strategy | `collector-unblock-capture-ready-candidate` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 4 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-92a714ad03318e4b0d35c6b4` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | PASS |
| Candidate | `loop-10-543cf61f25fc` |
| Product completion | IN PROGRESS |
| Review scope | `phase` |
| Findings | 6 (0 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `fnd.section3-external-journey-unproven-loop10` | MINOR | INTERNAL | HIGH | PRD section-3 first-run path is corroborated at runtime but not proven as an external user journey: typed-form registration, email-link follow, and sign-out/sign-in restart persistence remain runtime-unproven even though the host-triggered product self-drive concluded pass 16/16 twice. |
| `fnd.deeplink-activation-unproven-loop10` | MINOR | INTERNAL | MEDIUM | Out-of-process focusboard:// activation remains runtime-unproven: external_deeplink is not_run in both receipts and the probe's deeplink counters stay unread (warm=0, cold=0). |
| `fnd.state-board-states-uncaptured-loop10` | MINOR | DEGRADED | MEDIUM | Section 9.2 recovery and responsive states are still uncaptured at runtime on the frozen candidate: network-error-with-retry, narrow-viewport Today, and the email-settings detail (test email action, reminder timing) have no bound live capture; all captures are 1280x800 success states. |
| `fnd.surface-hydration-marker-page-loop10` | NOTE | INTERNAL | LOW | Host receipt hydration flags are miscalibrated: all 12 signed-in surface rows report hydrated=false/matched=false because the polled active_nav marker returns 'page', while the screenshots prove correct headings and nav highlight on every surface. |
| `fnd.isolated-store-probe-foreign-path-loop10` | NOTE | INTERNAL | LOW | The isolated_native_store advisory still probes the foreign com.focusboard.app data path (product identifier is app.focusboard.desktop), so its false result stays inconclusive and unlabeled. |
| `fnd.whitebox-advisory-stale-layout-loop10` | NOTE | INTERNAL | LOW | Advisory white-box checks remain miscalibrated to the candidate layout: source-contract fails on absent commands.rs and semantic-implementation reports 1/5 while rust units (20/20), rusqlite linkage, and visible durable data contradict it. |

---

## Loop 09

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-09-ea4db9b9ae7e` | `development_snapshot:attempt-287476a390d05d43e27697d6` | FAIL |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Make the trigger-driven golden-path lane receipt-drivable on the new candidate: a standard WebDriver click on #golden-path-trigger starts the drive, a passing drive ends signed-in on a stable surface with all six section-8 surfaces reachable via the primary nav for per-surface capture, a failing drive reports its step and reason without stranding the session, and the deeplink counters stay readable — so one extended-replay receipt bound to this candidate hash can show product_ui_separation=pass, probe passed=16 of 16, and live signed-in surface captures, with the replay extension and advisory calibration as Host follow-up. |
| Strategy | `registered-receipt-terminal-contract` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 8 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-287476a390d05d43e27697d6` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-09-ea4db9b9ae7e` |
| Product completion | IN PROGRESS |
| Review scope | `phase` |
| Findings | 6 (4 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `fnd.replay-collector-cannot-conclude-loop09` | MAJOR | INTERNAL | HIGH | Host gate defect persists into loop 9: the registered replay still never triggers the golden-path drive and golden_path_observation() still hardcodes state=unavailable, so no candidate — including this one whose drive is now unit-proven and receipt-drivable by design — can conclude golden_path=pass through the registered lane. |
| `fnd.first-run-gui-drive-missing-loop09` | MAJOR | INTERNAL | HIGH | Release-critical PRD section-3 journey remains runtime-unproven (8th loop): the bound replay drove only the anonymous invalid-email smoke with the probe idle 0/16; the new terminal contract is implemented and unit-proven but has no live receipt, and the blocking cause is the host-side collector gate. |
| `fnd.signedin-views-no-live-capture-loop09` | MAJOR | INTERNAL | HIGH | Bound pixel evidence still covers only the anonymous sign-in shell; no live captures exist of any signed-in surface or the section 9.2 recovery/responsive states, so section 8/9 conformance for those surfaces stays unjudgeable on the frozen candidate (status remains blocked). |
| `fnd.deeplink-activation-unproven-loop09` | MINOR | INTERNAL | MEDIUM | follow-the-link return and expired/reused-token recovery stay runtime-unproven: no bound receipt exercises an out-of-process focusboard:// activation and the probe's deeplink counters remain unread; unit coverage for token parsing and recovery stays green on this candidate. |
| `fnd.advisory-whitebox-stale-loop09` | MINOR | INTERNAL | LOW | Advisory white-box checks remain miscalibrated to this candidate's layout in the loop-09 receipt (missing commands.rs; storage.rs/App.ts citations), contradicting verified source facts; inconclusive infrastructure noise, not a product signal. |
| `fnd.isolated-store-probe-mismatch-loop09` | NOTE | INTERNAL | LOW | Advisory isolated_native_store check still reports false in both loop-09 receipts due to the foreign com.focusboard.app probe path (product identifier is app.focusboard.desktop); inconclusive for this product, not a product signal. |

---

## Loop 08

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-08-d5a71171daed` | `development_snapshot:attempt-492e7ceb088de6fadcf20ac8` | FAIL |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Make the release-critical PRD section-3 first-run path measurable through the registered native replay lane by adding product-owned instrumentation: a hidden #golden-path-probe gate wired to the already-proven self-drive flow, honest step-attributed state reporting, and an observable focusboard:// activation state, while preserving the entire passing baseline. This lets the host recapture signed-in per-surface evidence and gives the pinned evidence-stalled issues a measurable acceptance for the first time in six loops. |
| Strategy | `strategy.product-owned-measurement-gate` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 11 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-492e7ceb088de6fadcf20ac8` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-08-d5a71171daed` |
| Product completion | IN PROGRESS |
| Review scope | `phase` |
| Findings | 8 (4 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `fnd.goldenpath-probe-fix-verified` | NOTE | INTERNAL | LOW | fixed_pending_verify confirmed: the #golden-path-probe element now renders and is readable by the registered collector, with trigger-only start and backend-held credentials; the product-side instrumentation defect is closed and its receipt-level acceptance is superseded by the collector limitation issue. |
| `fnd.replay-collector-cannot-conclude-golden-path` | MAJOR | INTERNAL | HIGH | New host-side gate defect: the registered replay never clicks #golden-path-trigger and golden_path_observation() hardcodes state=unavailable, so no candidate — even one whose drive passes — can ever conclude golden_path.state=pass through the registered lane. Host follow-up, not a candidate write target. |
| `fnd.first-run-gui-drive-missing` | MAJOR | INTERNAL | HIGH | Seventh loop with the release-critical PRD section-3 path unobserved at runtime: the bound replay again drove only the anonymous invalid-email validation; the probe that could prove the journey now renders but nothing triggers it through the registered lane. |
| `fnd.signedin-views-no-live-capture` | MAJOR | INTERNAL | HIGH | Bound pixel evidence still covers only the anonymous sign-in shell at 1280x800; no live captures exist of any signed-in surface or the section 9.2 recovery/responsive states, so section 9 conformance for those surfaces remains unjudgeable on the frozen candidate. |
| `fnd.golden-path-surfaces-missing` | MINOR | INTERNAL | HIGH | All section-8 signed-in surfaces plus focus/reminders/IPC backend exist in source with 20/20 unit coverage, but none of it is proven at runtime on the frozen candidate; the bound receipt's renderer observations cover only the auth shell. |
| `fnd.deeplink-scheme-not-registered` | MINOR | INTERNAL | MEDIUM | The focusboard:// activation route is implemented and unit-tested in source, but no bound runtime receipt exercises an out-of-process activation, so follow-the-link return and expired/reused-token recovery stay runtime-unproven; the probe now exposes deeplink counters but nothing external activates them. |
| `fnd.isolated-store-probe-path-mismatch` | NOTE | INTERNAL | LOW | The advisory isolated_native_store check still reports false in both loop-08 receipts because the host probe path targets com.focusboard.app while the product identifier is app.focusboard.desktop; the check stays inconclusive for this product and is not a product signal. |
| `fnd.advisory-whitebox-stale-layout` | MINOR | INTERNAL | LOW | The advisory source-contract and semantic-implementation white-box checks still fail against this candidate's actual layout in the loop-08 receipt (missing commands.rs; citations of storage.rs/App.ts), contradicting verified source facts; they remain inconclusive infrastructure noise, not product signals. |

---

## Loop 07

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-07-8da584af31e8` | `warm_start:loop-06-8da584af31e8` | DEVELOPER ATTEMPT INCOMPLETE |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Close gap:prd_implementation_review by shipping the product-owned golden-path probe gate: a hidden #golden-path-probe surface wired to the existing section-3 self-drive flow and publishing focusboard:// activation outcomes, so the registered native replay can drive the full signed-in first-run path out-of-process and conclude a real golden_path.state instead of unavailable, restoring runtime observability of the release-critical PRD path. |
| Strategy | `product-owned-goldenpath-probe` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | INCOMPLETE ATTEMPT |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 0 |
| Changed paths | 0 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-0b4ae76a9bef3f0410a11914` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | UNTESTED |

---

## Loop 06

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-06-8da584af31e8` | `development_snapshot:attempt-73a120af5cdb7d6fa28f5c85` | FAIL |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Deliver the first reviewable runtime evidence for the implemented Focusboard surfaces: repair the registered focusboard-verify white-box lane (import-time argv hijack) so one combined white_box and black_box receipt exists for the staged candidate, extend the candidate self-drive and the native replay scenario to observe warm and cold focusboard:// activation with single-use tokens and all six signed-in views plus the section 9.2 recovery states at 1280x800 and 640x480, and clear the rustfmt debt, while the native launch slice, least-privilege security posture, 27-command IPC surface, and data-testid hooks stay preserved. |
| Strategy | `repair-lane-then-bind-live-evidence` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 0 |
| Changed paths | 4 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-73a120af5cdb7d6fa28f5c85` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-06-8da584af31e8` |
| Product completion | IN PROGRESS |
| Review scope | `phase` |
| Findings | 9 (3 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `find.goldenpath-probe-instrumentation-missing` | MAJOR | INTERNAL | HIGH | Renderer still renders no #golden-path-probe element, so the registered native replay can only ever conclude golden_path.state=unavailable and the release-critical section-3 path cannot be proven through the registered lane. |
| `find.first-run-gui-drive-missing` | MAJOR | INTERNAL | HIGH | The release-critical PRD section-3 path remains unobserved at runtime for a sixth loop: the bound replay drove only an invalid-email validation on the anonymous shell; no signed-in step was ever driven through the live UI. |
| `find.signedin-views-no-live-capture` | MAJOR | INTERNAL | HIGH | Bound pixel evidence still covers only the anonymous sign-in shell; no live captures exist of any signed-in surface or the section 9.2 recovery/responsive states, so section 9 conformance for those surfaces is unjudgeable. |
| `find.deeplink-scheme-not-registered` | MINOR | INTERNAL | MEDIUM | The focusboard:// activation route is implemented and unit-tested in source, but no bound runtime receipt exercises an out-of-process activation, so follow-the-link return and expired/reused-token recovery stay unproven at runtime. |
| `find.golden-path-surfaces-missing` | MINOR | INTERNAL | HIGH | All section-8 signed-in surfaces plus focus/reminders/IPC backend now exist in source with unit coverage, but none of it is proven at runtime on the frozen candidate. |
| `find.isolated-store-probe-path-mismatch` | NOTE | INTERNAL | LOW | The advisory isolated_native_store check still probes com.focusboard.app while the product identifier is app.focusboard.desktop, so the check reports false in both loop-06 receipts and stays inconclusive for this product. |
| `find.advisory-whitebox-stale-layout` | MINOR | INTERNAL | LOW | New issue: the advisory source-contract and semantic-implementation white-box checks are calibrated to a foreign file layout (commands.rs/storage.rs/App.ts) and fail against this candidate regardless of product quality, contradicting verified source facts (store.rs SQLite schema, bundled rusqlite, invoke() IPC in six renderer modules). |
| `find.rustfmt-style-debt-closed` | NOTE | INTERNAL | LOW | Closed: rustfmt debt is resolved - cargo fmt --check exits 0 on the frozen candidate, independently re-verified this phase and matching the bound white_box rust-format pass in native.host.1. |
| `find.whitebox-lane-repaired` | NOTE | INTERNAL | LOW | Closed: the whitebox lane hijack is resolved - the doctor probe now returns the focusboard-verify payload and the bound capture_evidence receipt contains both white_box and black_box sections. |

---

## Loop 05

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-05-f4298baf5f14` | `development_snapshot:attempt-c7060e28ed2a07d328917aa0` | FAIL |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | golden_path |
| Objective | Complete the release-critical PRD section 3 golden path in the running app: replace the Calendar, Focus, and Settings placeholders with real views backed by Reminder and FocusSession entities, an idempotent reminder scheduler delivering through the configured local mail sink, and durable focus-session transitions, so a live driver can perform steps 6-10 with durable data and produce golden-path receipts that no longer report unavailable. |
| Strategy | `golden-path-surface-completion` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 21 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-c7060e28ed2a07d328917aa0` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-05-f4298baf5f14` |
| Product completion | IN PROGRESS |
| Review scope | `golden_path` |
| Findings | 7 (4 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `find.golden-path-not-driven-live` | BLOCKER | unlabelled | HIGH | The release-critical PRD section 3 path is still unobserved at runtime: both bound host receipts report golden_path.state=unavailable and the replay drove only an invalid-email validation on the anonymous auth shell; no signed-in step (project, task with reminder, focus session, completion, reminder email, sign-out/in persistence) was ever driven through the live UI. |
| `find.whitebox-lane-import-hijack` | MAJOR | INTERNAL | MEDIUM | The registered focusboard-verify white-box lane is still unreachable: this loop's probe of tools/bin/focusboard-verify --doctor returned the focusboard-native-webdriver doctor payload, proving the entry point again dispatched into the native-webdriver argparse at import time, so no receipt with white_box and black_box sections can be produced. |
| `find.goldenpath-probe-instrumentation-missing` | MAJOR | INTERNAL | HIGH | The candidate renders no #golden-path-probe element, so the registered host replay can only ever conclude golden_path.state=unavailable: its golden-path check is a 60s DOM wait on that product-owned element's data-goldenpath-state attribute, and the renderer never provides it despite the path surfaces now existing. |
| `find.signedin-views-no-live-capture` | MAJOR | INTERNAL | HIGH | Bound pixel evidence still covers only the anonymous sign-in shell: no live captures exist of the signed-in shell, Today, Inbox, Projects, Calendar, Focus, Settings, the section 9.2 empty-inbox and error-with-retry states, or narrow-width layouts, so section 9 conformance for those surfaces is unjudgeable. |
| `find.deeplink-runtime-unproven` | MINOR | INTERNAL | MEDIUM | The focusboard:// activation route is implemented in source (desktop-entry registration, warm socket forwarding, cold argv handling via PendingDeepLink, auto-verify without rendering the token) but no bound runtime receipt exercises an external activation, so follow-the-link return and expired/reused-token recovery stay unproven at runtime. |
| `find.golden-path-surfaces-missing` | MINOR | unlabelled | MEDIUM | The previously deferred surfaces are now implemented in source (FocusView/CalendarView/SettingsView, focus.rs, reminders.rs with a spawned scheduler, 27 IPC commands, no placeholder strings), but the issue cannot close because none of this is proven at runtime on the frozen candidate. |
| `find.isolated-store-probe-path-mismatch` | MINOR | INTERNAL | LOW | The advisory isolated_native_store check probes <profile>/data/com.focusboard.app/focusboard.sqlite3 while the product identifier is app.focusboard.desktop, so the check reports false in both loop-05 receipts regardless of whether SQLite persistence works; the advisory is inconclusive for this product, not a product failure. |

---

## Loop 04

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-04-8f9e83ef56cf` | `warm_start:loop-03-8f9e83ef56cf` | DEVELOPER ATTEMPT INCOMPLETE |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | golden_path |
| Objective | Make the release-critical PRD section 3 first-run path completable end to end: implement the Reminder entity with timezone-aware idempotent scheduling and mail-sink delivery, the FocusSession model with a full session lifecycle, and real Calendar, Focus, and Settings surfaces replacing the deferred placeholders, so the registered native replay can drive the whole path through live UI/IPC and recapture golden_path receipts that no longer report state unavailable. |
| Strategy | `golden-path-domain-implementation` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | INCOMPLETE ATTEMPT |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 0 |
| Changed paths | 0 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-fc58c07b072c16afce98603a` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | UNTESTED |

---

## Loop 03

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-03-8f9e83ef56cf` | `development_snapshot:attempt-9318dd41703dc74f2f2c8978` | PASS |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | golden_path |
| Objective | Make the PRD section 3 step-3 follow-the-link route literal: activating the emailed focusboard:// verification link on a stock desktop returns the user to the application with the single-use token consumed, while preserving the paste-the-link fallback, validated auth/persistence behavior, least-privilege security posture, and visual direction, so gap:prd_golden_path can be closed by the golden-path review. |
| Strategy | `deeplink-return-route-golden-path` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 12 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-9318dd41703dc74f2f2c8978` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-03-8f9e83ef56cf` |
| Product completion | IN PROGRESS |
| Review scope | `golden_path` |
| Findings | 6 (3 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `find.golden-path-surfaces-missing` | BLOCKER | USER BLOCKING | HIGH | The release-critical PRD section 3 path is incomplete: Focus, Calendar, and Settings are deferred placeholder surfaces and no Reminder entity or scheduler exists, so steps 6-8 and 10 (reminder with timezone, calendar placement, focus session, reminder email) cannot be performed. |
| `find.golden-path-not-driven-live` | MINOR | INTERNAL | MEDIUM | Still unverified through the live UI: all bound host receipts (replay, capture_evidence, and this loop's fresh preflight/prepare_test/tester_lifecycle) report golden_path.state=unavailable, so the completable portion of the first-run path was never driven through the live UI/IPC boundary. |
| `find.deeplink-runtime-unproven` | NOTE | DEGRADED | LOW | The focusboard:// scheme handler is now implemented in code (session desktop-entry registration, warm socket forwarding, cold argv handling, auto-verify without rendering the token), but no bound runtime receipt exercises an external activation, so the follow-the-link route stays unproven at runtime. |
| `find.rustfmt-debt-persists` | NOTE | INTERNAL | LOW | Rust sources remain not rustfmt-clean: cargo fmt --check still fails across all five Rust files, and the new deep-link code in lib.rs added further hunks to the diff. |
| `find.signedin-views-still-uncaptured` | MINOR | INTERNAL | MEDIUM | Bound pixel evidence still covers only the anonymous auth shell; the signed-in shell, Today/Inbox/Projects, deferred-surface empty states, and section 9.2 live states have no live captures. |
| `find.whitebox-lane-still-hijacked` | MINOR | INTERNAL | LOW | The registered focusboard-verify white-box lane is still unreachable: re-probed this loop, the entry point again dispatched into the native-webdriver argparse at import time. |

---

## Loop 02

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-02-fb2048172510` | `warm_start:loop-01-ead43b846561` | DEVELOPER ATTEMPT INCOMPLETE |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | golden_path |
| Objective | Make the PRD section 3 first-run golden path true in the product: register focusboard:// at the OS level so a verification email link followed by the user reaches Focusboard and its single-use token is consumed through the existing verify-continuation flow, with the paste-the-link fallback preserved, so the frozen prd_golden_path review can pass on live register-verify-signin-quickadd-restart evidence while the preserved baselines (tauri_source_manifest, native launch chain, white-box persistence tests, security posture) stay green. |
| Strategy | `deeplink-golden-path` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | INCOMPLETE ATTEMPT |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 0 |
| Changed paths | 0 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-49c4c294ff6fbe9b4d14dfe6` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | UNTESTED |

---

## Loop 01

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-01-ead43b846561` | `development_snapshot:attempt-6ea4842339edd60591614b94` | PHASE PASS / PRODUCT IN PROGRESS |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Turn the bootstrap scaffold into the first runnable Focusboard vertical slice: a native Tauri desktop app with durable account registration and email verification via a local mail sink, sign-in sessions that persist, SQLite-backed task quick-add and project creation, and the PRD section 9 visual direction on the auth shell, Inbox, Today, and Projects, so the prd_implementation_review gate has real candidate-bound runtime, interface, persistence, security, and visual evidence to evaluate. |
| Strategy | `first-run-foundation-slice` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | PASS |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 41 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-6ea4842339edd60591614b94` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | PASS |
| Candidate | `loop-01-ead43b846561` |
| Product completion | IN PROGRESS |
| Review scope | `phase` |
| Findings | 5 (0 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `find.first-run-gui-drive-missing` | MINOR | INTERNAL | MEDIUM | The release-critical register-verify-signin-quickadd-restart path was never driven through the live UI and IPC boundary; it is verified only at store/function level by independently re-run cargo tests plus native launch and validation evidence. |
| `find.whitebox-lane-import-hijack` | MINOR | INTERNAL | LOW | The registered focusboard-verify white-box lane is unreachable: a backward-compat shim hijacks execution at import time, so the entry point degenerates to the native-webdriver probe only. |
| `find.rustfmt-style-debt` | NOTE | INTERNAL | LOW | Rust sources are not rustfmt-clean: cargo fmt --check (pinned 1.90.0 toolchain) reports a 526-line diff; advisory style debt only. |
| `find.deeplink-scheme-not-registered` | NOTE | DEGRADED | LOW | The focusboard:// verification link has no OS-level scheme handler; verification completes through the designed paste-the-link fallback, deferring true follow-the-link to the delivery loop. |
| `find.signedin-views-no-live-capture` | MINOR | INTERNAL | MEDIUM | No live capture exists for the signed-in shell, the three implemented views, or the section 9.2 live states; the visual bar is runtime-verified only on the auth shell. |

---

## Current pointers

- Latest candidate: `loop-14-6b853ced8f4f`
- Accepted candidate: `none`
- Latest attempted: `loop-14-6b853ced8f4f`
- Latest warm start: `loop-14-6b853ced8f4f`
- Last published: `loop-13-6b853ced8f4f`
- Best verified: `loop-13-6b853ced8f4f`
- Generated by the HoH host from immutable run evidence.
