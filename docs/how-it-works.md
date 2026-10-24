# How WORTH Works

> The platform's mechanics, from the type-level substrate up to the life of
> one application request. [Philosophy](philosophy.md) explains *why* WORTH is
> shaped this way. This document explains *what actually happens*, and what
> each part guarantees to the code that runs on top of it.

## In brief

WORTH is layered. At the bottom are two substrate crates. **`worth-proof`**
makes legality a type, so an illegal step fails to compile instead of failing
in production. **`worth-foundational`** is the shared dictionary: values,
identities, and digests mean the same thing in every runtime. Above them sit
four runtimes, each owning one kind of authority:
- **Relational** owns committed truth.
- **Signal** owns derived computation.
- **Runtime Bridge** owns the causal link from truth changes to computation.
- **Runtime World** owns the product branch that ties exact component states together.

**Query** sits on top and turns application meaning into governed work. An
application declares its schema and operations. Query installs them, admits
each request against current facts, lets a handler build a candidate change,
checks that candidate against the declared ceiling, commits it atomically
through Relational, and publishes a receipt that says exactly what changed.
At every step, *reporting* stays separate from *authority*, *proposed* from
*committed*, and *performed* from *settled*.

## Contents

1. [How to read this document](#1-how-to-read-this-document)
2. [The shape of the platform](#2-the-shape-of-the-platform)
3. [The substrate, part one: `worth-proof`](#3-the-substrate-part-one-worth-proof)
4. [The substrate, part two: `worth-foundational`](#4-the-substrate-part-two-worth-foundational)
5. [The placement rule](#5-the-placement-rule)
6. [The runtimes](#6-the-runtimes)
7. [One change through the whole stack](#7-one-change-through-the-whole-stack)
8. [State versus truth, as the code enforces it](#8-state-versus-truth-as-the-code-enforces-it)
9. [Query: the life of one request](#9-query-the-life-of-one-request)
10. [The touched graph](#10-the-touched-graph)
11. [Outcomes: every way a request can end](#11-outcomes-every-way-a-request-can-end)
12. [Branches, programs, and adoption](#12-branches-programs-and-adoption)
13. [Workflows](#13-workflows)
14. [Aftermath and recovery](#14-aftermath-and-recovery)
15. [Durability and the other surfaces](#15-durability-and-the-other-surfaces)
16. [Misconceptions](#16-misconceptions)
17. [Summary for AI agents](#17-summary-for-ai-agents)

---

## 1. How to read this document

- **Audience.** Application authors, and AI agents that write application
  code. The running example is an ordinary business application: orders,
  approvals, and invoices. WORTH is domain-neutral infrastructure; nothing
  here is specific to any one domain.
- **Names.** Every Rust name in this document exists in the current source.
  A name is not necessarily on a consumer facade, though. The
  [API Map](api.md) lists what an application may import. As a rule,
  application code imports only `worth-query-decl` and `worth-query-host`.
- **Snippets.** Code blocks show the *shape* of a call. They are not
  compiled, and they leave out setup. The crate guides linked from the
  [API Map](api.md) contain compiled examples.
- **Guarantee language.** "Guarantees" means something the types or the
  runtime enforce. It does not mean a convention. "You cannot" means the
  compiler or a typed denial stops you. It does not mean "please don't".
- **Terms.** Words such as *basis*, *admission*, *performed*, and *settled*
  have exact meanings. See the [Glossary](glossary.md).
- **Two paths through this document.** Building an application? Read
  section 2, then sections 9 to 14, then the summary in section 17. Sections
  3 to 8 explain *why* the guarantees hold: the substrate, the runtimes, and
  how one change moves through them. Read them when you need to trust a
  guarantee rather than just use it.

---

## 2. The shape of the platform

### 2.1 Layers

```text
                ┌───────────────────────────────────────────────────┐
  Application   │  your schema, operations, handlers, workflows     │
                └───────────────────────────────────────────────────┘
                          │ worth-query-decl     │ worth-query-host
                ┌───────────────────────────────────────────────────┐
  Query         │  declaration → installation → admission →         │
                │  execution → commit → publication                  │
                └───────────────────────────────────────────────────┘
                          │
                ┌───────────────────────────────────────────────────┐
  Composition   │  Runtime World: product branches over exact       │
                │  component bases; coordinated publication          │
                └───────────────────────────────────────────────────┘
                     │                 │                   │
                ┌──────────┐   ┌───────────────┐   ┌──────────────┐
  Runtimes      │Relational│──▶│Runtime Bridge │──▶│    Signal    │
                │  truth   │   │ correspondence│   │  derived     │
                │          │   │ + causality   │   │  computation │
                └──────────┘   └───────────────┘   └──────────────┘
                ┌───────────────────────────────────────────────────┐
  Substrate     │  worth-foundational: shared meaning                │
                │  worth-proof:        legality as types             │
                └───────────────────────────────────────────────────┘
```

The arrows between the runtimes show the direction of *causality*: a committed
truth change flows through the Bridge into Signal. They are not the direction
of compile-time dependency, which [section 2.2](#22-dependency-order) gives.

### 2.2 Dependency order

These are the real `[dependencies]` edges between workspace crates.

| Crate | Depends on |
|---|---|
| `worth-proof` | nothing |
| `worth-foundational` | `worth-proof` (plus `serde`, `serde_json`, `sha2`) |
| `worth-signal` | proof, foundational |
| `worth-relational` | proof, foundational |
| `worth-runtime-bridge` | proof, foundational, relational, signal |
| `worth-runtime-world` | proof, foundational, relational, runtime-bridge, signal |
| Query engine crates | foundational at minimum; `worth-query-execution`, `worth-query-publication`, and the internal `worth-query` engine also depend on the runtimes |
| `worth-query-decl` | `worth-query-declaration` only |
| `worth-query-host` | Query admission, declaration, execution, installation, and publication |

Two edges are worth knowing:

- **Truth knows nothing of the computation it feeds.** Relational and Signal
  do not depend on each other or on the Bridge. The Bridge depends on both: it
  defines the contracts a truth source must satisfy (`CommittedPatchSource`,
  `SnapshotReadSource`, `TruthBranchHeadSource`) and owns the Relational
  adapter that satisfies them through Relational's public `change_source`
  API.
- **The consumer facades do not depend directly on `worth-proof`.**
  `worth-query-decl` and `worth-query-host` do not list Proof as a
  dependency. Application code receives authority from the owners that mint
  it; it never constructs it. Proof is how the platform keeps its promises. It is not something you
  call.

### 2.3 Who owns what

Every kind of authority has exactly one owner. Nothing else may mint it.

| Owner | Owns | Does not own |
|---|---|---|
| `worth-proof` | The *laws* of legal progression: witnesses, phases, freshness, performed evidence, linear lifecycles | Any clock, counter, registry, storage, or execution |
| `worth-foundational` | The *meaning* shared across boundaries: values, aspects, canonical basis, digests, identity categories, branch-reference nouns, evidence vocabulary | Any runtime state, commit, or merge execution |
| Relational | Committed truth: entities, relations, aspects, immutable branch roots, branch-local linearization, commit history, durable publication settlement | Product authorization, application meaning, derived computation, external completion |
| Signal | Derived computation: dependency tracking, scoped invalidation, recompute, rollback, condition decisions, performed execution receipts | Truth, domain entities, product currentness, relational mutation |
| Runtime Bridge | Installed correspondence from truth dependencies to exact Signal targets, causal routing, lowering of conditional meaning | Relational facts, Signal decisions, application policy |
| Runtime World | Product branch references, immutable single-parent composite history, coordinated publication across component owners | Application authorization, component truth or settlement, durable restart |
| Query | Application meaning: declaration, installation, admission, execution, commit coordination, publication, workflows, aftermath | Lower-runtime truth; it goes through the owners above |

---

## 3. The substrate, part one: `worth-proof`

### 3.1 What it is

`worth-proof` is the platform's law layer for legality. Using only types, it
lets every runtime say:

- what phase a value is in;
- what has been proven about it;
- who was allowed to prove it;
- whether that proof is still fresh;
- that the steps happened in a legal order.

Illegal progressions fail to compile rather than fail at run time. The
crate has **zero dependencies** and deliberately owns no clock, counter,
registry, storage, diagnostics, or execution engine. Laws belong here. The
machinery that enforces them at run time belongs to the runtime that owns it.

It exists because each runtime once hand-rolled its own typestate wrappers,
sealed constructors, and staleness flags, and those copies drifted apart.
Proof is the one shared answer.

### 3.2 Concepts

| Concept | What it guarantees | Key names | Deliberately does not |
|---|---|---|---|
| **Witnesses** | A zero-sized value proving the caller is in an authorized *lane*. It can be minted only by surrendering a marker value, so a witness is exactly as sealed as its marker's constructor. | `AuthorityMarker`, `CapabilityMarker`, `AuthorityWitness<A>`, `CapabilityWitness<C>` | Identify *which instance*. Two runtimes of the same type accept each other's witnesses. Owners that need instance identity carry it separately. |
| **Sealed markers** | Macros generate a marker with a private field and private mint functions. Only the declaring module can mint a witness. | `authority_marker!`, `capability_marker!` | Seal a hand-written `pub struct Foo;` marker. A compile-fail test proves that consumers cannot mint a macro-sealed marker. |
| **Proofs** | `Proof<P, A>` is type-level evidence that fact `P` holds, authorized by `A`. It is mintable only when `A: AuthorityProves<P>`. | `ProofMarker`, `Proof`, `ProofSet`, `NoProofs`, `AuthorityProves` | Act as a runtime bag of facts. There is no dynamic lookup. |
| **Phase-typed artifacts** | `Artifact<P, T, S, A>` carries a payload plus its phase, proof set, and assumption basis. A function that needs phase `P` takes `Artifact<P, ..>`, so passing the wrong phase is a type error. | `Artifact`, `PhaseMarker` | Prove anything by itself. `Artifact::new` is public, so owners wrap artifacts in private types to make them unforgeable. |
| **Recipes** | Symbolic-to-executable progression: `Unresolved → Resolved → Lowered → Admitted`, then ready and executed. Each step consumes the previous stage. | `Recipe`, `ExecutionReadyRecipe`, `ExecutedRecipe` | Run anything. Execution stays with the owning domain. |
| **Checked transitions** | A six-way result: `Success`, `Denied`, `Deferred`, `Stale`, `RebindRequired`, `Failed`. Failure is never flattened into one error. | `Transition`, `TransitionOutcome` | Replace an owner's named outcomes. Owners may use it as the outer shape and keep their own variants. |
| **Freshness** | A value records *what was assumed*. Its type records *how current* that assumption is: `CurrentValidity`, `StaleReadable`, `RebindRequired`, or `AuthorityRevalidationRequired`. Stronger APIs accept only `CurrentValidity`. | `AssumptionBasis`, `FreshnessScopedBasis` | Seal the moment the basis was established. The owner seals its own basis. |
| **Source-bound freshness checks** | `evaluate_freshness` samples the owner's source *itself*, so a caller cannot pass in a flattering clock reading. | `FreshnessSource`, `FreshnessPolicy`, `evaluate_freshness` | Own a clock. The source belongs to the runtime. |
| **Trust boundaries** | Crossing a trust boundary weakens a current basis into `BoundaryBridged<..>`. Getting back to current requires explicit readmission with a witness. Readmission is never ambient. | `bridge_trust_boundary`, `readmit_with_authority`, `rebind_with_authority` | Decide *whether* readmission is warranted. That is owner policy. |
| **Binding axes** | Records the facts a capability was issued against. A missing comparison axis is a compile error, and a mismatch names the first axis that drifted. | `binding_axes!`, `Binding::ensure_matches` | Turn a successful match into a transferable token. |
| **Performed evidence** | `Performed<Action, Authority, Outcome>` is evidence that an action *ran*, which is different from evidence that it was *allowed*. It is minted only with the owner's witness, is not `Clone`, and is `#[must_use]`. | `ActionMarker`, `Performed` | Stand in for admission. *Admitted* is not *performed*. |
| **Causality** | `DerivedFrom` proves that a predecessor ran. `Inverts` proves that an action completed as the inverse of another. Both are minted only from a `Performed`. | `DerivedFrom`, `Inverts`, `prove_derivation`, `prove_inversion` | Grant current authority. A redo is admitted afresh. |
| **Linear resources** | A resource exists once and ends once. `terminate(self)` consumes it, so ending it twice does not compile. | `LinearResource`, `TerminalReceipt` | Keep a registry or detect leaks on `Drop`. Those belong to the owner. |
| **Proof-carrying collections** | Shapes whose facts are proven once, at construction. | `NonEmpty`, `ExactlyOne`, `UniqueVec`, `CanonicalVec`, `DisjointPair` | Hide collection cost. |
| **Brands** | Type-level instance identity inside one `with_brand` scope, using an invariant lifetime. | `with_brand`, `Brand`, `Branded` | Outlive the scope. Long-lived handles need a runtime-owned identity value. |

### 3.3 How the runtimes use it

Proof is not decorative. The runtimes are built out of it:

- Relational, Signal, the Bridge, and Runtime World each declare their own
  sealed authority markers. No other crate can open their governed doors.
- Inside the internal `worth-query` engine, installed operations move through
  about twenty phase markers, from bound to published. Each phase is an
  `Artifact` that only the engine's private authority can mint.
- Query installation admits packages as an `Admitted` `Recipe`.
- Query's recovery handle is a `LinearResource`. Its live table stays in
  Query, and the law that it ends exactly once comes from Proof.
- Query's recovery binding compares seven `binding_axes!`: schema, branch,
  binding generation, installed operation, governed input, principal scope,
  and installed aftermath.
- An undo carries `Inverts<..>`, which proves history but grants no current
  permission.
- Relational's publication returns `Performed`.

### 3.4 Honest limits

- A generic `AuthorityWitness` opens no governed door. Every governed surface
  requires a concrete, owner-sealed type. Relational and Signal both have
  compile-fail tests showing that a generic witness is refused.
- `AuthorityMarker` is an open trait. It is sealed only by a private
  constructor, which the macros generate.
- `evaluate_freshness` checks the observation, not the honesty of the
  original basis.

---

## 4. The substrate, part two: `worth-foundational`

### 4.1 What it is

`worth-foundational` is the platform's shared dictionary. Anything that must
mean the same thing on both sides of a boundary is defined here, once. That
covers values, aspects, canonical basis, digests, identity categories,
locators, branch-reference nouns, diagnostics, provenance, lineage, receipts,
support truth, profiles, and performance claims. Its only WORTH dependency is
`worth-proof`, and it uses Proof carriers for its own stronger values.

The design notes put the split in one line: *Proof defines how truth-bearing
artifacts progress; Foundational defines the shared language those artifacts
speak.*

Foundational exists to stop *duplicate ontology*: two crates that mean almost
the same thing with slightly different types, which drift apart and then
disagree at the boundary.

### 4.2 Concepts

| Concept | What it guarantees | Key names | Deliberately does not |
|---|---|---|---|
| **Canonical values** | One closed scalar language: null, booleans, integer widths, canonical floats, decimals, big integers, rationals, strings, bytes, UUIDs, dates, times, timestamps, entity references, and content references. | `AspectValue`, `CanonicalDecimal`, `CanonicalTimestamp` | Treat a raw value as validated meaning. |
| **Aspects** | An aspect contract states a semantic shape. A binding states where that meaning appears in graph truth. A change kind states what a committed change meant. All three are portable: no storage address, no runtime id, no Signal slot. | `AspectKey`, `FieldKey`, `AspectContract`, `AspectBinding`, `AuthoritativeAspectChangeKind` | Allocate Signal's runtime-local slots. Signal keeps its own. |
| **Masks** | Visibility is contract law, split into three modes: projection, mutation, and diagnostic. | `AspectMask<Mode>`, `ProjectionMask`, `MutationMask`, `DiagnosticMask` | Act as presentation sugar. |
| **Validation phases** | A raw value becomes a `ContractValidatedAspectValue`, then admitted `AuthoritativeRecordAspectState`. Patches separate *set* from *clear*. | `ContractValidatedAspectValue`, `AuthoritativeRecordAspectState`, `AuthoritativeRecordAspectPatch` | Publish a patch. Relational does that. |
| **Canonical basis** | An ordered, versioned, domain-tagged sequence of entries. It is the primary surface for identity: two parties that build the same basis mean the same thing. | `CanonicalBasisSequence`, `CanonicalBasisEntry`, `CanonicalBasisDomain`, `prepare_canonical_basis_sequence` | Stand in for a debug string. |
| **Digests** | SHA-256 is the only admitted algorithm. The digest front door binds algorithm, input shape, domain, rule version, and a work budget to the basis. | `canonicalization().digest()`, `CanonicalDigestId`, `CanonicalDerivedDigest` | Let a runtime call a hashing library directly for cross-crate identity. A digest is an *address*, never permission. |
| **Comparison** | Comparison outcomes are at least three-way: `Equivalent`, `Mismatched`, `Unsupported`. | `CanonicalComparisonOutcome` | Let a digest comparison substitute for canonical comparison. |
| **Identity categories** | Current authority identity, admitted, bridged, revalidated, projection, digest evidence, and external tokens are distinct types. Admitting one requires an authority witness. | `FoundationalAuthorityIdentity`, `admit_foundational_authority_identity`, `readmit_foundational_authority_identity` | Accept a string, digest, projection, bridged identity, or external token as authority. Compile-fail tests prove it. |
| **Branch references** | An exact, descriptive observation: branch id, target, and generation. It is generic over the owner's target type, so Relational and Signal share one meaning for "which branch, at which generation". | `FoundationalBranchReferenceObservation<T>`, `FoundationalBranchTarget`, `FoundationalBranchReferenceGeneration`, `FoundationalBranchId` | Move currentness, retention, or authority into Foundational. An observation *describes*. Acting on it requires the owner's sealed basis. |
| **Commit and transition nouns** | Commit ids, transition locators, and receipt vocabulary are shared as *evidence addresses*. | `FoundationalCommitId`, `FoundationalTransitionLocator` | Perform commits or merges. The runtimes own those. (Foundational also defines stronger commit-receipt, merge-verdict, and committed-authority artifacts. Today no production runtime consumes them; real commits use runtime-owned receipts.) |
| **Evidence axes** | Provenance explains basis. Lineage makes continuity claims (attested, replay-derived, restored, promoted, partial). Receipts split into planning, executed, and completed. Support truth stays honest about degraded operation. Each keeps its own strength. | `FoundationalBoundaryEvidence*` artifacts, `FoundationalBoundaryEvidenceSupportTruthKind` | Let a weaker claim pass as a stronger one. A planning receipt does not mean the work ran. A completed closeout does not mean it succeeded. |
| **Diagnostics** | Typed row families, with a *denial* class kept separate from a *breach* class. | `FoundationalDiagnosticOutcomeKind` | Count as receipts or authority. |
| **Profiles** | One composed policy object in which every family is assigned, progressing from requested to admitted to materialized. | `FoundationalProfileSet` | Act as a mutable settings bag. |
| **Compatibility input** | An explicit, typed lane for JSON input that lowers into native meaning, recorded as migration debt. | `compatibility().json()` | Offer a parser shortcut around validation. |

### 4.3 Where you meet it

A Query consumer rarely imports Foundational directly, but its meaning is
everywhere:

- Query admission derives the identity of request parameters by building a
  `CanonicalBasisSequence` and digesting it. The same request has the same
  identity in every runtime.
- Relational publishes `AuthoritativeAspectChangeKind`, and the Bridge and
  Runtime World consume it.
- Relational's branch-reference observation *is* a
  `FoundationalBranchReferenceObservation<RelationalBranchTarget>`. Signal
  builds its observations from the same types.
- The internal `worth-query` engine binds each operation-phase proof to an
  admitted Foundational authority identity.

---

## 5. The placement rule

When new vocabulary is needed, one rule decides where it lives. Ask three
questions in order. The first *yes* wins.

1. **Does it decide whether an operation is legal?** Put it in `worth-proof`.
2. **Would it mean the same thing crossing a boundary into another runtime?**
   Put it in `worth-foundational`.
3. **Does it need a clock, a counter, a live table, or `Drop` to exist?** Put
   it in the owning runtime, never in a substrate crate.

The rule splits one idea into its law, its meaning, and its machinery:

| Idea | Law (Proof) | Meaning (Foundational) | Machinery (runtime) |
|---|---|---|---|
| A recovery handle ends exactly once | `LinearResource` | — | Query's recovery-handle registry |
| A basis goes stale over time | `FreshnessSource`, `FreshnessPolicy`, `evaluate_freshness` | — | Query's runtime clock |
| Which branch, at which generation | — | `FoundationalBranchReferenceObservation` | Relational's sealed fork authority. Its serializable fork-source descriptor carries the observation but cannot fork anything by itself. |
| Instance identity | `Brand`, within one scope | — | A process-unique identity owned by the runtime |
| Who may act on Relational | the witness mechanism | — | Relational's own sealed markers. Proof never defines a marker on a runtime's behalf. |

One more instruction goes with the rule: *a substrate that a crate cannot see
in its own manifest will be hand-rolled instead.* Check the dependency before
concluding that the vocabulary does not exist.

---

## 6. The runtimes

### 6.1 Relational: committed truth

Relational is a standalone truth runtime for transactional, inspectable,
replayable graph state. Truth mutation, history, and identity live here.
Other systems may observe, project, accelerate, or react to truth. They do
not redefine it.

**Model.**

- **Entities and relations** have separate identity systems. Record ids are
  generational.
- **Aspects** are stable, queryable change surfaces on entities and
  relations. Committed changes are published aspect-precisely: exactly, or
  with a declared widening.
- **A branch** selects one runtime-owned reference cell. There is no implied
  `main`: governed operations never accept a missing target or a branch
  called `"main"` by name.
- **A branch root** is complete, immutable state. Readers never see a
  partially installed root. Roots are copy-on-write, and forking copies no
  truth bytes.
- **The branch reference cell is the unit of blocking exclusion**, not the
  runtime. A publication enters the critical section of its own branch and
  waits on no other branch. One runtime-wide reservation, held only for the
  moment the reference moves, orders published patches. A publisher that
  finds it taken is `Deferred`; it does not wait.

**The write path.**

1. **Observe.** An owner-issued `AdmittedRelationalBranchBasis` names one exact
   observation and keeps its root resident. A `RelationalBranchBasisDescriptor`
   is the portable copy. It is data, not authority, and copying it weakens its
   freshness.
2. **Transact.** A `BranchBoundRelationalTransaction` is move-only and bound to
   one admitted basis. It holds no runtime borrow. It owns an overlay, a
   declared read/write footprint, and its retained basis.
3. **Prepare.** Preparation validates schema, invariants, and footprint,
   assembles the commit, materializes the next root, and checks capacity. The
   result is a `PreparedRelationalCommitCandidate`: opaque, single-use,
   branch-bound, and retained. Preparing or discarding moves nothing.
4. **Compare and publish.** This is Relational's linearization point. In one
   branch-local critical section it compares the candidate's full expected
   observation with the branch cell, installs one complete next root, and
   advances the generation exactly once. The candidate is consumed whatever
   the outcome: `Performed`, `Stale`, `Denied`, `Interrupted`, `Deferred`, or
   `Failed`.
5. **Settle.** A `PerformedRelationalCommit` proves the reference moved, but
   settlement is still owed. A pending-settlement record is installed *before*
   movement, so a moved head always has a recovery record. Concurrent callers
   converge on one `RelationalCommitReceipt`.

**Conflict scope.** Conflicts are detected per branch, not per record. A
candidate is `Stale` if the branch moved at all after the observation it was
prepared from, even if the other commit touched unrelated records. The
footprint drives validation and sparse materialization, not concurrency. The
owner never silently retries, never rebases, and never hands back a reusable
candidate: the caller starts a fresh transaction. `Deferred` from reservation
contention is not staleness, so the caller may reuse the admitted basis with a
fresh transaction.

**Also here.** Retention leases keep a basis resident without claiming it is
current. The runtime also provides forking, canonical commit history,
replay from canonical commit envelopes, and a contained merge lane.
Automatic rebase and semantic merge are not provided.

**Change source.** `facade::change_source` is how another runtime reads what
a commit changed. It selects a commit reachable from an observation, checks
the commit's published aspect changes against its patch, and only then mints
a `RelationalChangeReceipt`. A receipt cannot be built any other way, so
holding one proves the checks passed. The API names no consumer: Relational
does not know who reads it.

**Today's limits.** Relational is memory-resident. Durability across
restarts belongs to Store ([section 15](#15-durability-and-the-other-surfaces)).

### 6.2 Signal: derived computation

Signal is a deterministic, incremental runtime for derived work. Signal never
owns source data: it has no access to Relational. Under Query, source data
stays in Relational, and Signal receives it as the snapshots and committed
patches the Bridge reads from there. Signal owns dependency tracking,
invalidation, recompute, rollback, diagnostics, replay, installed condition
decisions, and the decision about whether an output changed meaningfully.
It consumes snapshots and emits derived refreshes.

- **Graph.** `SignalGraph` is the map of what depends on what, and
  `SignalRuntime` executes it.
- **Aspects in Signal are runtime-local slots.** They are not Foundational
  semantic identities and are never persisted in portable Query packages.
- **Change intent.** `mark_changed` says that a producer-local aspect *may*
  need recomputing. It does not claim the producer emitted a semantic change.
- **Scoped invalidation.** Only a meaningful difference in committed output
  creates downstream causes. The producer declares output equivalence, and the
  consumer declares a dependency comparator. Invalidation marks only the
  changed sources it is given. A node's causes for its direct subscribers
  are derived from its own committed output delta and published with that
  commit, so a downstream hop starts only after the node above it commits.
  A change the consumer's comparator finds not meaningful yields no cause.
- **Transactions and rollback.** A failed transaction rolls back cleanly.
  Observers are notified only after a successful commit.
- **Receipts.** `SignalInvalidationExecutionReceipt` proves what Signal
  observed after *performed* execution. A planning estimate cannot stand in
  for it.
- **Branch bases.** Signal has its own component branches, bases, retention
  leases, and owner-service ports. Retention answers *is it available?*
  Readmission answers *is it current?* Being current never denies retention.

Signal docs sometimes say "truth" for Signal's own committed branch state.
Read that as *Signal's committed derived state*. It is never domain truth.

### 6.3 Runtime Bridge: correspondence and causality

The Bridge is the causal protocol boundary between Relational truth and
Signal computation. It translates across that boundary without giving either
side the other's authority. It translates, coordinates, and preserves
causality. It does not become a second truth runtime or a second scheduler.

- **Correspondence.** An installed correspondence binds a portable truth
  dependency to exact Signal targets: graph, node, partition, and aspect.
  Precision is `Exact` or `DeclaredWidening`. An unsupported precision or an
  ambiguity is a denial, never a reason to fall back to whole-graph
  invalidation.
- **Lowering.** Portable conditional meaning is lowered into an installed
  Signal contract, together with a set of providers. A missing or extra
  provider denies installation.
- **Authoritative delivery.** A committed Relational change carries its aspect
  identity and revision, binding, change kind, field path, and precision. The
  envelope that carries it holds the source and commit identity. The Bridge
  checks the source and the commit, then matches each change against the
  installed correspondence, all before any Signal target updates. A change
  that matches no installed correspondence delivers nothing to Signal.
- **Evidence.** Signal decides whether output changed. The Bridge returns
  Signal's evidence without restamping it.
- **Indexes.** The correspondence allocation index can be rebuilt. It is
  acceleration, not authority.
- **The Relational adapter.** The Bridge depends on Relational, never the
  reverse. `RuntimeBridgeRelationalSource` implements the Bridge's source
  contracts over Relational's public `change_source` API. It lowers each
  change receipt into a Bridge envelope and never re-derives what a commit
  meant.

Application code never calls the Bridge directly for Query-installed
operations.

### 6.4 Runtime World: the product branch

Runtime World is the memory-resident composition authority for one exact
Relational and Signal world. It owns product branches and coordinated
publication across the real component owners. The component owners keep
their own mutation, settlement, and basis authority.

- **Product branch.** A branch's identity, lifecycle incarnation, reference
  generation, and selected commit are four separate meanings. A
  `ProductBranchObservation` compares them all at once. A product reference
  selects its retained exact tuple of component bases, even if a component
  owner has since moved on.
- **Composite basis.** `CompositeBasisKey` binds exact, owner-issued
  admissions from Relational, Signal, and the Bridge. Unchanged components are
  retained exactly and need no contact with their owner.
- **Composite history.** Commits are immutable and single-parent. The mutable
  product reference is not the commit, and inserting history is not the same
  as moving the reference.
- **Coordinated publication** runs in a fixed order: frozen intent, resolved
  head, admitted basis, lowered plan, reserved attempt, owner settlement,
  ready, outcome. The World reserves everything (history slot, recovery
  record, pins) *before* any component effect. It never holds a lock across a
  call into a component owner.
- **Exactly three outcomes:**
  - `Performed`: the product compare-and-swap installed the successor.
  - `NoEffect`: no component owner and no product reference moved.
  - `ProductUnpublished`: some owner effects happened, but the product head did
    not move. This is a retained recovery obligation. It is not a rollback
    token.
- **Recovery** continues an already-performed settlement or exposes cleanup.
  It never calls an unperformed sibling and never performs a product
  compare-and-swap.

---

## 7. One change through the whole stack

This is the verified order for one ordinary application commit.

```text
Query            World                 Relational            Bridge → Signal
─────            ─────                 ──────────            ───────────────
build candidate
on exact basis ──▶ reserve attempt
                   (slot, record, pins)
                   prepare ───────────▶ prepare candidate
                   recheck head
                   publish ───────────▶ compare-and-publish
                                        (branch cell moves)
                                        settle
                   recheck head
                   product CAS
                   (head moves)
◀── receipt
deliver change ─────────────────────────────────────────▶ match correspondence
                                                          update exact targets
                                                          Signal decides
                                                          ◀── evidence
```

1. Query builds a Relational transaction on the product observation's exact
   Relational basis and prepares it into a candidate. Nothing moves.
2. Query hands the candidate to the World inside a
   `CompositePublicationIntent`. The World reserves the attempt before any
   component effect.
3. The World runs its fixed order: prepare in Relational, recheck the product
   head, publish and settle in Relational, recheck, run any Signal leg,
   recheck, and finally publish the product.
4. The final product compare-and-swap checks the complete expected product
   observation before recording history. A stale comparison does neither.
5. Only after the product publication has *performed* does Query deliver the
   committed change through the Bridge. The Bridge matches it against
   installed correspondence and updates the exact Signal targets. Signal
   decides whether output changed meaningfully and returns evidence.

If the commit also changes a conditional definition, the Signal leg runs
inside the World's order, after Relational settlement.

Two different things follow a commit, and they sit at different points in
this order. *Query's native marking*, the record of which settled outputs
the commit made stale ([§10.5](#105-marking-and-currentness)), is part of
step 3. It is prepared before Relational enters the branch's critical
section and installed in the same cutover that moves the branch cell. It is
therefore in place before Relational settles, before the product
compare-and-swap, and before any Bridge delivery. *Signal delivery* is step
5. It runs after the product publication has performed, and Query can start
it only with the receipt a performed publication produces.

**What is atomic, and what is not.**

| Atomic | Not atomic |
|---|---|
| One Relational branch-cell move, together with Query's native marking for that commit | A combined Relational-plus-Signal publication |
| One Signal branch advance (it rolls back cleanly before commit) | The sequence of owners as a whole: it is *ordered*, not atomic |
| One World product-head compare-and-swap | Signal delivery, which follows the commit as a separate delivery step |

If the product compare-and-swap loses after a component has moved, the result
is `ProductUnpublished`. A stale product head after an owner effect is kept as
a retained partial, never rolled back. Cancellation before a linearization
point leaves nothing moved, with a typed outcome. After the linearization
point, the movement wins and the late cancellation is recorded.

This is the platform's answer to distributed state. It does not pretend
several owners can commit as one. It makes the order fixed, the partial
states nameable, and the recovery explicit.

---

## 8. State versus truth, as the code enforces it

[Philosophy](philosophy.md) argues that truth, state, and authority must stay
apart. These are the places the code enforces that separation.

| Separation | Where it is enforced |
|---|---|
| **Derived is not truth** | Signal never owns truth. Derived indexes, including the Bridge's allocation index, are acceleration. |
| **Portable is not current** | Copying a Relational basis descriptor weakens its freshness. A descriptor, commit id, digest, or branch name cannot select a branch. |
| **Resident is not current** | A retention lease keeps a basis available. It does not claim the basis is still current. |
| **Proposed is not committed** | A prepared candidate moves nothing. Failed work is not a commit. |
| **Admitted is not performed** | `Performed` is separate evidence from any admission. |
| **Durable is not live** | A commit becomes live only after its durable medium accepted it. An unknown durable answer installs nothing yet may be durable (`Indeterminate`). |
| **Committed is not externally complete** | A commit records an outbox entry. Delivery to the outside world has its own posture. |
| **History is not authority** | History presence is not performed authority. A cloned receipt is descriptive. |
| **Reporting is not authority** | Readiness reports, basis tokens, discovery answers, and diagnostics grant nothing. |
| **Causality is not permission** | An undo proves inversion. A redo must be admitted again. |

---

## 9. Query: the life of one request

This section follows one request through the consumer facades. The running
example is an order application with an operation for approving an order. That
operation reads `Order.status`, writes `Order.status`, and links the order to
an `Approval`.

### 9.1 Declare

The application describes its schema, operations, queries, capabilities,
workflows, and aftermath with `worth-query-decl` macros and types. Nothing
runs. A declaration cannot install, authenticate, authorize, execute, or
publish.

- Schema: `worth_query_application_schema`, `worth_query_entity`,
  `worth_query_aspect`, `worth_query_field`, `worth_query_relation`.
- Operations: `worth_query_operation!(... access { ... })` declares a concrete
  schema operation's reads, effects and expectations once. Install those same
  facts with `schema.declared_operation::<Operation>(definition)`, retaining
  explicit effect/aftermath policy. See [operation access](build-an-application.md#21-declare-operation-access-once).
- Bindings: `worth_query_mutation_binding`, `worth_query_query_binding`.
- Capabilities and policy: `worth_query_capability`, `worth_query_ability`,
  `worth_query_policy`, `worth_query_principal_binding`.
- Workflows: `worth_query_workflow`.
- Features and programs: each `ApplicationFeature` owns actions through an
  `ApplicationFeatureSpec`. An `ApplicationProgramDefinition<Schema>` lists
  its contributions, features, output graph, and rules.
  A required Initial/Preserve production pair uses one
  `WorthQueryRequiredFeaturePair` and the feature's `required_pair` projection;
  contribution contracts, setup and readiness associations consume that same
  declaration. See [required roles](build-an-application.md#required-initial-and-preserve-output-roles).
  `ApplicationProgramAuthoring::<Schema, Program>::begin().validated_program()`
  returns a `ValidatedApplicationProgram`, whose `revision()` is the
  content-addressed `ApplicationProgramRevision`.

The exact calls, with real code, are in
[Build an Application §2–§3](build-an-application.md#2-declare-the-schema-and-its-contributions).

**Guaranteed.**

- What an operation declares it touches becomes its *touch ceiling* at install
  ([section 10](#10-the-touched-graph)).
- The operation builder is a typestate. Before `finish()`, the author must
  choose `external_effect(..)` or `no_external_effect()`, and `aftermath(..)`
  or `no_aftermath()`. Forgetting either is a compile error.
- An escaping external effect cannot be declared reversible.
- A decoded program description cannot be cast into a validated program.

### 9.2 Install

The host composes contributions with a schema, compiles each operation
contract, derives graph obligations, admits the package, and installs it into
an execution runtime with bounded resources.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let runtime = in_memory_program(
    program,        // ValidatedApplicationProgram<Schema, Program>
    declaration,
    configuration,  // handlers, invariants, producers, conditional nodes
    limits,         // WorthQueryInMemoryApplicationLimits
    initial_state,
)?;
```

- `in_memory_program` installs an application with a program. This is the
  ordinary path.
- `initial_state` is a closure that seeds the primary graph. Installation
  calls it once, before the first publication, with three arguments and
  expects `Ok(())` or a `WorthQueryPrimaryGraphInstallationDenial`:

  <!-- illustrative -->
  ```rust,ignore
  // Illustrative: not compiled.
  |_phase, graph, installed| {
      // _phase:    &WorthQueryBootstrapAdvancementPhase<'_>
      // graph:     &mut WorthQueryPrimaryGraphBootstrap<Schema>
      // installed: &WorthQueryInstalledApplicationSchema<Schema>
      Ok(())
  }
  ```

  The phase is installation's own execution custody
  ([§9.9](#99-resources-execution-and-cost)). It is borrowed for the call
  and cannot be kept. `graph` receives the seeded records. `installed`
  resolves installed names, such as a principal binding, for the seed. The
  types are exported from the host facade modules
  `application_contribution`, `primary_graph`, and `domain`, in that order.
- `in_memory_rostered_program` supports several program revisions against one
  installed schema.

- `*_from_checkpoint` variants restore from a `WorthQueryApplicationCheckpoint`.
- Worked installs with real limits and a roster:
  [Build an Application §4](build-an-application.md#4-install-the-application-graph).

Contribution setup registers handlers (`setup.handler::<Binding, _>(..)`),
invariants, producers, and conditional nodes.

**Guaranteed.**

- A missing handler fails installation before `initial_state` runs.
- The cross-product of reads and touches is computed once, at install. At run
  time, overlap is an O(1) lookup.
- The set of graph obligations is sealed. It cannot be forged.

**You can** inspect every installed contract through the `domain` module:
`touches()`, `graph_reads()`, `read_touch_overlap()`, `external_effect()`,
`aftermath()`, `authorization()`, and `graph_obligations()`.

**You cannot** mint a product branch, retain the primary-graph integration
handle, publish the graph yourself, or construct a touch scope. Each of these
has a compile-fail test.

#### Advanced program-less installation

Use the [ordinary program/owning entry](build-an-application.md#53-execute-through-the-program)
for applications. `in_memory` installs schema meaning without a program and offers
only deliberate native mutation control through `request.advanced()`.

### 9.3 Request entry

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let request = runtime
    .request(&principal, &scope)   // WorthQueryApplicationRequestExt
    .on_branch(branch);            // optional; defaults to current_world()
```

Constructing a request selects no state and resolves no principal. It is a
description of intent. From here, the request becomes one of:

| Call | Purpose |
|---|---|
| `.query(intent)` | Read |
| `.mutate(intent)` | Capture original action meaning; retain an owning operation before its effects |
| `.demand(demand)` | Ask for a program output to be produced |
| `.programs()` | Inspect or adopt program revisions ([section 12](#12-branches-programs-and-adoption)) |
| `.at(&observation)`, `.at_commit(&receipt, max_work)` | Read at a historical basis |
| `.retain_read()` | Capture the exact read occurrence for later `.at(&observation)` reads. It grants no mutation authority |

`inspect_application_readiness()` and `basis_token()` are optimistic reports.
They carry no authority.

### 9.4 Admission

Admission resolves the request against the installed binding, in a fixed order:

1. Select the branch snapshot.
2. Resolve the authenticated principal.
3. Resolve the scope entity: the entity the intent names through its
   binding's scope field. (`WorthQueryRequestScope` is different: it carries
   only the request's deadline and cancellation token.)
4. Authorize the operation, including declared preconditions.
5. Bind the source expectation and the idempotency key.

A request does not keep admission, or a selected world, between executions.
Every `execute()` is admitted afresh.

**Queries** narrow result cardinality with `.limit_results(max_results)`, where
`max_results` is a `NonZeroUsize`. A value above the installed binding ceiling
returns a typed `Limit` denial. This control is not a query-work estimate.

**Mutations** declare what they were based on, which is a typestate step:

#### Advanced native source preparation

Use the [ordinary owning entry](build-an-application.md#53-execute-through-the-program)
for fixed and source-only actions; it retains this source meaning in its descriptor.
The native example below deliberately selects `advanced()`.

The planar geometry example uses the installed program `Program<E>` and its
declared `RegionTotalsDemand`. The current call order is:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let outcome = request
    .advanced().mutate(RegionTotalsDemand {
        scope_key: "anchor-a".to_owned(),
        entries: "example".to_owned(),
        replacement_y: length(3),
    })
    .expect_source(observed.observed_sources()[0].clone())
    .idempotency(&1_u64)
    .execute_in_program::<Program<E>>(
        &application,
        worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
```

Choose `expect_source`, `expect_result_set`, or `without_source` as the binding
permits, and supply `idempotency` before execution. This example executes in
its program. On a runtime without a program, use `execute(allocation_policy)`
instead.

#### Original source meaning

- **Source expectation.** A `WorthQueryObservedSource<Query>` comes from a
  published query row. It records the branch, basis, query identity,
  parameters, and a *footprint* of the entities, field revisions, and
  adjacency revisions that were read. It is descriptive input, not read
  authority. At admission it is rejected if it is missing, foreign, retired,
  changed, or incomplete. Edits outside its footprint do not invalidate it.
  `without_source()` is available only when the binding declares no source.
- **Idempotency.** The key binds the input identity, plus the source and
  workflow transition when present. The same key with different input returns
  `IdempotencyIntentDrift`. The key is resolved *before* the handler runs, so a
  replay never runs the handler again.

### 9.5 Execution

**Queries** run once against the selected snapshot and are published as a
disclosed result. `rows()` gives the results. `observed_sources()` gives the
sources a later mutation can pass to `expect_source`.

**Mutations** run the handler. A handler implements `OperationHandler`:

The current contract includes the default `prepare_decision`. A handler with
no compute step implements `decide` and `build_candidate`; the default
preparation wraps its decision in an immediate
plan. A computing handler overrides preparation, as the
[compiled build-guide handler](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation)
does.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
pub trait OperationHandler<Schema, Binding>: Send + Sync + 'static
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn decide(
        &self,
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Binding::Decision, Binding::Denial>;

    /// Prepare every read on the owner and return compute plus owner completion.
    /// A handler without computation keeps its decision implementation unchanged.
    fn prepare_decision(
        &self,
        input: &Binding::Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> WorthQueryDecisionPlan<Binding::Decision, Binding::Denial> {
        WorthQueryDecisionPlan::immediate(self.decide(input, reader))
    }

    fn build_candidate(
        &self,
        input: &Binding::Input,
        decision: Binding::Decision,
        candidate: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Binding::Result, Binding::Denial>;
}
```

1. **Decide.** `prepare_decision` reads through `DecisionReader` and returns a
   `WorthQueryDecisionPlan`. Its default calls `decide` and returns an immediate
   decision, so a noncomputing handler needs no change. A computing handler
   overrides it with the installed computation's `decision_plan`, which owns
   preparation, compute dispatch and completion under the original request.
   It may deny with a domain denial, such as "the order is already approved".
2. **Build.** The handler writes the actual candidate through `CandidateWriter`
   without forecasting candidate bytes or validator work.
   Nothing outside the writer can change the candidate.

`HandlerResult` distinguishes `Completed`, `DomainDenied`, `ExecutionDenied`,
`Cancelled`, and `DeadlineExceeded`.

**Ordinary owning actions.** A declared action binds original typed input, key,
preconditions, row/result-set source and branch once. `request.mutate` exposes
construction, followed by `retain_operation::<Program, Action>`. Fresh ordinary
requests supply authority at every advance, delivery and cleanup attempt; the
descriptor carries no grant. Publication delegates existing source, retention,
output and cleanup owners. Fixed settlement and declared projection share one
currentness/read observation. Refusal retains the complete owner. Source-only
completion cannot prove required outputs. Delivery and drop do not certify cleanup.

#### Advanced native mutation control

Use the [ordinary owning entry](build-an-application.md#53-execute-through-the-program)
for fixed, discovered, source-only and guarded workflow actions. Ordinary guarded
workflow actions retain the original issued requirement in their owning operation.
The native lanes below require `request.advanced().mutate(intent)` and their
existing explicit allocation policy.
Native owners retain effect, custody and unresolved-publication authority.

**Native substrate.** Every lane takes an `ExecutionAllocationPolicy`
(`worth_query_host::facade::runtime`): `SystemAllocation` or
`Execution(&lease)`. The caller chooses; there is no default. The program lanes
take the installed program runtime first.

| Lane | Use it when |
|---|---|
| `execute(allocation_policy)` | The runtime was installed without a program (`in_memory`). On a runtime installed with a program, it refuses every mutation with `ApplicationProgramRequired`. |
| `execute_in_program(&program_runtime, allocation_policy)` | The runtime was installed with a program (`in_memory_program` or `in_memory_rostered_program`). This deliberate native lane executes under the selected program. The runtime reads which program the branch runs; the caller never names it. |
| `execute_capability_in_program` | As above, through a capability. |
| `execute_retained` / `execute_retained_in_program` | You want to commit and keep a retained read of the result. `execute_retained` is for a runtime without a program, like `execute(..)`; on a program runtime use `execute_retained_in_program`. |
| `execute_performed` / `execute_performed_discovered` | You need the program outputs the operation requires to be produced after commit. |

`commit_for_program`, reached through `WorthQueryProductEntry::transaction()`,
is an advanced product-transaction lane for hosts. Ordinary applications use the owning entrance above.

### 9.6 Candidate check

Before commit, the candidate goes through Relational validation, receipt
closure, required invariants, and **touch admission**, which
[section 10](#10-the-touched-graph) covers.

### 9.7 Commit

The validated candidate is compared against the branch head and committed
atomically, through the path described in
[section 7](#7-one-change-through-the-whole-stack). If the operation declares
an external effect, a dispatch outbox record lands in the same commit.

The receipt, `WorthQueryApplicationCommitReceipt`, says what happened:

| Question | Accessor |
|---|---|
| Which branch and commit? | `product_branch()`, `commit_reference()`, `committed_product_publication()`, `basis_descriptor()` |
| What changed? | `committed_changes()`, `changed_record_count()`, `emitted_effect_count()` |
| Which records did the commit actually touch? | `mutation_work()`, an `Option`, then `touched_records()` on the evidence inside |
| What must be sent to the outside world? | `dispatch_outbox()`, `external_dispatch()` |
| Under what authority? | `authority_binding()`, `installed_operation()`, `principal_scope()`, `idempotency_binding()` |
| What can be undone? | `published_aftermath_posture()`, `retained_preimage()`, `aftermath_causality()` |

Cloning a receipt drops its performed-change witness. The clone is
descriptive history.

### 9.8 Publication, live reads, and output demand

- **Live reads.** `query.subscribe(WorthQueryApplicationLiveLimits::bounded(..))`
  opens a bounded subscription. Each `next(&fresh_request)` re-resolves the
  principal and scope. A request whose principal or scope has changed is
  refused (`StalePrincipal`, `StaleScope`). An overflow reports how many commit
  batches were missed. Updates are never dropped silently. Close with
  `close()`.
- **Output demand.** `request.demand(d).controls(c).start()` opens a demand.
  `advance(&fresh_request)` returns `Pending` or `Settled`, or a typed denial.
  One call can settle a dependency chain and progress other queued required
  outputs within its budget. `Pending` means work remains for a further
  admitted call. `settle(&fresh_request)` repeats that step inside one
  request, up to the attempt count in the demand's controls, and returns as
  soon as the demand settles. No background sweeper runs: the authenticated
  request stays the principal.
  Within that call, Query prepares ready independent producers on the owner,
  computes their managed tasks together, then completes and publishes them in
  canonical order. The caller pumps progress; a worker does not publish.
- **Fixed required outputs.** The ordinary operation delegates performed publication,
  original-carrier retention, native output advancement and complete settlement.
  Registered replay resumes without another source effect. Delivery checks the
  actual required set and reads at one observation in one phase. Refusal retains
  settlement; stale delivery exposes history/close without pumping closed outputs.
  Manual publication, output start and registered recovery are
  [advanced native controls](#advanced-native-mutation-control).
- **Discovered outputs.** A declared discovered action selects the ordinary
  owning `retain_operation` entrance. Each fresh-request `advance` delegates one
  native handoff, preserving original descriptor and genuine partial or
  committed-retention failure. It does not rerun a committed handler. Registered
  reply-loss recovery rejoins actual native custody; missing registration never
  manufactures Fresh result or one-shot delivery. Current scalar/batch delivery
  checks the complete original discovery and all settlements at the same
  observation and phase as the declared read. No-projection delivery validates
  that aggregate without a dummy query. Stale owners retain original history and
  cleanup; closed handles cannot advance. Deliberate native source/start/recovery
  controls require `advanced()`.
- **Reuse.** An output whose settlement is unmarked on a continuous basis,
  with no recorded requirement for full verification, can
  be reused without contacting its producer or re-running its source query.
  That clean reuse depends on the marks alone; no producer contract gates it.
  A dirty output re-verifies its marked facts. Equal declared input can avoid
  producer execution (input cutoff). Input cutoff is opt-in per producer: the
  producer declares a `WorthQueryProducerInputReuseContract` in its
  `INPUT_REUSE` constant, which is `None` unless declared. The contract names
  the producer's determinism and which of the key, the principal, and the
  scope its result depends on. Only canonical-bitwise determinism qualifies.
  A run that reads through a raw reader, takes a checkpoint, runs a managed
  computation, or uses a context value the contract did not declare, leaves
  no input-cutoff candidate. Managed partition state has its own reuse rules
  below. Stable republication of the output that input cutoff
  kept can clear pending marks below the producer without calling consumers.
  [§10.5](#105-marking-and-currentness) gives the rules.

### 9.9 Resources, execution, and cost

Declared resource limits and independent algorithm controls are enforced by their
owners; caller limits can only restrict the installed policy where that policy applies.

Ordinary callers declare semantic result and effect scope, not Query's internal
traversal algorithm. Installed host profiles bound the advancement's work and
memory; a computation may declare a tighter ceiling. Candidate preparation,
validation and reads spend that same request budget. Installed invariants also
retain their own algorithm controls and actual execution evidence.
Composed demand admission intersects host, caller and child ceilings, then checks the child's
required resources. Work counters are safeguards and diagnostics, not claims
about elapsed performance or total process memory.

The governed controls include:

- install limits;
- query limits;
- live limits;
- demand controls;
- source-comparison budgets;
- history-selection budgets;
- candidate reservations.

**Execution authority.** A process constructs one physical execution authority. It owns
its computation workers and the process memory ledger. On a target without
OS threads it builds no worker pool. An installed execution
policy admits request leases with worker, charged-memory, and work ceilings.
Child leases share ancestor worker and memory limits. Nested work inherits
its caller's remaining work. Managed partition execution descends from the
advancement's request, and nested work descends from its active member. An
unrelated lease cannot open a second budget inside that member. Under serial placement,
execution uses a bounded lease with serial backing: it owns no physical pool,
runs on the caller thread, and admits one worker. Children share its ledger
and inherit its serial posture, cancellation, deadline and limits. Both
backings charge and report performed work through the checked execution doors.

**One request per advancement.** An *advancement* is the work of one public
request entry: a query, a mutation, a live read, an adoption step, or one
`advance` of a demand. Query opens one execution request for it, and every
runtime below works under that request.

**Stopped reads.** A stopped request is not a missing record. Read boundaries
preserve `Stopped(cause)` separately from `Refused(kind)`; consumer surfaces
report their own typed cancellation, deadline or execution-resource outcome.
A pinned read consults the request at the pin, before capturing native read
custody, and keeps the selected immutable basis rather than consulting a newer
live index generation. Handle the typed stop; do not turn it into empty input.

- **Opening.** A request-scoped entry takes its cancellation and deadline
  from the caller's request. A host-owned entry (an inbound occurrence, a
  clock observation, a staged run) opens with neither. Both take
  their ceilings from the installed execution policy. The request is opened
  before the advancement's first read. A policy that grants no work or no
  memory refuses the advancement there, before any read.
- **Placement.** The host decides placement when it builds the Runtime
  World: it installs an execution policy, and optionally the process
  authority. With an authority whose limits admit the policy, every
  advancement is *leased*: it holds a request lease from that authority.
  An authority that cannot admit the policy refuses the World build. With
  none, every advancement is *serial*. Query reads the placement from the
  World. The one exception is adoption publication after the World owner
  has expired, which runs serially.
- **One work ceiling.** The policy's work ceiling bounds the whole
  advancement in both placements. Nested computations draw from what
  remains.
- **Custody.** Code inside the advancement reaches the request only through
  a *phase*: a value borrowed for the duration of one callback. It cannot be
  returned, stored, or handed to a product, and each of those is a compile
  error. A phase lends its request only to the installed runtime that opened
  it; another runtime is refused with
  `Resource(ForeignAdvancementPhase)`. A public call that
  tries to open a second advancement on the same thread is refused with
  `NestedOpening`. Descendant workers carry the active host's owner marker
  and also refuse nested openings; an unrelated thread owns its own request.
- **Seams.** A seam is a function where one runtime calls another. Every
  seam that starts computation takes the request as a required argument.
  Publishing or settling an already prepared candidate takes neither request
  nor lease; the World consults the request before it contacts those doors.
  The Bridge takes the request to read a committed patch or a snapshot and
  to deliver a change. Signal takes it to advance a branch and to receive a
  committed patch. The Runtime World takes it to run a publication, and
  hands it to the Relational and Signal legs. Relational query execution,
  preparation, validation, commit execution, and index construction use the
  caller's request rather than opening another budget. A native caller that
  has no process lease still supplies a bounded request with serial backing.
- **Closing.** The request closes when the callback returns. Delivery of a
  performed change through the Bridge needs the phase, so the final delivery
  happens inside the advancement.
- **Demand calls.** `advance` opens one advancement per call. `settle` opens
  one advancement and repeats the same step inside it, up to the attempt
  count in the demand's controls, returning `Settled` as soon as the demand
  settles and `Pending` otherwise.
- **Installation.** Installation is not an advancement of the installed
  runtime. It runs serially under the host's memory and work bounds, and the
  phase handed to `initial_state` cannot open an installed runtime's public
  entries.

**Pinned read.** A read selects and retains one exact immutable source root.
Later branch movement cannot change that read's rows. Releasing its read hold
ends that access; a later read must select a basis again. A pin establishes
snapshot stability, not present currentness or permission to write.
A pin does not make reads concurrent: reads of one installed source still run
one at a time. Commit validation still compares
the selected basis and the attempt's consumed facts before any effect.

Admission checks the original request before dispatching kernels. Ordinary
installed managed computations use native allocation owners without an
author-supplied result-byte ceiling or `ChargedBytes` estimate. Actual request
memory refusals and allocator failures remain typed execution failures, distinct
from domain outcomes, cancellation, deadline and kernel panic. A busy worker
pool can admit inline execution without creating another worker.
Native read, index, and commit execution preserve these causes through their
Query denial. A stopped read is not an empty result, an absent fact, or an
authorization decision. Match the typed cause at the call that stopped.

The native canonical reducer owns its actual tree and temporary-storage
allocations, preserves retained custody and checks encoding sizes. Explicit
bounded lower-level Execution APIs still accept their own payload capacities;
those contracts are separate from the ordinary installed computation path.
Neither path turns allocation admission into permission to publish.

**Determinism.** For the same admitted inputs and declared computation, without
an interruption arriving during compute and with sufficient memory, results,
charged work and structural span do not depend on worker count or scheduling. The default
contract preserves canonical result bits; an explicitly installed equivalence
contract at the execution layer supplies its comparison predicate. Query
managed-computation installation currently refuses `ContractEquivalent`.
Partition identity comes from the
data, never the worker. Results are settled and published in canonical
partition order. If several partitions fail, the least failing identity is
reported. An execution map reports the accepted canonical prefix through its
failure; later results are discarded and their work is a physical discarded-work
metric. A producer wave differs: member denials travel as data, so every member
that computed stays charged even when owner completion keeps only a prefix.
A work ceiling stops at its canonical boundary. Cancellation and deadlines are live controls, so
changing when they arrive can change how far a run progresses. Worker use,
queue activity, and peak memory are physical metrics, not invariant results.

**Work and span.** Work is the sum of charged operation units. Span is the
structural length of the dependent computation: sequential stages add span,
independent branches take their maximum, and nested work contributes to its
calling branch. These are cost measures, not elapsed time; serial placement
preserves structural span. The executable
[serial and nested-parallel cost example](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/parallel_history/structural_cost.rs)
is `waves_add_and_nested_independent_children_take_the_maximum`: its declared
checkpoint expression has work 36 and span 21. Coordination and sequential reduction
also contribute their own charged costs. The execution layer exposes work and
span in its execution report. A managed partitioned computation exposes
charged work; its application-facing result does not expose span.
An advancement has one report, across all its waves, including a failed
advancement. Admission refusal contributes zero work and span.
`advance` returns `Pending`, `Settled`, or a typed denial; it does not return
the report.

**Partitioned managed computation.** A *partitioned managed computation*
splits one input into partitions, computes each partition on its own, and
reduces the partition results to one result. An installed *owner* supplies
its functions; the
[build guide](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation)
shows how to write one. The *calling thread* is the thread on which the
operation handler runs the computation.

| Owner function | Runs | Does |
|---|---|---|
| `partitions` | Calling thread, once | Names the input's items. Each item has a stable identity of its own, never its position, so a reordered input plans the same partitions. |
| `partition_key` | Calling thread, once per item, in ascending item identity | Returns the key of the partition the item belongs to. |
| `gather` | Calling thread, once per partition, in ascending partition identity | Reads one partition's data. |
| `compute_partition` | Any worker, once per partition | Computes one partition from what `gather` returned. It is handed no reader. |
| `reducer` | Calling thread | Supplies the identity value and the `combine` function that merge partition results. |
| `complete` | Calling thread, once | Turns the reduced result into the output. |

The first three functions read through the handler's reader, and Query
records which call read which fact. What `partitions` reads is a *membership
fact*: a change to it changes which items there are. What `partition_key`
reads is that item's *key fact*: a change to it can move the item to another
partition. What `gather` reads is that partition's fact, and a fact two
partitions read is a fact of both.

A partition's identity is derived from the digest of its key's canonical
encoding. Two keys are one partition exactly when their digests are equal.
The reduction runs over a canonical tree that the set of partitions fixes,
and `combine` sees partitions in partition identity order. That order is
deterministic and is not chosen by the author. A result that depends on
association, such as a floating-point sum, therefore has the same bits on
every run and at every worker count.

No partition is computed unless every partition gathers. A run that computes
every partition dispatches them through one execution map: on a child of the
request's lease under leased placement, or serially within the request's
policy. Either way, partitions settle in identity order with the charges any
worker count would settle on.

**Partition execution and advancement.** Kernels within one partitioned managed
computation can run concurrently. A completed producer run keeps its partition
state with the output it recorded. The next run of the same installed producer,
at the same producer edition and on an equal input value, gathers and computes
only the partitions whose facts changed. A changed fact behind partition
membership or behind an item's key redoes only that membership or that key,
and regathers only the partitions an item left or joined. A first run, a
restored record, a reinstalled owner, a changed input value, or state too
large to keep computes every partition; *Per-partition reuse* below lists
every cause. Reuse requires the [owner's purity contract](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation).

**Parallel advancement.** One caller-pumped `advance` can run several waves
of required outputs. The required-set owner admits each wave against its
selected source basis: a member's consumed upstream outputs must already be
committed on that basis. Members at the same dependency depth do not form a
wave unless each one's upstream outputs are committed on that basis.
After a commit moves the basis, readiness is checked on the new basis before
another wave is dispatched. The call keeps one bounded request for all waves;
each member's nested computation shares its remaining work and resource bounds.

Preparation reads declared facts on the owner thread and produces inert compute
inputs. Independent members' compute steps can run concurrently. Completion,
candidate authoring, commits, and publications run on the owner in canonical
key order, never worker completion order. The member key orders by producer
identity, then by observed source epoch; partition kernels within a member use
their own canonical partition identities. A handler that has no compute step
continues to return its immediate decision without an application change.

A wave stops at a request-wide stop or at a failure of the caller's own member.
The least canonical such member decides the reported failure, and no member
after it is committed or published. A failure of another queued member is held
with that member; its siblings in the wave still commit and publish.
Application keeps only the applied prefix. A compute request stop charges only its
canonical compute prefix. An earlier owner-completion failure can be reported
instead, but does not refund computation already settled, including work past
that earlier member. The applied prefix and the canonical compute prefix are
therefore distinct. Earlier performed effects remain performed and retain their
recovery obligations. A queued member's domain refusal can be held while other
required work progresses; the held member leaves the runnable queue for the
rest of this call and is requeued when the call ends. A
request-wide stop ends this call with its distinct cause. With sufficient
memory and no live interruption, serial and parallel placement produce the
same result, applied order, and charged work. Live cancellation or a deadline
can change the reached prefix, but never allow publication beyond its boundary.

Closing a demand releases that holder's interest; another holder can keep the
output required. Drop has no request budget: it releases this holder's interest
and that row's own claims, follows closed claims through at most two rows,
charges no work, starts no computation, and never scans the whole registry.
It may hand on at most one existing closed-leaf claim. Cleanup it cannot finish
stays recorded for a later funded call. A reader already sees the successor
output before that cleanup runs; deferral does not weaken currentness.

**Per-partition reuse.** A *producer* is an installed binding between an
output and the operation that produces it. When a producer's handler runs a
partitioned computation, the run keeps its state, and the producer's next run
recomputes only what changed. A run that no producer started keeps nothing
and is never compared.

- **What is kept.** A completed run leaves its state on the output record its
  attempt published: its items and their digests, the routing of items to
  partitions, each partition's key, the work every owner call charged, each
  kernel's accepted checkpoints, the reduction tree, and the facts the owner
  calls read. A record lacking usable kernel accounting takes the full-run path.
- **When it applies.** The next run compares against that state only when
  three things match: the installation of the owner, the producer edition,
  and the digest of the input value. A reinstall is another installation even
  when its types and declaration are the same. The *producer edition* is the
  meaning of the installed producer declaration: its declared contract and
  the types bound to it.
- **What is redone.** Each retained fact is compared as it stands at this
  attempt's snapshot, at three levels:
  - *Membership.* `partitions` is called again when a fact it read changed.
  - *Item.* An item is keyed and routed again when it is new, its digest
    changed, or a fact its key read changed. Every other item keeps its
    route.
  - *Partition.* A partition is gathered and computed again when an item left
    or joined it, one of its items' digests changed, or a fact its gathering
    read changed.

  Every other call is *carried*: its retained answer stands in for the call.
- **Reduction.** The next tree is the retained tree with departed partitions
  deleted, new partitions inserted, and each changed path recombined. A
  recomputed result with the same canonical bits as the retained one replaces
  nothing. When the declared work cannot fit the edits, the tree is built
  again from every leaf, so the run fails where a full run fails.
- **Dispatch.** One map contains carried and recomputed partitions in canonical
  identity order. Carried slots replay their accepted kernel charge; other
  slots compute. Both use the request's execution.
- **Cost.** An outcome never depends on reuse. A carried call charges what it
  charged, at the place a full run makes that call, so a work ceiling stops
  at the partition a full run would name. A call the remaining work would not
  pass is made instead, and fails as a full run fails. The declared work bounds
  retained-fact comparison before reuse is admitted. `charged_work()` reports
  carried and recomputed partitions alike; the enclosing member receives their
  full-run-equivalent work and span. Memory is physical: a carried partition
  reserves what it holds, so a memory denial may depend on reuse and placement.
- **Seal.** When the attempt seals its reads, every fact a carried call read
  must still be the fact the prior run read. If one changed, the attempt is
  denied.
- **Full runs.** Every other run gathers and computes every partition. The
  causes are: no output record is published at this address; the record came
  from a checkpoint restore or from a stable republication, neither of which
  keeps computation state; the prior run stopped before it completed, or ran
  no partitioned computation; the installation, the producer edition, or the
  input value differs; the handler ran several partitioned computations, or
  an earlier call in the same handler already consumed the prior state; the
  state could not be measured, or policy chose not to retain it; retained
  capacity refused it, or a successor record took its reservation; comparing
  its facts would exceed the run's declared work; or routing met a partition
  identity collision. Neither the handler nor the owner is told which kind
  of run it was.
- **Retention bounds.** Kept state is charged by its bytes against a bounded
  retention ledger: a maximum number of bytes that output records may keep.
  A refusal drops only the incoming state, and existing holders keep theirs.
  A successor can take its exact displaced predecessor's reservation only
  when that state is unpinned and solely held, after growth is admitted and
  the prior state is consumed. A
  record that a forked branch shares keeps its own reservation, and the
  successor reserves in full. A branch with no record of its own takes its
  prior state from the record at its fork point, and the same three matches
  and fact comparisons still decide what is carried.
- **Identity collision.** Two different keys that derive one partition
  identity deny every run while both keys are in the input
  (`PartitionIdentityCollision`). An incremental run that meets one starts
  over as a full run, so the collision is named as a first run names it.

*Executable example.* A record-backed aggregation changes one entry while
keeping its input identity. In
[`one_entry_edit_gathers_and_computes_one_partition_at_every_size`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/counts.rs),
the next run makes no membership or key calls and gathers and computes one
partition. The [warehouse quantity and sensor peak examples](build-an-application.md#36-declare-and-install-a-partitioned-managed-computation)
show the same owner interface in two domains; their input contains the values,
so changing that input requires a full run rather than fact-granular reuse.

**Workflow frontier.** A *staged run* executes one installed domain operation
as named stages. A *frontier* is a canonical set of ready stages advanced by one
call. The engine opens one execution request for the whole frontier and
takes every stage through three phases that its types keep apart.

| Phase | Runs | Sees |
|---|---|---|
| Prepare | Owner, in canonical stage order | The stage's input facts, fixed before the frontier starts. No executor, no execution context, and no workspace. It returns an owned task. |
| Compute | Any worker | Its task and a work meter: checkpoints and work accounting, no lease for starting nested work. It is a plain function pointer, never a closure over owner state. |
| Apply | Owner, in canonical stage order | The owner and the computed result. Every effect happens here. |

- **Canonical order** is ascending stage identity. The multi-stage frontier
  entry sorts identities and refuses fewer than two members or duplicates.
  Ordinary single-stage advancement uses the same metered map.
- **What runs in parallel.** Across stages, only the compute steps run
  concurrently. They go through one owned-map dispatch under the frontier's
  request, in serial placement too. A stage's apply runs alone on the owner,
  and the graph work it starts runs under the same request.
- **Least failure.** Preparation stops at the first stage in canonical order
  that fails to prepare. Later stages are neither prepared nor computed;
  earlier stages compute exactly once. Application then runs the canonical
  prefix through the least failure across all three phases. A failing
  application keeps the partial owner effects it made. Computation is
  charged for the canonical prefix through its least failure, and a later
  apply failure does not refund it.
- **Purity.** Prepare and compute must be pure functions of the values they
  are given: no clock, no static or global state, no environment, no
  interior-mutable state shared with the owner or another stage, and no
  effects. The types remove the executor, the context, and the workspace.
  They cannot remove ambient state, so an impure function makes results
  depend on execution order and worker count.
- **Determinism and interruption.** With sufficient memory, worker count
  changes neither results nor charged work when no cancellation or deadline
  arrives during compute. The frontier's entry opens a host-owned request,
  which carries no caller cancellation and no deadline. Under a request that
  carries one, an interruption during compute can change how far the
  frontier gets. Application is then a canonical prefix that ends at or
  before the interrupted stage, with the receipts and effects a serial run
  produces for that prefix, and the cause stays cancellation or deadline.
- **Stop causes.** Cancellation, deadline, work exhaustion, a nested stop, a
  panic, result capacity, and admission each surface as their own cause.

*Executable example.*
[`reversed_completion_still_applies_canonical_effects_and_receipts`](../workspaces/worth-query/crates/worth-query/src/domain_installation/operation_execution/workflow_execution/progression/frontier/tests/dispatch.rs)
forces `middle` to finish before `left`; receipts still occur as `start`,
`left`, `middle`, `right`. This tests owner application order independently of
worker completion order.

The staged-run surface belongs to the engine beneath the consumer facades,
which do not export it. It is not the workflow kernel of
[§13](#13-workflows), which takes one step per request.

**Parallelism at each layer.** Every row runs under the advancement's one
execution request. "Ordered" names what stays canonical whatever the worker
count.

| Layer | Runs in parallel | Stays ordered |
|---|---|---|
| Execution | A map's kernels and independent child computations may use authority workers under automatic posture. Serial backing runs inline. | A scan carries one value through ordered items. Rounds are a sequence of barriers. A map settles its results in canonical partition order. |
| Relational | With a lease: the packets of a query plan, index builds (one packet per index definition), and the invariant packets of commit validation. | Each merges in one ordered pass on the calling thread, and index generations publish there afterward. Without a lease, packets run in order on the calling thread. |
| Signal | Within one epoch of a stage: read preparation for the tasks that need computing, and application of tasks, each declaring its write keys. An epoch is a resource-admitted prefix of the stage's remaining tasks, none of which reads another's node. | The request is one ordered pass over stages. A stage is an ordered sequence of epochs; each publishes through the owner before the next starts. The owner reduces after the join: commits in task order, snapshots in stage order. Read preparation runs serially when the epoch's tasks that need computing are fewer than its configured minimum; application does when the epoch's tasks are fewer than its own. |
| Query: partitioned computation | Partition kernels (`compute_partition`). | Naming, keying, gathering, and completing run on the calling thread. Results settle in partition identity order. |
| Query: workflow frontier | The compute steps of a frontier's stages. | Prepare and apply run on the owner in canonical stage order. |
| Query: derived collections | Rebuilding dispatches one partition per unique root. Pinned immutable read custody lets workers read without holding the mutable runtime owner. | The map settles in canonical order. Projection and link checks complete on the owner in entity identity order; retention changes only on success. |
| Query: advancement | Managed compute tasks of independent ready producers in one wave. | Prepare, complete and publish run on the owner in canonical order; waves progress inside the caller's `advance`. |


[`a_real_diamond_agrees_across_widths_and_failure_phases`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/history.rs)
executes this through public output demand. The companion
[`a_completion_failure_charges_a_dispatched_suffix_kernel`](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/concurrent_waves/charge_prefix.rs)
asserts that a computed suffix's work stays charged after earlier completion
fails. Use `advance` to make one admitted progression call; use `settle` when
the demand's bounded repeat policy is appropriate.

Sessions, runs, subscriptions, continuations, leases, checkpoints, recovery
handles, and admitted capacity are *managed resources*. Close them explicitly.

---

## 10. The touched graph

The touched graph is the exact set of changes a commit made. Relational seals
changed records, aspect field paths, adjacency changes, observable revision
bumps, and old/new index keys into the canonical commit envelope, beside the
patch. An ordinary commit seals this exact graph, and so does a merge
executed under native merge authority. A commit that carries a schema
transition seals none, and neither does a merge without that authority. For
such a commit Query records a *discontinuity* in place of marks: a point its
marks do not cover, across which readers fully verify
([§10.5](#105-marking-and-currentness)). The touched graph also supplies the
cause of invalidation: the runtime intersects it with consumed facts to
determine affected consumers.

Widening, the delivery of a change at a coarser precision than it was made,
exists only in a Bridge correspondence that declares it
([§6.3](#63-runtime-bridge-correspondence-and-causality)). Query installs its
application field mappings in the Bridge with widening disallowed.

Scope paths belong to Signal. A scope path is a host-defined semantic
locality that a Signal subscription names exactly or by subtree; it is never
a work partition. Signal's `Visited` observation tier reports consideration
during transaction processing; the commit's touched graph reports actual
changes.

"What did this operation change?" has three answers in WORTH. They are
different questions, and the platform keeps them apart.

| Layer | Question | Type | Created by |
|---|---|---|---|
| 1. Declared ceiling | What *may* this operation change? | `WorthQueryOperationTouchContract` | Installation, from the operation declaration |
| 2. Admitted candidate touches | What does this candidate *propose* to change, and is it within the ceiling? | Checked by `admit_validated_application_touches` | Relational validation of the candidate, then Query touch admission |
| 3. Commit-sealed touched records | What *did* the commit change? | `WorthQueryTouchedRecordIdentity`, read through `receipt.mutation_work().map(\|work\| work.touched_records())` | The commit seal, from Relational's changed records |

The law, in one sentence: **declared touches are the legal ceiling, candidate
validation is not performed evidence, and only commit-sealed touched records
say what the committed attempt actually changed.**

### 10.1 Layer 1: the declared ceiling

`WorthQueryOperationTouchContract` is either `NotRequired` or
`Declared { graph_roles, scopes }`. Each scope is a
`WorthQueryOperationTouchScope`:

| Scope | Covers |
|---|---|
| `CreateEntity`, `DeleteEntity` | An entity kind |
| `WriteField` | One field of one aspect contract of an entity kind |
| `LinkRelation`, `UnlinkRelation` | A relation kind |
| `DeclaredDomain` | A domain-declared scope identity |

At install, WORTH builds a `WorthQueryOperationReadTouchOverlapIndex`, which
records which declared reads intersect which declared touches:

| Read | Touch | Intersects when |
|---|---|---|
| Entity | `CreateEntity` / `DeleteEntity` | Same schema and entity |
| Native projection | `WriteField` | Same schema, entity, and contract revision, and the mask covers the field |
| Relation | `LinkRelation` / `UnlinkRelation` | Same schema, relation, and endpoints |
| Anything else | — | Never |

Each operation whose touch contract is `Declared` also produces one
graph-obligation row of kind `MutationTouch`, whose terminal requirement is
touched-scope evidence. A `NotRequired` contract produces no row.

### 10.2 Layer 2: candidate admission

After Relational validates the candidate, Query admits its touches:

1. It checks that the overlap index matches the installed reads and touches.
2. It resolves every installed scope. A `DeclaredDomain` scope cannot be
   resolved to graph records, so it is rejected here. Duplicate scopes are
   rejected. A field write must name a single-segment path.
3. For every touch the candidate proposes:
   - a create, delete, field write, link, or unlink on a kind this
     application governs must match an installed scope;
   - a touch on a kind the application does not govern is skipped;
   - a revalidation or an unrepresentable mutation on a governed kind is
     refused. An operation cannot change governed records in a way its
     installed contract never admitted.

### 10.3 Layer 3: commit-sealed records

`WorthQueryTouchedRecordIdentity` is built only from the commit's changed
records. It is never supplied by a caller. It lives inside
`WorthQueryPrimaryMutationWorkEvidence`, which is built only from the commit
seal and also counts the work done: facts examined, invariant work, index
maintenance, touch targets materialized, and scopes compared.

Undo depends on this layer. A `RecordedInverse` or `Compensation` undo is
denied with `TouchedRecordsRequired` if the commit sealed no touched records.

### 10.4 Executable example

[`owner_validated_application_touch_outside_installed_ceiling_is_denied`](../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/tests/application_attempt/terminal_failures.rs)
adds an undeclared application touch after the owner validated the candidate.
Commit is denied at invariant execution, and the account stays open. Validation
cannot promote a proposed touch outside the installed ceiling into performed
evidence. Read what actually changed from the committed receipt, never from
the declaration or a planning estimate.

### 10.5 Marking and currentness

The touched graph causes invalidation. Native commit publication carries
Query's marking state with the source image, so visible source changes and
their marks agree, whoever the writer was.

- **Reverse index.** When an output settles, Query indexes its consumed facts:
  field revisions, index keys, selection and absence facts, and consumed
  upstream outputs. A settlement recorded against an older read replays
  retained deliveries since that read. Each graph fact is indexed under
  the exact keys a commit can touch: an entity's lifecycle, an entity or
  relation kind, an aspect or field revision, a predicate field, a relation
  membership, an adjacency, an index membership, or an index definition.
  A consumed upstream output is an edge to that output's settlement, not a
  key. A fact that has no key, such as a retired output entity, cannot be
  marked, so an output that consumed one always verifies in full.
- **Marking.** Delivery looks up the touched fact keys and marks matched
  settlements dirty. Settlements that consumed their outputs become
  pending-upstream, transitively. The index selects affected settlements and
  downstream edges; delivery does not scan all settlements or waiting work.
  Marks belong to a branch lineage. Their maintenance is counted and bounded.
  If retained marking cannot cover a delivery, currentness requires full
  verification rather than treating missing marks as clean.
- **Clean reuse.** On a continuous retained basis, an unmarked settlement with
  no recorded full-verification requirement can
  certify currentness without re-reading its consumed facts. The marks alone
  decide this. The producer's
  [input reuse contract](#98-publication-live-reads-and-output-demand)
  gates input cutoff, not clean reuse.
- **Cutoff.** Dirty verification compares marked facts. Unchanged facts can
  establish currentness without recomputation. Equal declared input can avoid
  calling a producer. Stable republication of the retained output then clears
  pending marks below it without calling those consumers. The executable
  [`equal_clean_successor_passes_the_cutoff_and_verifies_the_full_handler_prefix`](../workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/output_lineage/invalidation/native_journey_tests/equality/pending_resolution.rs)
  requires the exact equal, clean successor and verifies the handler's full
  consumed prefix; an unrelated clean successor cannot clear the obligation.
- **Full verification.** When marking cannot certify the selected basis,
  currentness compares all consumed facts. Checkpoint restore
  can require this path. Verification is counted and budgeted.
- **Commit checks.** A commit re-compares its attempt's consumed read facts.
  A fact that changed ends the attempt as stale, with nothing committed.
  Demand-time marking does not replace that check.
- **Certification.** The owner tests compare marking with full verification
  and reject disagreement; diagnostic comparison is not a consumer mode.

The neutral [installed locality court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/exact_invalidation/locality.rs)
grows independent copies through ordinary commits at 1, 10 and 100 copies in
one runtime. It compares equivalent source edits, checks the exact contacted
output roots and consumed-output decisions, and verifies clean unrelated
outputs through their retained demand handles. The owned Query execution
observation also exposes native delivery preparation reports: exact affected
lineage identities and installed output bindings, logical marking work and
the actual marking meter charge. These describe preparation; a later refusal
can discard them, so they grant no publication or currentness authority.
The marking meter pays matching, propagation and selected hint preparation;
its charge excludes navigation, checkpoints and touch extraction. Matched
posting visits count before stale-posting or already-dirty filtering, while
marked ordinals and closure vertices describe distinct affected identities.
The court independently checks published values and distinguishes the marked
A/B/C closure from the A/B producer contacts that stop propagation at equal B.
Growth's ordinary mutation also retains its own output on A, distinct from
A's demanded producer. Physical lineage generations remain in the reports;
the court judges semantic root/binding sets without pinning alias counts.
An eviction fallback without a completed delivery report exposes unknown
work, rather than a fabricated zero. An interruption before delivery
preparation returns has no marking report; these funded baseline intervals
do not certify failed-preparation cost or the remaining resource matrix.
One commit can report provisional exact marking before a later retention
admission selects fallback. Commit identity alone cannot identify the selected
preparation; the native owner court checks the installed image independently
and keeps provisional work distinct from the fallback's unknown work.

Mutation and demand intervals separately drain those reports and actual fact
examination observations. Source admission, postcommit rebase preparation,
resolution and unanswered classification, source-currentness comparison,
value equality, retained work-bound walks, prior computation walks, native
fact probes and carried-fact sealing each have their own boundary counter.
Walks count facts before skipping or filtering; adding these counters would
double-count facts examined in several phases. Source-currentness work is the
existing meter's debit within comparison, including stopped comparisons, not
the whole request bill. Reports are passive thread-local diagnostics, taken
once and cleared when the final execution observation guard drops. They add
no admission policy or caller authority. The existing
[partition court](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/computation_partition/oracle/counts.rs)
compares equivalent authored value edits as an installed population grows and
reports the real retained-fact walks, separately from gather and kernel calls.

Source-query entries, request charges and whole-registry visits remain
separate observations. The baseline reports whole-registry visits during
ordinary advancement, so exact producer contacts do not certify total
locality. Its
[lifecycle cases](../workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/topology_entry/src/checkpoint_recovery/required_chain/exact_invalidation/locality/lifecycle.rs)
carry ordinarily authored values through native fork divergence and checkpoint
reopening, then demand the results of new edits. These cases do not certify
application product-branch merge or implementation-edition changes.

Query's native marking is distinct from Signal delivery after product
publication. Native marking is part of commit publication: Relational
prepares it before entering the branch's critical section and installs it in
the cutover that moves the branch cell. The number of readers a commit
affects never refuses a legal writer. A commit whose marking exceeds the
installed marking ceiling or the retained capacity is recorded as a counted
discontinuity, and readers verify in full across it. Signal delivery is a
later step
([§7](#7-one-change-through-the-whole-stack)).

---

## 11. Outcomes: every way a request can end

WORTH never collapses "didn't work" into one error. Each family calls for a
different response.

`execute(..)` and `execute_in_program(..)` both return
`Result<WorthQueryApplicationMutationOutcome, ..RequestMutationDenial>`:

- The `Err` side covers refusals **before any effect**: binding, principal,
  scope, authorization, source, idempotency, capability installation, product
  selection, program selection or mismatch, workflow requirements, and handler
  execution denials. A `Handler` denial arrives after the handler ran, but
  before anything was written. Every refusal returns before the commit, so
  none of them claims the idempotency key.
- The `Ok` side is the outcome of an attempt.

| Family | Variants | What to do |
|---|---|---|
| Landed | `Committed { receipt, result }`, `AlreadyCommitted(receipt)`, `PreviouslyCommitted(historical)` | Use the receipt. `AlreadyCommitted` is an idempotent replay. `PreviouslyCommitted` means the same intent committed earlier, but its live receipt and handler result are not retained: read current state with a fresh query. |
| Domain | `DomainDenied(denial)`, `IdempotencyIntentDrift` | Report it. New intent needs a new key. |
| Stopped | `Cancelled`, `DeadlineExceeded` | Nothing landed. |
| Not a clean landing | `Commit(WorthQueryApplicationUncommitted)` | Match the inner commit outcome (below). Some of these *did* move the branch. Never treat this family as "retry". |

Inside `Commit(..)`, the commit outcome is one of:

| Commit outcome | Meaning | What to do |
|---|---|---|
| `Stale`, `ProductStale` | The branch moved after your basis | Re-read, then retry with a fresh source |
| `ProductUnpublished` | Some owners moved, but the product head did not | Recover publication ([section 14.5](#145-product-publication-recovery)) |
| `NoEffect(cause)` | Nothing was written (owner denied before effect, capacity, owner unavailable, and so on) | Act on the typed cause |
| `Denied(denial)` | The commit was denied, with a typed kind | Act on the kind |
| `Deferred(..)` | A capacity or lifetime limit was reached, or a patch-position reservation was contended | Retry later; the kind names the cause |
| `Cancelled`, `TimedOut`, `Aborted` | The attempt stopped | Nothing landed |
| `Indeterminate(..)` | The landing is unresolved: *not* failed | Recover; `recovery()` says which way |

`WorthQueryApplicationCommitOutcome::landed()` splits any commit outcome into
`(receipt, is_replay)` or `WorthQueryApplicationUncommitted`. Note that
`landed()` returns `Err` for `Indeterminate`, whose commit may have landed:
"not a clean landing" is not the same as "nothing happened".

The two most important rules:

- **`Indeterminate` is not failure.** Do not retry blindly as if nothing
  happened. Recover.
- **An unknown durable answer is `Indeterminate`.** The commit was not
  installed but may be durable. The Relational owner refuses further
  publication until it is reopened, and reopening establishes the fate.

---

## 12. Branches, programs, and adoption

The calls, with real code:
[Build an Application §7](build-an-application.md#7-adopt-a-new-program-on-a-branch).
Full guide:
[Programs and Adoption](../workspaces/worth-query/crates/worth-query/docs/foundations/programs-and-adoption.md).
See also
[Branches and Previews](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md).

### 12.1 Product branches

A product branch is one live occurrence of the application's world, for
example a preview in which to try a change before it goes live.

- `WorthQueryProductBranch` is a `Copy` token. It names a branch and **grants
  nothing**. Every use selects the branch through the owning runtime again, so
  a closed, retired, or foreign branch is refused at selection.
- `current_world()` returns the live branch, as an ordinary branch token.
- `branches().fork(..).components(..).create()` forks a branch. For each
  component, the caller chooses to fork it or to reuse its exact basis. Leaving
  a component undecided is `ComponentsIncomplete`.
- A raw branch identity cannot select a branch. A compile-fail test proves
  this.

### 12.2 Programs

A program is a revision of the application's authored meaning. Three
identities stay separate:

| Identity | Means |
|---|---|
| Revision | What the program means |
| Support | This host can run it (it is rostered) |
| Activation | This branch runs it |

Each branch runs exactly one activated revision. A host may support several at
once. **A mutation never names its program.** The runtime reads which program
the selected branch runs and commits under it. If an adoption lands between
selection and commit, the commit is denied (`ProgramNotActiveOnOccurrence`)
before any effect.

### 12.3 Adoption

Adoption moves one exact branch from its active revision to a target
revision, in one product publication:

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled.
let programs     = request.on_branch(branch).programs();
let requirements = programs.compare(&target)?;          // what adoption needs
let adoption     = programs.adopt(&requirements)?;
let dispositions = WorthQueryWorkflowDispositions::carry_compatible(); // explicit fresh native policy
let prepared     = adoption.workflow(dispositions).prepare(max_work)?;
let outcome      = prepared.publish(runtime, &principal, &scope); // native publication outcome
```

- You must compare before adopting: the requirements are an input to `adopt`.
- Every pending custody item needs a supported disposition.
- Native preparation privately checks a total legal disposition for every
  current definition and live instance. Submit opaque-reference choices or
  explicit carry-compatible intent. The complete safety inventory is not
  disclosed. Consumer reads use an explicit typed definition and its installed
  governed read binding; metadata alone grants no enumeration. Some custody
  needs migration or blocks adoption until it settles.
- Adoption across a **set of branches is not atomic**. Each branch publishes
  on its own, and progress can be resumed.
- Support for a revision can be retired only when no branch and no retained
  custody still uses it.

---

## 13. Workflows

The calls, with real code:
[Build an Application §6](build-an-application.md#6-author-publish-and-run-workflows).
Full guide:
[Workflows](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md).

A workflow is a branch-local, versioned graph of steps. Its node kinds are defined by the
[source-derived workflow vocabulary](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md). There is no
read-only node kind; a read that feeds a decision is an assessment or a
condition. A workflow is authored against an installed vocabulary. A purchase
request, for example, goes from draft, through budget and vendor checks, to a
manager's approval, and then to placing the order.

Design choices in the workflow kernel (the part of Query that authorizes,
adjudicates, and records workflow steps; applications reach it through the
workflow runtime value, `WorthQueryWorkflowApplicationRuntime`):

- **Every workflow action is a governed mutation.** Each control step is a
  declared mutation intent whose binding sets `WORKFLOW_CONTROL`, gated by
  one installed control capability. The authoring capability publishes and
  retires definitions. The instance-start capability starts, cancels,
  migrates, and continues instances on a fork. The advance capability
  advances and navigates back. An approval's own capability authorizes it,
  and a fresh authentication event signs it. The
  kernel authorizes and records these control steps itself. No handler may
  serve one: registering a handler for one is refused.
- **Progress is caller-pumped.** There is no background scheduler, and no
  timer advances an instance. Each advance is one request. An unauthorized
  caller sees `AwaitingActor`.
- **An inbound completion never runs a step.** A definition can wait at an
  `await_inbound` node for the external effect of an earlier operation node.
  The installed inbound source completes that original operation. It does
  not run a workflow node, and neither callback bytes nor a correlation
  token can resume the instance. Until the owner accepts the operation's
  receipt, advance returns `AwaitingOperation`. A later advance observes the
  terminal result at the node and may enter its successor.
- **Advance names what it waits for.** `WorkflowProgressOutcome` returns
  `Completed` when a step was recorded, or one of `AwaitingActor`,
  `AwaitingAssessment`, `AwaitingCondition`, `AwaitingOperation`,
  `AwaitingEvidence`, `AwaitingProposal`, and `AwaitingApproval` for what the instance needs next.
  A request that recorded no step says why: `Application` (the commit did not
  land), `PreparationDenied`, `AuthenticationDenied`, or `IdempotencyDenied`.
  `ProjectionDenied` is different: the commit landed, carries its receipt,
  and only its projection was refused.
- **Approvals need fresh authentication and a signature.** Replaying a
  recorded key is the only exception. An approval is evidence, not
  permission, and a guarded operation needs one only when its binding or
  definition asks for it ([§13.1](#131-who-may-run-a-guarded-operation)).
- **Definitions have explicit predecessors.** Publishing names the revision it
  expects to replace. There is no last-writer-wins overwrite.
- **Instances pin their definition revision.** A new revision supersedes the
  old one for new starts. Running instances keep their revision.
- **Cancellation is not rollback.** Canceling ends an instance where it
  stands, and the outcome names every step whose effect was performed.
- **Owner custody blocks unsafe moves.** While an external operation is
  unsettled, cancellation, migration, and fork continuation are refused.
- **Everything is bounded:** steps and evidence bytes per instance lineage (an
  instance and the instances it was migrated or continued from), live
  instances per definition lineage, and an optional total deadline on the
  trusted clock.

### Recorded actors and exact-basis observation

The workflow record owns canonical proposer, advancer, performed-operation
executor and approver identity. Admission authors each role using the complete
native principal identity; a later settler does not replace the original actor.
Positive recorded-actor requirements extend the existing capability pipeline:
the exact required occurrence must be present and satisfy its declared comparison.
A graph prohibition alone cannot establish positive presence. Current grants,
scope and revocation still decide whether the acting request may proceed.

Observation selects an exact retained data basis and separately admits the current
reader. Subject disclosure and actor audit have independent rules. A withheld
audit does not erase an approval or an incomplete owner obligation. Read targets,
pages and original-start intents carry data rather than permission. They do not
convert to an operation need or approval authority.

Live marking reuses the native envelope/touched-fact subscription. A relevant
change marks the subscription without replacing the old retained basis. Refresh
requires a new retained read and reacquisition. Releasing a subscription is
separate from settlement and cleanup of the original operation. The
[mixed workflow journey](build-an-application.md#67-capture-approve-archive-and-observe-one-workflow)
shows these receivers in one owning application.

### 13.1 Who may run a guarded operation

A workflow-bound business operation passes up to three checks. They answer
different questions, are declared in different places, and none grants another.

| Check | Question it answers | Declared by | Checked when |
|---|---|---|---|
| Operation authorization | May this principal perform this business operation on this scope? | The operation's capability or policy binding | Ordinary mutation admission, on every execution, including workflow-bound ones |
| Workflow occurrence authority | Is this exactly the operation, input, and branch the instance is waiting for now, and not already spent? | `workflow_authority required` on the mutation binding | Issued by an advance that reaches a guarded Operation node (`AwaitingOperation`); bound by `for_workflow_operation`; rechecked at commit |
| Required approval evidence | Must a current signed Approve decision also exist? | The binding's `workflow_requirements` plus any `approval_authority` edge authored in the definition | Resolved by the advance before authority is issued; the approver's grant is rechecked at commit |

An unguarded mutation needs check 1 only. A guarded mutation needs checks 1
and 2, always, and check 3 when the installed requirement asks for an approval
or the definition wires an `approval_authority` edge to it. Approving needs the
approval capability plus a fresh signature; advancing needs the advance
capability. Neither satisfies check 1: permission to pump an instance is not
permission to execute the business operations in it.

| `workflow_requirements` declared | Meaning |
|---|---|
| Guarded requirement omitted | Compile refusal; manual bindings must implement the requirement method explicitly. |
| `ApplicationWorkflowExecutionRequirements::not_applicable()` | Explicit non-workflow applicability; refused for a guarded binding. |
| `ApplicationWorkflowExecutionRequirements::occurrence_only()` | No additional evidence. Zero or one approval edge; an authored edge is still enforced. |
| `ApplicationWorkflowExecutionRequirements::requiring_approval::<Cap>()` | An approval by `Cap` is required, and an edited definition cannot remove it. |

Guarded bindings explicitly choose `occurrence_only()` or a mandatory approval
capability. Omission is a compile refusal; it has no legacy interpretation.

### 13.2 Propose, authorize, execute

An Operation node is a proposal slot or a guarded effect, decided by its
binding. At an unguarded Operation node, `prepare_workflow_proposal` records
the operation, input type, an input-identity digest, and the node path. It
runs no handler, writes no business state, and does not retain the input value.
Assessments and conditions take their subject from the proposal
(`proposal_for_assessment`, `condition_subject`) and read that subject's
current state on the branch, never the proposed value. At a guarded node (`workflow_authority required`), an advance returns
`AwaitingOperation` carrying single-use authority for that occurrence; nothing
is committed. The ordinary caller captures the original input fallibly with
`idempotency_owned`, binds the description with `for_workflow_operation`, and
retains the installed declared action. Each owning `advance` takes a fresh
request and delegates the existing native admission, proposal-input matching
and atomic business-write/node-settlement funnel. Refusal and pending results
return the original owner; close releases its native cleanup separately.
Manual `execute_in_program` uses the explicit advanced entrance. A subsequent
workflow advance selects the successor; source completion alone is not that
workflow advance.

Full rules:
[Workflows, Operations](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md#operations).
Real calls:
[Build an Application §6](build-an-application.md#6-author-publish-and-run-workflows).

---

## 14. Aftermath and recovery

### 14.1 Declared aftermath

Each operation declares its aftermath posture: **reversible**,
**compensatable**, **reconcilable**, or **irreversible**. A typed correction
binding connects that meaning to an installed operation and a disclosure query.
Recorded inverses restore captured prior truth; compensations create an offsetting
change; reconciliations bring local records into agreement with an outside party.
External effects use their declared compensation or reconciliation policy.

### 14.2 Undo, redo, reconciliation

Start at `request(&principal, &scope).on_branch(branch).corrections()`.
Select the retained occurrence to correct. Inspection runs the binding's
disclosure query under current authority; preparation derives its declared
inverse from the retained input and captured pre-image, then admits the new
operation on the selected branch. A successful correction appends a child of
the current branch head and preserves intervening unrelated edits.

`alternatives()` pages the retained correction lineage. Finish its continuation
before selecting an alternative; `reapply()` then executes the retained original
input under current authority and current handler rules. This supports choosing
an older alternative after further edits and correction cycles. Each command
uses its own idempotency key, so retrying a performed command returns its
existing result.

The returned ordinary operation owns publication, required-output delivery and
close. Observe source publication separately from derived-output completion.
If retained material has expired or current admission refuses the operation,
handle the typed outcome and let the user choose a currently supported action.

See [Recorded source correction](../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md)
for binding declarations, complete caller examples, alternatives, retry and recovery.

### 14.3 Unresolved durability

Relational makes a commit durable before it installs it live, so a performed
commit always settles. When the durable medium cannot say whether a commit
became durable, nothing is installed, the commit may still be durable, and the
Relational owner refuses every later publication until it is reopened.
Reopening reads the medium's recovered journal, which decides the commit's
fate. Query reports this as `Indeterminate` (commit recovery required); a
World publication retains it as `ProductUnpublished` with cause
`RelationalDurabilityUnresolved`, whose only continuations are release,
inspection and closing the owner. Never retry such a commit blindly.

### 14.4 External effects

A commit that declares an external effect writes a dispatch outbox record in
the same atomic commit. The effect's posture is `NotDeclared`,
`PendingDispatch`, `Acknowledged`, `Completed`, or `Unresolved`. **A commit is
not external completion.** A committed order confirmation has not been
delivered until its posture says so.

### 14.5 Product publication recovery

A `ProductUnpublished` partial, from the World or from adoption, carries a
recovery value. That value cannot be duplicated, and it exposes the next
actions that are allowed. Recovery continues the performed effects. It never
rolls them back.

---

## 15. Durability and the other surfaces

Everything above runs in memory. Relational, Signal, Runtime World, and Query
hold their state in the process. This section covers the surfaces beside the
stack: where durability will come from, how applications reach the network
and the screen, and how the platform proves itself. The
[Platform README](../README.md) has the full crate map.

### 15.1 Store: durable physical survival

**WORTH Store** (`workspaces/worth-store`, 42 crates) is the durable physical
foundation. Its job is narrow and hard: make accepted physical records survive
process failure, reopen persisted roots in a fresh process, and say plainly
when something is corrupt or when an outcome is unknown.

| Store owns | Store does not own |
|---|---|
| The write-ahead log and the write path | Whether a Query operation is legal |
| Page, segment, and blob formats; the buffer pool; I/O scheduling | What Relational truth means |
| Fresh-process recovery from WAL and checkpoints | Treating a checksum as proof of authenticity |
| Checksums, scrub, quarantine, isolation, tenant security | Inventing a semantic result when bytes are missing |
| Retention, tiering, replication, backup, repair | |

The same laws apply here as everywhere else. Recovery is an explicit path with
its own types, not a silent retry. An indeterminate write is reported as
indeterminate. Independent verifiers (`worth-store-offline-verifier`,
`worth-store-offline-integrity-observer`) read persisted bytes without trusting
the writer.

**Current state:** no crate outside the Store workspace depends on Store yet.
Query's store-backed execution parity and durable restore are planned work.
Nine crates for semantic durable programs (snapshots, branch deltas, schema
lineage, live-query sync, subscriptions, bulk ingest, extensions, analysis,
claim boundaries) exist but are dormant. Until the wiring lands, treat Query
applications as memory-resident.

To survive a restart today, use the app-owned checkpoint lane. The installed
runtime captures an opaque `WorthQueryApplicationCheckpoint` with
`capture_application_checkpoint_with_sections`; the application stores and
transports its bytes, and reinstalls with
`in_memory_rostered_program_from_checkpoint` (or another `*_from_checkpoint`
constructor), which readmits committed truth and accepted-output identities.
Live handles and retained observations are not restored.
See [Build an Application §4.4](build-an-application.md#44-checkpoint-program-transitions).
Do not build host persistence of platform truth as a substitute.

### 15.2 Server: the transport boundary

`worth-server` (`worth_server::facade::{WorthServer, WorthServerBuilder}`)
serves a Query application over the network. It offers WORTH-native
operations, plus a strict HTTP compatibility boundary for reads, writes,
streams, uploads, and downloads.

The server is transport, not authority. It carries a request to Query's
admission and carries the typed outcome back. It does not decide legality. In the
Bank reference world, identity-provider tokens end at the HTTP adapter; what
reaches Query is an authenticated principal and a request scope. `worth-server-client-generation`
renders TypeScript and Python clients from the server's protocol catalog.
Guide: [Server docs](../crates/worth-server/docs/README.md).

### 15.3 UI: authored presentation

**WORTH UI** (`workspaces/worth-ui`) owns authored UI meaning, compilation,
the active application, planning, mounting, interaction, native and headless
hosts, and read-only inspection. It keeps three distinctions that most UI
stacks blur:

- **Native input is not user intent.** A click is a mechanical event. UI
  admission decides whether it becomes an intent.
- **UI admission is not domain admission.** The UI may admit an intent and
  Query may still deny the operation. Both answers are typed.
- **Inspection grants nothing.** An inspection receipt shows what happened. It
  cannot be used to change anything.

`worth-ui-query-binding` is the only crate that turns Query products into UI
registrations and observations. Applications import `worth-ui` and, for a
native window, `worth-ui-native-platform`. Guide:
[UI AI discovery](../workspaces/worth-ui/AI_README.md).

### 15.4 Signal in the browser

`worth-signals-wasm` is Signal compiled to WebAssembly, with a TypeScript
package on npm. It is worker-first: state, resources, forms, and routing run
off the main thread, with React integration. The laws do not change in the
browser: derived work is still derived, and still reconstructible. Guide:
[Signal Wasm README](../crates/worth-signal-wasm/README.md).

### 15.5 Harness: how the platform certifies itself

`worth-harness` provides scenario, capture, replay, comparison, and workload
infrastructure. The runtimes use it to test themselves at their real
boundaries: run a scenario, capture what happened, replay it, and compare the
two exactly. It is not a product API. Query's certification audience
(`worth-query-replay`, `worth-query-certification`) is the application-level
counterpart. See [API Map §4](api.md#4-certification-api).

### 15.6 The Bank reference world

`workspaces/worth-query-bank-world` is a complete application built the way an
outside team would build one. It has a domain crate, a server, an HTTP adapter
with OIDC, independent per-user client processes, and a separate-process
external service that injects faults on purpose. It imports Query only
through `worth-query-decl` and `worth-query-host`.

It exists to prove the consumer boundary under pressure: money that must not
be lost or duplicated, external effects that can fail halfway, and users acting
concurrently from separate processes. Banking is the test load, not the
subject. Read its
[public consumer contract](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md)
to see the two facades used end to end.

---

## 16. Misconceptions

| Misconception | Reality |
|---|---|
| "Any `AuthorityWitness` opens a governed operation." | Governed doors require concrete, owner-sealed types. Query's authority is not even nameable from the consumer facades. |
| "A digest or descriptor is authority." | It is an address or a description. Only an owner-admitted basis selects or grants. |
| "Foundational owns commits and merges." | It owns the *nouns*. Relational, Signal, and Runtime World perform and mint them. |
| "One commit updates everything atomically." | Each owner's step is atomic. The whole is ordered, and a gap is a named, recoverable partial. |
| "Conflicts are detected per record." | Relational conflicts are per branch. Any movement since your observation is `Stale`. |
| "The runtime retries or rebases for me." | It never does. Start a fresh transaction or request. |
| "Signal invalidation is part of the commit." | It is a separate delivery step after the product publication performs. Query's native marking is the part that moves with the commit: it is installed in the cutover that moves the branch cell. |
| "The Bridge decides whether output changed." | Signal decides. The Bridge carries the evidence. |
| "A branch token is a capability." | It names a branch and grants nothing. |
| "The declared touch ceiling says what changed." | Only commit-sealed touched records say that. |
| "A clean output is reused only if its producer allows it." | Clean reuse depends on the marks alone. The producer's `INPUT_REUSE` contract gates input cutoff, a different saving. |
| "Serial placement is unbounded." | It is the same request with one worker. The policy's memory limit, the request's cancellation and deadline, and every work ceiling still bind it. |
| "Reuse makes a partitioned computation cheaper to charge." | An outcome never depends on reuse. A carried partition charges what it charged in a full run. |
| "`Indeterminate` means it failed." | It means unresolved. Recover. |
| "Committed means the email was sent." | Commit records the outbox entry. External dispatch has its own posture. |
| "Canceling a workflow undoes its work." | Cancellation names performed effects and stops. It does not roll back. |
| "Upgrading the host upgrades every branch." | Adoption is explicit and per branch. |
| "Closing the last observer stops the workflow." | Observers hold interest only. The instance keeps its state. |

---

## 17. Summary for AI agents

- Import only `worth-query-decl` (to declare) and `worth-query-host` (to
  install and run). Everything in this document below Query is how the
  platform keeps its promises. You do not call it directly.
- Write application code from
  [Build an Application](build-an-application.md): features, program,
  installation, workflows, and adoption, with real calls.
- Declare exactly what each operation reads and touches. The declaration is
  the ceiling, and admission enforces it.
- Build every mutation from an observed source (`expect_source`) unless the
  binding declares none. Always pass an idempotency key.
- Match every outcome variant. Treat `Indeterminate` as recovery, never as
  failure. Treat `Stale` as "re-read, then retry".
- Use the [request and retained-operation guide](capability-index.md#use-requests-and-retained-operations)
  to submit, advance, recover and close application work.
- Read touched records, external dispatch posture, and aftermath from the
  receipt. Do not infer them. Touched records sit behind
  `receipt.mutation_work()`, which is an `Option`.
- Workflows move only when a request pumps them. Plan for `Awaiting*` outcomes.
  An inbound completion finishes an operation; it never runs a step.
- Keep `partitions`, `partition_key`, `gather`, and `compute_partition` pure. A partition
  whose facts did not change keeps its last result without being gathered or
  computed again, so an impure function yields a stale result.
- Expect one execution request per public call. With sufficient memory,
  worker count never changes a result or its charged work; cancellation and
  deadlines can change how far a call gets.
- When a name you need is not on the audience facades, the answer is a
  missing platform capability to report. It is not a reason to reach into an
  internal crate. Do not build a substitute. Follow the missing-capability
  procedure in the [Capability Index](capability-index.md) before reporting.
- A guarded workflow operation still needs ordinary operation permission.
  An approval is evidence, not permission, and is required only when the
  binding or definition asks for one
  ([§13.1](#131-who-may-run-a-guarded-operation)).

---

## Related documents

- [Philosophy](philosophy.md): why the platform is shaped this way
- [Glossary](glossary.md): exact definitions of every term used here
- [Build an Application](build-an-application.md): the application story, end to end, in real code
- [API Map](api.md): which crate to import, and where the reference documentation is
- [Platform README](../README.md): the crate map
- [Query engine architecture map](../workspaces/worth-query/crates/worth-query/docs/AI_README.md): internal; where each request stage lives inside the Query crates, for maintainers
- [Coding Guidelines](coding-guidelines/): the laws that bind platform code
