# Milestone 20 Engineering Spec: The Bridge Owns The Relational Adapter

> **Status:** Planned.
>
> **Roadmap parent:** [WORTH_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Supersedes:** the adapter-direction rule in [Milestone 1 §5.8](./milestone-1.md#58-bridge-owned-adapter-traits), for Relational only.
>
> **Relational counterpart:** [WORTH_relational_roadmap.md](../WORTH-relational/WORTH_relational_roadmap.md)
>
> **Purpose:** reverse the dependency between `worth-relational` and `worth-runtime-bridge`. The Bridge exists to link Relational and Signal, so the Bridge depends on both, and neither runtime knows the Bridge exists.

## Goal

When this milestone closes:

- `worth-runtime-bridge` depends on `worth-relational` and `worth-signal`.
- `worth-relational` has no dependency on `worth-runtime-bridge`: not
  normal, dev, or build. Its source, tests, docs, and facade never name the
  Bridge.
- Relational exposes a Bridge-free **change source** API: committed changes,
  exact reads at an observation, branch heads, lineage, and commit selection.
  Relational mints every authority-bearing value in that API.
- The Bridge owns everything that translates Relational into Bridge terms:
  `Truth*Identity` minting, committed-patch envelopes, snapshot read packets,
  error mapping, its source trait implementations, the identity binding
  tables, and grouped truth projection.
- The boundary checker denies `worth-relational -> worth-runtime-bridge` and
  allows the reverse.

Signal already has this shape. Signal never names the Bridge, and hosts
implement the Bridge's `InvalidationSink`. This milestone gives Relational the
same shape.

## Why This Milestone Exists

The Bridge was designed as the part that links truth and computation. Its
roadmap says:

- "The bridge's only job is to wire the runtimes together."
- The Bridge "must not define truth semantics that belong to
  `worth-relational`."

The code went the other way. The edge was added on 2026-04-05, in the same
commit that created the Bridge (`5ae3ca1e4d`, "starting on runtime bridge").
[Milestone 1 §5.8](./milestone-1.md#58-bridge-owned-adapter-traits) then
codified it:

- the Bridge owns narrow adapter traits;
- the parent runtimes implement them.

That rule was meant to keep the Bridge off wide parent facades. It had the
opposite effect: Relational took on the Bridge's vocabulary.

- About 7,300 lines in `crates/worth-relational/src/presentation/bridge/**`
  and `crates/worth-relational/src/grouped_truth/**` are written in Bridge
  types. 45 files are adapter code; 4 files are grouped truth projection.
- Relational's facade publishes `worth_relational::facade::bridge`
  (`crates/worth-relational/src/facade.rs:64-65`) and grouped truth exports
  (`facade.rs:18-27`).
- The boundary tooling locks the inversion in.
  `tools/boundary-check/config/road1.toml:107-112` denies exactly the target
  edge, `worth-runtime-bridge -> worth-relational`.
- The Bridge knows Relational by name anyway, without depending on it.
  - `crates/worth-runtime-bridge/src/relational_identity.rs` (330 lines)
    defines `RelationalBridgeRecordIdentityParts`,
    `RelationalBridgeSnapshotIdentityParts`, `from_relational_*`
    constructors, and Relational identity prefixes.
  - `RelationalBridgeSourceError` and `RelationalCommittedPatchRequest` are
    Bridge types.
  - So the Bridge already encodes Relational's shape. It just cannot see
    Relational's types to encode it honestly.

The cost is concrete:

- **The public constructors are the weak point.** Relational is a different
  crate, so every Bridge constructor it calls must be public. Examples:
  - `BridgeCommittedPatchEnvelope::new_with_authoritative_lowering`
    (`src/input/envelope/canonical.rs:212`);
  - `BridgeCommittedPatchItem::with_relational_semantic_change`
    (`src/relational_identity.rs:279`);
  - `BridgeAuthoritativeSourceProvenance::from_owner_partition_publication`
    (`src/input/envelope/source_authority.rs:71`);
  - `BridgeProducerMetadata::registered_authoritative_source`
    (`src/input/envelope/core.rs:35`);
  - `BridgeHistoricalLineageAuthority::try_new`
    (`src/adapter/continuity_lineage.rs:68`).

  Relational is the only non-test caller of most of them. Once the adapter
  lives in the Bridge, they can be narrowed: crate-private where only the
  adapter calls them, and feature-gated where tests and fakes still need them
  (D11).
- **Relational semantics use Bridge error types.**
  `presentation/bridge/patch_semantic_validation.rs` (391 lines) decides
  Relational meaning, but reports it as `BridgeRouteError`.
- **Consumers learn the wrong model.** Query, Runtime World, and UI import
  `worth_relational::facade::bridge::RuntimeBridgeRelationalSource`. That
  teaches every reader that the Bridge is a feature of Relational.
- **The public docs describe the edge as unintended.** `README.md` and
  `docs/how-it-works.md` §2.2 already call it "the reverse of the intended
  direction".

## Governing Summaries

- `MENTALITY.md`: fix the foundation before more is built on it. Query,
  Runtime World, and UI already sit on this edge; each milestone that passes
  adds to what must move.
- `arch_laws.md`:
  - The authority that decides a meaning mints its proof. Relational decides
    what a committed change is, so Relational mints the change receipt.
  - The Bridge decides what a committed-patch envelope is, so the Bridge mints
    the envelope.
  - Neither mints the other's proof.
- `composition_laws.md`: the adapter already has separate homes for
  committed patches, snapshot reads, branch heads, lineage, observation
  bindings, and publication. Keep those homes when the code moves; do not
  merge them into one `relational.rs`.
- `domain_structure_laws.md`: a type lives with the authority that owns its
  meaning.
  - Relational owns commit, change, observation, record, and lineage meaning.
  - The Bridge owns routing, envelope, identity-correspondence, and
    grouped-projection meaning.
- `AGENTS.md` placement rule:
  - The binding tables that map Bridge snapshot identities to retained
    Relational observations are live tables, so they belong to a runtime.
  - They bind a Bridge identity, so the runtime is the Bridge.
  - The retention lease they hold is Relational's.
- No wrapper, alias, or compatibility surface. The flip removes
  `worth_relational::facade::bridge` outright. Every downstream import changes
  in the same commit. No re-export stays behind "for migration".

## Current Shape (Evidence)

All paths are relative to the repository root. `RA` means
`crates/worth-relational/src/presentation/bridge`.

### Bridge traits Relational implements today

| Trait (defined in the Bridge) | Definition | Relational implementation |
|---|---|---|
| `CommittedPatchSource` | `crates/worth-runtime-bridge/src/adapter/truth_sources.rs:4-15` | `RA/runtime_source/committed_patches.rs:126-176` |
| `SnapshotReadSource` | `truth_sources.rs:17-22` | `RA/runtime_source/snapshot_reads.rs:8-25` |
| `TruthBranchHeadSource` | `truth_sources.rs:33-38` | `RA/runtime_source/branch_heads.rs:8-40` |
| `RelationalBridgeSource` (blanket) | `truth_sources.rs:40-48` | blanket |
| `TruthSnapshotReader` | `crates/worth-runtime-bridge/src/snapshot/context.rs:5-12` | `RA/snapshot_reading.rs:39-54`, `RA/runtime_source/retained_snapshot.rs:47-58` |
| `ContinuityLineageSource` | `crates/worth-runtime-bridge/src/adapter/continuity_lineage.rs:14-19` | `RA/runtime_source/continuity_lineage.rs:17-26` |
| `GroupedProjectionMemberSource`, `GroupedProjectionSource` | `crates/worth-runtime-bridge/src/source/grouped_contract.rs:80-94` | `crates/worth-relational/src/grouped_truth/grouped_projection.rs:66-149` |

Many Query, UI, and Bridge test fixtures implement these traits as well, so
they stay public Bridge traits after the flip. Examples:

- `workspaces/worth-query/crates/worth-query/src/harness/fixtures/preview_bridge.rs`
- `workspaces/worth-ui/crates/worth-ui-query-binding/src/scalar_projection_async_fixture.rs`

### Relational internals the adapter reaches

The adapter uses Relational's `pub(crate)` internals. Moving it into the
Bridge therefore needs a public Relational API first. The internals it
reaches:

- `RelationalVisibilityRuntimeAuthority` (`crates/worth-relational/src/visibility/runtime_authority.rs:6-35`)
- `RelationalRuntime::runtime_instance_id` (`runtime/state/runtime_state/branch_authority.rs:91`)
- visibility `allocate_snapshot_id` (`runtime/state/subsystems/visibility/mod.rs:94`)
- commit ancestry: `inspect_commit_ancestry`, `classify_commit_in_ancestry`,
  `CommitAncestryPosture` (`history/access/ancestry.rs:35-110`)
- commit surfaces: `commit_envelope` through `CommitEnvelopeSource`,
  `canonical_stream_position`, `next_commit_id`
  (`capabilities/history/commit_envelopes.rs:6`, `history/access/commit_surfaces.rs:134-217`)
- lineage: `lineage_access`, `resolve_record_history_for_observation`,
  `visible_entity_ids_for_lineages_for_observation`
- exact-state record reads (`visibility/materialization/read_records/reader/truth_record_access.rs:4-22`)
- observation `selected_root` / `admitted_basis` (`mvcc/observation.rs:28-101`)
- schema aspect plans (`branch/root.rs:279`, `branch/root_schema.rs:91-159`)
- `PublishedAuthoritativePatchEnvelope::from_canonical` and the full-grammar
  operation accessors (`publication/patch/data/**`)
- `identity_authority::relational_source_truth_authority` and
  `RelationalSourceTruthAuthorityIdentity`
- the publication proof chain in `RA/authoritative_publication_witness.rs`.
  It has private `AuthorityMarker` and `CapabilityMarker` structs over
  `worth_proof::Recipe`, and mints `RelationalBridgePatchPublication`
  (`RA/publication_outcome.rs:82-95`).
- the Bridge-only performance counter
  `count_bridge_observation_commit_selection`
  (`performance/access/bridge_counters.rs:4`), and the public counter fields
  named after it, `bridge_observation_commit_{selections,ancestry_visits}`
  (`runtime_complexity_counters.rs:133-134`)
- the runtime handle. `RuntimeBridgeRelationalSource` holds the crate-private
  enum `RelationalVisibilityRuntimeAuthority` (`Immutable(Arc)` or
  `Shared(Arc<Mutex>)`) and reaches it through the crate-private `with_runtime`
  (`RA/runtime_source/mod.rs:20-36`). `branch_basis.rs`,
  `committed_patches.rs`, `continuity_lineage.rs`,
  `retained_entity_projection.rs`, and `snapshot_reading.rs` all go through
  it.
- counter reads for the cost tests. `branch_head_selection_cost.rs:37-81` and
  `fork_ancestry_selection.rs:83` read counters through the crate-private
  `performance_access()`. `.counters()` and `reset_counters` exist only under
  `#[cfg(test)]`, and `inspect_commit_ancestry(..).total_work()` is
  crate-private. The only public counter read is
  `CommitResult::complexity_delta`.
- an inherent impl. `RA/authoritative_patch_publication.rs:33` is
  `impl RelationalRuntime`, including the public
  `admit_opaque_aspect_bridge_widening`. Rust does not allow an inherent impl
  in another crate, so this cannot move as it is.

These are already public and need no new API:

- `observe_branch`, `readmit_branch_basis`, `retain_component_basis`;
- `read_truth`, `history`;
- the observation identity, version, commit, canonical commit, and canonical
  patch accessors;
- `RelationalBranchRetentionLease::release`;
- `runtime_instance_id` on the basis and the branch identity.

### Downstream users of `worth_relational::facade::bridge`

| Crate | Files |
|---|---|
| `worth-query-execution` | 13 |
| `worth-query` (engine) | 12 |
| `worth-runtime-world` | 8, including `examples/runtime_world_publication/correspondence.rs` and `tests/runtime_world_certification/court/correspondence.rs` |
| `worth-ui-query-binding` | 1 |

These counts come from a multi-line search, because several files use brace
imports:

```text
rg -U -l 'worth_relational::facade::(\{[^}]*)?bridge|RuntimeBridgeRelationalSource|RelationalBridge(ObservationLease|BranchHeadLease|ObservationReleaseReceipt|SourceConfigurationError)'
```

The names used are `RuntimeBridgeRelationalSource`,
`RelationalBridgeObservationLease`, `RelationalBridgeBranchHeadLease`,
`RelationalBridgeObservationReleaseReceipt`, and
`RelationalBridgeSourceConfigurationError`. Representative call sites:

- `worth-query-execution/src/execution_runtime/product_world/source_owner.rs:28`
- `worth-runtime-world/src/lifecycle/owner/owned_async_product.rs:102`

No code outside Relational uses any other `facade::bridge` export. The
publication outcomes, the widening admission, and
`bridge_snapshot_identity_for_*` are used only inside Relational.

Grouped truth is used by 16 source files, all in `worth-query`, for example:

- `projection_consumption/extraction/mod.rs:18`
- `source/constructors.rs:10`

It is also named in
`docs/domain-capabilities/declaration-relational-truth-routing.md:147-149`.

**Some paths are strings, and some of those strings feed digests.**

- `application/declaration_relational_routing/lower.rs:31` and `:41` name
  Relational facade paths as strings.
- `basis_lifecycle/reuse.rs:257-267` holds `"worth-relational"` and
  `"worth_relational::facade::RuntimeBridgeRelationalSource"`. That path is
  already stale. These strings are hashed into `row_digest` and
  `matrix_digest` (`reuse.rs:49-60`, `142-147`), so rewriting them changes
  those digests.

**Query's public enums name the source.**
`WorthQueryDeclarationRelational{AuthorityFamily,TruthClaim,Binding}` have
`GroupedTruth` and `BridgeSource` variants
(`declaration_relational_routing/contract.rs:23-57`, `artifact.rs:26-38`),
exported at `exports_application.rs:116`. After the flip, both surfaces live
in the Bridge. D9 decides what happens to the names.

`worth-server` and `worth-ui-certification` will reach Relational
transitively through the Bridge after the flip. No rule forbids that.

### Bridge constructors called from outside the Bridge

Relational is not the only outside caller:

- `with_authoritative_source` and `from_owner_publication`:
  `worth-query/src/domain_installation/dependency_impact/compiled/invalidation_manifest/structural_manifest_tests.rs:192-193`.
- `BridgeHistoricalLineageAuthority::try_new`:
  `worth-query/src/correspondence/bridge_lowering_tests.rs:31,65`, and
  `.../phase_six/readmission_support/source.rs:190`. That file is a fake of
  `ContinuityLineageSource`, a trait D5 keeps public, so fakes must still be
  able to build one.
- `tests/ui/committed_patch_item_literal_private.stderr` names these
  constructors.

### Cycle check

`worth-proof`, `worth-foundational`, `worth-signal`, and `worth-harness`
depend on neither the Bridge nor Relational. Once Relational stops depending
on the Bridge, the edge `Bridge -> Relational + Signal` creates no cycle.

Cargo rejects a two-way edge, so the manifest swap must land in one commit.
By then, Relational code must already be free of Bridge types.

A Relational dev-dependency on the Bridge is not an escape hatch:

- It would compile two copies of Relational's types into Relational's unit
  tests.
- `tools/boundary-check/src/configured_dependency_denials.rs:13-23` counts
  every dependency kind.

Bridge-using Relational tests therefore move into the Bridge.

## Target Shape

```text
worth-proof
  └── worth-foundational
        ├── worth-signal ─────────────┐
        └── worth-relational ─────────┤
                                      └── worth-runtime-bridge
                                            └── worth-runtime-world
                                                  └── Query, UI, Server
```

### Relational: the change source API (Bridge-free)

Relational adds one facade module, provisionally
`worth_relational::facade::change_source`. It is written only in Relational
and Foundational vocabulary. The final names are settled in Phase 3, with the
[dx laws](../../docs/coding-guidelines/dx_laws.md) review. The exports live
in a `#[path]` submodule from the start, because Relational's `facade.rs` is
already 380 lines. Planned contents:

0. **Runtime handle.** A public handle over a Relational runtime that covers
   both of today's sharing modes, `Immutable(Arc)` and `Shared(Arc<Mutex>)`.
   It states its locking rules and exposes the runtime instance id as the
   `u64` every public Relational accessor already uses; a newtype on the
   handle alone would give one value two vocabularies. It
   replaces the adapter's use of the crate-private
   `RelationalVisibilityRuntimeAuthority`, `with_runtime`, and
   `runtime_instance_id`. It needs a live runtime, so by the three-question
   rule it stays in Relational.
1. **Committed change publication.** Given a retained observation, with an
   optional partition, Relational returns a typed outcome.
   - On success, the outcome carries a **Relational-minted receipt**. Relational
     mints it through the existing proof chain in
     `authoritative_publication_witness.rs`, which moves into this module.
   - The receipt carries only Relational meaning:
     - the canonical `PublishedAuthoritativePatchEnvelope`;
     - the commit, version, branch, and snapshot ids;
     - the runtime instance id;
     - the partition;
     - the Relational parts of the source basis;
     - the source-truth authority identity.
   - The provenance `source_basis` string interleaves Relational fields with
     the graph role and truth partition, so the Bridge assembles it, byte for
     byte, from the receipt fields and its own concepts.
   - It does not carry the graph role, `TruthPartitionRole`, or the adapter
     semantic identity (`relational_bridge_adapter_semantic_identity()`).
     Those are Bridge concepts. The Bridge owns them and supplies them when
     it lowers the receipt.
   - The receipt's rules:
     - it is `#[must_use]`;
     - the Bridge can read it but cannot construct it;
     - Bridge lowering consumes it by value;
     - a committed-patch envelope with Relational provenance can be minted
       only from a receipt.
   - The outcome keeps today's `TransitionOutcome` families: `Stale`,
     `RebindRequired`, `Deferred`, `Denied`, and `Failed`.
2. **Change consistency.** The Relational self-consistency checks in
   `patch_semantic_validation.rs` move here and report a Relational denial
   enum. They decide Relational meaning:
   - declared aspects;
   - field grammar;
   - lifecycle and endpoint consistency.

   They run inside receipt minting, so holding a receipt proves they passed.
   The rest of that file is Bridge meaning and moves to the Bridge (D2).
3. **No widening admission in Relational.**
   `RelationalOpaqueAspectWideningAdmission` is used only in tests, and the
   precision loss it records happens in Bridge lowering. The Bridge mints its
   own widening admission from its Relational source, scoped to a runtime and
   a graph role exactly as today, and
   `admit_opaque_aspect_bridge_widening` is deleted from
   `impl RelationalRuntime`. Minting it from a receipt would narrow it to one
   commit and reorder every call site, which is a change in meaning.
4. **Exact reads at an observation.** Aspect values, lifecycle, relation
   endpoints, and the declared-aspect check for one record, at an exact
   retained observation.
5. **Lineage at an observation.** Record history and the visible entities
   for a set of lineages, resolved at an exact observation.
6. **Commit selection at an observation.** Ancestry classification and
   selection. The call returns a typed work receipt (commits selected,
   ancestry visits), so callers and cost tests can assert on the work without
   crate-private counters. `count_bridge_observation_commit_selection` and the
   `bridge_observation_commit_{selections,ancestry_visits}` fields get neutral
   names.
7. **Snapshot allocation.** Retain a basis and allocate its snapshot id as
   one operation that returns a Relational lease. Relational never sees a
   Bridge snapshot identity.

Every item needs a clock, a counter, a live table, or `Drop` (the runtime
handle, the retention leases), or decides Relational legality. So every item stays in Relational;
none belongs in Foundational or Proof. If an item turns out to mean the same
thing in another runtime, move it to `worth-foundational` instead, and record
that decision in the Phase 3 closeout.

### Bridge: the Relational adapter

The adapter moves to one Bridge module,
`crates/worth-runtime-bridge/src/relational_source/`, and its public items
are exported flat from `worth_runtime_bridge::facade`.

The Bridge facade has hidden `everyday`, `advanced`, and `specialist` modules
that re-export everything with `pub use super::*`. A new top-level
`pub mod relational` would get three alias paths automatically, and the
no-alias rule forbids that. The Phase 3 DX review settled on the flat export,
the same shape Phase 2 gave grouped truth: it adds no module path, and every
exported name already says `Relational`. The existing export files are near
the cap (`facade/exports_core.rs` is 375 lines), so the new exports get their
own `#[path]` export file, `facade/exports_relational.rs`.

The adapter keeps its existing homes:

- `committed_patches`
- `snapshot_reads`
- `branch_heads`
- `continuity_lineage`
- `observation_bindings`
- `branch_head_bindings`
- `retained_snapshot`
- `patch_envelopes`
- `snapshot_values`
- `source_profile`

It then:

- implements `CommittedPatchSource`, `SnapshotReadSource`,
  `TruthBranchHeadSource`, `TruthSnapshotReader`, and
  `ContinuityLineageSource` for `RuntimeBridgeRelationalSource` over the
  change source API;
- owns the observation and branch-head binding tables, which map Bridge
  snapshot identities to Relational leases;
- lowers a Relational change receipt into a `BridgeCommittedPatchEnvelope`;
- maps Relational denials into `BridgeRouteError` /
  `RelationalBridgeSourceError`;
- owns the opaque-widening gate and the lowering counters
  (`patch_semantic_validation.rs:85-97` and `:345-383`, which fill
  `BridgeAuthoritativePatchLoweringCounters`);
- reproduces the provenance strings byte for byte
  (`authoritative_patch_publication.rs:228-245`);
- absorbs `relational_identity.rs`. The `from_relational_*` constructors keep
  their primitive `u32`/`u64` parameters. Callers reach them through
  `worth_query_host::facade::primary_graph`, and about 40 files use them, so
  retyping them would change the host facade. Adding typed versions next to
  the primitive ones would create a parallel lane, so there are none.

`RuntimeBridgeRelationalSource` keeps its name, because it names what it is
in Bridge terms. Only its path changes:

- before: `worth_relational::facade::bridge::RuntimeBridgeRelationalSource`
- after: `worth_runtime_bridge::facade::RuntimeBridgeRelationalSource`

The same applies to the lease, receipt, and configuration-error types.

### Bridge: grouped truth projection

`crates/worth-relational/src/grouped_truth/**` has 4 files and 886 lines.

- It is a pure transform from Bridge snapshot packets into row sets and
  grouped projections. It touches no runtime.
- Its only Relational internal is `aspect_wire::encode_u32`, which is
  `u32::to_le_bytes`.

It moves into the Bridge **as its own module**, next to the Bridge's grouped
truth, not merged into it. The two are layers, not duplicates:

- The Bridge already has `row_set.rs`, `row_set/`, `grouped_truth_view.rs`,
  `packet_set_digest_basis.rs`, and `grouped_contract.rs`. Its digest type
  is `BridgeMaterializedRowSetDigest`; Relational's is
  `RelationalRowSetDigest`.
- Relational's `grouped_projection.rs:66,127` implements the Bridge's
  `GroupedProjectionSource`, so one layer sits on the other.
- Both crates define `GroupedProjectionContract`, with different fields
  (`grouped_projection.rs:13`, `grouped_contract.rs:44`). Query already
  renames the Relational one on import
  (`milestone_eight_certification/mod.rs:52`,
  `grouped_truth_projection.rs:6`).

Merging them because they look alike would give one name two meanings and
would change digests. So the moved contract and digest get names distinct
from the Bridge's, settled in the Phase 2 DX review. Parity means the same
function produces the same bytes before and after the move.

## Decisions

| # | Decision | Rejected alternative | Why |
|---|---|---|---|
| D1 | Relational mints the change receipt; the Bridge mints the envelope | Bridge mints both from raw Relational data | The authority that decides a meaning mints its proof. A Bridge-minted "committed change" would be a second truth authority. |
| D2 | Split `patch_semantic_validation.rs`. The consistency checks stay in Relational and run inside receipt minting. The opaque-widening gate and the lowering counters move to the Bridge. | Move the whole file to one side | The file mixes meanings. The consistency checks decide Relational meaning, and the Bridge roadmap forbids the Bridge from defining truth semantics. The widening gate and `BridgeAuthoritativePatchLoweringCounters` describe Bridge lowering. |
| D3 | The Bridge owns the identity binding tables | Relational keeps them behind an opaque key | They bind Bridge identities. Relational must not learn what a Bridge snapshot identity is. |
| D4 | Grouped truth moves to the Bridge as its own module, with contract and digest names distinct from the Bridge's | Fold it into `grouped_truth_view`; or rewrite it over Relational read types and keep it in Relational | Its inputs and outputs are Bridge packets and identities, and its only consumer is Query, which already depends on the Bridge. It is a layer over the Bridge's grouped truth, not a copy of it, so folding would merge two meanings. |
| D5 | The Bridge's source traits stay public | Make them `pub(crate)` once Relational stops implementing them | More than 20 Query and UI test fixtures implement them as fakes. |
| D6 | No re-export of the old path | Leave `worth_relational::facade::bridge` re-exporting for a while | That would be a compatibility surface, which the constitution forbids. It would also need a Relational dependency on the Bridge, which is the edge being removed. |
| D7 | Relational's Bridge-using tests move into the Bridge | Keep them in Relational behind a dev-dependency | A dev-dependency would create duplicate types, and the boundary checker counts it. |
| D8 | Milestone 1 §5.8 is superseded for Relational only | Rewrite §5.8 wholesale | Signal already has the target shape. §5.8's intent, keeping the Bridge off wide facades, still holds; the Bridge consumes only the narrow change source module. |
| D9 | Query's `GroupedTruth` and `BridgeSource` variants keep their names. Their rustdoc states that "relational" names the truth source, not the owning crate. | Rename the variants in this milestone | The variants classify where truth comes from, and that is still Relational. Renaming would change the host facade and the declaration digests with no change in meaning. For the same reason `WorthQueryDeclarationEntryLowerOwnerCrate::WorthRelational` stays on every `RelationalTruthRouting` row, grouped truth included, and its rustdoc says it names the truth authority. Phase 2 checks `declaration_entry_seam/digest.rs` to confirm that no declaration digest changes beyond D12. |
| D10 | The `basis_lifecycle/reuse.rs` matrix digest changes, and the change is accepted | Keep the stale path string to preserve the digest | The string is already wrong (`worth_relational::facade::RuntimeBridgeRelationalSource`). Keeping a wrong path to protect a digest would make the digest certify a falsehood. |
| D11 | Constructors that only the adapter calls become `pub(crate)`. Constructors that Query tests or public-trait fakes need go behind the `certification-construction` feature. | Make all of them `pub(crate)` | Query tests and `ContinuityLineageSource` fakes build these values today. The feature already gates `pub mod certification` (`lib.rs:73-74`). |
| D12 | The relational routing digest of `GroupedTruth` and `BridgeSource` rows changes, and so does the declaration-entry inspection digest that folds it in (`relational:{routing_digest}`). The change is accepted. | Keep the old binding strings in `declaration_relational_routing/lower.rs` to preserve the digests | The strings name where the bound type lives, and `lower.rs:31` and `:41` name paths that this milestone deletes. As with D10, a digest that certifies a deleted path certifies a falsehood. For the common-lane `GroupedTruth` route over `edge:42`, the routing digest moves from `36e3b23db193c977071144a65974dab7ab44a5930c087ae0ec2769ffdc7fac65` to `f5c2328a993b679c819074ccef1e9851fa3a64596051a4bf526c59ebe8d8733d`. `:41` changes in Phase 4, when its path moves. |

## Phases

Each phase leaves the workspace green:

- the boundary check and its tests;
- the agent-context check;
- the line caps;
- fmt and clippy for the touched crates;
- the owner tests for every touched crate.

Phases 0 to 3 change no manifests.

### Phase 0: Bridge housekeeping (completed)

- Move `worth-harness` from `[dependencies]` to `[dev-dependencies]` in
  `crates/worth-runtime-bridge/Cargo.toml`. It is used only by the
  `#[cfg(test)] mod harness` (`src/lib.rs:102-103`).
- Update the `worth-runtime-bridge` row in `docs/how-it-works.md` §2.2.

**Exit:** `cargo test -p worth-runtime-bridge` passes, and
`cargo tree -p worth-runtime-bridge -e normal` does not list `worth-harness`.

### Phase 1: Move the Bridge-only compile-fail tests

- `crates/worth-relational/tests/ui/phase_2a/*` holds four trybuild cases and
  their `.stderr` files. They test only Bridge `from_relational_*`
  constructors. Move them into `crates/worth-runtime-bridge/tests/ui/`.
- Move the driver lines from
  `crates/worth-relational/tests/phase_boundaries_compile_fail.rs:13-17` into
  the Bridge's `tests/phase_boundaries_compile_fail.rs`.
- Move `tests/ui/branch_reference/raw_commit_cannot_publish_bridge.rs` and
  its `.stderr` (driver `branch_reference_compile_time.rs`) the same way. If
  the case proves a Relational rule rather than a Bridge one, rewrite it in
  Relational terms instead, and record which in the closeout.
- Regenerate the `.stderr` files with the Bridge's trybuild workflow; never
  hand-edit them.

**Exit:** both crates' compile-fail suites pass, and Relational's `tests/ui`
tree no longer names the Bridge.

### Phase 2: Move grouped truth into the Bridge

- Move `grouped_truth/{canonical_digest,grouped_projection,row_set,snapshot_aspect_reads}.rs`
  into their own Bridge module beside `source/`, not into
  `grouped_truth_view`. Inline `encode_u32` as `to_le_bytes`.
- Give the moved `GroupedProjectionContract` and `RelationalRowSetDigest`
  names distinct from the Bridge's (D4), and drop Query's import renames.
- Export grouped truth from the Bridge facade. Delete
  `crates/worth-relational/src/facade.rs:18-27` and the `grouped_truth`
  module declaration in `crates/worth-relational/src/lib.rs`.
- Update the 16 `worth-query` source files, the string paths in
  `application/declaration_relational_routing/lower.rs:31` and `:41`, and
  `docs/domain-capabilities/declaration-relational-truth-routing.md:147-149`.
- Retarget `crates/worth-relational/tests/bridge_identity_boundary.rs` so it
  stops scanning `src/grouped_truth`. Move the grouped-truth half of that
  scan into the Bridge.
- Add the D9 rustdoc to the `GroupedTruth` and `BridgeSource` variants.
  Confirm with `declaration_entry_seam/digest.rs` that no declaration digest
  changes.

**Exit:**

- Canonical digests are byte-identical before and after. Add a digest parity
  test that runs the same function on the pre-move commit and the post-move
  commit, and record both results.
- The `worth-query` owner tests pass.

### Phase 3: Build the change source API and move the adapter onto it, inside Relational

- Add `worth_relational::facade::change_source` with items 0 to 7 above.
- Move the publication proof chain into it.
- Split `patch_semantic_validation.rs` (D2). The consistency checks move into
  receipt minting and report Relational denials. The widening gate and the
  lowering counters stay in `RA/**`, ready to move with the adapter.
- Delete `admit_opaque_aspect_bridge_widening` from `impl RelationalRuntime`.
  The widening admission becomes Bridge-side, inside `RA/**`: the Relational
  source mints it for its own runtime and graph role, and publication checks
  both exactly as today.
- Rename `count_bridge_observation_commit_selection` and the
  `bridge_observation_commit_*` counter fields to neutral names, and return
  the typed work receipt from commit selection (item 6).
- Rewrite the adapter in `RA/**` so it uses only these:
  - the Relational facade, the new module included;
  - `worth_runtime_bridge::facade::*`;
  - `worth_foundational::facade::*` and `worth_proof::*`.

  Inside Relational the facade is spelled `crate::facade`, so the rule is:
  no `crate::` path other than `crate::facade::*` and
  `crate::presentation::bridge::*`.
- Prove the adapter is ready to move with a **probe crate**, deleted in
  Phase 4. It is a scratch crate outside the workspace members that depends
  on Relational and the Bridge and includes the `RA/**` files through
  `#[path]`. It compiles only if the adapter uses nothing crate-private. A
  text search cannot prove this, because it cannot see calls to
  crate-private methods such as `with_runtime`.
- Split `crates/worth-relational/src/tests/transactions/core/struct_field_patch_authority.rs:119-138`.
  The Relational half asserts on the change receipt; the envelope half is
  marked for the move.
- Split `crates/worth-relational/tests/relational_certification/basis/read_cutover.rs`
  the same way. Lines 18-99 use the Bridge; lines 101-184 are
  Relational-only. The moved half imports the Relational fixture
  `super::world::supply_chain`. Rebuild what it needs on public Relational API
  in the moved support module, rather than depending on Relational test code.
- Rewrite the `crate::tests::support` helpers the adapter tests use (about 40
  uses) on public Relational API, in a support module that will move with
  them.
  `RelationalVisibilityRuntimeAuthority::immutable` is the only crate-private
  helper; replace it with the public constructor path.

**Exit:**

- The probe crate compiles.
- Every existing adapter test passes unchanged in meaning.
- The DX review of the change source names, and of the Bridge-side facade
  path, is recorded in the phase closeout.
- The closeout names a destination for every remaining `RA/**` file.
- The change source module's public items have rustdoc. Relational has no
  entries in `tools/boundary-check/snapshots/facades.toml` today, and this
  milestone adds none. The Bridge entries added in Phase 4 are snapshotted.

### Phase 4: The flip (one commit)

1. Move the rest of `RA/**` into
   `crates/worth-runtime-bridge/src/relational_source/`, following the
   destinations in the Phase 3 closeout. Phase 3 already moved the proof
   chain and the consistency checks out and deleted the inherent impl. The
   destination list must cover the files the adapter homes above do not name:
   `identities`, `partition_projection`, `publication_outcome`,
   `authoritative_patch_publication`, `snapshot_reading`, `branch_basis`,
   `selected_commit_resolution`, and `retained_entity_projection`. Move the
   adapter tests and their support with it. Move `test_catalog.rs` into
   Bridge test support.
2. Absorb `crates/worth-runtime-bridge/src/relational_identity.rs` into the
   new module.
3. Delete from Relational:
   - `crates/worth-relational/src/facade/bridge.rs`;
   - `pub mod bridge` in `facade.rs:64-65`;
   - `presentation::bridge`;
   - the Phase 3 probe crate;
   - the rest of `tests/bridge_identity_boundary.rs`. Its scan of
     `src/presentation/bridge` moves into the Bridge.
4. Manifests:
   - remove `worth-runtime-bridge` from `crates/worth-relational/Cargo.toml`;
   - add `worth-relational` to `crates/worth-runtime-bridge/Cargo.toml`;
   - update every tracked lock file that contains `worth-relational`, found
     with `git ls-files '*Cargo.lock' | xargs grep -l 'name = "worth-relational"'`.
     Today that is eight files: the root, `workspaces/worth-query`,
     `workspaces/worth-ui`, `workspaces/worth-query-bank-world`,
     `workspaces/worth-store`, the `consumer_entry` fixture, and the
     `compile_contracts` and `runtime_effect_adapter` fixtures. CI runs
     `compile_contracts` with `--locked`
     (`scripts/ci/run_worth_ui_compile_contracts.py`), so a stale copy fails
     the build.
5. `tools/boundary-check/config/road1.toml:107-112`:
   - replace the denial of `worth-runtime-bridge -> worth-relational` with a
     denial of `worth-relational -> worth-runtime-bridge`;
   - add the matching boundary-check test next to
     `configured_dependency_denials.rs:239-256`.
6. Update every downstream import found by the multi-line search above to
   the new Bridge path, including the Runtime World example and
   certification test. Rewrite the stale string in
   `basis_lifecycle/reuse.rs:257-267` and record the matrix-digest change
   (D10).
7. Update the descriptive path tables in
   `crates/worth-relational/src/identity_authority/phase_one_root_break_targets.rs:26-39`
   and `phase_one_family_map.rs:63-112`.
8. Regenerate `AGENT_CONTEXT.md` files with `tools/agent-context`; never
   hand-edit them.

**Exit:**

- `cargo tree -p worth-relational` does not list `worth-runtime-bridge`, and
  `cargo tree -p worth-runtime-bridge` lists `worth-relational`.
- This search finds nothing in `crates/worth-relational`:

  ```text
  rg 'worth_runtime_bridge|Runtime ?Bridge|RelationalBridge|_for_bridge|bridge_observation'
  ```

  A plain `rg -i bridge` is not the test and cannot pass. Relational has
  legitimate names that contain the word: `SchemaBridgeabilityClassification`,
  `SchemaBridgeDescriptor`, `ContinueWith{Visible,Transparent}Bridge`, and
  `BoundaryBridgedRelational…`. The mock in
  `tests/performance_profiles/bridge_runtime_support.rs` is renamed to say
  what it mocks. Prose that names the Bridge as a consumer in docs is
  allowed only in "Who consumes this" sections.
- Owner tests pass for Relational, the Bridge, Runtime World,
  `worth-query-execution`, `worth-query`, and `worth-ui-query-binding`, plus
  the Query and UI certification suites that exercise the Bridge.
- The boundary check, including its new denial test, passes.

### Phase 5: Tighten and document

- Narrow the Bridge constructors in three groups (D11):
  - **Crate-private**, because only the adapter calls them:
    `BridgeCommittedPatchEnvelope::new_with_authoritative_lowering` and
    `BridgeCommittedPatchItem::with_relational_semantic_change`.
  - **Behind the `certification-construction` feature**, because Query tests
    and `ContinuityLineageSource` fakes build these values:
    `BridgeHistoricalLineageAuthority::try_new`,
    `BridgeAuthoritativeSourceProvenance::from_owner_publication` and
    `from_owner_partition_publication`, and
    `BridgeProducerMetadata::registered_authoritative_source` and
    `with_authoritative_source`. Their Query callers enable the feature
    through a dev-dependency.
  - **Public**, with rustdoc saying why: `from_relational_commit_id`,
    `from_relational_lineage_id`, and `from_relational_publication`.

  Add compile-fail tests proving that the crate-private constructors cannot
  be called from outside the Bridge, and that the gated ones cannot be
  called without the feature. Regenerate
  `tests/ui/committed_patch_item_literal_private.stderr`.
- Delete `RelationalBridgePresentation{ExportIdentityKind,DigestIdentityBasis}`
  (`crates/worth-relational/src/identity_authority/kinds.rs:10-25`); nothing
  uses them.
- Update the docs:
  - `README.md` (dependency tree and the "reverse of the intended
    direction" note);
  - `docs/how-it-works.md` §2.2 (table rows and note), §6.1, and §6.3;
  - `docs/api.md` §6;
  - `crates/worth-relational/README.md:104-113` and `API_OVERVIEW.md:83`;
  - Relational's `BRANCH_LOCAL_MVCC.md:272`, `OWNER_COMPONENT_PORT.md:8-10`
    and `:499`, and `TESTING_WORLDS.md:509-515`. Each either moves to the
    Bridge docs or becomes a "Who consumes this" note;
  - `crates/worth-runtime-bridge/README.md:28-33` and the anti-pattern at
    `:165`;
  - this roadmap and the Relational roadmap. The supersession note at
    [Milestone 1 §5.8](./milestone-1.md#58-bridge-owned-adapter-traits) is
    already in place.

**Exit:**

- The public docs show one direction: the Bridge depends on Relational and
  Signal.
- The compile-fail tests for the narrowed constructors pass.

## Must Preserve

- **Truth stays authoritative in Relational.**
  - Relational alone decides whether a commit is a legal change, what it
    changed, and which observation it is valid at.
  - The Bridge never re-derives Relational meaning from raw data.
- **Forged authority opens no doors.** Two requirements hold:
  - The Relational change receipt cannot be built outside Relational.
  - A narrowed Bridge envelope cannot be built outside the Bridge.

  Each has a compile-fail test.
- **Byte-identical artifacts.** The following must be identical before and
  after every phase: canonical committed-patch envelopes (including their
  provenance strings and `BridgeAuthoritativePatchLoweringCounters`),
  snapshot identities, grouped-truth digests, declaration digests, route
  records, and replay bundles. Any change is a failure, not a re-baseline.
  The planned exceptions are the `basis_lifecycle/reuse.rs` matrix digest
  (D10) and the relational routing and declaration-entry inspection digests
  of `GroupedTruth` and `BridgeSource` rows (D12). Each changes only because
  an input string that names a type's path is corrected.
- **Every outcome family survives.** Today's publication outcomes (`Stale`,
  `RebindRequired`, `Deferred`, `Denied`, `Failed`) and source errors keep
  their meaning.
- **Cost and bounds hold.**
  - Commit selection, branch-head selection, and lineage costs keep their
    counters and bounds.
  - The existing cost tests (`bridge_source_tests/branch_head_selection_cost.rs`,
    `fork_ancestry_selection.rs`, `fork_lineage_scale.rs`) move with the
    adapter and still pass. They assert on the item 6 work receipt instead
    of crate-private counters. A cost test that cannot be written that way
    stays in Relational, against Relational's own API.
- **Signal is untouched.** No Signal file changes in this milestone.
- **Query's public facades do not change.** `worth-query-host` and
  `worth-query-decl` exports stay the same:
  - `RelationalBridgeRecordIdentityParts` stays a Bridge type re-exported
    through `worth_query_host::facade::primary_graph`, with its primitive
    constructors;
  - the `GroupedTruth` and `BridgeSource` variants keep their names (D9);
  - the Query entries in `tools/boundary-check/snapshots/facades.toml` do
    not move.

## Acceptance Evidence

This milestone is complete only when:

- `cargo tree` shows `worth-runtime-bridge -> worth-relational` and no path
  from `worth-relational` to `worth-runtime-bridge`.
- `road1.toml` denies `worth-relational -> worth-runtime-bridge`, and a
  boundary-check test proves it.
- Relational's source, tests, facade, and README do not name the Bridge,
  except for a "Who consumes this" note.
- The digest parity test from Phase 2 and the artifact parity checks from
  Phase 4 pass, and their before-and-after results are recorded.
- The Phase 5 compile-fail tests pass.
- Every moved test passes, and none was deleted to make the move easier. Any
  test that could not move is listed in the closeout with the reason.
- The public docs (README, How WORTH Works §2.2, API map) show the new
  direction with no "unintended" caveat.

## Risks

| Risk | Mitigation |
|---|---|
| The change source API leaks Relational internals, which would make Relational's public surface worse | Phase 3 DX review. Each item must be meaningful without the Bridge. If an item only makes sense to the Bridge, it is shaped wrong. |
| The grouped-truth move changes a digest | It moves as its own module (D4). Phase 2 parity test; stop and surface, never re-baseline. |
| The flip commit is large, and review misses a semantic change | After Phase 3 the probe crate proves the adapter uses only public API, so Phase 4 is mostly a move. It is not a pure move: the lock files, the downstream imports, and the D10 digest change land with it. Review it with a rename-aware diff (`git diff -M`), and list every hunk that is not a move in the closeout. |
| A stale lock file fails CI after the flip | Phase 4 updates every tracked lock file found by search, not a fixed list. |
| Other branches in flight touch `presentation/bridge` or its importers | Announce the flip window. Rebase in-flight work after Phase 4 instead of carrying old paths. |
| The Bridge's compile time grows because it now builds Relational | Measure `cargo build -p worth-runtime-bridge` before and after with an A/B/A run. Every current Bridge consumer already builds Relational, so the whole-workspace cost should not move. |

## Out Of Scope

- Changing Signal or the Signal side of the Bridge.
- Moving Runtime World, or changing what it owns.
- Renaming Bridge types other than the path move, the grouped-truth names
  (D4), and the `relational_identity.rs` absorption.
- Retyping the `from_relational_*` constructors.
- Redesigning the Bridge's source traits.
- Any change to Query's public facades, including renaming the
  `GroupedTruth` and `BridgeSource` variants (D9).

## Completion Note

Phase 0 is complete: `worth-harness` is a Bridge dev-dependency, and
`cargo tree -p worth-runtime-bridge -e normal` no longer lists it.

Phase 1 is complete, with one deferral. The Bridge-only compile-fail tests
moved, except `raw_commit_cannot_publish_bridge.rs` and
`fork_basis_cannot_publish.rs`. Both name `publish_commit_for_bridge`, which
Phase 3 replaces, so Phase 3 rewrites them against the change receipt.

Phase 2 is complete.

- Grouped truth lives in `worth_runtime_bridge::relational_grouped_truth`,
  with the D4 names `RelationalGroupedProjectionContract`,
  `RelationalGroupedProjectionDigest`, and `RelationalRowSetDigest`.
- The Bridge exports it flat from its facade. The flat export adds no new
  module path; the `everyday`, `advanced`, and `specialist` exposure
  predates this change.
- Canonical digests are byte-identical. `canonical_digests_match_the_pinned_bytes`
  pins the row-set and grouped-projection digests captured on the pre-move
  commit, and passes unchanged in the Bridge.
- The routing digests change as D12 records. `lower.rs:31` is updated;
  `lower.rs:41` still names `worth_relational::facade::bridge`, which exists
  until Phase 4, so it changes there.
- The grouped-truth identity scan is a Bridge unit test beside the code it
  guards, and also bans the crate-private identity minters.

Phase 3 is complete.

- `worth_relational::facade::change_source` holds items 0 to 7. It names no
  Bridge concept, and every public item has rustdoc. The publication proof
  chain, partition projection, and consistency checks live there; receipt
  minting runs the checks, so holding a `RelationalChangeReceipt` proves they
  passed.
- The adapter uses only the Relational facade, the Bridge facade,
  `worth_foundational::facade`, and `worth_proof`. The probe crate
  `tools/relational-adapter-probe` compiles the adapter and its tests as an
  outside crate, and all 46 adapter tests pass there.
- Compile-fail tests in `tests/ui/change_source` prove that a raw
  `CommitId` or a fork basis cannot mint a receipt (E0308), and that the
  receipt, the selected commit, and the retained observation have no
  struct-literal constructor, `Default`, or `Clone`.
  `tests/ui/change_source_pass` holds the valid counterpart. These replace
  the two Phase 1 deferrals.
- `struct_field_patch_authority.rs` and `read_cutover.rs` are split. Each
  Relational half asserts on Relational state: the change receipt for the
  first, and history and visibility at both observations for the second.
  The Bridge halves are adapter tests
  (`bridge_source_tests/struct_field_patch.rs` and `read_cutover.rs`), built
  on the adapter's own support module instead of the supply-chain world.

**DX review.** The change source names say what each item is in Relational
terms: `RelationalRuntimeHandle`; `select_reachable_commit` and
`select_exact_commit`, which return a `RelationalCommitSelection` holding a
`RelationalSelectedCommit` or a `RelationalCommitSelectionDenial`, plus a
`RelationalCommitSelectionWork`; `mint_change_receipt`, which returns a
`RelationalChangeReceiptOutcome`; `RelationalChangeConsistencyDenial` and
`RelationalChangeConsistencyWork`; `retain_observation_snapshot`, which
returns a `RelationalRetainedObservation`; and the four runtime reads at an
observation, which refuse a foreign observation with
`RelationalObservationReadDenial`. The facade module's doc lists these entry
points in the order a consumer uses them. The Bridge-side path is the flat
facade export described above. `bridge_snapshot_identity_for_commit` and
`bridge_snapshot_identity_for_handle` keep their names: their parameters are
Relational types, and Query callers already use them.

**Destinations.** Every remaining `RA/**` file moves in Phase 4 to the same
relative path under `crates/worth-runtime-bridge/src/relational_source/`:
`mod.rs`, `change_publication.rs`, `identities.rs`, `lowering_precision.rs`,
`patch_envelopes.rs`, `publication_outcome.rs`, `snapshot_reading.rs`,
`runtime_source/**`, and `snapshot_values/**`. The test modules
(`*_tests.rs`, `bridge_source_tests/**`), `relational_test_support/**`, and
`test_catalog.rs` move beside them as Bridge test code. Nothing stays in
Relational. `partition_projection.rs`, `patch_semantic_validation.rs`,
`authoritative_patch_publication.rs`, and
`authoritative_publication_witness.rs` no longer exist in `RA/**`; Phase 3
moved their Relational meaning into the change source and their Bridge
meaning into `change_publication.rs` and `lowering_precision.rs`.

Deviations from the plan:

- Publication no longer re-checks for an empty graph role (the old
  `UnsupportedProducerEnvelope` denial). The source refuses a blank or padded
  role when it is built, so the check could not fire.
- The widening publication path ignores the source's partition, as it did
  before.
- The receipt carries the selected branch as well as the authoring branch,
  so a fork publication keeps the branch it was selected on.
- `RelationalCommitSelectionDenial::ForeignObservation` is new. Commit
  selection refuses an observation another runtime issued, before it reads
  any history. The exact reads and the lineage reads do the same with
  `RelationalObservationReadDenial::ForeignObservation`; before, they resolved
  a foreign commit against this runtime's lineage and ancestry and answered
  wrongly. The two `*_kind_declares_aspect` reads take no runtime; they read
  the observation's own retained schema.
- The receipt carries commit, version, and branch ids but not the snapshot
  id. The snapshot id belongs to the retained observation, and the adapter
  carries it beside the receipt in `RelationalBridgeSelectedCommitObservation`.
- The adapter mints the receipt under the runtime lock and lowers it
  outside the lock.
- The consistency checks all run at receipt minting, over every record,
  before any lowering check. The old per-record order was count, opaque
  gate, match and widened claim, coverage, posture. Two cases now report
  `InvalidAuthoritativePatchSemantics` with Relational's work where they
  reported `UnsupportedAuthoritativePatchPrecision` with lowering's: one
  record with an unadmitted opaque change and any later breach, and an
  unadmitted opaque record followed by an inconsistent one. A count mismatch
  still wins, as before. `consistency_lowering_tests.rs` pins both cases and
  one mapping per rule.
- `PublishedAuthoritativePatchEnvelope::check_change_consistency` is public,
  so a patch held by another route is checked by the same rules. The
  adapter's decoded-publication lowering applies them first, as receipt
  minting does.
- `UncoveredOperation` is gone. Equal counts and one expected change
  consumed per matched change leave nothing uncovered, so it could not
  fire; an uncovered operation shows as an unjustified change.
- The adapter source sums its commit selections' work across clones. The
  cost tests assert those sums after each real `load_*` call, against the
  3N - 1 work of one walk down an N-commit linear history.
- `RelationalRuntimeHandle` reads the runtime instance id once when it is
  made, so the id and `Debug` take no lock. The source no longer keeps its
  own copy.
- `RecordStructuralChange` is non-exhaustive outside Relational, so lowering
  denies an unknown structural change with `InvalidLoweringContract` instead
  of guessing. Inside Relational that arm is unreachable and carries
  `#[allow(unreachable_patterns)]`, which Phase 4 drops.
- The copied-metadata and endpoint-label consistency tests moved from the
  adapter to `change_source/consistency_tests.rs`, because receipt minting
  owns those checks. The fork-lineage scale test moved to
  `tests/lineage/fork_scale.rs`, because it measures Relational internals.
- Lowering tests that need patches no commit produces decode them from
  Relational's wire shape (`relational_test_support/publication.rs`), rather
  than widening Relational's constructors.
- The adapter's retention assertions use the public
  `branch_retention_cost_counters`.

Follow-up outside this milestone: an earlier Phase 3 run saw six Relational
wall-clock timing tests fail under concurrent machine load. The full suite
passes at the Phase 3 commit.

Phase 4 is complete.

- The adapter lives in `crates/worth-runtime-bridge/src/relational_source/`,
  at the destinations above. `relational_identity.rs` became
  `relational_source/identity_parts.rs`. The Bridge facade exports the
  adapter, grouped truth, and the identity parts from one module,
  `facade/exports_relational.rs`; downstream callers import them from
  `worth_runtime_bridge::facade`.
- Relational no longer has `facade::bridge`, `presentation::bridge`,
  `tests/bridge_identity_boundary.rs`, or the `worth-runtime-bridge`
  dependency. The probe crate is deleted: the adapter now compiles as Bridge
  code against the Relational facade, which is the property the probe proved.
  The identity scan is the Bridge unit test
  `relational_source/identity_boundary_tests.rs`.
- `road1.toml` denies `worth-relational -> worth-runtime-bridge`, and
  `configured_fence_keeps_relational_off_the_bridge_and_lets_the_bridge_read_relational`
  proves the denial and that the reverse edge passes.
- All eight lock files carry the flipped edges. The two UI certification
  fixture locks are already stale for reasons outside this milestone, so only
  the two edges were changed there.
- The performance mock is renamed for what it mocks: a downstream runtime
  (`downstream_runtime_support.rs`, `downstream_runtime_mock_matrix/`, and
  `geometry_commit_downstream_wave_*` scenario ids).
- `RelationalBridgePresentation{ExportIdentityKind,DigestIdentityBasis}` and
  the `bridge_presentation_export` family were deleted here rather than in
  Phase 5, because the family map named the moved paths. The family-map
  frontier strings now name the change source.
- D10 and D12: `basis_lifecycle/reuse.rs` and
  `declaration_relational_routing/lower.rs` name
  `worth_runtime_bridge::facade::RuntimeBridgeRelationalSource`. Neither
  digest is pinned; both are computed from these strings.
- `tools/boundary-check/snapshots/facades.toml` does not change: it lists no
  Bridge path, and `RelationalBridgeRecordIdentityParts` keeps its name.

Hunks that are not a move or a path rewrite:

- `fork_ancestry_selection.rs` asserts the work after evaluation.
- `patch_envelopes.rs` drops a stale doc paragraph and the
  `#[allow(unreachable_patterns)]`, which is no longer needed outside
  Relational.
- `consistency_lowering_tests.rs` adds
  `a_consistent_unadmitted_opaque_record_is_unsupported_precision`, the
  single-fault baseline for the two double-fault cases.
- `relational_source/mod.rs` declares `identity_parts` and the identity scan.
- Four Runtime World test fixtures import `RelationalRuntimeApi` from
  Relational and the adapter from the Bridge.
- Relational's README and `API_OVERVIEW.md` move the Bridge flow into "Who
  consumes this" sections.

Phase 5 is complete.

- Crate-private: `BridgeCommittedPatchEnvelope::new_with_authoritative_lowering`,
  `BridgeCommittedPatchItem::with_relational_semantic_change`, and
  `BridgeProducerMetadata::new`. The spec did not list `new`, but a public
  `new(RegisteredAuthoritativeSource, ..)` would have bypassed the gate on
  `registered_authoritative_source`; only Bridge code calls it.
- Behind `certification-construction`:
  `BridgeAuthoritativeSourceProvenance::from_owner_publication` and
  `from_owner_partition_publication`, and
  `BridgeProducerMetadata::registered_authoritative_source` and
  `with_authoritative_source`. The `certification_constructor!` macro
  (`src/certification_constructor.rs`) emits each one as `pub` with the
  feature and `pub(crate)` without it. Query and the UI certification crates
  already enable the feature.
- `BridgeHistoricalLineageAuthority::try_new` stays public, which departs
  from D11. Its caller is Query's public acceptance suite
  (`worth_query_lower_runtime_acceptance_suite`, through the phase-six
  readmission fixtures), not a test-only path, so a feature gate would have
  put it behind a dev-dependency it cannot use. It validates that the
  identities are canonical and distinct, and the authority basis comes from
  the request.
- `from_relational_commit_id`, `from_relational_lineage_id`, and
  `from_relational_publication` stay public, with rustdoc saying why.
- Compile-fail tests: `tests/ui/crate_private_constructors/` and, without
  the feature, `tests/ui/certification_constructors/`. With the feature,
  `tests/pass/certification_constructors_with_feature.rs` must compile.
  `committed_patch_item_literal_private.stderr` regenerated unchanged.
- The two `&str` record-identity compile-fail cases were the same case; the
  grouped-row-label copy is deleted.
- The dead presentation kinds were already deleted in Phase 4.
- The docs listed above now show one direction, and this roadmap marks the
  milestone complete.
