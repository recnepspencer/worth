# Milestone 9.6 Closeout Evidence

> **Branch:** `query-repair`
>
> **Date:** 2026-06-16
>
> **Verdict:** **CLOSED for non-spatial 9.6 identity-boundary scope** â€”
> `worth-spatial public_api_contract` postponed as a named external gate.

## Before / after (lib regression burn-down)

| Gate | Before (2026-06-16 QA re-open) | After (WS-1â€“WS-4 closeout) |
|------|--------------------------------|----------------------------|
| `cargo test -p worth-query --lib` | 2277 pass / **47 fail** | **2327 pass / 0 fail** |
| `cargo test -p worth-kernel --lib construction` | folklore on basis/stop-class paths | **172 pass / 0 fail** |

## WS-3 inventory hardening (evidence)

- Expanded `EXACT_ZERO_FORMAT_DIGEST_PATHS` with `query_context/basis.rs` and crate-root `preview/mod.rs`.
- Migrated remaining `hash_parts` / `terminal_projection_for_reporting()` folklore on those surfaces to typed `WORTHQueryEvidenceIdentity` composition (`preview/workflow_context_identity.rs`).
- Added `EXCLUDED_FOLKLORE_DEFERRALS` with named owner milestones for each `EXCLUDED_FOLKLORE_PATHS` prefix.
- Added `identity_boundary_certification_gate.rs` with `MILESTONE_9_6_CERTIFICATION_GATE_PATHS` and `MILESTONE_9_6_LIB_CERTIFICATION_GREEN`; support `Closed` now requires certification gate certified.
- Widened `EXACT_ZERO_STRING_MATCHING_PATHS` to worth-kernel construction consumer tests (`branch_preview_basis.rs`, `construction.rs`).

## Verification matrix (2026-06-16)

```text
cargo test -p worth-query session_label --lib          â†’ 21/21
cargo test -p worth-query evidence_identity --lib      â†’ 19/19
cargo test -p worth-query stop_class --lib             â†’ 22/22
cargo test -p worth-query identity_boundary --lib      â†’ 27/27
cargo test -p worth-query --lib --test-threads=2       â†’ 2327/0
cargo test -p worth-kernel --lib construction          â†’ 172/0
cargo check -p worth-query --lib                       â†’ ok
cargo test -p worth-query application::support::identity_boundary_inventory::tests --lib â†’ 4/4
cargo test -p worth-query application::support::tests::identity_boundary_support_report --lib â†’ 4/4
cargo test -p worth-query public_doc_coverage::tests::identity_boundary_docs --lib â†’ 1/1
```

`cargo fmt --check --all` reports pre-existing drift outside worth-query (worth-topo); worth-query sources were formatted with `cargo fmt -p worth-query`.

## Hostile QA (Closure Gate Â§1â€“5)

| Check | Result |
|-------|--------|
| Covered worth-query surfaces + new feeders (`query_context/basis.rs`, `preview/mod.rs`) | No same-class digest folklore in inventory scan |
| Support `Closed` derives from residue + hostile matrix + lib certification gate | Proven by `identity_boundary_support_report` tests |
| worth-kernel construction basis/stop-class consumer paths | Typed `admission_identity()` + `stop_class()` matching; string-matching scan clean |
| Named exclusions | Documented in `EXCLUDED_FOLKLORE_DEFERRALS` (9.7/9.8 owners) |

## Mandatory scope closeout (Phases 8â€“11 â€” Milestone 9.6, not 9.7)

| Prefix | Phase | Attack plan |
|--------|-------|-------------|
| Bridge-truth Phase 10 | 8 | WS-5 **done** â€” spatial P10-4 postponed |
| `projection_consumption/` | 9 | WS-6 **done** â€” removed from exclusions; 50 modules in inventory scan |
| `workflow/` | 10 | WS-7 **done** â€” removed from exclusions; production files in inventory scan |
| `domain_capabilities/` | 11 | WS-8 **done** â€” production files in inventory scan |

**Invalid deferrals:** assigning worth-query integration dirs to Milestone 9.7 is
prohibited. **worth-spatial `public_api_contract`** is the one valid postponement
â€” separate agent, harness optimization, not part of this non-spatial closeout.

## Former deferrals (still out of scope â€” different milestone class)

| Prefix | Owner |
|--------|-------|
| `harness/milestone_nine_five_` | Milestone 9.5 harness-only |
| `runtime/intent/declaration.rs` | Milestone 9.8 intent declaration lowering |

worth-kernel `prepare_branch_basis_digest` still composes typed admission digests via `digest_owned_parts` for test parity labels â€” scoped follow-on for 9.7/9.8 prep per closeout plan risk register, not ordinary-path production in worth-query.
