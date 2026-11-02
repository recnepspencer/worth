# Query 9.19 Phase 5: Native Source preparation groups plan

Status: implementation plan, before Rust edits. Independent reviewer accepted the
internal extraction as a coherent prerequisite. Boundary:
[milestone-9.19-phase5-source-groups-boundary-review.md](milestone-9.19-phase5-source-groups-boundary-review.md).

## Intended result

Native Source routing, authored history and pre-invariant staging consume exact
installed Source metadata without carrying invariant callbacks or descriptors.
Existing public declarations/builders and runtime behavior remain identical.
This is preparation for public-only maintenance, not that capability itself.

## Shape and ordered implementation

1. Add `validation/candidate_source/preparation_group.rs`, an immutable private
   group of exact programs, access contract and maximum work metadata. Its sole
   constructor derives from a real invariant registration; no public caller can
   invent a group, access audience or work limit.
2. Freeze one group roster inside `FrozenCustomInvariantRegistry`. Filter only
   Source-empty rules while preserving the relative order of every nonempty
   group. Compile Source postings and the historical catalog from that roster;
   make postings resolve group slots. Rule inventory/index/get/iter is unchanged.
3. Cut both actual Source preparation receivers over to groups. Change only the
   metadata type/accessors/names. Preserve their existing program planning loop,
   first affected owner, sibling scope union, work meter lease, participant and
   completeness checks. The historical catalog deduplicates the same exact
   program identities in the same encounter order. No detached fallback remains.
4. Review complete diff independently for ordering, exact program affinity,
   access/work custody and composition. Run focused actual courts and owner
   tests. Do not create implementation-mirroring tests for the extraction; the
   existing real Source/history/lifecycle and accounting courts are stronger.
5. Save measured evidence and independent qa-loop/qa-tests/code-quality-qa gate;
   commit this slice. Phase 5/6 stay unfinished.

## Evidence and compile discipline

First run rustfmt on touched Rust, dirty line caps and function scrutiny, strict
UTF8/C0/added-mojibake and `git diff --check`. Use configured Rust/sccache and
`C:/worth-build-cache/target`, original profiles/features, runtime stack 67108864.

Build the existing external Native target once:
`cargo test -p worth-court-relational-certification --test worth_court_relational_certification --no-run`.
Run its real `relational::managed_source` group (13 baseline cases), plus existing
`source_preparation_custom_and_native_work_share_one_original_invoice` and
`source_preparation_distinguishes_installed_allowance_and_inherited_parent_cancellation`
accounting courts. The latter distinguish original invoice/installed allowance
and inherited cancellation; retain their actual original controls.

Build `cargo test -p worth-relational --lib --no-run` once. Run actual inventoried
`validation::custom_registry::tests`, `candidate_source::program::bindings::tests`
and authored-write-history tests affected by the roster/catalog handoff. Existing
external fixtures contain single-program groups, so they do not independently
prove multi-program or shared-program first-owner behavior: those loop algorithms
must remain unchanged and receive static preservation review. Add a genuine
compact court only if implementation changes that algorithm or exposes missing
behavior evidence. No Bank/Query/50k rebuild for this private-only extraction.

Run the required actual Cargo boundary-check and agent-context commands and final
cached rescans. Review dirty caps/scrutiny/text again after evidence edits.

## Deferred obligations

Public Source declaration/policy, v1-v4 portable byte preservation and malformed
new versions; explicit runtime Source registration and receiving-registry
validation; builder/install/fork/recovery/codec transition and genuine roster
receipt authority; Bank pinned compatibility and real adoption; public topology
box/contact adoption, lifecycle/scale qualification; stable facades/guidance;
single final private Phase 6 cutover. This slice supplies none of those claims.
