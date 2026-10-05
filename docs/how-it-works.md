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
- **The branch reference cell is the unit of exclusion**, not the runtime.
  Unrelated branches make progress concurrently.

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

Signal is a deterministic, incremental runtime for derived work. Its rule
is: *Signal never owns source data.* Under Query, Relational owns it. Signal owns dependency tracking,
invalidation, recompute, rollback, diagnostics, replay, installed condition
decisions, and the decision about whether an output changed meaningfully.
Signal is never truth. It consumes snapshots and emits derived refreshes.

- **Graph.** `SignalGraph` is the map of what depends on what, and
  `SignalRuntime` executes it.
- **Aspects in Signal are runtime-local slots.** They are not Foundational
  semantic identities and are never persisted in portable Query packages.
- **Change intent.** `mark_changed` says that a producer-local aspect *may*
  need recomputing. It does not claim the producer emitted a semantic change.
- **Scoped invalidation.** Only a meaningful difference in committed output
  creates downstream causes. The producer declares output equivalence, and the
  consumer declares a dependency comparator. A downstream hop starts only
  after a node commits its own output. Nothing is marked transitively ahead
  of time.
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
  identity and revision, binding, change kind, field path, precision, source,
  and commit identity. It must match an installed correspondence before any
  Signal target updates. Labels, equal numeric slots, and digests cannot
  authorize delivery.
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

**What is atomic, and what is not.**

| Atomic | Not atomic |
|---|---|
| One Relational branch-cell move | A combined Relational-plus-Signal publication |
| One Signal branch advance (it rolls back cleanly before commit) | The sequence of owners as a whole: it is *ordered*, not atomic |
| One World product-head compare-and-swap | Invalidation, which follows the commit as a separate delivery step |

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
| **Performed is not settled** | A moved branch reference still owes durable settlement and publication (`SettlementDeferred`). |
| **Committed is not externally complete** | A commit records an outbox entry. Delivery to the outside world has its own posture. |
| **History is not authority** | History presence is not performed authority. A cloned receipt is descriptive. |
| **Reporting is not authority** | Readiness reports, basis tokens, discovery answers, and diagnostics grant nothing. |
| **Causality is not permission** | An undo proves inversion. A redo must be admitted again. |

---

## 9. Query: the life of one request

This section follows one request through the consumer facades. The running
example is an order application with an operation `approve_order`. That
operation reads `Order.status`, writes `Order.status`, and links the order to
an `Approval`.

### 9.1 Declare

The application describes its schema, operations, queries, capabilities,
workflows, and aftermath with `worth-query-decl` macros and types. Nothing
runs. A declaration cannot install, authenticate, authorize, execute, or
publish.

- Schema: `worth_query_application_schema`, `worth_query_entity`,
  `worth_query_aspect`, `worth_query_field`, `worth_query_relation`.
- Operations: `worth_query_operation` and the family that declares what it
  touches: `worth_query_operation_reads`, `_writes`, `_creates`, `_deletes`,
  `_links`, `_unlinks`, `_emits`, `_requires`, `_expects_fact`,
  `_expects_version`.
- Bindings: `worth_query_mutation_binding`, `worth_query_query_binding`.
- Capabilities and policy: `worth_query_capability`, `worth_query_ability`,
  `worth_query_policy`, `worth_query_principal_binding`.
- Workflows: `worth_query_workflow`.
- Features and programs: each `ApplicationFeature` owns actions through an
  `ApplicationFeatureSpec`. An `ApplicationProgramDefinition<Schema>` lists
  its contributions, features, output graph, and rules.
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

```rust
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
- `in_memory_rostered_program` supports several program revisions against one
  installed schema.
- `in_memory` installs schema meaning without a program.
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

### 9.3 Request entry

```rust
let request = runtime
    .request(&principal, &scope)   // WorthQueryApplicationRequestExt
    .on_branch(branch);            // optional; defaults to current_world()
```

Constructing a request selects no state and resolves no principal. It is a
description of intent. From here, the request becomes one of:

| Call | Purpose |
|---|---|
| `.query(intent)` | Read |
| `.mutate(input)` | Change |
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

**Queries** narrow their limits with `.limits(max_results, max_work)`. Limits
only narrow: an attempt to widen them is a typed `Limit` denial.

**Mutations** declare what they were based on, which is a typestate step:

```rust
let outcome = request
    .mutate(ApproveOrder { order })
    .expect_source(observed_source)   // or .expect_result_set(..) / .without_source()
    .idempotency(&key)
    .execute()?;   // runtime installed without a program; see "Ways to execute" below
```

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

```rust
impl OperationHandler<Schema, ApproveOrderBinding> for ApproveOrderHandler {
    fn decide(&self, input, reader: &mut DecisionReader<..>)
        -> HandlerResult<Decision, Denial> { .. }

    fn candidate_requirements(&self, input, decision)
        -> ApplicationCandidateRequirements { .. }

    fn build_candidate(&self, input, decision, candidate: &mut CandidateWriter<..>)
        -> HandlerResult<Result, Denial> { .. }
}
```

1. **Decide.** The handler reads through `DecisionReader` and decides. It may
   deny with a domain denial, such as "the order is already approved".
2. **Reserve.** The handler states its resource requirements. The runtime
   reserves them against the installed ceiling *before* any allocation.
3. **Build.** The handler writes the candidate through `CandidateWriter`.
   Nothing outside the writer can change the candidate.

`HandlerResult` distinguishes `Completed`, `DomainDenied`, `ExecutionDenied`,
`Cancelled`, and `DeadlineExceeded`.

**Ways to execute a mutation.**

| Lane | Use it when |
|---|---|
| `execute()` | The runtime was installed without a program (`in_memory`). On a runtime installed with a program, it refuses every mutation with `ApplicationProgramRequired`. |
| `execute_in_program(&program_runtime)` | The runtime was installed with a program (`in_memory_program` or `in_memory_rostered_program`). Every mutation on such a runtime uses this lane or one of the program lanes below. The runtime reads which program the branch runs; the caller never names it. |
| `execute_capability_in_program` | As above, through a capability. |
| `execute_retained` / `execute_retained_in_program` | You want to commit and keep a retained read of the result. `execute_retained` is for a runtime without a program, like `execute()`; on a program runtime use `execute_retained_in_program`. |
| `execute_performed` / `execute_performed_discovered` | You need the program outputs the operation requires to be produced after commit. |

`commit_for_program`, reached through `WorthQueryProductEntry::transaction()`,
is an advanced product-transaction lane for hosts. Ordinary applications use
the request lanes above.

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
| Which records did the commit actually touch? | `mutation_work()` → `touched_records()` |
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
  `advance(&fresh_request)` returns `Pending` or `Settled`. One call
  progresses every dirty and pending-upstream output in the *required set*, in
  dependency-ready waves, until it settles or its budget runs out. When the
  budget runs out first, the typed `Pending` outcome names the remaining work.
  The application neither re-demands its output tree nor polls, and no
  background sweeper runs: the authenticated request stays the principal.
- **Required outputs.** After `execute_performed`, `start_required_outputs`
  produces the outputs the operation requires, and `recover_required_outputs`
  resumes that work. Those outputs join the required set until their demand
  closes.
- **Reuse.** An output whose settlement is unmarked on a continuous basis is
  reused without contacting its producer or re-running its source query. A
  dirty output re-verifies only its marked facts; when the value its producer
  would receive is unchanged, the producer is not called (input cutoff), and
  when a recomputed value encodes identically, its consumers stay current
  (output cutoff). [§10.5](#105-marking-and-currentness) gives the rules.

### 9.9 Resources and budgets

Every resource is bounded, and caller limits can only restrict installed policy.

Ordinary callers do not calculate Query's internal traversal or validator work.
Installed host profiles supply finite operational safeguards; declarations state
semantic result/effect scope and may add deliberate tighter caps. Candidate
validator allowances derive from the installed invariant closure. Composed demand
admission intersects host, caller and child ceilings, then checks the child's
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

Sessions, runs, subscriptions, continuations, leases, checkpoints, recovery
handles, and admitted capacity are *managed resources*. Close them
explicitly.

---

## 10. The touched graph

The touched graph is the exact set of changes a commit made. Relational seals
changed records, aspect field paths, adjacency changes, observable revision
bumps, and old/new index keys into its published patch envelope. Every writer
contributes that commit evidence. It also supplies the cause of invalidation:
the runtime intersects it with consumed facts to determine affected consumers.
A producer that declares coarser precision carries and reports the widening.

Scope paths narrow precision within touched records. Shards determine
placement. Signal's `Visited` observation tier reports consideration during
transaction processing; the commit's touched graph reports actual changes.

"What did this operation change?" has three answers in WORTH. They are
different questions, and the platform keeps them apart.

| Layer | Question | Type | Created by |
|---|---|---|---|
| 1. Declared ceiling | What *may* this operation change? | `WorthQueryOperationTouchContract` | Installation, from the operation declaration |
| 2. Admitted candidate touches | What does this candidate *propose* to change, and is it within the ceiling? | Checked by `admit_validated_application_touches` | Relational validation of the candidate, then Query touch admission |
| 3. Commit-sealed touched records | What *did* the commit change? | `WorthQueryTouchedRecordIdentity`, read through `receipt.mutation_work().touched_records()` | The commit seal, from Relational's changed records |

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

Each touch contract also produces a sealed graph-obligation row of kind
`MutationTouch`, whose terminal requirement is touched-scope evidence.

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

### 10.4 Example

`approve_order` declares that it writes `Order.status` and links an order to
an `Approval`.

- A candidate that also writes `Order.total` fails touch admission, because
  `Order.total` is outside the ceiling.
- Suppose the handler decides no link is needed this time. A candidate that
  writes only `Order.status` passes. After commit,
  `touched_records()` lists only that order record, even though the ceiling
  also allowed a link.
- A typed *emit* from the operation is not a graph touch.

### 10.5 Marking and currentness

The touched graph is also the cause of invalidation. Query owns one Bridge
subscription that receives every committed patch envelope on a branch, in
commit order and synchronously with commit visibility, whoever the writer was.

- **Reverse index.** When an output settles, Query records the facts it
  consumed (field revisions, index keys, selection and absence facts) in a
  reverse index from fact to settlement. A settlement recorded against an
  older read replays the deliveries since that read before it is indexed.
  Facts come from native query, adjacency or indexed-selection evidence at
  the smallest sound scope.
- **Marking.** A delivered commit looks up only its touched keys. Each match
  marks that settlement dirty. Every settlement that consumed a marked
  output is marked pending-upstream, transitively. No commit scans
  settlements or waiting work, and marks are per branch lineage. A wake or
  touched set narrows which deliveries reach a settlement, but never
  replaces admission's currentness proof.
- **Clean reuse.** On a continuous basis (same lineage, read basis still
  inside the retained commit window, demand at or after that basis), an
  unmarked settlement is current. Nothing is re-read or hashed.
- **Dirty recompute.** A dirty settlement re-verifies only its marked facts.
  If they changed, it recomputes, subject to input cutoff. When an upstream
  republishes an equal value, the pending marks below it clear without any
  producer contact.
- **Full-verification fallback.** Revision comparison of every consumed fact
  remains only where marking cannot be trusted: checkpoint restore or reopen,
  a read basis outside the retained window, a switch of branch lineage, a
  foreign basis, program or schema installation, a contract-revision or
  index-definition change, or a delivery gap. It runs once per affected
  output and is counted and reported.
- **Commit-time checks.** A commit still re-compares its own attempt's read
  facts. Marking replaces only demand-time re-verification.
- **Equivalence mode.** A certification feature runs full verification beside
  marking and fails on any difference.

Work budgets charge marking and dirty verification, never a scan of clean
state. Live queries are not invalidated: they are caused by committed
application emissions and re-read at the cause's observation.

---

## 11. Outcomes: every way a request can end

WORTH never collapses "didn't work" into one error. Each family calls for a
different response.

`execute()` and `execute_in_program(..)` both return
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
| Landed | `Committed { receipt, result }`, `AlreadyCommitted(receipt)` | Use the receipt. `AlreadyCommitted` is an idempotent replay. |
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
| `SettlementDeferred(..)` | **The branch moved**, but durability or publication did not finish | Recover settlement ([section 14](#14-aftermath-and-recovery)) |
| `Indeterminate(..)` | The landing is unresolved: *not* failed | Recover; `recovery()` says which way |

`WorthQueryApplicationCommitOutcome::landed()` splits any commit outcome into
`(receipt, is_replay)` or `WorthQueryApplicationUncommitted`. Note that
`landed()` returns `Err` for `SettlementDeferred` even though the branch moved:
"not a clean landing" is not the same as "nothing happened".

The two most important rules:

- **`Indeterminate` is not failure.** Do not retry blindly as if nothing
  happened. Recover.
- **`SettlementDeferred` is performed but not settled.** The change happened.
  Finish the settlement. Do not redo the change.

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

```rust
let programs     = request.on_branch(branch).programs();
let requirements = programs.compare(&target)?;          // what adoption needs
let adoption     = programs.adopt(&requirements);
let inventory    = adoption.workflow_inventory(max_work)?;
let dispositions = /* a decision for every workflow, built from `inventory` */;
let prepared     = adoption.workflow(dispositions).prepare(max_work)?;
let outcome      = prepared.publish();                  // Performed | NoEffect | ProductUnpublished
```

- You must compare before adopting: the requirements are an input to `adopt`.
- Every pending custody item needs a supported disposition.
- Every workflow definition and instance on the branch needs a legal
  disposition (carry, retire, or cancel). Some need migration, and some block
  adoption until they settle.
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

A workflow is a branch-local, versioned graph of steps: operations,
assessments, conditions, approvals, evidence joins, and terminals. There is no
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
- **Progress is caller-pumped.** There is no background scheduler, no callback,
  and no inbound-completion API. Each advance is one request. An unauthorized
  caller sees `AwaitingActor`.
- **Advance names what it waits for.** `WorkflowProgressOutcome` returns
  `Completed` when a step was recorded, or one of `AwaitingActor`,
  `AwaitingAssessment`, `AwaitingCondition`, `AwaitingOperation`,
  `AwaitingEvidence`, and `AwaitingApproval` for what the instance needs next.
  A request that recorded no step says why: `Application` (the commit did not
  land), `PreparationDenied`, `AuthenticationDenied`, or `IdempotencyDenied`.
  `ProjectionDenied` is different: the commit landed, carries its receipt,
  and only its projection was refused.
- **Approvals need fresh authentication and a signature.** Replaying a
  recorded key is the only exception.
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

---

## 14. Aftermath and recovery

### 14.1 Declared aftermath

Each operation declares its aftermath posture: **reversible**,
**compensatable**, **reconcilable**, or **irreversible**. An escaping external
effect cannot be reversible. Provisional aftermath is never accepted state.

### 14.2 Undo, redo, reconciliation

The host `provisional_aftermath` module provides `admit_undo`,
`progress_admitted_undo`, `progress_admitted_redo`,
`progress_admitted_reconciliation`, and related functions.

- An undo request is one of three kinds:
  - `RecordedInverse`: the recorded inverse of the change;
  - `Compensation`: a new change that offsets it;
  - `Reconciliation`: brings recorded state back in line with an outside
    party.
- Undo is built from the commit-sealed touched records and the retained
  pre-image, never from caller claims.
- An irreversible effect cannot be undone (`deny_irreversible_undo_attempt`).
- A completed undo proves inversion. It does not authorize a redo: a redo is
  admitted afresh.

### 14.3 Settlement recovery

`SettlementDeferred` means the branch moved but durability or Query
publication did not finish. Its `next_action()` is always the same:

```rust
runtime.recover_deferred_application_settlement(&deferred)
```

This returns the Relational commit receipt, or a typed recovery error. The
deferred value's `relational_settlement_commit()` is diagnostic only. It
cannot authorize dispatch or current-state reads.

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
| "Signal invalidation is part of the commit." | It is a separate delivery step after the product publication performs. |
| "The Bridge decides whether output changed." | Signal decides. The Bridge carries the evidence. |
| "A branch token is a capability." | It names a branch and grants nothing. |
| "The declared touch ceiling says what changed." | Only commit-sealed touched records say that. |
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
- Match every outcome variant. Treat `Indeterminate` and `SettlementDeferred`
  as recovery, never as failure. Treat `Stale` as "re-read, then retry".
- Never pass a program to a mutation. The branch decides which program runs.
- Never treat a report, token, digest, receipt clone, or discovery answer as
  authority.
- Read touched records, external dispatch posture, and aftermath from the
  receipt. Do not infer them.
- Workflows move only when a request pumps them. Plan for `Awaiting*` outcomes.
- When a name you need is not on the audience facades, the answer is a
  missing platform capability to report. It is not a reason to reach into an
  internal crate.

---

## Related documents

- [Philosophy](philosophy.md): why the platform is shaped this way
- [Glossary](glossary.md): exact definitions of every term used here
- [Build an Application](build-an-application.md): the application story, end to end, in real code
- [API Map](api.md): which crate to import, and where the reference documentation is
- [Platform README](../README.md): the crate map
- [Query engine architecture map](../workspaces/worth-query/crates/worth-query/docs/AI_README.md): internal; where each request stage lives inside the Query crates, for maintainers
- [Coding Guidelines](coding-guidelines/): the laws that bind platform code
