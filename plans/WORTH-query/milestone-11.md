# Milestone 11: Runtime Persistent Naming Over Entities, Relations And Aggregates

**Status:** design brief, not scheduled. Nothing here is implemented or
qualified. It is in the plan so that generic naming and lineage are built once,
in the runtime, instead of separately in each application. Milestone 11 is new
scope. It is not a revival of the Query 10–13 portfolio that
[milestone-10.md](milestone-10.md) removed, and it does not displace Milestone
10, 9.18.2 history work or the Store roadmap. Proposed names are targets, not
existing APIs or passed tests.

## Goal and scope

An application declares what its things mean: an entity, a relation, or a
group of records. The runtime then gives each of them a persistent name. The
name keeps meaning the same thing through:
- edits and regeneration;
- undo, redo and history checkout;
- feature reorder;
- a from-scratch rebuild;
- branches and merges;
- a process restart from Store.

**What the application supplies** is domain meaning only:
- which outputs correspond to which predecessors;
- which group kinds exist;
- what keeps a group "the same";
- typed discriminators for split pieces.

**What the runtime owns** is the rest:
- minting names;
- many-to-many lineage;
- resolving a held name to live records;
- typed refusals when a name has no unique answer.

The first consumer is WORTH Proprietary CAD, which names faces, edges, loops,
shells and features. The same capability serves BIM elements and spaces, MEP
runs and systems, and UI selection continuity.

Scope covers:
- Relational: name minting, lineage and resolution authority;
- Query: declaration lowering, posture and reads;
- Bridge: continuity;
- Store: Milestone 7 durability, which this milestone consumes.

Applications keep using Decl/Host.

## Adversarial constraint

A held name must keep resolving to the same meaning through an arbitrary
interleaving of:
- edits, regenerations, splits and merges;
- undo, redo and checkout;
- feature reorder;
- a fresh-process rebuild;
- replay at any worker count;
- a branch merge.

When it cannot, it returns a typed refusal that names its candidates or its
retiring commit. It must never rebind silently.

The names themselves must satisfy three properties:
- **Allocation independence.** Names never depend on allocation, worker order,
  slot reuse or branch.
- **Bounded size.** Names stay bounded in size however many times their subject
  has been split.
- **Local resolution.** Resolution cost grows with the lineage path from the
  name to the selected basis, not with history length or the number of named
  records. This must hold at a million names.

## Generational IDs and names: two layers, one authority

`RecordId { partition, local slot, generation }` stays the only live handle: it
is stale-safe and unforgeable. Restoration keeps allocating a fresh generation
under a retained logical key ([build-an-application.md](../../docs/build-an-application.md),
authored restoration), so an old handle never aliases a new incarnation.

A persistent name is the stable meaning above the handle. It resolves to the
current generation at a basis. A name is never an ID, and an ID is never a
name, for three reasons:
- **Restore.** It changes the generation but keeps the meaning.
- **Rebuild.** A rebuild allocates differently but must reproduce the names.
- **Branches.** Two branches can allocate one slot to different records.

**Single authority.** The authority is the lineage events in the canonical
commit envelope. Every birth event carries its name, and that one place covers
entities, relations and groups alike. Per-record name reads and the
name-to-generation index are derived. They can be rebuilt from events, and
Store persists events, not indexes.

## What the investigation established

These are actual guarantees on master `5ca4739320`. Paths are relative to the
repository root. `<effect_program>` means `workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_attempt/effect_program`.

| Existing owner and source | Starting guarantee | Gap and required change |
|---|---|---|
| `crates/worth-relational/src/identity/data/record_ids.rs` | Generational native IDs | Reused unchanged as the live handle. |
| `crates/worth-relational/src/lineage/` `data/events.rs`, `authority/commit_finalization.rs` | `LineageEventKind` has Create, Replace, Split, Merge and Retire, with many-to-many sources and targets. | Finalization drafts only Create, Retire and Replace. Relation mutations write no lineage (`MutationIntent::Relation(_) => {}`). 11.2 writes the rest. |
| `crates/worth-relational/src/lineage/access/resolution.rs` (`event_positions_for_sources`), `crates/worth-relational/src/change_source/lineage_at_observation.rs` | An indexed lineage walk over sources, and lineage at an observation. | 11.4 extends these into name resolution. No second walker. |
| `crates/worth-relational/src/durability/authority/runtime_rebuild/checkpoint_restore.rs` | Lineage is checkpointed and restored. | 11.6 starts from this and Store Milestone 7. |
| `crates/worth-relational/src/aggregate_identity/` (`transition.rs`, `transition/mint.rs`, `stored_lineage.rs`) | Orbit and component names, split and merge lineage, worker-count determinism (9.18.1). | The name hashes the anchor's native partition, slot and generation, and the keeper is chosen by native member order, so names depend on allocation. Lineage reads are privileged and start with a slot scan; `latest_identity()` follows merges only; names are not portable after a fresh installation ([relation-laws.md](../../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md)). 11.3 re-keys this owner. It adds no second group identity. |
| Query output posture `<effect_program>/output_correspondence/candidate.rs` (`WorthQueryApplicationOutputPosture`) and role families (`workspaces/worth-query/crates/worth-query-declaration/src/application_operation/binding/output_role/family.rs`) | Create, Preserve and Retire by role or family member. | No Split or Merge posture. 11.2 extends this vocabulary and adds no parallel intent set. |
| Authored identity families ([semantic-correction-authored-identity.md](../../docs/plans/semantic-correction-authored-identity.md)) | A logical key declared once survives retire and restore. | Covers authored roots only. In 11.1 these keys become the producer part of every name. |
| 9.14 `workspaces/worth-query/crates/worth-query/src/domain_installation/operation_lineage/naming.rs` | `admit_persistent_name` binds a name that the domain wrote to trace-bound evidence. | Consumed by one court and no Decl/Host application. Superseded in 11.4. |
| Milestone 7 `workspaces/worth-query/crates/worth-query/src/identity_evolution/` | Typed antecedent, successor, split, merge and break vocabulary, with advisory honesty. | `execution/lineage.rs` derives its outcomes from `IdentityEvolutionSyntheticScenario`, not from Relational lineage. 11.4 replaces that executor with real lineage and keeps the vocabulary. |
| `crates/worth-runtime-bridge/src/adapter/continuity_lineage.rs` | `ContinuityLineageSource`, which owns UI selection continuity, with single, split, merge and ambiguous topologies. | In 11.4 it consumes name resolution as its source. |
| `crates/worth-relational/src/mvcc/retained_entity_selection.rs`, 9.18.2 history | Checkout copies payload and keeps current identity. | Covered by 11.5. |
| `workspaces/worth-query/crates/worth-query/src/ordinary/workflow/branch_merge/` | Workflow-family merge declaration: a target and a source branch. | It does not rerun producers under merged inputs. 11.5 adds this as a requirement on this owner. |
| [Store roadmap Milestone 7](../worth-store/runtime-integration-roadmap.md) | Persists structural identity and lineage and correspondence events. | 11.6 defines what it persists for names. |

**Consumer evidence** (private repository, not verifiable here). WORTH
Proprietary CAD implements its own copies of four things:
- a name graph;
- lineage-fact storage;
- held-reference resolution (all descendants, or bounded by a carrier, with an
  unchosen-branch refusal);
- least-member merge survival.

Only its correspondence decisions and role vocabulary are CAD-specific.

## Ownership decisions and supersessions

1. **The domain declares policy; Relational mints.**
   - The domain states naming policy as installed declarations: name schemes,
     continuity rules, postures and discriminators.
   - Relational evaluates them and mints names. Query lowers the declarations
     and stays domain-agnostic, consistent with
     [runtime-generic-graph-authoring-plan.md](runtime-generic-graph-authoring-plan.md):
     "lower runtimes remain authoritative for … naming".
   - No handler, provider or restore path can write a name.
   - This **supersedes** 9.14's binding of domain-executed naming mutations
     (`milestone-9.14.md`, "Persistent naming policy remains domain-owned; Query
     owns the proof-bearing binding").
2. **One lineage story.** The envelope's lineage events are the canonical
   artifact. 9.14's trace-bound lineage evidence, Milestone 7 queries, Bridge
   continuity, replay and invalidation all derive from them. This supersedes
   9.14's separate trace-bound evidence as the naming authority.
3. **Advisory stays advisory.** Similarity may propose a correspondence for
   adjudication. It never becomes continuity without an admitted posture.
   "Never rebinds silently" is therefore only as strong as the domain's
   postures. Validation checks Preserve and Merge keepers against the
   performed effects, not against their meaning.
4. **Plural stays plural.** Choosing among split successors needs a declared
   selection rule or a typed refusal.
5. **No correction rewrites lineage.** Corrections append events and never edit
   old ones.

**Closure duties.** These documents must be updated:
- `milestone-9.14.md` warnings;
- the Current Limits in
  `workspaces/worth-query/crates/worth-query/docs/domain-capabilities/installed-operation-lineage-and-promotion.md`;
- the naming wording in `runtime-generic-graph-authoring-plan.md`;
- the 9.18.1 aggregate identity documentation.

## Name grammar

Every name is built the same way. Proposed type: `WorthQueryName<K>`, opaque and
typed by kind. Its parts are:
- the declared scheme;
- the producer's authored occurrence key, from an authored identity family. It
  is never a bare operation identity, so two fillets never collide;
- the declared role path. Nested family members are added once #32 lands;
- for a split or merge successor, the identities of its parent names;
- a typed local discriminator, built from Foundational values and from
  references to other names.

**Fixed size.** Every referenced name enters as its fixed-size canonical
identity, never as its full parts. Relational encodes and hashes the typed
parts to mint that identity. A collision refuses typed at birth.

**What a name excludes.** A name never derives from:
- payload, geometry, coordinates or strings;
- allocation, slot, generation, commit position or worker order.

The 9.14 warning still holds: digests of payload or representation are never
continuity authority. The identity hash is only the canonical encoding of
declared meaning, and on its own it is only a selector, as in
[relation-laws.md](../../workspaces/worth-query/crates/worth-query/docs/modeling/relation-laws.md).
A held name is admitted only by matching a birth event whose recorded parts
re-encode to its identity. A bare identity with no event refuses typed.
Identity collisions are checked at birth and again at branch merge.

**Provenance.** The birth event records the full typed parts for inspection,
including the parent identities, so each name's provenance can be read.

**Well-founded discriminators.** A discriminator may reference only:
- names that exist at the posture's basis; or
- outputs of the same occurrence that come earlier in the scheme's declared role
  order.

Installation checks that order for acyclicity. It refuses a cyclic scheme, and
a scheme that declares no discriminator for split successors. Whether the
values make the discriminator unique depends on what is produced, so that is
checked at commit: two successors minting one identity in one commit refuse
typed, naming both. A non-convex face that leaves two pieces on one side
needs a further discriminator, such as the names of the bounding carriers.

**Consequences:**
- **Reorder.** Reordering features keeps every name whose parents are
  unchanged. A reorder that makes a cut split a different parent changes the
  parent identity, so the old name resolves as `Retired` or refuses typed. It
  never rebinds silently to a piece of another face.
- **Branches.** Two branches cutting one face with different features are
  different occurrences, so they mint different names.
- **Relations.** Relations are named the same way, by producer, role,
  discriminator and parents, never by endpoint names. Parallel same-role
  relations, such as a seam or two edges between one pair of faces, differ by
  discriminator.

## Phases

| Phase | Usable result | Requires | Parallel constraints |
|---|---|---|---|
| 11.1 Name schemes | Runtime-minted, bounded names for declared entity, relation and group kinds | Role families, authored identity families | Freeze the grammar before 11.2 and 11.3. |
| 11.2 Many-to-many lineage | Split and Merge postures validated against effects; lineage events for entities and relations | 11.1 | Envelope format change; coordinate with concurrent envelope work. |
| 11.3 Group meaning | `AggregateIdentity` re-keyed to its anchor's name; keeper chosen by declared continuity | 11.1, 9.18.1 | Beside 11.2. |
| 11.4 Resolution | Unprivileged, tracked, local name resolution; Milestone 7 and Bridge continuity consume it | 11.2, 11.3 | Deletes the 9.14 binding. |
| 11.5 Correction, history, branches | Names through undo, redo, restore, checkout and merge | 11.4, 9.18.2, workflow branch merge rerunning producers | After the 9.18.2 history-tree work settles. |
| 11.6 Durable names | Store reopen and fresh-installation admission of retained names | 11.4, Store Milestone 7 | Store owns persistence. |

Every phase keeps the existing request semantics:
- **Interruption.** Minting and lineage writes are part of the commit, so a
  stopped request leaves no partial name or event.
- **Retry.** A retry mints identical names, because names are derived and never
  allocated.
- **Cost.** Resolution and index maintenance are charged to the original
  request under its existing limits.

Each phase updates the consumer guide for what it ships: the build-an-application
naming section, the relation-laws aggregate section and the Decl/Host reader
docs.

### 11.1 Name schemes

- **Owner.** Relational mints the name at the birth event. Query lowers the
  scheme declaration and binds it to output roles and families.
- **Commit check.** A forged or colliding name refuses typed at commit, naming
  both holders.
- **Tests:**
  - an identical history built in two fresh processes, and at 1, 2 and 8
    workers, gives byte-identical names;
  - a feature reorder keeps every name whose parents are unchanged;
  - a reorder that moves a cut onto another parent retires or refuses the held
    name instead of rebinding it;
  - installation refuses a cyclic scheme or one with no split discriminator,
    and a commit producing duplicate identities refuses;
  - regenerating a preserved parent's split keeps its piece names as Preserve;
  - a handler that writes a name fails to compile;
  - a forged birth refuses.

### 11.2 Many-to-many lineage

- **Postures.** `WorthQueryApplicationOutputPosture` gains
  `Split { parent, discriminator }` and `Merge { parents }`. Create, Preserve
  and Retire stay; Replace stays Relational's `EntityMutationIntent::Replace`.
  Relations get the same postures and lineage nodes.
- **Validation.** Commit validates each posture against the performed
  creations, retirements and endpoint changes, then writes Split and Merge
  events with full sources and targets. A contradicting claim refuses typed,
  naming the posture and the effect.
- **Merge naming.** The `Merge` posture names its keeper: one of its parents, or
  none. A named keeper keeps its name; with none, a new name is born with the
  parent identities. A keeper outside the parents refuses typed. This is the
  domain's correspondence decision, the same kind as choosing what to
  Preserve. Groups take their keeper from 11.3's continuity rules instead.
- **Tests:**
  - a corruption court per posture;
  - mutation: drop the validation, and the courts go red;
  - replay audits compare the lineage rows.

### 11.3 Group meaning

- **One identity per aggregate.** The existing `AggregateIdentity` owner stays
  the single identity of an orbit, component or declared group.
- **Allocation-independent minting.** `AggregateIdentity` becomes the group
  kind's name identity in the grammar above. Groups have no authored
  occurrence, so their parts are:
  - the group scheme;
  - the birth anchor's name identity, in place of the native carrier;
  - the continuity rule.

  The commit occurrence is dropped.
- **Birth anchor.** The anchor is the member that fills a declared anchor role,
  or, when the scheme declares none, the member with the least name identity.
  It is never chosen by native order.
- **Projected members.** A projected member's anchor is named by the projection
  declaration's scheme and its carrier's name, not by domain or case strings.
- **Re-forming.** A group that forms again around the same anchor is revived
  under its old name. This is intended.
- **Compatibility closure duties.** Migrate portable record grammar 18 and the
  accepted `identity()` / `bytes()` API.
- **Keeper choice.** On split and merge, the keeper is chosen by the group
  kind's declared continuity rule, not by native member order.
- **Continuity rules.** The closed set is decided at phase start from real
  consumers. Candidates:
  - **anchor:** a loop follows its face side;
  - **boundary:** a space follows its enclosing walls;
  - **producer role:** a shell follows its body feature.
- **Lineage.** `stored_lineage.rs` events move into the envelope lineage, so
  there is one story.
- **Tests:**
  - split and merge under each rule;
  - group names survive undo and restore;
  - two groups claiming one name refuse;
  - a group re-forming around its anchor is revived under its old name;
  - mutation: revert to native-order keeping, and the reorder court goes red.

### 11.4 Resolution

Target caller shape (proposed):

```rust,ignore
let held: WorthQueryName<Face> = reader.name_of(face)?;      // tracked read
// ... edits, undo, checkout ...
match reader.resolve(&held, Selection::AllDescendants)? {      // tracked read
    Resolved::Exact(handle) => use_face(handle),
    Resolved::Plural(successors) => use_faces(successors),
    Resolved::Retired { at } => report_deleted(at),
}
// Ambiguous { candidates } and Unavailable arrive as typed refusals.
```

- **Selection rules.** These are declared on the reference: all descendants,
  bounded by named carriers, or keep a named side.
- **Reads.** Resolution is an ordinary scoped and tracked read, so a decision
  that resolved a name revalidates when its resolution changes.
- **Locality.** The derived index is ordinary branch-local MVCC state, so a
  historical basis reads its own version. Retention follows that basis, and a
  pruned basis keeps its existing typed refusal. Resolution walks only the
  lineage path, extending `lineage/access/resolution.rs`. The privileged
  slot-scan path is removed.
- **Consumers.** Milestone 7's executor reads real lineage, and
  `ContinuityLineageSource` resolves through names.
- **Deletions.** The 9.14 `admit_persistent_name` binding and its court are
  deleted.
- **Tests:**
  - resolution cost stays flat as history and record count grow (growth class
    with headroom, never exact counts);
  - ambiguity refuses with candidates;
  - mutation: resolve by least member, and the courts go red.

### 11.5 Correction, history and branches

- **Restore.** Undo, redo and authored restore append a proposed `Restore`
  lineage kind that links the retired name to its new generation.
- **Re-minting.** A name's identity includes its occurrence and its parents.
  - **Regeneration.** A commit that retires a holder and mints the same
    identity again records `Preserve` continuity, as when a preserved parent's
    split is regenerated.
  - **Correction.** Re-minting the identity of a holder retired by an earlier
    commit, through undo, redo or authored restore, records `Restore`.
  - **Conflict.** Minting an identity that still has a live holder not retired
    in the same commit refuses as a conflicting birth.
- **Checkout.** Names resolve at the selected step.
- **Branch merge.** The merge owner is Query's workflow branch merge
  (`workspaces/worth-query/crates/worth-query/src/ordinary/workflow/branch_merge/`).
  Today it declares a target and a source branch, and nothing in it reruns
  producers under the merged inputs. This phase therefore requires that owner
  to rerun the application's ordinary producers under the merged authored
  inputs. That is a new requirement on the owner, not an existing guarantee.
  Held names from either branch then resolve against the result, or refuse
  typed, naming the occurrence whose outputs differ.
- **Until that exists,** a branch merge that touches named outputs refuses
  typed. Generic Native branch reconciliation keeps its existing typed
  refusal.
- **Independent features stay distinct.** Two independently authored features
  with the same meaning keep two names. Unifying them takes an adjudicated
  correspondence posture; similarity alone never does it.
- **Tests:**
  - a hostile undo/redo/checkout/merge interleaving: every held name resolves
    or refuses typed;
  - byte comparison shows that no old lineage row changes.

### 11.6 Durable names

- **What Store persists.** Store Milestone 7 persists the lineage events,
  extending the existing checkpoint restore. The index is rebuilt from them.
- **Fresh installations.** Retained names become admissible in a fresh
  installation of the same schema lineage.
- **Tests:**
  - crash and reopen at each lineage write boundary resolves identically;
  - fresh-installation admission matches a recorded birth event, and a bare
    identity without its event refuses;
  - a corrupted index is rebuilt from events.

## Acceptance

A public cell-complex fixture, extended with groups and relations, and the
proprietary CAD consumer both pass one journey:
1. Hold names to a face, an edge, a seam relation, a loop, a shell and a
   feature.
2. Split, merge, regenerate and reorder.
3. Undo, redo and check out.
4. Merge a branch. Without the producer-rerunning merge this step must refuse
   typed; with it, held names resolve.
5. Rebuild in a fresh process and reopen from Store.
6. Check that every held name resolves to its meaning or refuses typed with
   its candidates.

The proprietary consumer then deletes its generic naming machinery and keeps
only postures, discriminators and role vocabulary.

## Out of scope

- **Domain vocabularies and geometric correspondence algorithms.** These stay
  application meaning.
- **Ordered structural-event membership** for "live at step k" reads (#70). It
  is adjacent; 11.4 may consume it.
- **Physical persistence formats.** The Store roadmap owns them.
