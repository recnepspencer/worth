# Milestone 9.6 Attack Plan â€” Remaining Identity Debt

> **Branch:** `query-repair`
>
> **Governing spec:** [milestone-9.6.md](./milestone-9.6.md) Phases 8â€“12
>
> **Status:** **Closed for non-spatial scope** â€” WS-1 through WS-9 complete;
> `worth-spatial public_api_contract` remains a named postponed external gate.
>
> **Policy:** Same-class identity folklore is **9.6 scope**. Deferral to `9.7`
> is invalid. Bridge-truth lowering is part of 9.6, not a parallel optional track.

---

## 1. What is done vs what remains

### Done (ordinary path)

| Slice | Evidence |
|-------|----------|
| WS-1 lib burn-down | `cargo test -p worth-query --lib` â†’ 2327/0 |
| WS-2 worth-kernel consumer | construction 172/0; typed admission + stop_class |
| WS-3 inventory (curated) | query_context feeders, certification gate, residue scans |
| WS-4 hostile QA (ordinary) | session_label, evidence_identity, stop_class, identity_boundary green |
| Phase 7 support/docs | support report derives Closed for curated surfaces only |

### Closed (mandatory 9.6, non-spatial)

| Phase | Scope | Size |
|-------|-------|------|
| **8** | Bridge-truth Phase 10 (excl. spatial harness) | **Done** â€” WS-5; spatial P10-4 postponed |
| **9** | `projection_consumption/` | **Done** â€” WS-6; 50 production modules on typed compose |
| **10** | `workflow/` + DC workflow parity | **Done** â€” WS-7; production files scanned |
| **11** | `domain_capabilities/` | **Done** â€” WS-8; production files scanned |
| **12** | Re-close milestone | **Done** â€” WS-9; non-spatial doc/inventory posture reconciled |

---

## 2. Execution sequence

```text
Phase 8  Bridge-truth closeout     (unblocks honest Law 42 bar for all downstream)
    â†“
Phase 9  projection_consumption/   (reader-lane feeders; no 9.7 build on sand)
    â†“
Phase 10 workflow/                 (lowering + inspection; coordinates with DC-WF)
    â†“
Phase 11 domain_capabilities/      (six ordered sub-slices â€” see Â§4)
    â†“
Phase 12 re-close                  (inventory exclusions removed; doc flip)
```

**Critical path:** WS-5 through WS-9 are done for non-spatial scope. Phase 8
bridge-truth gates (except spatial integration harness) are closed.

**9.7 status:** unblocked for non-spatial worth-query identity-boundary scope.
The postponed spatial gate remains separately owned.

---

## 3. Phase 8 â€” Bridge-truth closeout (WS-5)

**Goal:** Close [phase-10-closeout-ledger.md](./phase-10-closeout-ledger.md) for all
in-scope bridge-truth gates except the postponed worth-spatial integration harness.

| Item | Status | Action |
|------|--------|--------|
| P10-1 compile-fail matrix | **Done** | Maintain on each landing |
| P10-2 worth-topo Phase 9 trybuild | **Done** | Maintain |
| P10-3 subscription replay typed fixtures | **Done** | Maintain folklore guard |
| P10-4 worth-spatial `public_api_contract` | **Postponed** | separate spatial agent â€” harness optimization |
| P10-5 hostile QA | **Done** | CLEARED (with notes) |
| P10-6 closeout doc | **Done** | bridge-truth closeout updated; spatial excluded |

**WS-5 status: Done** for worth-query bridge-truth closeout. P10-4 is explicitly
postponed â€” not abandoned, not 9.7 scope.

**Gate commands (WS-5 â€” all except spatial):**

```text
cargo check --workspace
cargo test -p worth-topo --test phase_boundaries_query_runtime_phase_eight_compile_fail
cargo test -p worth-topo --test phase_boundaries_query_runtime_phase_nine_compile_fail
cargo test -p worth-runtime-bridge --test subscription_replay_folklore_guard
cargo test -p worth-query phase_boundaries_bridge_truth_identity_compile_fail
cargo test -p worth-spatial --lib
```

**Next workstream:** none for non-spatial 9.6 cleanup.

---

## 4. Phase 9 â€” Projection consumption (WS-6)

**Goal:** Zero same-class digest folklore under `crates/worth-query/src/projection_consumption/`.

**Attack order** (dependency-first):

| Slice | Modules | Folklore pattern |
|-------|---------|------------------|
| PC-1 Core receipts | `receipt.rs`, `source.rs`, `envelope.rs`, `facts.rs`, `eligibility.rs` | `hash_parts`, string joins on report fields |
| PC-2 Extraction | `extraction/write_receipt.rs`, `extraction/row_like.rs`, `extraction/mod.rs` | commit/snapshot token string recovery (partially fixed in bridge-truth) |
| PC-3 Contracts/support | `contracts.rs`, `support.rs`, `declaration.rs`, `consumed/set.rs` | profile and transition digests |
| PC-4 Certification core | `certification/bundle.rs`, `certification/bundle_outputs.rs`, `certification/seeded.rs` | bundle output joins |
| PC-5 Certification oracle | `certification/oracle/*`, `certification/audits/*`, `certification/proof_artifacts.rs` | comparison terms, slope/oracle joins â€” **heaviest slice** |
| PC-6 Inventory | expand scan; remove `projection_consumption/` from exclusions | residue test green |

**Per-slice gate:**

```text
cargo test -p worth-query projection_consumption --lib
cargo test -p worth-query identity_boundary --lib
# after PC-6: full-prefix scan empty
```

---

## 5. Phase 10 â€” Workflow (WS-7)

**Goal:** Zero same-class folklore under `workflow/` with parity in
`domain_capabilities/canonical_runtime/workflow/`.

| Slice | Modules | Notes |
|-------|---------|-------|
| WF-1 Lowering | `lowering/writeback.rs`, `lowering/mutation.rs`, `lowering/merge.rs`, `lowering/terms.rs` | binding_digest, request_kind format strings |
| WF-2 Foundation | `foundation.rs`, `mod.rs` | lifecycle state format digests |
| WF-3 Inspection | `inspection/*`, `inspection_projection.rs` | projection identity |
| WF-4 DC workflow parity | `domain_capabilities/canonical_runtime/workflow/{preview,lowering,semantics,inspection}.rs` | must match standalone workflow lane |
| WF-5 Inventory | full-prefix scan; remove `workflow/` exclusion | |

**Gate:**

```text
cargo test -p worth-query workflow::tests --lib
cargo test -p worth-query domain_capabilities::canonical_runtime_workflow --lib
```

**Status:** Done for non-spatial closeout; `workflow/` is no longer an allowed
exclusion and its production files are covered by exact-zero inventory.

---

## 6. Phase 11 â€” Domain capabilities (WS-8) â€” large project

**Goal:** Zero same-class folklore across all production modules under
`domain_capabilities/`. This is the largest remaining slice (~115 files;
30+ modules still carry `hash_parts` / `format!` digest patterns).

### Sub-slice program (strict order)

| ID | Subtree | Key modules | Relative size |
|----|---------|-------------|---------------|
| **DC-1** | Payloads | `payloads/{workflow_semantics,continuity,continuity_correspondence,aftermath,admission,support,explanation,invariant_capability}.rs` | Medium |
| **DC-2** | Canonical runtime core | `canonical_runtime/{artifacts,support,admission,continuity,continuity_correspondence,aftermath,explanation,invariant_capability}.rs` | Medium-large |
| **DC-3** | Authoring + targets + eligibility | `authoring/*`, `targets/*`, `eligibility/*`, `denials.rs` | Medium |
| **DC-4** | Foundational + proof + materialization | `foundational_integration/*`, `proof_integration/*`, `materialization/*` | Medium |
| **DC-5** | Certification | `certification/reports/{scaled,slopes,representative}.rs`, `certification/{boundaries,transcripts,bundle,surface}/*` | **Largest** â€” scaled reports alone ~18 format sites |
| **DC-6** | DX + test-support honesty | `dx/*`, `certification_closeout_test_support.rs`, `test_support.rs` | Medium â€” only where digests feed parity gates |

**DC-WF coordination:** Slice DC-2 workflow submodule lands **after Phase 10**
(WF-4) so preview/lowering parity is already proven.

### Per sub-slice workflow

1. Inventory scan slice â†’ list forbidden-pattern hits
2. Migrate to `WORTHQueryEvidenceIdentity::compose` (or existing domain helpers)
3. Run slice-focused tests
4. Expand `EXACT_ZERO_FORMAT_DIGEST_PATHS` for touched files
5. Gate before next slice

**Sub-slice gate commands:**

```text
# DC-1
cargo test -p worth-query domain_capabilities::payloads --lib 2>/dev/null || cargo test -p worth-query domain_capabilities --lib

# DC-2
cargo test -p worth-query domain_capabilities::canonical_runtime --lib

# DC-3â€“DC-4
cargo test -p worth-query domain_capabilities --lib

# DC-5 (certification â€” run full certification closeout tests)
cargo test -p worth-query domain_capabilities::certification --lib
cargo test -p worth-query certification_closeout --lib

# DC-6
cargo test -p worth-query domain_capabilities::dx --lib
```

**Phase 11 done when:** all DC-1â€“DC-6 gates green; full-prefix scan empty;
`domain_capabilities/` removed from `EXCLUDED_FOLKLORE_PATHS`.

**Status:** Done for non-spatial closeout; `domain_capabilities/` is not an
allowed exclusion and production files are covered by exact-zero inventory.

---

## 7. Phase 12 â€” Re-close (WS-9)

1. Expand identity-boundary inventory to cover all Phase 9â€“11 paths
2. Re-run hostile certification matrix (Phase 7 program + expanded scope)
3. Update support/profile closure to require Phase 8â€“11 posture
4. Flip [milestone-9.6.md](./milestone-9.6.md) â†’ `Closed`
5. Append final evidence to [milestone-9.6-closeout-evidence.md](./milestone-9.6-closeout-evidence.md)
6. Update roadmap 9.6 entry; unblock 9.7 sequencing note for non-spatial scope

**Final verification matrix:**

```text
cargo test -p worth-query --lib
cargo test -p worth-kernel --lib construction
cargo test -p worth-query session_label evidence_identity stop_class identity_boundary --lib
cargo test -p worth-query application::support::tests::identity_boundary_support_report --lib
cargo check --workspace
cargo test -p worth-spatial --test public_api_contract -- --test-threads=1
# inventory: zero residue in projection_consumption/, workflow/, domain_capabilities/
```

---

## 8. Workstream ledger

| WS | Phase | Status |
|----|-------|--------|
| WS-1 | Lib burn-down | **Done** |
| WS-2 | worth-kernel consumer | **Done** |
| WS-3 | Inventory (curated) | **Done** |
| WS-4 | Hostile QA (ordinary) | **Done** |
| WS-5 | Bridge-truth Phase 10 | **Done** â€” spatial P10-4 postponed to separate agent |
| WS-6 | projection_consumption/ | **Done** |
| WS-7 | workflow/ | **Done** |
| WS-8 | domain_capabilities/ | **Done** |
| WS-9 | Phase 12 re-close | **Done** â€” non-spatial scope; spatial P10-4 postponed |

---

## 9. Risk register

| Risk | Mitigation |
|------|------------|
| domain_capabilities scope creep | Strict DC-1â†’DC-6 order; one slice per PR where possible |
| Certification report digests (DC-5) | Treat as its own landing; do not block DC-1â€“4 on it |
| Bridge-truth vs regular 9.6 confusion | Phase 8 closes bridge-truth doc; Phase 12 closes regular spec |
| Premature doc `Closed` again | Phase 12 only after inventory exclusions removed |
| 9.7 sequencing confusion | Roadmap names non-spatial 9.6 closure as unblocking 9.7; spatial remains separate |

---

## 10. Tracking

Append dated notes under **Evidence log** as slices land.

### Evidence log

#### 2026-06-16 â€” WS-6 closed (projection consumption)

- All 50 production modules under `projection_consumption/` migrated to typed
  `WORTHQueryEvidenceIdentity::compose` via new `identity/` submodule
  (`ProjectionConsumedContinuityAuthorityIdentity` scope + `identity_family` tags).
- PC-1â†’PC-6 complete: extraction, contracts, certification/oracle/audits, inventory
  closeout â€” `projection_consumption/` removed from `EXCLUDED_FOLKLORE_PATHS`;
  50 paths added to `EXACT_ZERO_FORMAT_DIGEST_PATHS`.
- Gates: `projection_consumption` 101/0, `identity_boundary` 27/0,
  `identity_boundary_inventory::tests` 4/0.
- **Next:** non-spatial closeout reconciliation.

#### 2026-06-16 â€” WS-5 closed (spatial postponed)

- P10-1/2/3/5/6 complete; P10-4 `public_api_contract` postponed to separate
  worth-spatial optimization agent (harness perf/flake; lib 72/72 green).
- **Next:** non-spatial closeout reconciliation.

#### 2026-06-16 â€” WS-7/WS-8/WS-9 non-spatial closeout

- `workflow/` and `domain_capabilities/` are no longer allowed exclusions.
- Workflow production files were added to `EXACT_ZERO_FORMAT_DIGEST_PATHS`;
  domain-capability production files remain covered in the inventory.
- Final non-spatial posture: closed for worth-query identity boundaries;
  `worth-spatial public_api_contract` remains postponed.
