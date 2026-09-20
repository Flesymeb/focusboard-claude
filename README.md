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
| Open | 5 |
| Closed | 12 |
| All | 17 |

---

## Loop 17

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-17-cc5832463442` | `development_snapshot:attempt-51e2868241f1594e3fa28136` | NEEDS VISUAL QA |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | golden_path |
| Objective | Convert the blocked golden-path journey receipt into a mechanically capturable lane: build the candidate-side first-run journey measurement bridge (a declarative journey manifest plus opt-in isolated retrieval of delivered email action links), settle the three pinned evidence-stalled dispositions including a bounded waiver of the structurally impossible deeplink receipt, and keep every phase-gate asset green so the next host recapture can produce per-step journey evidence. |
| Strategy | `candidate-journey-measurement-bridge` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 3 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-51e2868241f1594e3fa28136` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | UNTESTED |

---

## Loop 16

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-16-c57aa7499336` | `development_snapshot:attempt-cddf766ed4a44fb3ba722751` | PASS |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | golden_path |
| Objective | Close gap:prd_golden_path by making the PRD section-3 journey externally measurable and discharging the three pinned evidence debts with explicit dispositions: rewrite the golden-path acceptance onto host WebKitWebDriver per-step typed-journey receipts, waive the absent external focusboard:// activation lane while preserving its environment-unavailable disposition, bind white-box evidence to the green host-run instruments, and keep the candidate journey-complete from a fresh opt-in isolated profile with no normal-startup test behavior. |
| Strategy | `external-lane-acceptance-rewrite` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 5 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-cddf766ed4a44fb3ba722751` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-16-c57aa7499336` |
| Product completion | IN PROGRESS |
| Review scope | `golden_path` |
| Findings | 5 (3 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `fnd.loop16.section3-external-journey-unproven` | MAJOR | INTERNAL | HIGH | The PRD section-3 external user-journey receipt remains unproducible: the host collector still has no typed-registration, email-link-follow, or sign-out/sign-in stage, so the golden path is proven white-box and advisory-black-box but never as an external journey. Blocked disposition — missing evidence, not an observed product defect. |
| `fnd.loop16.state-board-states-uncaptured` | MINOR | INTERNAL | MEDIUM | PRD 9.2 recovery and responsive states remain runtime-uncaptured on the loop-16 evidence: no network-error retry, narrow-viewport Today, or email-settings delivery-state capture among the 15 bound images. Evidence debt against the PRD quality bar, not an observed product failure. |
| `fnd.loop16.auth-recovery-journey-uncaptured` | MINOR | INTERNAL | MEDIUM | Forgot/reset password flows remain journey-uncaptured: the backend reset chain is unit-proven and the Forgot password? link renders, but no bound receipt drives forgot → email-link reset → sign-in through the real UI and no email action page is captured. |
| `fnd.loop16.deeplink-external-activation-lane-absent` | NOTE | INTERNAL | LOW | External focusboard:// activation stays receipt-unproven: both receipts mark external_deeplink not_run because no out-of-process activation stage exists in this environment; product-side scheme and token handling remain unit-proven. Blocked lane, tracked. |
| `fnd.loop16.advisory-whitebox-miscalibrated` | NOTE | INTERNAL | LOW | Advisory white-box lanes remain miscalibrated: source-contract blocks on the absent src-tauri/src/commands.rs and semantic-implementation reports 1/5 while the same receipt shows a real SQLite store, linked rusqlite, and 46/46 tests. Instrument failure mislabeled as candidate failure; non-gating, tracked. |

---

## Loop 15

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-15-0e8bf8de7271` | `development_snapshot:attempt-2b4c8f3bf37a3fc0e7c9b863` | PHASE PASS / PRODUCT IN PROGRESS |

### Project Planner

| Field | Value |
| --- | --- |
| Status | PLANNED |
| Focus | phase / prd_implementation |
| Objective | Close the last missing PRD-8 Settings capability (password change, session management, account-deletion request) as a working, security-preserving product increment, and add the opt-in, isolated candidate-side surfaces that make the Host's external lanes drivable: focusboard:// activations that increment the delivery counters, and verification/reset/reminder links readable from the local mail-sink outbox by an external typed journey. |
| Strategy | `settings-depth-plus-drivable-activation-surfaces` |
| Tasks | 1 |

### Developer

| Field | Value |
| --- | --- |
| Status | NEEDS VISUAL QA |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 11 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-2b4c8f3bf37a3fc0e7c9b863` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | PASS |
| Candidate | `loop-15-0e8bf8de7271` |
| Product completion | IN PROGRESS |
| Review scope | `phase` |
| Findings | 8 (0 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `fnd.settings-depth-verified-loop15` | NOTE | INTERNAL | LOW | Settings depth verified: password change, session management with revocation, and account-deletion request are implemented, registered, host-unit-proven, and the password plus sessions sections are captured signed-in in two independent runs; closure of fnd.prd-settings-depth-missing with a documented residual (delete-account section below the capture fold). |
| `fnd.deeplink-external-lane-unavailable-loop15` | MINOR | INTERNAL | MEDIUM | External focusboard:// activation remains receipt-unproven: the collector lane for out-of-process activation does not exist in this environment, so the decisive evidence cannot be produced; product-side scheme, opt-in counter, outbox, and token-parsing links all pass host-run unit tests. |
| `fnd.section3-external-journey-lane-missing-loop15` | MINOR | INTERNAL | MEDIUM | The PRD section-3 release-critical path still lacks an external user-journey receipt: the 16/16 self-drive remains explicitly untrusted authority, while host-collected navigation proves the signed-in shell but not typed registration, email-link follow, or sign-out/sign-in persistence; the journey lane is host-side and absent from this environment. |
| `fnd.auth-recovery-journey-uncaptured-loop15` | MINOR | INTERNAL | LOW | Forgot/reset password flows remain journey-uncaptured: only the Forgot password? link is visible in anonymous captures; the full backend chain is host-unit-proven but no bound receipt drives forgot, reset, and sign-in through the UI, and no email action page is captured. |
| `fnd.state-board-states-uncaptured-loop15` | MINOR | INTERNAL | LOW | PRD 9.2 recovery and responsive states remain runtime-uncaptured: all 13 bound captures across both runs are 1280x800 success or anonymous-auth states; no network-error retry, narrow-viewport Today, or email-settings detail capture exists, and the fixed collector scenario cannot produce them. |
| `fnd.advisory-whitebox-miscalibrated-loop15` | MINOR | INTERNAL | LOW | Advisory white-box lanes remain miscalibrated: source-contract blocks on an absent src-tauri/src/commands.rs and semantic-implementation reports 1/5 while the same receipt shows rusqlite linked, a real SQLite store created, and 45 host-run tests passing; host follow-up, non-gating. |
| `fnd.isolated-store-probe-pass-loop15` | NOTE | INTERNAL | LOW | Host isolated-store probe repaired: both loop-15 receipts report isolated_native_store pass on the identifier-derived path app.focusboard.desktop/focusboard.sqlite3; find.isolated-store-probe-path-mismatch is verified fixed and closed. |
| `fnd.hydration-calibration-pass-loop15` | NOTE | INTERNAL | LOW | Host hydration calibration repaired: all six signed-in surfaces in both receipts report clicked=true, correct active_nav, matched=true, hydrated=true; fnd.surface-hydration-marker-page-loop10 is verified fixed and closed. |

---

## Loop 14

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-14-74fe45da5118` | `development_snapshot:attempt-86608b1727319cf7f20a2c45` | FAIL |

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
| Status | PASS |
| Summary | Implementation details are retained in the private HoH Runtime audit. |
| Completed tasks | 1 |
| Changed paths | 15 |
| Blocker | See the QA outcome below. |
| Attempt | `attempt-86608b1727319cf7f20a2c45` |

### QA Tester

| Field | Value |
| --- | --- |
| Status | FAIL |
| Candidate | `loop-14-74fe45da5118` |
| Product completion | IN PROGRESS |
| Review scope | `phase` |
| Findings | 8 (5 blocking focus items) |

### Evidence / QA findings

| ID | Severity | Impact | Release risk | Summary |
| --- | --- | --- | --- | --- |
| `fnd.prd-settings-depth-still-missing` | MAJOR | USER BLOCKING | HIGH | Settings still offers only profile fields and reminder deliveries; PRD 8 requires password change, session management, and an account-deletion request, none of which exist in source or in this loop's signed-in runtime captures from either run. |
| `fnd.section3-external-journey-still-unproven` | MAJOR | INTERNAL | HIGH | The PRD section-3 release-critical path is still evidenced only by the product's renderer self-drive (16/16, explicitly untrusted authority); no bound receipt drives typed-form registration, email-link verification, focus lifecycle, or sign-out/sign-in persistence as an external user journey. |
| `fnd.deeplink-external-activation-still-unproven` | MAJOR | INTERNAL | HIGH | External focusboard:// activation is not_run in both loop-14 receipts (warm=0, cold=0, pending=false), so the reminder-email link journey remains receipt-unproven; scheme registration is proven only by the built-config unit test. |
| `fnd.state-board-states-still-uncaptured` | MINOR | INTERNAL | MEDIUM | PRD 9.2 recovery and responsive states remain runtime-uncaptured: all 13 bound captures across both runs are 1280x800 success or anonymous-auth states; no network-error retry, narrow-viewport Today, or email-settings detail capture exists. |
| `fnd.auth-recovery-journey-still-unproven` | MINOR | INTERNAL | MEDIUM | Forgot/reset password flows are implemented with host-run unit coverage, but no bound receipt drives forgot -> reset -> sign-in as an external UI journey; only the Forgot password? link is visible in the anonymous captures. |
| `fnd.host-isolated-store-foreign-path-persists` | NOTE | INTERNAL | LOW | Host collector's isolated_native_store check again reports false on the foreign com.focusboard.app data path while the candidate identifier is app.focusboard.desktop; the product-side identifier-derived data dir stays unit-proven. Host follow-up, measurement debt only. |
| `fnd.host-whitebox-advisory-stale-layout-persists` | NOTE | INTERNAL | LOW | Advisory white-box lanes remain miscalibrated to the single-crate layout: source-contract blocks on an absent src-tauri/src/commands.rs and semantic-implementation reports 1/5 while the crate links rusqlite, passes 39 host-run unit tests, and captures prove durable SQLite storage. Host follow-up. |
| `fnd.host-hydration-marker-miscalibration-persists` | NOTE | INTERNAL | LOW | Host hydration calibration still reports active_nav='page', matched=false, hydrated=false for all six signed-in surfaces while every bound capture shows the correct heading and highlighted nav item; navigation itself is proven (clicked=true, correct headings). Host follow-up. |

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

- Latest candidate: `loop-17-cc5832463442`
- Accepted candidate: `none`
- Latest attempted: `loop-17-cc5832463442`
- Latest warm start: `loop-17-c57aa7499336`
- Last published: `loop-17-cc5832463442`
- Best verified: `loop-16-c57aa7499336`
- Generated by the HoH host from immutable run evidence.
