# Milestone 11: Persistent naming over entities, relations, and groups

Tracking issue: `#71`. Revised 2026-11-21 from the earlier Nov 5 design brief
to reflect the current `origin/master` architecture, concrete target APIs,
law bindings, and bounded property tests.

## Goal

An application declares what its things mean — an entity, a relation, or a
group of records — along with its correspondence postures, group continuity
rules, and split discriminators. Public WORTH mints bounded,
allocation-independent persistent names (`WorthQueryName<K>`), validates and
records many-to-many lineage events, and resolves held names to current
generational handles or typed refusals.

A held name keeps resolving to the same meaning across:
- edits, regenerations, splits, and merges;
- undo, redo, and history checkout;
- feature reorder (when a name's parents are unchanged);
- a from-scratch rebuild and replay at any worker count;
- a process restart from Store (Phase 7).

When a held name has no unique live answer, resolution returns a typed refusal
naming its candidates or its retiring commit. It never rebinds silently by
nearest geometry, slot order, or payload digest.

The first consumer is WORTH Proprietary CAD, which names faces, edges, loops,
shells, and features. The same capability serves BIM elements and spaces, MEP
runs and systems, and UI selection continuity. Applications keep using
`worth-query-decl` and `worth-query-host`.

**Primary rule (Architecture 1, 10, 18):**
Public WORTH owns name minting, lineage authority, derived name-to-generation
indexing, and tracked resolution. Applications supply domain correspondence
decisions, typed split discriminators, and role vocabulary — and never build
private generic name graphs or lineage walkers.

## Scope and current boundary

### Verified on `origin/master` (2026-11-21)

| Existing owner and path | Starting guarantee | Gap addressed in this milestone |
|---|---|---|
| `crates/worth-relational/src/identity/data/record_ids.rs` | Generational `RecordId { partition, local_slot, generation }` | Reused unchanged as the live handle; never conflated with persistent names. |
| `crates/worth-relational/src/lineage/data/events.rs` & `authority/commit_finalization.rs` | `LineageEventKind` has `Create`, `Replace`, `Split`, `Merge`, `Retire` with `sources`/`targets: Vec<LineageId>` | Finalization only emits `Create`, `Retire`, `Replace` for entities; relation mutations emit no lineage; no birth-name provenance or `Restore` kind. Closed in Phases 1, 2, 5. |
| `crates/worth-relational/src/lineage/access/resolution.rs` | Indexed lineage walk over sources (`event_positions_for_sources`) | No branch-local name-to-generation derived index or unprivileged tracked resolution. Extended in Phase 4 (no second walker). |
| `crates/worth-relational/src/aggregate_identity/transition/mint.rs` & `stored_lineage.rs` | Orbit and component `AggregateIdentity` with worker-count determinism | `mint()` hashes `CommitId` + anchor's native `(partition, local_slot, generation)` (allocation-dependent); keeper uses native member order; lineage sits in `stored_lineage.rs` instead of envelope events. Re-keyed in Phase 3. |
| `workspaces/worth-query/crates/worth-query-declaration/src/application_operation/binding/output_role/action.rs` | `ApplicationMutationOutputPosture` (`Create`, `Preserve`, `Retire`) and `WorthQueryApplicationOutputRole` | No `Split` or `Merge` posture, and no declared name schemes or discriminators. Extended in Phases 1 and 2. |
| `crates/worth-runtime-bridge/src/adapter/continuity_lineage.rs` | `ContinuityLineageSource` UI selection continuity (single, split, merge, ambiguous) | Consumes public name resolution in Phase 4. |
| `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore.rs` | Checkpoints and restores Relational lineage | Extended in Phase 7 to persist name birth/lineage events and rebuild the derived name index on reopen. |

*(Audit note vs the Nov 5 brief: legacy `domain_installation/operation_lineage/naming.rs`
(`admit_persistent_name`), `identity_evolution/`, and `ordinary/workflow/branch_merge/`
were retired with the legacy engine crate and are already absent from `master`.)*

### Out of scope

- **Domain geometric correspondence algorithms:** Owned by the application;
  public WORTH never interprets geometry.
- **Physical Store serialization formats:** Owned by Store; Phase 7 consumes
  Store's durable event log/checkpoint interface.
- **Tree-based undo/redo implementation:** Specified in
  [tree-based-undo.md](tree-based-undo.md); Phase 5 integrates persistent name
  continuity with `HistoryDocument` once its Phase 2 lands.

## Shared definitions

- **Two layers, one authority:**
  - `RecordId` (generational native handle): live, stale-safe, allocation-bound.
  - `WorthQueryName<K>` (persistent name): opaque, kind-typed, fixed-size
    canonical identity minted by Relational from declared meaning parts.
  - A name is never an ID, and an ID is never a name (Architecture 10).
- **Name grammar (frozen in Phase 1):** Every entity or relation name is minted
  from:
  1. declared scheme identity;
  2. producer's authored occurrence key (from an authored identity family —
     never a bare operation identity, so two independent features never collide);
  3. declared role path;
  4. parent name identities (for `Split` or `Merge` successors, entering only as
     fixed-size canonical identities so name size stays O(1) at arbitrary split
     depth);
  5. typed local discriminator (Foundational values and/or references to other
     names that exist at the basis or earlier in the scheme's acyclic role order).
  - **Exclusions:** Never depends on payload, geometry, coordinates, floats,
    allocation, partition, local slot, generation, commit position, or worker
    order.
- **Canonical authority (Architecture 18):** Birth and lineage events in the
  canonical commit envelope are the sole authority. Per-record name reads and
  the name-to-generation index are derived MVCC state rebuildable from events.
  A bare `WorthQueryName<K>` with no matching birth event refuses typed.
- **Well-founded discriminators:** Installation checks scheme role-reference
  order for acyclicity and requires every `Split`-capable scheme to declare a
  discriminator. Value uniqueness among siblings is checked at commit; two
  successors minting the same identity in one commit refuse typed, naming both.

## Phase overview

*(Phase 6 is the downstream CAD consumer cutover tracked in the consumer
repository; keeping the shared phase numbers 1–5 and 7 keeps cross-repository
tracking aligned.)*

| Phase | Usable result | Requires | Establishes for later phases | Parallel constraints |
|---|---|---|---|---|
| **1. Name schemes and runtime minting** | Declared entity/relation/group name schemes; Relational mints fixed-size, allocation-independent `WorthQueryName<K>` at every birth event and rejects cycles, collisions, and handler writes. | Current `origin/master` | Frozen name grammar, `WorthQueryName<K>`, birth provenance on lineage events, Relational minting authority. | Freeze grammar before Phases 2 and 3. |
| **2. Many-to-many lineage for entities and relations** | `Split` and `Merge` output postures validated against actual performed effects; canonical many-to-many `LineageEventRecord`s emitted for both entities and relations. | Phase 1 | Effect-validated entity and relation lineage for Phase 4 resolution. | Can run in parallel with Phase 3. |
| **3. Allocation-independent aggregate and group names** | `AggregateIdentity` re-keyed to `(scheme, anchor_name, continuity_rule)`; split/merge keepers chosen by declared continuity (`Anchor`, `Boundary`, `ProducerRole`); `stored_lineage.rs` unified into envelope lineage. | Phase 1 | Rebuild- and worker-order-stable group names (orbits, components, declared groups). | Can run in parallel with Phase 2. |
| **4. Tracked local name resolution and Bridge continuity** | Unprivileged, dependency-tracked `reader.name_of` and `reader.resolve` over a branch-local derived index (O(lineage path), flat at 1M records); `ContinuityLineageSource` consumes it. | Phases 2, 3 | Complete public read/resolve/continuity API ready for downstream adoption. | Sequential after Phases 2 and 3. |
| **5. History, correction, and rebuild continuity** | `Restore` lineage kind on undo/redo/authored restore; `Preserve` continuity on in-commit regeneration; names hold across `HistoryDocument` undo/redo/checkout and fresh-process rebuilds. | Phase 4; [tree-based-undo.md](tree-based-undo.md) Phase 2 for cross-retirement history tests | Full in-process continuity across edits, corrections, history navigation, and rebuilds; unblocks downstream consumer cutover (Phase 6). | Sequential after Phase 4. |
| **7. Durable names on Store reopen and branch merge** | Store checkpoint/journal persists birth and lineage events; fresh installations admit retained names by birth event and rebuild the index; branch merges touching named outputs rerun producers or refuse typed. | Phase 5; Store lineage persistence readiness | Cross-restart and cross-branch durable persistent naming; closes `#71`. | Independent of downstream Phase 6; waits only on Store durability interface. |

---

## Phase 1: Name schemes and runtime minting

### Outcome, scope and dependencies

Applications declare typed name schemes for entity, relation, and group kinds
through `worth-query-declaration`, declaring role order and split discriminator
types. At commit, `worth-relational` evaluates the scheme over each birth's
admitted occurrence key, role path, parents, and discriminator values, minting a
fixed-size opaque `WorthQueryName<K>` and recording both the canonical identity
and full typed birth parts on the canonical `Create` lineage event.

Handlers and providers can never write, mutate, or forge a name. Installation
refuses cyclic discriminator dependencies and split-capable schemes lacking a
discriminator. Commit refuses colliding births naming both holders.

**Requires:** Current `origin/master`.
**Out of scope:** `Split`/`Merge` postures (Phase 2); `AggregateIdentity`
re-keying (Phase 3); resolution queries (Phase 4).

### Target API and caller DX

**Proposed declarations (`worth_query_declaration::facade::application_naming`):**

```rust
use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, ApplicationNameScheme, ApplicationNameSchemeBuilder,
    ApplicationNameSchemeDenial, WorthQueryName,
};

// 1. Declare a typed discriminator (Foundational values + fixed-size name references only;
//    no floats, no RecordIds, no raw payloads).
#[derive(Clone, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
pub enum FaceSplitDiscriminator {
    SidedCut {
        cutter: WorthQueryName<Face>,
        side: CutSide,
        boundary_piece: Option<WorthQueryName<Edge>>,
    },
}

// 2. Declare the scheme on the schema builder (checked at install time for acyclicity).
schema.application_name_scheme::<Face>(
    ApplicationNameSchemeBuilder::<S, Face>::new("worth.example.face.v1")
        .role_order(&["authored", "cut_piece", "seam_patch"])
        .split_discriminator::<FaceSplitDiscriminator>()
        .require_authored_occurrence()
        .build()?,
)?;
```

- **Compile-time authority enforcement (Arch 3, 10, 16; DX 8):**
  `WorthQueryName<K>` has no public constructor from raw bytes or arbitrary
  strings; only Relational birth/readmission and declared query reads produce an
  admitted `WorthQueryName<K>`. `WorthQueryName<K>` carries a
  `#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]` fixed-size canonical
  digest plus phantom kind `K` so a `WorthQueryName<Face>` cannot be passed
  where `WorthQueryName<Edge>` is expected.

### Authority, typed progression and architectural laws

- **Domain declares policy; Relational mints (Arch 1, 3, 10, 13):** Query lowers
  `ApplicationNameScheme` into Relational's installed schema layout; Relational
  alone evaluates minting during commit finalization. No handler, provider, or
  caller can pass a pre-minted name on a birth.
- **Allocation independence & bounded size (Arch 10, 17; Perf Core Law 1):**
  Minting hashes canonical domain-tagged bytes of `(scheme_id, occurrence_key,
  role_path, sorted_parent_identities, canonical_discriminator)` into a fixed
  32-byte identity. Referenced names enter only by their 32-byte identity, so
  name byte size is O(1) regardless of lineage depth.
- **Fail closed at earliest boundary (Arch 4, 16; DX 17):** Cyclic role
  references and missing split discriminators refuse at installation
  (`ApplicationNameSchemeDenial::{CyclicRoleOrder, MissingSplitDiscriminator}`);
  runtime collisions refuse at commit
  (`LineageNameDenial::DuplicateBirth { identity, first, second }`) leaving zero
  committed effects.

| Transition | Exact input / proof | Granting owner and admission | Output / proof | Next permitted consumer | Refusal and effect posture |
|---|---|---|---|---|---|
| Declare & install scheme | `ApplicationNameScheme<S, K>` | `worth-query-installation` (checks valid names, acyclic role order, split discriminator presence) | Installed naming layout in Relational schema | Birth minting at commit, Phase 2 postures | Refuses `CyclicRoleOrder` / `MissingSplitDiscriminator` / `DuplicateScheme` before runtime starts. |
| Mint at birth | Admitted authored occurrence key, role path, parent names, discriminator value | `worth-relational::lineage::authority::commit_finalization` | `WorthQueryName<K>` + birth event with full typed parts | Derived name index (Phase 4), downstream lineage | Refuses `DuplicateBirth` / `InvalidDiscriminatorBasis` before commit seal; zero partial rows. |

### Operational behavior and cutover

1. Installation validates every declared scheme's role order DAG and registers
   the lowered naming contract in Relational's immutable schema layout.
2. During commit finalization, for each created entity/relation whose kind has
   an installed scheme, Relational resolves referenced discriminator names (from
   basis or earlier role in the same occurrence), encodes canonical typed parts,
   mints the 32-byte identity, checks for live-holder or same-commit collisions,
   and appends the birth record to the commit's canonical lineage events.
3. Interruption or commit refusal leaves zero birth events and zero name state
   (Arch 22). Retry with the same inputs deterministically mints byte-identical
   names.

### Ownership and meaningful placement

- `workspaces/worth-query/crates/worth-query-declaration/src/application_naming/`:
  new module (`scheme.rs`, `discriminator.rs`, `name.rs`) defining
  `WorthQueryName<K>`, `ApplicationNameScheme`, and validation.
- `workspaces/worth-query/crates/worth-query-installation/src/naming.rs`:
  acyclic role-order verification and lowering into Relational layout.
- `crates/worth-relational/src/lineage/`: extend `data/events.rs` (birth parts
  and minted `PersistentNameIdentity`) and `authority/commit_finalization.rs`
  (minting & collision guard).

### Phase 1 test matrix

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|
| Fresh-process & worker-count determinism (Arch 17; Perf Core Law 15) | Same authored sequence built in two fresh processes and at 1, 2, and 8 workers | Mint entity + relation names across multi-step program | Byte-identical `WorthQueryName<K>` across all runs and worker counts | Exposes allocation, slot, or worker-order leakage into minting | Decl/Host court (`worth-query-court-decl-host`, `persistent_naming`) |
| Feature reorder invariance vs parent change | Two independent features `F1`, `F2`, plus a cut `C` splitting `F1.face` | Reorder `F1`/`F2` (independent); then reorder `C` so it splits `F2.face` instead | Independent reorder keeps identical names; parent-changing reorder mints a distinct name (old name never silently rebinds) | Exposes commit-position leakage or silent cross-parent aliasing | Decl/Host court (`persistent_naming`) |
| Installation validation & commit collision (Arch 4, 16) | Cyclic role order; split scheme without discriminator; two siblings emitting identical discriminator | Install invalid schemes; commit colliding sibling outputs | Install refuses `CyclicRoleOrder` / `MissingSplitDiscriminator`; commit refuses `DuplicateBirth` naming both holders with 0 effects | Exposes missing install DAG check or silent overwrite on collision | Installation + Decl/Host court |
| Compile-time non-forgeability (Arch 10, DX 8; Testing 15) | Compile-fail doctest in `worth-query-declaration` / `worth-query-publication` | Attempt to construct `WorthQueryName<Face>` from raw bytes, pass `WorthQueryName<Edge>` as `Face`, or write a name field from a handler | Fails to compile at intended boundary; valid counterpart compiles | Exposes forged-name or cross-kind bypass | Doctest (`worth-query-declaration`, `worth-query-publication`) |
| Bounded size under repeated splits | Chain of 64 successive splits | Inspect byte size of `WorthQueryName<K>` and birth-event parts at depths 1, 16, 64 | Constant `WorthQueryName<K>` size (32 bytes) and O(1) birth-part size per split | Exposes recursive full-tree copying inside names | Relational unit/court (`worth-relational`) |
| Allocation/reorder invariance laws (`proptest` 1.x; Testing 2) | 32–64 bounded `proptest` DAGs (`proptest-regressions/` committed) of independent and dependent births | Permute creation order of independent occurrences and vary dummy pre-allocations before running each DAG | Every occurrence's minted `WorthQueryName<K>` set is byte-identical across all valid topological permutations and slot offsets | Exposes subtle hash sensitivity to insertion order, slot numbers, or map iteration | Relational + Decl/Host court (`persistent_naming`) |

### Completion gate and documentation

- **Verification:** Focused `worth-relational` lineage tests,
  `worth-query-court-decl-host` (`persistent_naming`), doctests,
  `./loop gate --step guard-diff`, `--step line-caps`, check-compile of
  dependent targets.
- **Docs:** Add the naming declaration section to `docs/build-an-application.md`
  and `docs/capability-index.md`.

---

## Phase 2: Many-to-many lineage for entities and relations

### Outcome, scope and dependencies

`ApplicationMutationOutputPosture` (and its Decl/Host output-role actions) gains
`Split { parent, discriminator }` and `Merge { parents, keeper }` alongside
`Create`, `Preserve`, and `Retire` — uniformly for **both entities and
relations**. Commit finalization validates every declared posture against the
actual performed creations, retirements, and relation endpoint mutations before
writing canonical `LineageEventRecord`s (closing `MutationIntent::Relation(_) => {}`
in Relational finalization). Any posture contradicting the performed effects
refuses typed before commit, naming both posture and effect.

**Requires:** Phase 1.
**Parallel constraint:** Can run in parallel with Phase 3.
**Out of scope:** Aggregate/group continuity rules (Phase 3); resolution queries
(Phase 4).

### Target API and caller DX

**Extended posture vocabulary (`worth_query_declaration::facade::application_operation`):**

```rust
use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputPosture, WorthQueryMergeKeeper, WorthQueryMergeOutput,
    WorthQuerySplitOutput,
};

// Output posture variants on ApplicationMutationOutputPosture:
//   Create
//   Preserve
//   Retire
//   Split { parent: WorthQueryName<K>, discriminator: D }
//   Merge { parents: Vec<WorthQueryName<K>>, keeper: WorthQueryMergeKeeper<K> }
//
// WorthQueryMergeKeeper<K>:
//   KeepParent(WorthQueryName<K>)  // must be a member of `parents`; keeps that parent's name
//   MintNew { discriminator: D }   // mints a new name carrying all parent identities
```

- **Uniform entity & relation surface (Arch 12, DX 1):** Relations use the exact
  same `Create` / `Preserve` / `Retire` / `Split` / `Merge` posture contract as
  entities, and parallel same-role relations between the same pair of endpoints
  are distinguished by discriminator, never by endpoint names alone.

### Authority, typed progression and architectural laws

- **Postures validated against performed effects (Arch 4, 6, 16):**
  - `Split { parent, discriminator }`: parent must be retired (or replaced by
    disjoint split pieces) in this commit and at least one successor created
    with a valid discriminator; claiming `Split` on an unretired parent or with
    zero created targets refuses `LineagePostureMismatch`.
  - `Merge { parents, keeper }`: every listed parent must be retired in this
    commit (or the one named `KeepParent` preserved in place while every other
    parent is retired); a `KeepParent` outside `parents` refuses
    `InvalidMergeKeeper` before commit.
  - `Preserve`: target must exist at basis and not be newly created from
    nothing; `Retire`: target must exist and be retired in this commit.
- **Advisory stays advisory (Arch 10):** Similarity/heuristics never emit
  lineage events; only an explicitly admitted posture produces continuity.

### Operational behavior and cutover

1. Output-role lowering attaches typed `Split` / `Merge` postures to both entity
   and relation mutation intents.
2. Relational `commit_finalization.rs` validates every entity and relation
   posture against the transaction's actual net mutation set, mints any new
   successor names (Phase 1) or preserves the admitted merge keeper's name, and
   appends canonical `LineageEventRecord`s (`Create`, `Replace`, `Split`,
   `Merge`, `Retire`) with full `sources` and `targets`.
3. Replay audits verify that re-running the same commit produces identical
   lineage event rows.

### Ownership and meaningful placement

- `workspaces/worth-query/crates/worth-query-declaration/src/application_operation/binding/output_role/`:
  extend `action.rs` and `family.rs` with `Split` and `Merge` postures and
  relation output roles.
- `crates/worth-relational/src/lineage/authority/`: extend
  `commit_finalization.rs` and `event_emission.rs` with relation lineage
  emission and posture-vs-effect validation.

### Phase 2 test matrix

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|
| Entity & relation Split/Merge happy path (Arch 6, 12) | Model with face + 2 parallel seam relations | Split 1 face into 2 pieces (with distinct discriminators); merge 2 faces keeping parent 1; merge 2 relations minting a new name | Canonical `Split` and `Merge` `LineageEventRecord`s recorded for both entities and relations with exact `sources`/`targets` and names | Exposes missing relation lineage (`MutationIntent::Relation(_) => {}`) or broken merge keeper naming | Decl/Host court (`persistent_naming::lineage`) |
| Corruption court per posture (Arch 4, 16; Testing 4) | Valid base entities & relations | Submit 5 invalid claims: `Split` without retiring parent, `Merge` with `KeepParent` outside `parents`, `Merge` with live unretired non-keeper, `Preserve` on fresh create, `Retire` on unretired entity | Each refuses typed (`LineagePostureMismatch` / `InvalidMergeKeeper`) with zero committed effects; dropping validation makes every court go red | Proves effect validation is real and cannot be bypassed | Decl/Host court (`persistent_naming::lineage`) |
| Parallel same-role relations distinguished by discriminator | Two edges/seams between the same `(FaceA, FaceB)` pair | Create both with distinct discriminators; split one without affecting the other | Distinct `WorthQueryName`s; splitting seam 1 leaves seam 2's name untouched | Exposes endpoint-only relation naming collision | Decl/Host court (`persistent_naming::lineage`) |
| Many-to-many lineage conservation laws (`proptest` 1.x; Testing 2) | 32–64 bounded `proptest` sequences (`proptest-regressions/` committed) of `Create`, `Preserve`, `Split`, `Merge`, `Retire` on entities + relations | Apply random valid posture graphs and inspect committed `LineageEventRecord`s vs live/retired sets | Every live named entity/relation has exactly one live lineage head; every retired source appears in `Retire`/`Split`/`Merge` `sources`; zero dangling targets or duplicate live names | Exposes edge-case accounting bugs when Splits and Merges chain in one commit or across commits | Relational + Decl/Host court (`persistent_naming::lineage`) |

### Completion gate and documentation

- **Verification:** `worth-relational` lineage tests, `worth-query-court-decl-host`
  (`persistent_naming::lineage`), `./loop gate --step guard-diff`,
  `--step line-caps`, check-compile of dependent targets.
- **Docs:** Update `docs/build-an-application.md` output-posture section with
  `Split`, `Merge`, and relation lineage.

---

## Phase 3: Allocation-independent aggregate and group names

### Outcome, scope and dependencies

`worth-relational`'s existing `AggregateIdentity` owner is re-keyed so group
names (orbits, connected components, and declared groups such as face loops,
shells, or spaces) are allocation-independent, worker-count-invariant, and
rebuild-stable:
- `mint()` derives `AggregateIdentity` from `(group_scheme, anchor_name_identity,
  continuity_rule)` — dropping `CommitId` and native `(partition, local_slot,
  generation)`.
- The birth anchor is chosen by declared anchor role or least member
  `WorthQueryName` identity (never native slot order).
- Split and merge keeper selection uses the group kind's declared continuity
  rule from a closed, consumer-driven set (`Anchor`, `Boundary`, `ProducerRole`)
  instead of native member order.
- A group that dissolves and later re-forms around the same anchor revives under
  its original name.
- Group split/merge/birth/retirement events move from `stored_lineage.rs` into
  the canonical commit-envelope lineage events so entities, relations, and
  groups share one lineage authority.

**Requires:** Phase 1 (can run in parallel with Phase 2).
**Out of scope:** Group resolution queries (Phase 4).

### Target API and caller DX

**Extended aggregate/group declaration (`worth_query_declaration::facade::application_naming` & `worth_relational::facade::aggregate_identity`):**

```rust
/// Closed consumer-driven continuity rule for group split/merge keeper selection.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationGroupContinuityRule {
    /// Follows its anchor carrier (e.g., a loop follows its host face).
    Anchor,
    /// Follows its enclosing boundary members (e.g., a space follows its walls).
    Boundary,
    /// Follows its owning producer role (e.g., a shell follows its owning body feature).
    ProducerRole,
}

// Installed group scheme binds group kind + optional anchor role + continuity rule:
// AggregateIdentity::from_group_name(scheme, anchor_name, continuity_rule)
// (No CommitId, no partition, no local_slot, no generation.)
```

- **Compatibility closure (Arch 21):** Migrate portable record grammar 18 and
  keep the accepted `AggregateIdentity::bytes()` surface intact while updating
  the canonical hash domain tag (explicit version bump, never silent
  reinterpretation of old bytes).

### Authority, typed progression and architectural laws

- **One identity per aggregate (Arch 10, 12):** Re-keys the existing
  `worth-relational/src/aggregate_identity/` owner rather than adding a second
  group-identity subsystem.
- **Projected members named by carrier name (Arch 10):** Projected group members
  derive their anchor identity from the projection declaration's scheme and
  their carrier's `WorthQueryName`, never from domain/case strings or native
  slot order.
- **One lineage authority (Arch 18):** `stored_lineage.rs` moves into canonical
  commit-envelope lineage events; `latest_identity()` is superseded by Phase 4's
  full lineage resolution.

### Operational behavior and cutover

1. On group birth, Relational identifies the birth anchor (declared anchor role,
   else member with least `WorthQueryName`), hashes `(scheme, anchor_name,
   continuity_rule)`, checks for same-commit collision between two distinct live
   groups claiming one name (refuses typed), and emits a group birth lineage
   event.
2. On group split or merge, Relational evaluates the declared
   `ApplicationGroupContinuityRule` to pick the keeper (or mints new names when
   tied/unkept) and emits canonical `Split`/`Merge` lineage events.
   An unkept successor receiving members from live prior groups is named under
   `worth.relational.group-aggregate-transition.v2\0` from `(scheme, anchor,
   rule, count-prefixed sorted prior identities)`; only source-free groups use
   the pure `(scheme, anchor, rule)` name, and a source-free name equal to any
   live or same-commit-retiring prior group refuses typed.
   A continuity role counts only when the member holds it before and after,
   and every proposed group must contain its rule's required role member.
3. Re-forming a previously retired group around the same anchor revives its
   original `AggregateIdentity` cleanly.

### Ownership and meaningful placement

- `crates/worth-relational/src/aggregate_identity/`: update `transition/mint.rs`
  (drop `CommitId` and native carrier coordinates), `transition.rs` (continuity
  rule keeper choice), and retire separate `stored_lineage.rs` into
  `lineage/authority/commit_finalization.rs`.
- `workspaces/worth-query/crates/worth-query-declaration/src/application_naming/`:
  add `group.rs` (`ApplicationGroupContinuityRule` and group scheme builder).

### Phase 3 test matrix

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|
| Split & merge under each continuity rule (Arch 10, 12) | 3 group fixtures: `Anchor`, `Boundary`, `ProducerRole` | Split and merge groups while permuting native member creation order | Keeper matches declared continuity rule in every case; reverting to native-order keeping makes the reorder court go red | Proves continuity rule owns keeper selection, not slot order | Relational + Decl/Host court (`aggregate_identity`, `persistent_naming::groups`) |
| Allocation & rebuild invariance (Arch 17) | Same group topology built in 2 fresh processes with reversed member creation order and extra dummy allocations | Compare minted `AggregateIdentity` bytes across both runs | Byte-identical `AggregateIdentity` across all member orders and worker counts | Exposes residual `CommitId` or `RecordId` hash leakage in `mint.rs` | Decl/Host court (`persistent_naming::groups`) |
| Re-forming revival & live collision guard | Group `G` around anchor `A`; retire `G`, then re-form around `A`; separately attempt 2 simultaneous groups claiming anchor `A` | Check revived identity; check collision refusal | Revival produces exact original `AggregateIdentity`; simultaneous collision refuses typed naming both groups | Exposes broken revival or silent duplicate group aliasing | Relational court (`aggregate_identity`) |
| Permutation invariance property (`proptest` 1.x; Testing 2) | 32–64 bounded `proptest` group graphs (`proptest-regressions/` committed) with random splits/merges and member insertion orders | Permute insertion order of members and independent commits before running each split/merge step | Every group's `AggregateIdentity` and lineage events are invariant under all valid member insertion permutations | Exposes hidden native-order tie-breaking in keeper or anchor selection | Relational court (`aggregate_identity`) |

### Completion gate and documentation

- **Verification:** `worth-relational` `aggregate_identity` + lineage tests,
  `worth-query-court-decl-host` (`persistent_naming::groups`),
  `./loop gate --step guard-diff`, `--step line-caps`.
- **Docs:** Update `workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md`
  (or active relation modeling docs) to remove the "names are not portable after
  fresh installation" and slot-order limits.

---

## Phase 4: Tracked local name resolution and Bridge continuity

### Outcome, scope and dependencies

Applications read and resolve persistent names through ordinary, unprivileged,
dependency-tracked Query readers:
- `reader.name_of(handle)` returns the entity's, relation's, or group's
  `WorthQueryName<K>` as a tracked read.
- `reader.resolve(&held, selection)` resolves a held `WorthQueryName<K>` at the
  reader's basis using a declared `NameSelection` (`Exact`, `AllDescendants`,
  `BoundedByCarriers`, `KeepSide`), returning `ResolvedName::{Exact, Plural, Retired}`
  or a typed refusal (`Ambiguous { candidates }`, `BasisPruned`, `UnknownName`).
- Resolution uses a branch-local MVCC derived index (name identity → current
  generational handle + latest lineage event) rebuilt solely from lineage events,
  walking only the lineage path from the held name to the selected basis — so
  resolution cost is **O(lineage path)**, flat as total named records and
  unrelated history grow (up to 1M names).
- `worth-runtime-bridge`'s `ContinuityLineageSource` switches to consume this
  resolution directly. The privileged slot-scan aggregate-lineage path is
  deleted.

**Requires:** Phases 2 and 3.
**Out of scope:** `HistoryDocument` undo/redo/checkout `Restore` events
(Phase 5); Store disk reopen (Phase 7).

### Target API and caller DX

**Proposed reader API (`worth_query_execution` / `worth_query_host::facade::application_naming`):**

```rust
use worth_query_host::facade::application_naming::{
    NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
};

// Ordinary scoped, dependency-tracked read inside a Query / decision / producer reader:
let held: WorthQueryName<Face> = reader.name_of(&face_handle)?;

// Later, after edits, splits, or merges:
match reader.resolve(&held, NameSelection::AllDescendants)? {
    ResolvedName::Exact(live_handle) => use_face(live_handle),
    ResolvedName::Plural(live_handles) => use_faces(live_handles),
    ResolvedName::Retired { at_commit } => report_deleted(at_commit),
}
// When NameSelection::Exact hits a split with 2+ live successors, or a bounded
// selection matches multiple unchosen pieces:
//   Err(NameResolutionDenial::Ambiguous { candidates: Vec<WorthQueryName<Face>> })
// When a bare/unrecorded name or pruned basis is queried:
//   Err(NameResolutionDenial::UnknownName) / Err(NameResolutionDenial::BasisPruned)
```

- **Tracked reads & honest ambiguity (Arch 2, 5; DX 11, 13, 17):**
  Because `resolve` is a tracked read on the decision/query reader, any producer
  or projection that resolves a `WorthQueryName<K>` automatically invalidates
  when a later commit splits, merges, or retires that name's lineage. Plural
  results never silently collapse to a "least member" when the caller asked for
  `NameSelection::Exact`.

### Authority, typed progression and architectural laws

- **Derived index is never authority (Arch 18):** The branch-local
  name-to-generation index is derived MVCC state governed by the commit's
  canonical lineage events; destroying and rebuilding it from events reproduces
  identical resolution answers.
- **Local resolution cost (Perf Core Law 1, 6, 12):** Resolution touches only
  the derived index entry for `held` and the forward lineage path from `held`'s
  event to the reader's basis — never scanning unrelated records or unrelated
  commits. Charged to the calling request under existing execution limits.
- **No privileged slot-scan bypass (Arch 1, 16):** Removes the privileged
  slot-scan lineage lookup on `aggregate_identity` and extends
  `lineage/access/resolution.rs` as the sole walker.

### Operational behavior and cutover

1. On commit, Relational updates the branch-local MVCC derived index for every
   entity, relation, and group touched by the commit's lineage events.
2. `reader.resolve(&held, selection)` checks the derived index at the reader's
   basis:
   - If `held` is currently live at this basis → `ResolvedName::Exact(handle)`
     in O(1) index lookup.
   - If `held` was split/merged/replaced/retired → follows only `held`'s forward
     lineage edges in `lineage/access/resolution.rs` up to the reader's basis,
     applies `selection`, and returns `Exact`, `Plural`, `Retired { at_commit }`,
     or `Err(Ambiguous { candidates })`.
   - If `held` has no birth event visible at this basis → `Err(UnknownName)`.
3. `worth-runtime-bridge`'s `ContinuityLineageSource` resolves UI selections
   through `reader.resolve(...)`.

### Ownership and meaningful placement

- `crates/worth-relational/src/lineage/access/resolution.rs`: extend the single
  lineage walker with name resolution and selection evaluation; add branch-local
  derived name index under `crates/worth-relational/src/lineage/index.rs`.
- `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/`:
  expose tracked `name_of` and `resolve` on ordinary query/decision readers and
  register read dependencies for invalidation.
- `crates/worth-runtime-bridge/src/adapter/continuity_lineage.rs`: switch source
  to name resolution.

### Phase 4 test matrix

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|
| Exact, Plural, Retired, and Ambiguous resolution (Arch 5; DX 11) | Entity/relation/group split into 2, then 1 piece retired, then remaining merged | Resolve with `Exact` vs `AllDescendants` vs `BoundedByCarriers` at each stage | `Exact` refuses `Ambiguous { candidates }` on 2 live pieces (never picks least member); `AllDescendants` returns both; retiring all returns `Retired { at_commit }`; mutating resolver to pick least member turns court red | Exposes silent least-member rebinding or lost candidate lists | Decl/Host court (`persistent_naming::resolution`) |
| Tracked invalidation on lineage change (Arch 2) | Producer `P` resolves `held` via `reader.resolve`; later commit splits `held` | Run edit envelope committing the split | `P` is marked dirty and re-evaluates against the split outcome; unrelated producer `Q` is not re-run | Exposes untracked name resolution causing stale producer outputs | Decl/Host court (`persistent_naming::resolution`) |
| Derived index rebuild equivalence (Arch 18) | Branch with splits, merges, retirements | Destroy derived name index, rebuild from lineage events, compare `resolve` on every held name | 100% identical `ResolvedName` and refusal outcomes before and after rebuild | Exposes authority leaked into the derived index | Relational court (`lineage::index`) |
| Local resolution cost scaling (Perf Core Law 1, 12) | 1,000 vs 20,000 unrelated named records + 50 vs 500 unrelated commits | Resolve a held name with a 3-hop lineage path in both worlds; compare charged work units | Flat growth class across both total-record count and unrelated-history length (within bounded log headroom) | Exposes O(N) record scan or O(H) history scan | Cost court (`cost::persistent_naming`) |
| Bridge `ContinuityLineageSource` integration | Bridge UI selection holding a named entity through split & merge | Advance selection through split (reports split candidates) and merge (reports merged survivor) | Continuity events match Relational lineage exactly | Exposes Bridge/Query continuity mismatch | Bridge court (`worth-runtime-bridge`) |
| Resolution consistency laws (`proptest` 1.x; Testing 2) | 32–64 bounded `proptest` lineage DAGs (`proptest-regressions/` committed) of entities, relations, and groups | After each commit, resolve every previously minted `WorthQueryName<K>` against a simple graph-reachability oracle over the commit's canonical lineage events | Derived-index `resolve` agrees 100% with direct event-graph reachability for every name, every `NameSelection`, and every historical basis | Exposes subtle index-maintenance drift across complex Split→Merge→Retire chains | Relational + Decl/Host court (`persistent_naming::resolution`) |

### Completion gate and documentation

- **Verification:** `worth-relational` lineage/index tests,
  `worth-query-court-decl-host` (`persistent_naming::resolution`),
  `worth-runtime-bridge` continuity tests, cost court, `./loop gate --step guard-diff`,
  `--step line-caps`.
- **Docs:** Update `docs/build-an-application.md` (Decl/Host reader naming
  section) and `docs/capability-index.md`.

---

## Phase 5: History, correction, and rebuild continuity

### Outcome, scope and dependencies

Persistent names survive every in-process temporal and rebuild transition
without rewriting any historical lineage event:
- **Undo, redo, and authored restore** append a new `LineageEventKind::Restore`
  event linking a previously retired name to its newly allocated generation.
- **In-commit regeneration** (retiring a holder and re-minting the same name
  identity within one commit, such as regenerating a preserved parent's split)
  records `Preserve` continuity rather than a birth conflict.
- **Live conflict guard:** Minting an identity that still has a live holder not
  retired in the same commit refuses typed as `DuplicateBirth`.
- **History checkout:** Names resolve cleanly at any checked-out
  `HistoryDocument` node (including across recorded retirements and sibling
  branches from [tree-based-undo.md](tree-based-undo.md)).
- **Independent features stay distinct:** Two independently authored occurrences
  producing identical payloads keep distinct names unless explicitly unified by
  an adjudicated `Merge` posture.

**Requires:** Phase 4; [tree-based-undo.md](tree-based-undo.md) Phase 2 for
the cross-retirement `HistoryDocument` checkout integration courts.
**Out of scope:** Store disk reopen and workflow branch merge (Phase 7).

### Target API and caller DX

Extends `crates/worth-relational/src/lineage/data/events.rs` with one variant:

```rust
pub enum LineageEventKind {
    Create,
    Replace,
    Split,
    Merge,
    Retire,
    /// Undo, redo, history checkout, or authored restore re-activating a
    /// previously retired persistent name onto a fresh generational RecordId.
    Restore,
}
```

No new caller ceremony is required on `HistoryDocument` or `reader.resolve`:
existing `undo_step`, `redo_to`, `checkout`, and `reader.resolve(&held, ...)`
automatically follow `Restore` events and bind to the checked-out basis.

### Authority, typed progression and architectural laws

- **Append-only lineage (Arch 8, 14, 18):** Corrections, undos, and checkouts
  only append `Restore` / `Retire` lineage events; prior lineage event bytes are
  immutable and never edited or deleted.
- **Generational safety preserved (Arch 10):** `HistoryDocument` selection
  restores a record at its exact original full-generation `RecordId`
  ([tree-based-undo.md](tree-based-undo.md), Arch 14: ID regeneration on undo
  is a defect), so `Restore` binds the name to a fresh `LineageId` on that
  identity rather than to a new generation. A handle to a record the
  checked-out node does not hold still fails stale-handle checks, and no
  selection replaces another slot identity. (Repaired 2026-11-23, Slice 5B:
  the earlier "fresh generation" clause contradicted the landed undo spec.)
- **Single authority (Arch 1, 10, 18):** Native's owned-graph selection
  commit derives `Retire`/`Restore`/in-place continuity from the canonical
  selection and the branch naming basis; `HistoryDocument` supplies no naming
  plan and Query reconstructs no lineage.

### Operational behavior and cutover

1. When a commit retires a holder and re-mints the exact same identity in the
   same commit → emits `Preserve` continuity (no conflict).
2. When undo, redo, checkout, or authored restore brings back an identity
   retired in an earlier commit → appends `LineageEventKind::Restore` from the
   name's last retired lineage onto a fresh `LineageId` bound to the restored
   record (its exact original identity under `HistoryDocument` selection) and
   updates the branch-local derived index.
3. When any commit attempts to mint an identity whose previous holder is still
   live and not retired in this commit → refuses `DuplicateBirth`.

### Ownership and meaningful placement

- `crates/worth-relational/src/lineage/`: add `LineageEventKind::Restore` in
  `data/events.rs`, re-minting classification (`Preserve` vs `Restore` vs
  `DuplicateBirth`) in `authority/commit_finalization.rs`, and `Restore`
  traversal in `access/resolution.rs` and `index.rs`.
- `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/product_operation/document_history/navigation/`:
  ensure union restoration emits `Restore`/`Retire` lineage events cleanly for
  all restored/re-retired named entities, relations, and groups.

### Phase 5 test matrix

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|
| Append-only lineage across undo/redo/checkout (Arch 8, 14) | `HistoryDocument` with create → split → delete → undo delete → branch edit → checkout delete → checkout branch | Snapshot serialized prior lineage event bytes before each navigation; verify `resolve` on every held name after each step | Zero prior lineage bytes mutated; every held name resolves to exact live generation on live branch or `Retired` on deleted branch; stale pre-undo `RecordId` still rejected | Exposes mutated historical lineage or stale-generation aliasing | Decl/Host court (`persistent_naming::history`) |
| Regeneration vs restore vs live conflict | Split entity regenerated on edit; retired entity restored; live entity re-minted without retiring | Run all three commits | Regeneration records `Preserve`; restore records `Restore`; live re-mint refuses `DuplicateBirth` | Exposes conflation of regeneration with conflict or restore | Relational + Decl/Host court (`persistent_naming::history`) |
| Independent occurrences stay distinct | Two separate authored occurrences creating identical payloads | Inspect names of both occurrences' outputs | Distinct `WorthQueryName`s (different occurrence keys); no silent unification | Exposes payload-digest aliasing across independent occurrences | Decl/Host court (`persistent_naming::history`) |
| Hostile interleaving state-machine laws (`proptest` 1.x; Testing 2) | 16–32 bounded `proptest` sequences (`proptest-regressions/` committed) interleaving create, split, merge, regenerate, delete, undo, redo, and arbitrary `checkout` | Hold every minted `WorthQueryName<K>` across the entire sequence; after every step assert `resolve` matches the checked-out node's live/retired state and fresh-process rebuild of the checked-out node yields byte-identical names | 100% agreement at every step; zero silent rebinds; zero mutated prior lineage events | Exposes subtle interaction bugs between `Split`/`Merge` lineage and `HistoryDocument` branching checkouts | Decl/Host court (`persistent_naming::history`) |

### Completion gate and documentation

- **Verification:** `worth-relational` + `worth-query-court-decl-host`
  (`persistent_naming::history`), `./loop gate --step guard-diff`,
  `--step line-caps`, check-compile of dependent targets.
- **Docs:** Update `docs/build-an-application.md` to document persistent name
  behavior across undo/redo/checkout and rebuilds.

---

## Phase 7: Durable names on Store reopen and branch merge

> **Progress (2026-11-26, Phase 7B-1, refs #71; not closed):** Store-backed
> crash/reopen at every lineage boundary (checkpoint + tail and Journal only),
> fresh-installation admission and bare-identity refusal, branch-merge
> named-output refusals, `ReopenFromStore` in `PersistentNaming.tla` and the
> durable DST landed, with two durability fixes (checkpoint event pages
> refused in-place keeper merges; Journal replay lost in-commit
> regeneration). Held: group lineage lost on checkpoint reopen for non-head
> commits (Store cold-checkpoint owner). Phase 7B-2/7B-3 follow; this
> milestone stays open.

> **Progress (2026-11-27, Phase 7B-2 to 7B-4 public, refs #71; not closed):**
> The held non-head group reopen loss closed with Store M4.3f's paged `CHK24`
> naming and group state (#440, #468), which also landed retention-floor
> compaction, cutoff-indexed `fork_children` (#461) and copy-on-write table
> forks. Phase 7B-2 landed recorded-at-basis discriminators and structural
> identity (#449), tracked split and awaited-birth facts with delta carriers
> (#460), shared symbol texts (#469) and canonical delta group-lineage frames
> (#475). Step bounds are Lean 4 proofs in `proofs/lean/persistent_naming/`,
> pinned to the shipping Rust and checked against its traces by
> `lake exe conformance` (#477; `ForkChildrenCost` and `CowMapForkCost` for
> `forks_inheriting` and table forks). `worth-court-relational-cold-residency`
> `allocation_proofs::composed_cold_reopen_scaling` composes names, an Anchor
> group and forks over a cold `CHK24` reopen at 1x and 10x, and
> `allocation_proofs::naming_release_ceiling` is the manual release ceiling
> run: at 100,000 names over 10,000 commits with 1,000 groups and 64 branches,
> medians of three runs were cold reopen 1.03 s (104 MiB peak heap), fork
> 11 µs, name and group resolve 0.45 ms, one-member detach 1.2 ms and a birth
> 1.6 ms (77 ms when it first faults a cold partition). The same run found the
> checkpoint paging every fork's inherited tables in full (109,085 naming
> pages, 4.0 GB, a 47 MB manifest); that is the reopen's cost and is open. The
> private 7B-3b adoption and the unified acceptance journey remain; this
> milestone stays open.

*(Phase 6 is the downstream consumer cutover tracked in the consumer repository.)*

### Outcome, scope and dependencies

Extends Phases 1–5 with cross-restart durability and branch-merge safety:
1. **Store durability & fresh-installation admission:** Store persists canonical
   birth and lineage events via Relational's checkpoint and journal restore; on
   reopen or fresh installation of the same schema lineage, the derived
   name-to-generation index is rebuilt from persisted events, retained
   `WorthQueryName<K>` values are admitted only by matching a persisted birth
   event, and a bare identity without a birth event refuses typed.
2. **Branch merge:** Where a branch merge touches named outputs, either the
   workflow branch-merge owner reruns producers under the merged authored inputs
   (checking for name identity collisions at merge and resolving held names from
   either branch against the merged result) or — until producer-rerunning merge
   is active on that branch path — refuses typed naming the conflicting
   occurrence. Never silently rebinds across branches.

**Requires:** Phase 5; Store lineage persistence readiness (Store M3 / Store M7).
**Out of scope:** Physical Store file layout (owned by Store roadmap).

### Authority, typed progression and architectural laws

- **Events are durable truth; index is rebuildable (Arch 8, 18, 21):** Store
  persists canonical lineage events with explicit schema/format version. A
  missing or corrupted derived name index is rebuilt deterministically from
  events on open; unsupported format versions refuse typed before effects.
- **No bare-identity forgery across restarts (Arch 10):** Fresh-installation
  admission verifies that the supplied `WorthQueryName<K>` matches a persisted
  birth event whose typed parts re-encode to its 32-byte identity.

### Operational behavior and cutover

1. Checkpoint & journal persistence serialize birth parts, minted identities,
   and `LineageEventKind` (`Create`, `Replace`, `Split`, `Merge`, `Retire`,
   `Restore`) atomically with each commit.
2. On crash recovery or clean reopen, Relational replays checkpoint + journal
   lineage events to rebuild the derived name-to-generation index before serving
   reads.
3. On branch merge, collision checks run across merged birth events; any
   unsupported named-output merge path refuses typed naming the occurrence.

### Ownership and meaningful placement

- `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore.rs`:
  persist and replay name birth/lineage events and trigger derived index rebuild.
- `crates/worth-relational/src/lineage/index.rs`: deterministic rebuild from
  replayed events.
- `workspaces/worth-store/`: integrate event persistence through existing Store
  durability boundaries.

### Phase 7 test matrix

| Requirement / law | Setup and entry | Action or failure sequence | Expected result and effects | Decisive observation / defect exposed | Test boundary and lane |
|---|---|---|---|---|---|
| Crash & reopen at every lineage write boundary (Arch 8, 22) | Durable Store-backed home executing create, split, merge, retire, and restore | Kill/reopen after each commit boundary; resolve every held `WorthQueryName<K>` | Identical resolution outcomes before and after reopen; zero lost or partial lineage events | Exposes torn lineage persistence or stale index after recovery | Store + Decl/Host recovery court (`worth-store-court-application`) |
| Fresh-installation admission & bare-identity refusal (Arch 10, 21) | Reopen retained names in a fresh installation of the same schema lineage | Present a genuinely recorded `WorthQueryName<K>` vs a fabricated bare 32-byte identity with no birth event | Recorded name resolves identically; fabricated bare identity refuses `UnknownName` | Exposes unverified bare-identity acceptance | Decl/Host + Store court |
| Corrupted derived index rebuild (Arch 18) | Persisted Store home with multi-step lineage | Discard/corrupt derived name index before open; reopen and resolve all held names | Index rebuilds from lineage events and resolves 100% identically | Exposes hidden durability dependence on derived index | Relational + Store recovery court |
| Branch merge collision & refusal honesty (Arch 5, 10) | Two diverged branches modifying named outputs | Merge branches with producer-rerunning merge (or without, verifying typed refusal) | Without producer rerun: refuses typed naming occurrence; with producer rerun: held names from either branch resolve cleanly or refuse typed on collision | Exposes silent cross-branch rebinding | Decl/Host court (`persistent_naming::branches`) |

### Completion gate and documentation

- **Verification:** Store recovery courts, `worth-relational` checkpoint/restore
  tests, `worth-query-court-decl-host` (`persistent_naming::branches`),
  `./loop gate --step guard-diff`, `--step line-caps`.
- **Docs:** Update `docs/capability-index.md` and close `#71`.
