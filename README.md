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
| Open | 8 |
| Closed | 0 |
| All | 8 |

---

## Loop 06

> A host-derived record of one Planner → Developer → QA Tester cycle.

| Candidate | Base | Current state |
| --- | --- | --- |
| `loop-06-8da584af31e8` | `development_snapshot:attempt-73a120af5cdb7d6fa28f5c85` | NEEDS VISUAL QA |

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
| Status | UNTESTED |

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

- Latest candidate: `loop-06-8da584af31e8`
- Accepted candidate: `none`
- Latest attempted: `loop-06-8da584af31e8`
- Latest warm start: `loop-06-f4298baf5f14`
- Last published: `loop-06-8da584af31e8`
- Best verified: `loop-05-f4298baf5f14`
- Generated by the HoH host from immutable run evidence.
