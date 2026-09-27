# Milestone 9.6 Closeout Plan â€” Product Boundary Debt Closure

> **Branch:** `query-repair`
>
> **Governing spec:** [milestone-9.6.md](./milestone-9.6.md)
>
> **Status:** **Closed for non-spatial 9.6 identity-boundary scope** â€” WS-1â€“WS-9
> closed; `worth-spatial public_api_contract` remains a named postponed external
> gate. See [milestone-9.6-attack-plan.md](./milestone-9.6-attack-plan.md).
>
> **Bridge-truth:** In scope for Milestone 9.6 Phase 8 â€” closed for all
> non-spatial gates and not deferrable to 9.7.

---

## 1. Closeout definition

Milestone 9.6 closes when Query is the **ordinary-path owner** of three machine
boundaries â€” with **no surviving same-class folklore** in worth-query or the
primary downstream consumer (`worth-kernel` construction):

| Boundary | Ordinary-path contract |
|----------|------------------------|
| Evidence identity | Sealed `WORTHQueryEvidenceIdentity::compose`; no format-string / pipe-join digest assembly on covered surfaces and feeders |
| Stop-class matching | `WORTHQueryRuntimeError::stop_class()`; no message grep control flow on covered denial paths |
| Session label identity | `WORTHQuerySessionLabel` intake on preview/branch; typed collision posture |

**Honest closure requires all of:**

1. Every covered surface and same-class upstream/adjacent feeder uses typed boundaries (spec Closure Gate Â§1â€“2).
2. Support/profile derives `Closed` only from proof â€” not from a curated scan alone (Â§3).
3. Sibling boundaries architecturally consistent â€” no mixed model (Â§4).
4. Exclusions named with owner milestone; no hiding same-class debt (Â§5).
5. Acceptance evidence in spec Â§682â€“710 â€” especially full `cargo test -p worth-query --lib` green and worth-kernel consumer migration.

---

## 2. Current state (2026-06-16 QA)

### Green (worth-query certification lanes)

| Gate | Result |
|------|--------|
| `cargo test -p worth-query session_label --lib` | **21/21** (fixed: basis_admission tag `basis_evidence`) |
| `cargo test -p worth-query evidence_identity --lib` | 19/19 |
| `cargo test -p worth-query stop_class --lib` | 22/22 |
| `cargo test -p worth-query identity_boundary --lib` | 24/24 |

### Red

| Gate | Result |
|------|--------|
| `cargo test -p worth-query --lib` | **2277 pass / 47 fail** |
| worth-kernel construction consumer digests | Still `digest_owned_parts` + pipe-join (`branch_basis_digest.rs`) |
| worth-kernel denial tests | Still `error.to_string().contains(...)` (`construction.rs`) |
| Inventory honesty | Curated path allowlist + explicit exclusions; `query_context/basis.rs`, crate-root `preview/mod.rs` not scanned |

### Root cause

Bridge-truth identity lowering on `query-repair` changed production identity
composition (relational record authority, snapshot adapter requirement, evidence
field tags / projections) without completing downstream test and consumer
migration. Regular 9.6 **runtime core** is largely landed; **certification,
fixtures, inventory scope, and worth-kernel** are not.

---

## 3. Workstreams

### WS-1 â€” Lib test regression burn-down (47 failures)

**Goal:** `cargo test -p worth-query --lib` green.

Fix in dependency order (each slice lands with focused test run before next):

| Slice | Tests | Root cause | Primary touch |
|-------|-------|------------|---------------|
| **WS-1a** | 9 | Assembly builders missing `snapshot_identity` adapter â†’ `MissingSnapshotIdentityAdapter` | `runtime/tests/assembly/builder.rs`, support_profile + invariant_registration tests |
| **WS-1b** | 3 | Intent receipt manual composition helpers stale vs production encoder | `runtime/tests/intent_receipt_*_identity_composition*.rs` |
| **WS-1c** | 15 | Graph composition fixtures lack `relational_record_parts()` on entity endpoints | `runtime/tests/mutation/graph_composition*`, test entity mint helpers |
| **WS-1d** | 5 | Projection consumption fixtures: `SourceIdentityMismatch` (evidence-identity vs relational-commit labels) | `projection_consumption/tests/`, certification fixtures |
| **WS-1e** | 8 | Mutation continuity / naming / binding / batch â€” digest or entity identity drift | `runtime/tests/mutation/{continuity,existing_binding,naming,verified_update_existing}/` |
| **WS-1f** | 5 | Preview / query_context / workflow foundation basis identity | `preview/tests`, `query_context/scoped_tests`, `domain_capabilities/.../workflow_runtime_preflight_tests` |
| **WS-1g** | 4 | Declaration entry seam, harness workflow cert, view_shape_live grouped baseline | per failing module |
| **WS-1h** | 3 | Read composition temporal_async, shared_read pending state | `runtime/tests/read_composition/`, `runtime/tests/shared_read/` |

**Gate:** `cargo test -p worth-query --lib`

**Done when:** zero failures; no `@ignore` or suite narrowing.

---

### WS-2 â€” Primary consumer migration (worth-kernel)

**Goal:** Eliminate the folklore patterns 9.6 was written to kill in the
motivating downstream consumer.

| Item | Current defect | Target |
|------|----------------|--------|
| Basis admission parity digests | `digest_owned_parts([label, effect_policy, authority_lane, evidence.join("\|")])` in `construction/tests/support/branch_basis_digest.rs` | Compare against Query `basis_admission().admission_identity()` or recompose via public evidence encoder with same fields as production |
| Branch/preview digest chain | String labels from `preview_basis.label().to_string()` | Use `label_identity()` / typed admission identity |
| Denial matching | `construction.rs`: `error.to_string().contains(...)` | `stop_class()` typed matching with family payload |
| Residual construction digests | Widespread `digest_owned_parts` / `format!("{:?}")` in construction support | Triage: **in-scope** if feeding 9.6 boundaries (basis admission, report parity, certification comparison); **out-of-scope** only with named owner + spec note |

**Gate:**

```text
cargo test -p worth-kernel --lib construction
cargo test -p worth-kernel construction::tests::construction --lib
```

**Done when:** branch/preview basis digest tests use Query-owned identity; at least one consumer-shaped test proves stop-class matching without string ops on covered denial paths (mirrors worth-query Phase 4 bar).

---

### WS-3 â€” Inventory and closure-honesty hardening

**Goal:** Support report `Closed` means what the spec says.

1. **Expand scan coverage** â€” add to `EXACT_ZERO_FORMAT_DIGEST_PATHS` or a new feeder scan:
   - `query_context/basis.rs`
   - `preview/mod.rs` (crate root; distinct from `runtime/preview/mod.rs`)
   - Any path WS-1 touches that still carries `hash_parts` on ordinary surfaces

2. **Review exclusions** â€” same-class worth-query integration prefixes
   (`projection_consumption/`, `workflow/`, `domain_capabilities/`) are not
   excludable and must stay in exact-zero inventory. Only genuinely different
   milestone-class paths may remain excluded.

3. **Tie closure posture to certification** â€” extend `WORTHQueryIdentityBoundaryClosure::derived()` or add closeout gate test:
   - `Closed` requires targeted 9.6 suites green **and** zero lib failures **or** explicit documented deferral list empty.
   - Prevents scan-green / test-red overclaim.

4. **String-matching scan** â€” widen beyond single file `runtime/tests/stop_class/consumer_support/routing.rs` to worth-kernel construction tests once migrated.

**Gate:**

```text
cargo test -p worth-query identity_boundary --lib
cargo test -p worth-query application::support::tests::identity_boundary_support_report --lib
```

**Done when:** inventory covers feeders identified in QA; support report cannot read `Closed` while lib suite is red.

---

### WS-4 â€” Hostile re-QA and doc closeout

**Goal:** Replace stale 2026-06-10 closeout note with evidence-backed closure.

1. Re-run hostile QA prompt (spec Phase 7 + Closure Gate) â€” code inspection, not tests alone.
2. Update [milestone-9.6.md](./milestone-9.6.md):
   - Status â†’ `Closed` only after WS-1â€“WS-3 gates pass
   - Closeout note with date, branch, gate command outputs
3. Update [worth_query_roadmap.md](./worth_query_roadmap.md) Milestone 9.6 entry if posture changed during re-open.
4. Append evidence paths under `plans/worth-query/goal_mode_*` or new `milestone-9.6-closeout-evidence.md` (gate logs, failure count before/after).

**Gate:** Verdict `CLEARED` with numbered evidence; no `NOT CLEARED` blockers.

---

## 4. Execution sequence

```text
Phase A  WS-1a â†’ WS-1b          (mechanical + composition helpers; ~12 tests)
Phase B  WS-1c                  (graph composition fixture authority; ~15 tests)
Phase C  WS-1d                  (projection consumption fixtures; ~5 tests)
Phase D  WS-1e â†’ WS-1h          (remaining lib failures; ~15 tests)
Phase E  WS-2                   (worth-kernel consumer; parallel after WS-1a if desired)
Phase F  WS-3                   (inventory + closure honesty; after lib green)
Phase G  WS-4                   (hostile QA + doc closeout)
```

**Critical path:** WS-1 â†’ WS-3 â†’ WS-4. WS-2 can run in parallel with WS-1câ€“D but **must complete before WS-4** (spec requires consumer boundary closure).

**Do not:** fold bridge-truth Phase 10 items (worth-spatial, compile-fail matrix expansion) into this plan â€” track separately.

---

## 5. Verification matrix (final closeout)

All must pass on `query-repair` immediately before doc flip to `Closed`:

```text
# Milestone 9.6 named suites
cargo test -p worth-query session_label --lib
cargo test -p worth-query evidence_identity --lib
cargo test -p worth-query stop_class --lib
cargo test -p worth-query identity_boundary --lib

# Full lib (acceptance evidence)
cargo test -p worth-query --lib

# Consumer boundary
cargo test -p worth-kernel --lib construction

# Workspace sanity
cargo check -p worth-query --lib
cargo fmt --check --all
```

Optional regression guard (recommended once WS-3 lands):

```text
cargo test -p worth-query application::support::identity_boundary_inventory::tests --lib
cargo test -p worth-query public_doc_coverage::tests::identity_boundary_docs --lib
```

---

## 6. Risk register

| Risk | Mitigation |
|------|------------|
| Bridge lowering continues to churn identity composition | Freeze bridge-truth production changes in 9.6-owned files until WS-1 green; batch composition updates |
| Graph composition fixes require production changes not just fixtures | Triage per test: if production correctly enforces relational authority, fix fixtures only |
| worth-kernel scope creep (100+ `digest_owned_parts`) | WS-2 scopes to **construction basis/stop-class paths** from spec; broader digest migration â†’ named follow-on (9.7/9.8 prep) |
| Inventory exclusions used to avoid work | WS-3 requires explicit owner milestone per exclusion or removal |
| Doc re-closed without lib green | WS-3 gate ties support `Closed` to certification; WS-4 blocked on full matrix |

---

## 7. Closeout checklist

- [x] WS-1: `cargo test -p worth-query --lib` â€” 0 failures (2327/0)
- [x] WS-2: worth-kernel construction basis/stop-class folklore removed on covered paths
- [x] WS-3: inventory expanded (curated); exclusions documented
- [x] WS-4: hostile QA `CLEARED` (ordinary path)
- [x] `milestone-9.6.md` closeout note updated (ordinary path)
- [x] Roadmap 9.6 entry consistent (non-spatial identity boundaries closed)
- [x] WS-5: bridge-truth Phase 10 (P10-1/2/3/5/6 done; P10-4 spatial **postponed**)
- [x] WS-6: `projection_consumption/` identity closure (Phase 9) â€” 101/101 tests; inventory scan clean
- [x] WS-7: `workflow/` identity closure (Phase 10)
- [x] WS-8: `domain_capabilities/` identity closure (Phase 11)
- [x] WS-9: Phase 12 re-close â€” non-spatial milestone closed, exclusions removed
- [x] 9.7 unblocked per roadmap sequencing notes for non-spatial scope

---

## 8. Tracking

Use this doc as the ledger. Append dated notes under **Evidence log** as slices
land (test counts, commands, PR/commit refs).

### Evidence log

#### 2026-06-16 â€” WS-3/WS-4 closeout (query-repair)

- Hostile QA: **CLEARED**
- Lib: **2327/0**; named 9.6 suites: session_label 21/21, evidence_identity 19/19,
  stop_class 22/22, identity_boundary 27/27
- worth-kernel construction: **172/0**
- Inventory: added `query_context/basis.rs`, `preview/mod.rs`; certification gate;
  `EXCLUDED_FOLKLORE_DEFERRALS` with owner milestones
- Evidence: [milestone-9.6-closeout-evidence.md](./milestone-9.6-closeout-evidence.md)

#### 2026-06-16 â€” QA re-open

- Hostile QA: **NOT CLEARED** on `query-repair`
- Lib: 2277 pass / 47 fail (after session_label fix: 21/21 green)
- Session label fix: `basis_evidence` tag in test helper (`runtime/tests/session_label.rs`)
