# Milestone 9.14 Phase 21 Operational Identity Inventory

This inventory defines the migration closure for Phase 21. Each slice is
inventoried before its production path is changed. A slice is closed only when
its source owner mints the operational identity, every trust crossing weakens
or re-admits it explicitly, and copied representation cannot satisfy the
downstream operational API.

The adversarial constraint is uniform across all slices: a caller may copy
every label, digest, serialized field, debug string, numeric handle, and
projection key; may deliberately collide derived lookup keys; and may retain
stale handles across lifecycle changes. Those representations may support
reporting or candidate selection, but none may open an operational door without
current owner authority.

## Category Contract

| Category | Owner | Permitted power |
| --- | --- | --- |
| owner operational identity | Query, Relational, Runtime Bridge, or Signal | owner-named operations after current validation |
| Foundational admitted identity value | the owning runtime | intermediate admission only |
| Foundational authority identity | the owning runtime | shared typed carriage beneath a stronger owner artifact |
| Foundational boundary-bridged identity | the source owner | boundary transport; never current operational use |
| Foundational revalidated identity value | the source owner | reviewed input to explicit readmission |
| Foundational projection identity | no operational owner | display, diagnostics, and candidate lookup only |
| Foundational digest identity evidence | no operational owner | canonical evidence only |
| Foundational external identity token | external caller or host | unadmitted input only |

`worth-proof` supplies progression, freshness weakening, and witness-bearing
transitions. `worth-foundational` supplies the shared category vocabulary.
Neither replaces the stronger runtime artifact that owns an operation.

## Completion Audit

| Slice | Status | Owner-path closure | Hostile evidence |
| --- | --- | --- | --- |
| Relational truth to Query basis | closed | Relational publication produces owner evidence; Bridge admits and weakens it; Query mints current read identities from the retained truth handoff and current mutation identities only when one Bridge causality bundle matches the receipt's commit, snapshot, collection, Relational record, mutation kind, and Foundational aspect-touch set | raw Relational and Bridge projections cannot mint Query truth by themselves; wrong commit, snapshot, collection, record, kind, or touch evidence is denied; equal copied runtime identities fail named current-identity checks; retained historical admission rejects a wrong snapshot |
| Query conditional declaration through Signal decision | closed | Query declaration enters Bridge installed lowering; Signal mints the decision artifact; Bridge retains the decision; Query consumes the retained handoff | decision projections, copied node/graph coordinates, and raw commit projections cannot satisfy the owner APIs |
| Installed operation and execution progression | closed | Query wraps generic Proof progression with a private Query operation-progression authority identity | copied stage labels and exact-chain splices are rejected; direct and workflow paths preserve one owner law |
| Replay, aftermath, and lineage | closed | exact retained traces drive replay comparison; verified postconditions drive aftermath; exact effect receipts drive owner lineage | wrong target, false postcondition, semantic drift, generic reporting equality, unrelated historical correspondence, and replay-to-ordinary promotion are denied |
| Compatibility, rebind, sharing, and lease | closed | relationship-specific Query compatibility and move-only lease artifacts retain pair, generation, owner, and lifecycle authority | raw sharing registration, lower authority, copied labels, wrong consumer, stale generation, and foreign runtime attacks fail |
| Dependency, invalidation, collection, and delivery | closed for Phase 21 identity scope | compiled dependencies retain owner semantics; current owner epoch and exact lease mint and readmit invalidation authority | raw Foundational evidence cannot admit a delta or consequence; stale epochs and unrelated owners fail; collection-window and patch authority remain separate later capabilities |
| Reporting, diagnostics, indexes, and facades | closed | explicit projections and digests remain one-way; owner markers and raw operational parts are absent from public facades; indexes retain owner records or revalidate selected candidates | audience crates cannot import owner markers, extract raw Query identity parts, compare lineage through generic equality, or promote projections into authority |

Query commit, snapshot, and entity identities retain only the equality, ordering,
and hashing needed for legitimate representation-level collections. Those
traits do not establish currentness. Every operational door uses a named owner
comparison, retained authority artifact, lifecycle generation, or explicit
historical admission.

## Slice 1: Relational Truth To Query Basis

**Source authority**

- `worth-relational` live runtime commit, snapshot, entity, relation, version,
  branch, and workspace truth.

**Current operational path**

- `worth-relational/src/presentation/bridge/identities.rs` constructs Bridge
  truth identities from `CommitId`, `SnapshotId`, `VersionId`, and record
  storage parts.
- `worth-runtime-bridge/src/relational_identity.rs` publicly constructs
  `TruthCommitIdentity` and `TruthSnapshotIdentity` from raw numeric parts.
- `worth-query/src/memory_workspace/identities.rs` reconstructs Query commit,
  snapshot, and entity identities from the same raw parts and derives evidence,
  ordering, hashing, and historical matching from representations.

**Existing but non-participating vocabulary**

- Relational, Bridge, and Query `identity_authority` modules declare
  Foundational category aliases and family maps, but the production path above
  does not require them.

**Authority leaks to remove**

- public raw `from_relational_*` operational constructors;
- public constructible owner authority markers and public witness factories;
- Query ordering, hashing, or historical admission through reporting text;
- Bridge parsing that converts a representation back into source truth.

**Target handoff**

```text
Relational current truth
  -> Relational owner artifact
  -> weakened boundary export
  -> Bridge source admission and retained correspondence
  -> Query owner validation
  -> Query current basis identity
```

**Closure evidence**

- real Relational commit and snapshot publication reach Query through Bridge;
- a mutation receipt becomes current only when one Bridge-owned causality bundle
  retains its exact commit and snapshot plus the same collection, Relational
  record identity, mutation kind, and canonical Foundational aspect-touch set;
- Bridge binds the subject's canonical Foundational patch basis before lowering
  and denies a different effect intent before authority execution;
- raw numeric parts, projections, digests, external tokens, and merely bridged
  identities fail to satisfy the Query basis door;
- a Bridge bundle for a different mutation cannot authorize an otherwise
  well-formed Query delta, and one bundle cannot authorize a multi-delta
  receipt;
- stale/foreign runtime and generation cases are denied;
- projection destruction and rebuild do not change the admitted basis.

## Slice 2: Query Conditional Declaration Through Signal Decision

**Source authorities**

- Query owns portable installed-operation declarations and continuation.
- Runtime Bridge owns installed correspondence and lowering.
- Signal owns graph allocation, installed condition/comparator/reuse identity,
  evaluation attempt, and decision evidence.

**Current operational path**

- Query declarations carry condition/comparator fingerprints and graph-facing
  locations.
- Bridge lowers into Signal graph/node identities.
- Signal `NodeId`, installed condition identity, and installed comparator
  identity expose copy/equality/hash/serialization behavior and graph identity
  material.
- Signal conditional decision identity is assembled as `String` from snapshot
  and execution identity strings plus runtime fields.

**Authority leaks to remove**

- reconstructible graph/node/role tuples substituting for installed authority;
- caller-provided snapshot/execution strings participating in operational
  decision identity;
- serialized condition/comparator identity re-entering resolver admission;
- Query restamping or re-deriving Signal decision authority.

**Target handoff**

```text
Query admitted declaration
  -> Bridge installed lowering authority
  -> Signal graph-owned installed contract
  -> Signal owner evaluation and decision evidence
  -> Bridge retained decision handoff
  -> Query continuation consuming Signal evidence
```

**Closure evidence**

- one real end-to-end installed conditional operation;
- copied graph ids, node ids, role tuples, decision summaries, and colliding
  fingerprints cannot resolve or continue an operation;
- foreign graph, stale generation, and cross-role substitution are denied;
- reporting summaries rebuild without altering eligibility or decision class.

## Slice 3: Installed Operation And Execution Progression

**Source authority**

- Query installation generation, package admission, bound operating world,
  direct/workflow execution, effect receipts, and publication progression.

**Current identity families**

- operation/package identity and canonical operation identity;
- installation generation and bound capability identity;
- workflow, run, stage, effect, read, publication, and aftermath identities;
- operation definitions, condition/comparator fingerprints, and execution
  traces used by registries or comparisons.

**Authority leaks to remove**

- operation names, canonical digests, fingerprints, traces, or receipt labels
  accepted as operational lookup or progression authority;
- generic `Eq`, `Ord`, or `Hash` serving as compatibility decisions;
- raw identities supplied by executors and restamped by Query.

**Target handoff**

```text
installed generation authority
  -> bound operation authority
  -> execution-ready progression
  -> owner-produced stage/effect/publication evidence
  -> settled operation authority
```

**Closure evidence**

- direct and workflow execution use the same authority law;
- representation collision returns, at most, candidates requiring owner
  validation;
- wrong generation, operation family, stage, effect, or receipt cannot progress.

## Slice 4: Replay, Aftermath, And Lineage

**Source authorities**

- original admitted execution trace and basis;
- Query replay/certification progression;
- verified effect/aftermath evidence;
- owner-specific identity-evolution execution and Foundational lineage
  description at the shared boundary.

**Current identity families**

- replay basis, semantic trace, historical correspondence, aftermath receipt,
  lineage proof/report, promotion, and persistent naming identities.

**Authority leaks to remove**

- semantic equivalence inferred from digests or reporting summaries;
- historical correspondence admitted without endpoint authority;
- lineage or aftermath promoted from identities without effect proof;
- Foundational descriptive lineage replacing Query/Relational owner evidence.

**Target handoff**

```text
original owner evidence
  -> Proof boundary weakening/readmission
  -> replay execution and semantic comparison
  -> verified aftermath
  -> owner lineage result
  -> Foundational boundary description where portable meaning is needed
```

**Closure evidence**

- wrong-target, wrong-effect, wrong-basis, and same-summary/different-semantics
  attacks fail;
- replay/report projections can be rebuilt without changing correspondence;
- lineage consumers receive proof-backed owner results, not asserted vectors.

## Slice 5: Compatibility, Replacement, Rebind, Sharing, And Lease

**Source authorities**

- installed capability and generation;
- Query-owned compatibility decision;
- replacement/rebind transition;
- shared projection ownership and managed lease lifecycle.

**Current identity families**

- operation equivalence token, compatibility explanation, replacement target,
  rebind target, shared-resource key, lease label, release identity, and reuse
  identity.

**Authority leaks to remove**

- equality of names, digests, labels, fingerprints, or equivalence tokens
  substituting for compatibility;
- lease release/resume by copied label;
- shared-resource lookup returning authority directly;
- reuse admission from a cached diagnostic comparison.

**Target handoff**

```text
projection key
  -> authority-paired candidate set
  -> Query compatibility/current-generation validation
  -> replacement, rebind, share, reuse, release, or resume capability
```

**Closure evidence**

- deliberate key collisions cannot cross ownership or generation;
- stale lease, foreign runtime, wrong consumer, and rebuilt token attacks fail;
- index work and authority-validation work are counted separately.

## Slice 6: Dependency, Invalidation, Collection, And Delivery

**Source authorities**

- bound dependency contracts;
- Relational authoritative change publication;
- Bridge installed correspondence;
- Signal invalidation/decision evidence;
- Query consumer and collection-window lifecycle.

**Current identity families**

- dependency fingerprints, field/region keys, invalidation identities,
  collection cursor/window identities, patch identities, consumer continuity,
  and delivery/acknowledgement identities.

**Authority leaks to remove**

- dependency or delta fingerprints used as admitted change truth;
- collection cursor or patch label used to resume delivery;
- diagnostic Signal summaries used as invalidation authority;
- consumer continuity reconstructed from a reporting token.

**Target handoff**

```text
authoritative truth delta
  -> installed Bridge correspondence
  -> Signal invalidation/decision
  -> Query impact authority
  -> collection/patch/consumer delivery progression
```

**Closure evidence**

- full Relational-to-consumer end-to-end pressure;
- collision, stale cursor, wrong patch, wrong consumer, and replayed summary
  attacks fail;
- candidate lookup and authoritative delta validation remain distinct.

## Slice 7: Reporting, Diagnostics, Indexes, And Facades

**Source authority**

- retained owner artifacts from Slices 1 through 6.

**Current representation surfaces**

- evidence identities, terminal reporting projections, canonical digests,
  labels, formatted traces, diagnostic fingerprints, and public equality or
  ordering traits.

**Authority leaks to remove**

- public raw-value accessors on operational identities;
- public `Display`, serialization, `Eq`, `Ord`, or `Hash` where those traits
  substitute for a named operational decision;
- report fields stored beside and later preferred over retained authority;
- facade exports of owner markers, witness factories, or raw constructors.

**Target handoff**

```text
retained owner authority
  -> explicit Foundational projection/digest derivation
  -> reporting, diagnostics, or candidate-selection surface
```

There is no reverse arrow.

**Closure evidence**

- reporting projections remain deterministic and reconstructible;
- audience crates cannot extract or recreate operational key material;
- facade and compile-fail inventories prove exact category separation;
- residue scans find no ordinary representation-to-authority path.
