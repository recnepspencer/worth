# Milestone 9.16 Runtime Phase 8: Application Aftermath, External Effects, And Recovery

> **Status:** Completed. Its undo and redo sections are provisional; [Milestone 9.18](./milestone-9.18.md) owns their final semantics.

**Owner:** Runtime Hardening Track, Phase 8
**Developer guide:** [`Application Aftermath, External Effects, And Recovery`](../../workspaces/worth-query/crates/worth-query/docs/execution/application-aftermath-and-recovery.md)
**Canonical milestone:** [`milestone-9.16.md`](./milestone-9.16.md)
**Predecessor:** Runtime Phase 7, closed through Gate 7.7 Gate D
([`milestone-9.16-runtime-phase-7-closure-ledger.md`](./milestone-9.16-runtime-phase-7-closure-ledger.md))
**Inherited substrate:** Milestone 9.16.1 canonical graph-obligation and
provider-session authority ([`milestone-9.16.1.md`](./milestone-9.16.1.md))
**Unblocks:** Bank World Phase 5

This document refines the Phase 8 section of `milestone-9.16.md`. It does not
replace it, relax it, or renumber its gates. Where this document is more
specific, the specific text governs. Where it contradicts the milestone, the
milestone governs and this document is the defect.

The `R8.*` identifiers introduced here are the durable requirement names for
the Phase 8 closure ledger. They have no closure meaning until that ledger
exists.

---

## 1. Why This Phase Needs Its Own Specification

Phases 1-7 added authority to a path that always ended in a decidable answer:
admitted or denied, committed or stale. Phase 8 is the first phase whose
subject is the case where the runtime **does not know**, and the first phase
that must act on truth the runtime already committed.

That changes the shape of the work in four ways:

1. **The input is a past fact, not a request.** Undo consumes a committed
   receipt. The receipt must therefore carry enough to derive an inverse, and
   must remain unforgeable. Today it carries neither property (§4.3).
2. **The authority question inverts.** Every prior phase asked "may this
   principal do this now?" Phase 8 must additionally ask "does this committed
   past still permit this correction now?" — a question about drift, not
   permission.
3. **It reaches outside the process.** External-effect causality is the first
   Query boundary where the authoritative answer is owned by a system Query
   cannot interrogate synchronously. No such boundary exists anywhere in this
   repository today (§5.3).
4. **It is the phase most likely to be faked.** A generic `rollback()`, a
   provider repair call, a status string plus a retry button, or a replayed
   execution each look like a finished feature and each destroys the
   milestone's central claim. Three near-miss surfaces already exist in tree
   (§4.2, §4.3).
5. **It fixes a public vocabulary for every domain the platform will serve.**
   The aftermath classification is the one Phase 8 artifact that finance,
   medical, CAD, and simulation applications must all express themselves
   through. A taxonomy that fits banking and strains everywhere else cannot be
   corrected later without reclassifying every consumer. §6.1 shows the
   existing vocabulary is already straining, in tree, today.

The adversarial constraint for this phase, stated once:

> A copied receipt, an opaque wire identity, a matching digest, a lower-runtime
> notification, a Foundational transition artifact, a certification replay
> reconstruction, or a possessed status value must open no recovery, undo,
> redo, compensation, or completion door — and the honest paths must still be
> the easiest ones to call.

---

## 2. Central Claim

Phase 8 closes when the following is true through the public facade only:

An ordinary application can (a) classify every installed mutation's aftermath
at installation, (b) receive a typed, framework-owned handle for every outcome
the runtime could not resolve, (c) resolve that handle against authoritative
truth without guessing, (d) undo a committed operation through a freshly
admitted inverse or compensation that preserves the original history, (e) redo
that operation as a new authorized execution against current truth, and (f) be
told honestly and specifically when any of those is not available — while the
ordinary committing path pays none of the cost of any of them.

---

## 3. What Phase 8 Inherits

These are proved and must be consumed, not rebuilt. Rebuilding any of them is
a Phase 8 defect regardless of local convenience.

| Inherited authority | Owner in tree | Phase 8 consumption |
|---|---|---|
| Typed commit outcome family (`Committed`, `AlreadyCommitted`, `Stale`, `Cancelled`, `Denied`, `Aborted`, `PartialEffect`, `Indeterminate`) | `worth-query-execution/.../application_attempt/compare_and_commit.rs:138` | Extended in place with payloads; no parallel outcome enum |
| Graph-persisted idempotency record written inside the operation's own transaction | `.../primary_graph/provider/idempotency.rs:24` | The **sole** resolve-by-idempotency authority |
| Idempotency resolution taxonomy (`Unseen`, `AlreadyCommitted`, `IntentDrift`) | `.../application_attempt/idempotency_resolution.rs:14` | Reused verbatim by recovery resolution |
| Session-level `CommitRecoveryRequired` / `AbortRecoveryRequired` | `.../provider_session/protocol/terminal_outcome.rs:9` | Carried up, not re-derived |
| Managed-run lifecycle, interruption classification, readmission, disposal | `.../domain_computation/managed_run/` | The recovery handle is a managed resource in this registry |
| Capability, purpose, disclosure, conflict, elevation admission | Phase 7 `.../domain_computation/authorization/` | Re-entered fresh by undo and redo |
| Query-owned trusted time interval | `.../authorization/time_basis.rs:41` | Single time authority; see G6 |
| Typed branch affinity on every session, basis, and receipt | Milestone 9.16.1 | Carried by every aftermath artifact |
| Authoritative commit identity, ordered parents, branch head, ancestry, and canonical commit publication | `worth-relational` history and commit authority | The sole history and current-head authority. Query may bind operation-semantic causality to a commit; it may not maintain a parallel commit chain or head. |
| Cross-runtime causality transfer and lineage-aware continuity | `worth-runtime-bridge` | Transport and mapping only when causality crosses from truth to computation. Bridge admission never proves Relational currentness and never decides undo or redo legality. |
| Phase-separated canonical-work counters | `worth-query-installation/src/canonical_work.rs:105` | Extended with three lanes; see §8 |
| Foundational lineage, receipt, support-recovery, provenance vocabulary | `crates/worth-foundational/src/boundary_evidence/` | Lowering target only, never authority |

---

## 4. Current Boundary

### 4.1 Preserve

- Milestone 9.15 proposed-state and invariant authority.
- One installed operating-world root and one primary graph authority.
- Relational ownership of graph facts, transactions, commit identity, and
  history.
- Bridge ownership of installed correspondence; Signal ownership of policy-node
  evaluation.
- Cert-only replay imports. `worth-query-replay` may verify Phase 8 evidence
  and may not be named by any ordinary aftermath path.
- Phase 6 query identity, lane semantics, cursors, and publication contracts.
- Phase 7 capability, disclosure, conflict, and elevation semantics, including
  the rule that elevation cannot exceed its installed upper bound. Undo and
  redo are ordinary governed operations under that same bound.

### 4.2 Competing authority that must be reconciled, not extended

An aftermath authority **already exists** in the `worth-query` monolith:

```text
workspaces/worth-query/crates/worth-query/src/domain_installation/operation_aftermath/
    posture.rs          WorthQueryAftermathPosture, WorthQueryAftermathKind,
                        WorthQueryAftermathCounters, WorthQueryAftermathAdmissionDenial
    admission.rs
    admission/validation.rs
```

Its vocabulary is **not** the milestone's vocabulary, and its shape is the
evidence that motivates §6.1:

| Existing (`posture.rs:1`) | Destination | Required disposition |
|---|---|---|
| `ExactInverse { operation, lowering_family: String, postcondition }` | authority `RuntimeAlone` + mechanism `RecordedInverse` | Migrate; `lowering_family: String` becomes a typed bridge correspondence reference (§G8) |
| `Compensation { operation, postcondition }` | authority `RuntimeAlone` + mechanism `Compensation` | Migrate |
| `RebuildRequired { recovery_family: String }` | mechanism axis, unpopulated in 9.16 | **Resolved D3** — not a reconciliation; see §6.2 |
| `Irreversible` | authority `NotCorrectable` | Migrate |
| `ProvisionalDiscard` | *(none)* | **Resolved D4** — deleted from the family; see §6.3 |
| `DeclarationIncomplete` | *(none)* | Becomes an installation denial, never an installed posture |

The upstream declaration contract
(`worth-query-installation/src/domain_operation/semantic_contracts.rs:258`) is
worse than the posture enum: seven variants including both `ExactInverse` and
`ExactInverseWithPostcondition`, and both `Compensation` and
`CompensationWithPostcondition`. That is a flattened cross product of mechanism
against "does a postcondition exist." The collision has already happened once
and was resolved by enumerating the product. §6.1 resolves it structurally
instead.

**R8.0** — Phase 8 obeys the Milestone 9.16.1 reconciliation policy for this
surface: preserve the existing path as sole authority until the destination
proves feature, denial, lifecycle, receipt, and cost parity; then cut every
covered consumer atomically and retire exactly the predecessor authority in the
same slice. A second independently reachable aftermath classification,
admission, or denial lane is not lawful at any point, including transiently.

The Bank's descriptive declarations at
`workspaces/worth-query-bank-world/crates/bank-domain/src/estate/aftermath.rs:23`
(`EstateAftermath`, `aftermath_for()`) are the intended first consumer of the
destination contract. Its `NoMutation` variant has no milestone counterpart —
reads have no aftermath — and must disappear into "not a mutation," not into a
fifth classification.

### 4.3 Carriers that are currently insufficient

These are not missing features; they are load-bearing types that cannot support
what Phase 8 must prove. Each is a prerequisite inside Phase 8's causal
closure, not follow-on work.

**C1 — The committed receipt cannot derive an inverse.**
`WorthQueryApplicationCommitReceipt` (`compare_and_commit.rs:9`) carries
`outcome_identity`, `provider_runtime_instance_id`, `commit_id`,
`changed_record_count`, `emitted_effect_count`, `mutation_work`,
`precondition_comparison`, `canonical_work`, `terminal`. It names **no**
installed operation, **no** principal scope, **no** idempotency binding, **no**
touched record identity, and **no** input material. An inverse cannot be
derived from a count.

**C2 — Mutation work evidence is counters only.**
`WorthQueryPrimaryMutationWorkEvidence` (`provider/mutation_work.rs:2`) holds
six `usize`/`u64` counters. Nothing identifies *what* was mutated.

**C3 — `Indeterminate` and `PartialEffect` are payload-free.**
`WorthQueryApplicationCommitOutcome::{Indeterminate, PartialEffect}` are unit
variants (`compare_and_commit.rs:138`), even though the layer beneath them
carries real failure evidence
(`WorthQueryProviderCompareAndCommitOutcome::Indeterminate(WorthQueryProviderSessionFailure)`,
`commit_progression.rs:22`) and the session protocol distinguishes
`CommitRecoveryRequired` from `AbortRecoveryRequired`
(`terminal_outcome.rs:9`). Correlation evidence exists and is **discarded at
the application boundary**. Phase 8 must carry it, not rediscover it.

**C4 — Provider session identity is stringly.**
`WorthQuerySessionCommitOrAbortOutcome::Committed { plan_identity: String,
token_identity: String, provider_receipt: String, .. }`. These are diagnostic
grade. No recovery correlation may be keyed on them.

**R8.1** — C1-C4 are repaired as part of Phase 8, and each is **owned by a
named gate** rather than assumed to have happened:

| Carrier | Owning gate | Why there |
|---|---|---|
| C3 — `Indeterminate`/`PartialEffect` carry correlation | **8.2** | The correlation evidence is what that gate produces |
| C4 — typed provider correlation identity | **8.2** | Same; R8.5 is its requirement |
| C1 — receipt names operation, principal scope, idempotency binding | **8.3** | R8.28 binds the handle to exactly those fields; the handle cannot exist without them |
| C2 — mutation work names touched records | **8.4** | Deriving an inverse requires knowing what was mutated |

A carrier repair is complete only when the gate that owns it proves it. No
later gate may list a carrier as an entry condition unless an earlier gate's
`Establishes` produced it.

Receipt strengthening must preserve unforgeability: the strengthened receipt is
constructible only by the commit progression that produced it, exposes no
public constructor, and cannot be rebuilt from wire fields, `Debug` output, or
copied identities. The `compile_fail` hostility already guarding
`WorthQueryApplicationCommitTerminalEvidence` (`commit_terminal.rs:14`) is the
pattern to extend, not an exception to it.

---

## 5. Lower-Runtime Gap Inventory

This is the section the phase most needs. Each row states the gap, the
evidence, the owner, the admissible resolutions, and what is forbidden. A gap
whose resolution is chosen must be resolved **before** the gate that depends on
it, not worked around inside it.

| ID | Gap | Owner | Blocks gate |
|---|---|---|---|
| G1 | No before-image anywhere in Relational history or patches | Relational / Query | 8.4 |
| G2 | Version retention for pre-images is replay-named and replay-scoped | Relational | 8.4 |
| G3 | No external-effect boundary exists in the repository | Query (contract) + Bank (real boundary) | 8.2 |
| G4 | No typed provider correlation identity | Query provider protocol | 8.2, 8.3 |
| G5 | No provider commit-inquiry call | Query / provider contract | 8.3 |
| G6 | Trusted time is authorization-scoped | Query | 8.2, 8.3 |
| G7 | CDC is a candidate dispatch substrate, unevaluated | Relational | 8.2 |
| G8 | Bridge lowering family for inverse operations is a `String` | Runtime Bridge | 8.4 |
| G9 | Foundational vocabulary is sufficient — and partly forbidden | Foundational | all |
| G10 | Signal has no Phase 8 role and must not acquire one | Signal | all |
| G11 | Store durability is out of scope and must be typed as such | Store | 8.3 |
| G12 | No atomic Query aftermath-causality binding exists at the Relational commit boundary | Query semantics + Relational commit authority | 8.4, 8.5 |

### G1 — Relational exposes no before-image

**Evidence.** `AspectHistoryOrigin` / `AspectHistoryEntry`
(`crates/worth-relational/src/history/data/aspect_history.rs:19,40`) carry
`commit_id`, `version_id`, `branch_id`, `target`, `structural_change`,
`changed_aspects`, `contains_opaque_aspect` — *which* aspects changed, never
their prior values. `PublishedAuthoritativePatch`
(`crates/worth-relational/src/publication/patch/data/published_authoritative_patch.rs:10`)
exposes `scalar_set_for` / `struct_set_for` — post-values only. A grep for
`previous|prior|before_image|old_value` across the patch data module returns
nothing.

**Why it blocks 8.4.** A `Reversible` inverse that restores a prior field value
(the Bank's `UnfreezeAccount`, `RestoreRevokedCapability`) needs that prior
value. Deriving it from "the aspect changed at commit C" is not possible.

**Admissible resolutions.**

- **A (default, chosen).** Query retains the pre-image it already read. The
  decision read-set at admission necessarily contains the pre-state the
  invariants ran against. Bind the exact pre-image slice required by the
  installed inverse contract into the strengthened committed receipt (C1).
  This keeps authority in Query, adds no lower-runtime change, and makes the
  retained bytes bounded by the installed contract rather than by record size.
- **B.** Pinned historical read at the commit's parent version through the
  Phase 6.5 historical lane. Correct, but pays a reconstructive read on every
  undo and depends on G2.
- **C.** Relational adds before-images to patches or history. Largest blast
  radius; write amplification for every commit to serve a rare path. Rejected
  unless A and B both fail.

**R8.2** — Resolution A. The installed inverse contract declares exactly which
aspects it must restore; installation rejects an inverse contract whose
pre-image demand is unbounded or not covered by the operation's declared reads.
Retained pre-image bytes are counted and bounded per operation.

**Forbidden.** Reading the pre-image outside an admitted graph obligation.
Retaining "the whole record just in case." Deriving a pre-image from a live
read at undo time and calling it the original value.

### G2 — Version retention is replay-named and replay-scoped

**Evidence.** `HistoryAuthority::retain_version_for_replay` and the visibility
replay pins (`crates/worth-relational/src/history/logic/authority/replay_retention.rs:14`).
Versions not pinned are evictable through the visibility cache policy.

**Why it matters.** Under resolution A, G2 is not on the undo path — that is
the main reason A is chosen. It becomes live if resolution B is ever taken, and
it is live *now* for any Phase 8 inspection that reads at a historical basis.

**R8.3** — No ordinary Phase 8 path calls a `*_for_replay` retention API. If a
Phase 8 surface genuinely needs a pinned version, the pin is acquired through a
non-replay-named Relational retention contract, or the need is removed. A
`retain_version_for_replay` call in an ordinary lane is a residue-check
failure, both for authority and for the repository's cert-only replay rule.

### G3 — There is no external-effect boundary anywhere

**Evidence.** A repository-wide grep for
`external_effect|externaleffect|outbox|dispatch_attempt` across `crates/` and
`workspaces/` returns one unrelated UI test. The capability does not exist at
any layer.

**Why it blocks 8.2.** Gate 8.2 requires stable identity and causal links
across provider commit, emitted application event, dispatch attempt, external
acknowledgement, and external completion, plus classification of timeout,
disconnect, lost response, duplicate acknowledgement, and unknown outcome. The
milestone additionally requires a *real* controllable boundary: "An in-process
fake that shares the runtime's truth source cannot prove this gate."

**Ownership split.**

- **Query owns** the installed external-effect contract, the typed posture
  ladder, correlation identity, and the rule that no posture may be upgraded by
  possession of a lower one.
- **The Bank world owns** the real external service, its faults, and its
  transport. It is a Bank World deliverable that Runtime Phase 8.2 depends on,
  and it must be a separate process reachable over a real network boundary —
  the same standard Bank Phase 5 applies to user nodes.
- **No lower runtime owns** effect dispatch. Relational commits truth; it does
  not call the world.

**Decision D1 — dispatch atomicity: transactional outbox.** The dispatch intent
commits atomically with the mutation. The precedent already exists —
`provider/idempotency.rs:24` writes a Query-owned entity into the operation's
own `MutationIntent`, so a Query-owned dispatch-intent record is co-committed by
exactly the same mechanism.

This is structural rather than a cost preference. Under R8.55 every escaping
effect must be anchored by a committed local fact, and an operation whose only
domain effect is its dispatch record has no other anchor available. A post-commit
dispatch transition would leave those operations unanchored — no correlation
target, no idempotency record, no recovery anchor, and no authoritative local
answer to "did it happen?" — which is a second, weaker effect lane of exactly
the kind this milestone exists to prevent. A crash between commit and dispatch
would also lose the effect silently, making the "distinct typed postures"
requirement decorative.

**R8.4** — Operations that declare no external effect pay exactly zero
dispatch records, zero dispatch counters, and zero additional commit work.
Declaring an external effect is opt-in per operation and must be provable as
absent for ordinary money movement. What is *not* optional is the anchoring:
an operation that declares one gets the co-committed record, always (R8.55).

**Forbidden.** Treating local commit, transport success, HTTP 2xx, or receipt
possession as external completion. A retry loop that re-dispatches without a
correlation identity. An in-process fake as the gate's proof.

### G4 — No typed provider correlation identity

**Evidence.** C4 above: `plan_identity`, `token_identity`, `provider_receipt`
are `String`. `WorthQueryApplicationCommitOutcomeIdentity` is a `u64` stored
into the graph idempotency record as `AspectValue::UInt64`
(`provider/idempotency.rs:38`).

**R8.5** — Recovery correlation binds to typed Query identity — installed
operation identity, admitted attempt identity, idempotency binding, commit
outcome identity, branch — never to a provider-supplied string. Provider
strings may appear in diagnostics and may not appear in any equality that
decides whether a transition is admitted. Provider protocol identities are
either typed in this phase or explicitly quarantined to diagnostics with a
residue check.

### G5 — No provider commit-inquiry, and none is needed

**Evidence.** The commit provider surface is `admit_commit`
(`.../graph_provider/provider_contract.rs:30`) plus lifecycle
`prepare_provider_session` / `commit_prepared_session` /
`abort_provider_session` / `readmit_provider_plan`. There is no "was
transaction T committed?" call.

**Resolution.** None is required, and adding one would be wrong. The
authoritative answer to "did my commit land?" is the **graph-persisted
idempotency record**, which is written inside the operation's own transaction
and is therefore atomic with it. Resolution reads that record through an
admitted read; a provider memory lookup would be a second, weaker truth source.

**R8.6** — `resolve_by_idempotency` executes an admitted graph read of the
idempotency record and returns the existing
`WorthQueryApplicationIdempotencyResolution` taxonomy. A provider that cannot
prove the idempotency record co-commits atomically with the operation's effects
may not return `Committed` — this extends milestone decision 42 rather than
adding a new rule.

**Forbidden.** A provider-side recovery cache. Resolution that reads provider
memory. Resolution that infers commit from the absence of an error.

### G6 — Trusted time is authorization-scoped

**Evidence.** `WorthQueryAuthorizationClock` is
`pub(in crate::domain_computation)` and named for authorization
(`.../authorization/time_basis.rs:41`), fed by a host-published
`WorthQueryAuthorizationTimeSource`.

**Why it matters.** Recovery-handle expiry and dispatch timeout classification
need trusted time. Two clocks means two answers to "has this expired?"

**R8.7** — Phase 8 consumes the same single host-published time source. If that
requires generalizing the owner's name and visibility from
"authorization" to a Query runtime time authority, that rename is part of Phase
8 and must not fork the source. Callers and transport adapters still cannot
supply a sample or choose the evaluation moment — the Phase 7 rule is inherited
verbatim. Every expiry decision records its exact sample in the decision facts.

### G7 — Relational CDC as a dispatch substrate: evaluate before assuming

**Evidence.** `crates/worth-relational/src/publication/cdc/data/` contains
`subscriber_checkpoint.rs`, `subscriber_resume_request.rs`,
`subscriber_recovery_plan/`, `subscriber_failure.rs`,
`subscriber_stream_batch.rs`. This is a real at-least-once delivery substrate
with checkpoints and resume.

**Position.** CDC is a *Relational-truth* change stream for subscribers. An
application effect dispatch is not a Relational subscription: it carries
application meaning, application authority, and disclosure posture that CDC
does not model. Phase 8.2 may **derive** dispatch work from committed truth,
but the dispatch record, correlation identity, and posture ladder remain
Query-owned.

**R8.8** — If Phase 8.2 uses CDC as the delivery mechanism for the outbox, it
must state so explicitly, prove that no application authority or disclosure
decision is made by a CDC subscriber, and prove that a CDC checkpoint cannot be
readmitted as a Query dispatch posture. If it does not use CDC, it must not
reimplement a second change stream over Relational either.

### G8 — Bridge lowering family for inverse operations is a `String`

**Evidence.** `WorthQueryAftermathPosture::ExactInverse { lowering_family:
String, .. }` (`operation_aftermath/posture.rs:5`).

**R8.9** — The destination inverse contract references the installed Bridge
correspondence by typed reference, resolved and validated at installation.
Installation rejects an inverse whose lowering family does not resolve, does
not belong to the same installation generation, or does not match the original
operation's graph participation. A string that names a lowering family is a
diagnostic, not a binding.

Whether this needs a new Runtime Bridge surface or only a typed Query-side
reference to an existing one is a Phase 8.1 investigation, not an assumption.

### G9 — Foundational is sufficient, and partly forbidden

**Sufficient.** `FoundationalBoundaryEvidenceLineageOutcomeKind`
(`boundary_evidence/lineage/definitions.rs:4`) already distinguishes
`SingularContinuity`, `RestoredContinuity`, `ReconstructedEquivalence`,
`NamedGapPartialContinuity`, `Ambiguity`, `IdentityBreak`, `Denial`.
`FoundationalBoundaryEvidenceReceiptKind` (`receipts/definitions.rs:4`)
distinguishes `Execution`, `Restoration`, `CheckpointResume`, `Closeout`.
`FoundationalBoundaryEvidenceSupportRecoveryPosture`
(`support/definitions.rs:64`) covers degraded recovery publication. No new
Foundational vocabulary is required for lowering.

**Forbidden.** Per milestone F8 and the Phase 8 topology note: Foundational
branch, merge, scoped-merge, and cherry-pick artifacts
(`crates/worth-foundational/src/transitions/{branches,merges}/`) are not an
implementation path for linear aftermath. `BranchLocalReplacement`,
`MergeSuccessor`, and `PluralSuccessorPredecessor` lineage kinds are not the
ordinary Phase 8 completed-causality posture and may not be emitted by it.

**R8.10** — Each new identity-bearing Phase 8 family prepares its own ready
canonical basis under one named `CanonicalBasisDomain::Future(...)` constant
and one explicit `CanonicalizationRuleVersion` (milestone F4, F9). The families
are: installed aftermath contract, external-effect causality, recovery handle,
undo intent, redo intent. Co-committed aftermath causal facts carry the
already-derived identities of their endpoints and prepare no basis of their
own.

### G10 — Signal has no Phase 8 role

Signal evaluates installed policy conditions and mints decision evidence.
Nothing in recovery, undo, or redo changes that.

**R8.11** — No Signal decision, slot value, or explanation may classify an
external effect, admit a recovery transition, or authorize an inverse. Undo and
redo re-enter the ordinary Phase 7 admission path, which consults Signal
exactly as any other operation does — no more, no differently.

### G11 — Store is out of scope and must be typed as such

Milestone 9.16 explicitly excludes "durable recovery handles, restart-stable
cursors, or restart-stable undo/redo history before the Store handoff."

**R8.12** — Recovery-handle durability publishes as an explicit
`Store capability required` support posture through the milestone decision-58
vocabulary. Process-local handle lifetime is a stated, typed, inspectable
limit, never silence and never an implied guarantee. Phase 8 adds no
`worth-store` dependency to any Query crate.

### G12 — Aftermath causality has meaning in Query and history authority in Relational

`worth-relational` already owns authoritative commit identity, ordered parents,
branch heads, ancestry, and canonical commit publication. Its entity-lineage
authority separately owns create, replace, split, merge, retire, and admitted
correspondence of truth identities. Neither responsibility may be reconstructed
as an instance-local Query `Vec`, mutex-protected head, raw commit ordinal, or
post-commit append callback.

Query nevertheless owns meaning that Relational must not invent: that one
freshly admitted operation is the semantic inverse or compensation of an exact
original operation, or that one freshly admitted operation reapplies the exact
meaning established by a proved undo. This is **aftermath causality**, not a
second history or lineage system.

The locked boundary is:

| Responsibility | Owner | Authority status |
|---|---|---|
| Original/undo/redo operation meaning and legal transition | Query installation and execution | Query-owned semantic admission |
| Commit identity, ordered parents, branch head, ancestry, serialization, and publication | Relational | Sole canonical history authority |
| `undo-of` / `redo-of` fact prepared by Query and committed with the mutation | Query meaning, published through the Relational transaction | Canonical only after the ordinary Relational commit succeeds |
| Cross-runtime transport, continuity, remapping, and explanation | Runtime Bridge | Admitted projection; never Query legality or Relational currentness authority |
| Portable completed-continuity description | Foundational | Descriptive lowering only |

**R8.12a** — An original outcome is identified only by its authoritative
Relational-backed Query commit receipt. Every undo or redo prepares exactly one
private typed Query causal fact naming its exact semantic target, and that fact
is co-committed with the ordinary mutation. The resulting committed evidence is
sealed from the Relational commit result; no caller supplies its role, child
commit identity, parent list, branch head, or publication order.

**R8.12b** — Query owns no mutable aftermath chain or authoritative head. Redo
intent binds an owner-observed projection of the exact Relational branch head at
derivation. Redo compare-and-commit consumes that expected head and fails closed
if Relational current truth advanced; a preflight comparison without an atomic
commit precondition is insufficient.

**R8.12c** — Runtime Bridge has no ordinary Phase 8 aftermath-authority role.
It continues to own installed inverse correspondence under G8/R8.9. A later
cross-runtime consumer may ask Bridge to carry a completed, owner-admitted
aftermath-causality projection, but Bridge cannot create an undo/redo relation,
choose a commit parent or head, admit the operation, or publish history. Phase 8
adds no Bridge-owned aftermath chain, head, or legality type.

---

## 6. Product Decision Lock

### 6.1 The aftermath contract has two axes

Milestone decision 61 and architectural law 14 both name four postures:
`Reversible`, `Compensatable`, `Reconcilable`, `Irreversible`. Those four names
are three values of one axis wearing one hat:

- **Correction authority** — who can produce the corrected state? The runtime
  alone; the runtime plus a distinct actor or external truth owner; or nobody.
- **Correction mechanism** — how is the corrected state produced? From recorded
  inverse data; by a forward operation that neutralizes the effect; or by
  deterministic re-derivation from retained authoritative inputs.

`Reversible` and `Compensatable` share an authority value and differ only on
mechanism. `Reconcilable` differs only on authority. `Irreversible` is the
terminal authority value. The axes are independent, and flattening them has
already produced one cross-product enum in tree (§4.2).

Finance never needs a combination the flattening cannot express. The other
target domains do, immediately:

| Domain case | Authority | Mechanism | Expressible in four names? |
|---|---|---|---|
| Bank transfer reversal | runtime alone | compensation | yes — `Compensatable` |
| Account freeze | runtime alone | recorded inverse | yes — `Reversible` |
| Amend a signed clinical note | external attestation | compensation (addendum; original must remain visible) | **no** — files as `Reconcilable`, mechanism lost |
| Correct a released lab result | external (issuing lab) | recorded correction | **no** — same collapse |
| Regenerate CAD feature tree after a parameter correction | runtime alone | re-derivation | **no** — no honest slot |
| Re-elaborate a signed-off netlist | runtime alone | re-derivation | **no** — no honest slot |
| Released drawing, tapeout submission | not correctable | none | yes — `Irreversible` |

Two consequences follow, and the second is the expensive one.

The medical rows lose a **legal** requirement. "Original remains visible,
correction appended" is not a convention in a regulated record; it is the
requirement. Collapsed into `Reconcilable`, the framework cannot enforce it,
and MENTALITY §3 is explicit that an unenforced rule is hope.

The CAD and EDA rows lose a **cost** distinction. `compensate()` and
`reconcile()` both read as cheap next actions. A re-derivation is bounded by
the derivation, not by the semantic delta — potentially minutes of compute on a
chip design. `perf_laws` names this failure twice, as path conflation and
amplification blindness, and dx law 1 requires a weakened or expensive path to
state what it costs. Architectural law 12 forbids unifying constructs that do
not share authority, lifecycle, failure topology, **and** cost class;
re-derivation and reconciliation share none of the four:

| | `Reconcilable` | Re-derivation |
|---|---|---|
| Produces the correction | external owner or distinct actor | the runtime, alone |
| Determinism | none; depends on an outside answer | deterministic, and must prove it under arch law 17 |
| Cost class | unbounded wall-clock, bounded compute | bounded by the derivation, not the delta |
| Denial modes | no authority available; external disagreement | inputs not retained; budget exceeded; nondeterminism detected |

**R8.52** — The installed aftermath contract carries correction authority and
correction mechanism as two separate typed contracts. The four law-14 names
remain the published posture and are **derived** from the pair, never
hand-declared. The cross-product variants in
`semantic_contracts.rs:258` are retired in the same slice under R8.0; the
postcondition becomes a field of the mechanism contract rather than a variant
axis.

**R8.53** — The mechanism axis is populated with exactly `RecordedInverse` and
`Compensation` in Milestone 9.16. Re-derivation is **not** populated, because
no consumer exists yet. Domain structure law 2 requires the smallest populated
form of the committed destination topology, and equally forbids empty
placeholders for uncommitted possibilities. The axis is the commitment; a
future domain adds one leaf beneath a durable axis rather than migrating every
consumer.

**Governing amendments.**

*Taken.* Milestone decision 61 previously read "Every installed mutation
**declares** `Reversible`, `Compensatable`, `Reconcilable`, or `Irreversible`
aftermath posture." It now states the two declared axes and the derived
posture, so R8.52 refines its milestone rather than contradicting it. The
outcome decision 61 requires is unchanged: every installed mutation still
carries exactly one of the four, as installed meaning, with no default in
either direction.

*Pending, with the first re-derivation consumer.* Architectural law 14 names
four postures and admits no slot for deterministic re-derivation. Populating
that mechanism requires law 14 to gain the posture. That is a constitutional
change to `docs/coding-guidelines/arch_laws.md` and must be made deliberately,
not discovered as a local exception in whichever milestone first needs CAD,
EDA, or simulation aftermath. Phase 8 does not amend it, because Phase 8 does
not populate the mechanism (R8.53).

### 6.2 D3 — `RebuildRequired` is a mechanism, not a reconciliation

**Resolved: neither fold nor keep.**

`RebuildRequired` does not currently work. Both it and `ProvisionalDiscard`
terminate in aftermath admission denials
(`operation_aftermath/admission/validation.rs:127-131`); nothing implements
either. `RebuildRequired { recovery_family: String }` is a placeholder meaning
"no undo here, something must be rebuilt," with an untyped string naming the
something.

It is not a reconciliation (§6.1 table). It is the re-derivation mechanism,
arriving before its axis existed. Under R8.52 and R8.53 it is removed from the
posture family in 9.16 and returns as a mechanism value when a consumer needs
it. Any operation that genuinely requires it before then is `Irreversible` with
a typed cause, which is honest, or it is not installable, which is also honest.

The name does not survive either. `RebuildRequired` names a demand rather than
a semantic, and `Rederivable` collides with the platform's
authoritative-versus-derived vocabulary (MENTALITY §9), where "derived" means
non-authoritative — a re-derived artifact here **is** authority. The naming
decision belongs to the domain vocabulary owner at the point the mechanism is
populated.

### 6.3 D4 — `ProvisionalDiscard` is deleted from the family

**Resolved: delete, and make it unrepresentable.**

`operation_aftermath/discard.rs:66` denies with `ExecutedEffectsPresent`, which
confirms the reading: it is pre-commit discard of provisional transaction work.
Aftermath classifies committed truth. Nothing escaped, so there is nothing to
correct.

Keeping it in the posture family is the mislabeling architectural law 14's
final clause forbids. A caller matching on posture sees `ProvisionalDiscard`
and `Compensatable` as siblings and reasonably reads them as two flavours of
undo. One of them moves committed money. AI_README states the same boundary
from the other side: Relational savepoints and rollback "do not create
application authority or alter committed history."

**R8.54** — Provisional discard is a transaction-lifecycle transition and is
not reachable from any aftermath type. The Bank's `NoMutation` variant
(`bank-domain/src/estate/aftermath.rs:23`) is likewise deleted: an operation
that changes no truth carries no aftermath contract, expressed by the absence
of the contract rather than by a variant meaning "not applicable."

### 6.4 D8 — Every escaping effect is anchored by a committed local fact

**Resolved: the question was malformed, and its answer is a law.**

The original question — does the bank have an operation whose only effect is
external? — is the wrong scope. Across the platform's target domains such
operations are ordinary: send a lab order, page a clinician, submit a
regression to a compute farm, write to a PDM vault, drive a mill, transmit a
payment instruction.

An unanchored external dispatch loses four things at once. There is no local
record to correlate an acknowledgement against. There is no idempotency record,
so a retry fires the effect twice — a duplicate drug order, a second cut on a
billet. There is no anchor for a recovery handle. And "did it happen?" has no
authoritative local answer, which is the exact question Phase 8 exists to make
answerable.

**R8.55** — No operation may emit an escaping effect without a committed local
fact anchoring it. An application may declare an operation with no domain
mutation; the runtime still commits its dispatch intent, and that outbox record
is the anchor. There is no mutation-free external effect — only an operation
whose sole domain effect is its dispatch record. This is milestone decision 68,
added because the anchoring law is platform-grade rather than phase-local: it
is what makes decision 67's three distinct facts recoverable rather than merely
distinguishable.

**R8.56** — An operation declaring an external effect may not declare
correction authority `RuntimeAlone` with mechanism `RecordedInverse` — that is,
it can never publish as `Reversible`. Architectural law 14 confines reversal to
inverse data derived "without external reread," which an escaped effect
excludes by construction. Installation rejects the combination. This removes
the most tempting fake in the phase: an `undo()` reachable on an operation that
has already told the outside world something.

Milestone decision 63 already required this of the bank ("externally escaped
effects expose compensation, reconciliation, or irreversible posture rather
than a fake inverse"). Decision 68 generalizes it to every domain and R8.56
makes it mechanical, moving the rule from category 5 of MENTALITY §3's
enforcement hierarchy to category 2.

### 6.5 Remaining locked decisions

- **D1** — External dispatch uses a Query-owned transactional outbox
  co-committed with the mutation (§G3). **Structural, not optional:** under
  R8.55 an opt-in outbox would leave mutation-free external effects unanchored,
  which is the parallel authority lane the milestone exists to prevent.
  Operations declaring no external effect still pay exactly zero (R8.4).
- **D2** — Pre-images come from the retained decision read-set bound into the
  strengthened committed receipt (§G1, resolution A).
- **D5** — Recovery-handle expiry, dispatch timeouts, and Phase 7 grant
  validity all read one host-published time source (§G6).
- **D6** — Resolution authority for "did it commit" is the graph idempotency
  record, not the provider (§G5).
- **D7** — Redo invalidation on divergence is *invalidation*, not deletion, and
  it is a **Query policy evaluated against Relational's authoritative branch
  head**, not a property of the redo intent type and not a Query-owned history
  head. The intent becomes unusable; committed causal facts and both original
  journals remain. See R8.45 for why the distinction is load-bearing.

---

## 7. Destination Topology

The milestone's Phase 8 skeleton is correct but **incomplete in one place**:
domains must *declare* aftermath before installation can validate it, and the
declaration audience facade must re-export it. Phase 7's skeleton has its
declaration package; Phase 8's does not.

```text
worth-query-declaration/src/application_aftermath/       [ADDED HERE]
    contract.rs                 declared aftermath contract entry point
    correction_authority.rs     declared authority axis
    correction_mechanism/       declared mechanism axis
        recorded_inverse.rs     inverse operation and pre-image demand
        compensation.rs         compensating operation
    reconciliation.rs           declared reconciliation procedure
    external_effect.rs          declared external-effect posture and correlation demand

worth-query-installation/src/application_aftermath/
    canonical_basis.rs
    correction_authority.rs     installed authority axis
    correction_mechanism/       installed mechanism axis
        recorded_inverse.rs
        compensation.rs
    published_posture.rs        law-14 name derived from the axis pair
    next_action_contract.rs
    recovery_contract.rs
    external_effect_contract.rs

worth-query-execution/src/domain_computation/application_aftermath/
    recovery_handle.rs
    recovery_progression.rs
    undo_admission.rs
    undo_progression.rs
    redo_intent.rs
    redo_admission.rs
    causality/
        undo.rs                 Query-owned semantic `undo-of` admission
        redo.rs                 Query-owned semantic `redo-of` admission
        committed.rs            sealed receipt projection after Relational commit
        current_head.rs         linear-lane policy over Relational head evidence
    external_effect.rs

worth-query-execution/src/domain_computation/primary_graph/provider/
    application_causality/
        prepare.rs              lower admitted Query causality into the mutation
        commit_fact.rs          co-committed internal graph fact
        lookup.rs               owner read for duplicate/copy/divergence admission

worth-query-publication/src/application_aftermath/
    outcome.rs
    explanation.rs
    access_and_disclosure.rs
    boundary_evidence.rs
```

Facade obligations:

- `worth-query-decl` re-exports the declaration surface with no added identity
  or behavior layer.
- `worth-query-host` exposes recovery, undo, and redo as ordinary front-door
  capabilities; it exposes no runtime authority object and no raw handle
  internals.
- No consumer imports `worth-query-installation`, `-admission`, `-execution`, or
  `-publication` to use aftermath.

`correction_mechanism/` is a one-child-per-populated-mechanism directory today
and is deliberately not flattened. It is the stable parent axis whose next
sibling — deterministic re-derivation, arriving with the first CAD, EDA, or
simulation consumer — must be able to enter without reclassifying its siblings,
moving the facade, or changing authority direction (domain structure laws 2 and
3, and R8.53). No empty placeholder file for that sibling may exist before its
consumer does.

`published_posture.rs` derives the four architectural-law-14 names from the
installed axis pair (R8.52). It is the only place those names are produced, and
no declaration may state one directly.

Placement rules that carry real weight here:

- **Installation** owns operation-specific meaning and legal next actions.
- **Execution** owns attempt-bound progression and consumes current authority.
- **Relational** remains the sole owner of commit identity, ordered parents,
  branch head, ancestry, serialization, and publication. Query's committed
  causality is a co-committed semantic fact, never an independently mutable
  history surface.
- **Runtime Bridge** owns installed inverse correspondence and, for later
  cross-runtime consumers, transport of admitted causal projections. It owns no
  ordinary Phase 8 history, head, undo/redo legality, or commit currentness.
- **Publication** describes posture and available next actions and can
  manufacture none of them from wire identities.
- Branch-shaped aftermath is absent by design. No dormant directory, no
  placeholder module, no `branch` parameter that is always the ordinary branch.
  Ordinary aftermath still carries its 9.16.1 branch affinity on every artifact.
  Absence is not foreclosure: the Query causal relation names semantic targets
  without encoding a private linear history representation. Branch-shaped
  history remains additive because Relational owns history shape and a future
  lane may apply a different Query policy over that authority (R8.45).

---

## 8. Canonical-Work And Cost Contract

`WorthQueryCanonicalWorkPhases` (`worth-query-installation/src/canonical_work.rs:105`)
currently has: `installation`, `admission`, `execution`, `provider_commit`,
`projection`, `live_delivery`, `retry_resolution`, `recovery_inspection`,
`publication`. Gate 8.6 requires reporting external dispatch, undo, and redo
separately.

**R8.13** — Three phase slots are added: `external_dispatch`, `undo_admission`,
`redo_admission`. Adding them must break every construction site until each is
supplied (arch law 9); a defaulted-to-zero slot is not lifecycle completeness.

Required counter posture, asserted exactly, not by threshold:

| Lane | basis preparations | digest derivations | digest text materializations |
|---|---:|---:|---:|
| Installation of one aftermath contract | bounded, declared | 1 per new family | 0 |
| Fresh undo admission | 1 | 1 | 0 |
| Fresh redo admission | 1 | 1 | 0 |
| New dispatch/causality event | 1 | 1 | 0 |
| Ordinary commit with no external effect | 0 | 0 | 0 |
| Recovery handle lookup and inspection | 0 | 0 | 0 |
| Retry resolution | 0 | 0 | 0 |
| Delivery, acknowledgement, timeout classification | 0 | 0 | 0 |
| Aftermath causal-fact preparation, Relational current-head check | 0 | 0 | 0 |
| Publication | 0 | 0 | at explicit boundary only |

**R8.14** — Fan-out independence. Growing posting count, decision-fact count,
touched-record count, committed history breadth, and consumer count must not
change any count in the table. The undo and redo derivations are one each,
independent of what they undo.

**R8.15** — Lane separation. Reconstructive inspection and compensation are
measured in their own lanes and may not be amortized into ordinary commit.
Report materialization is never on the commit path. A planning or
policy-admission receipt cannot satisfy executed-cost evidence (milestone F7).

---

## 9. Gate Specifications

Gates are ordered and each consumes the prior gate's proved product. A
discovery that strengthens a closed gate becomes an append-only corrective
phase and blocks unfinished dependents; it does not become a local exception.

### Gate 8.1 — Installed Aftermath Classification And Legal Next Actions

**Entry.** Phase 7 closed; §4.2 reconciliation plan written.

**Establishes.**

- **R8.16** Every installed mutation carries exactly one correction-authority
  value and, where authority is not `NotCorrectable`, exactly one
  correction-mechanism contract (R8.52). Its published posture — `Reversible`,
  `Compensatable`, `Reconcilable`, `Irreversible` — is derived from that pair,
  with operation-specific typed next actions. Missing, contradictory,
  host-authored, or changed aftermath meaning is rejected at installation.
  There is no default and no fallback in either direction, on either axis.
- **R8.17** Semantic inverse, compensating operation, reconciliation procedure,
  and terminal denial are four distinct installed contracts, not one callback
  with a mode field. The postcondition is a field of the mechanism contract,
  never a variant axis (§4.2).
- **R8.18** The inverse contract declares its pre-image demand, and installation
  rejects a demand not covered by the operation's declared reads or exceeding
  its declared bound (§G1).
- **R8.19** Classification binds to exact operation, schema, package,
  compatibility generation, commit posture, and result contract.
- **R8.20** One Foundational canonical basis and structured comparison for the
  portable contract; compact digest admitted through Foundational's typed slot
  and retained inside Query's installed aftermath identity; derived once per
  installed or rebuilt meaning (§G9, milestone F10/F12).
- **R8.21** The public outcome type exposes only next actions installed for its
  exact posture. An irreversible operation has no `undo` method to call — this
  is a type-level absence, not a runtime denial.
- **R8.57** An operation declaring an external effect cannot install as
  `Reversible` (R8.56). The rejection happens at installation, names the
  escaping effect that caused it, and is proved by a negative case with a
  positive twin.
- **R8.58** `ProvisionalDiscard` and `NoMutation` are unreachable from every
  aftermath type (R8.54), proved by exhaustive match and by residue search
  rather than by their absence from a happy path.

**Exit proof.** Complete operation inventory with no unclassified mutation, and
no mutation whose authority and mechanism axes were inferred rather than
declared. Independent drift attacks on each axis separately, through
Foundational structured comparison — an authority change and a mechanism change
must produce different installed identities. Residue denial for direct hash and
debug-string identity grammars. Bank estate declarations installed through the
generic contract with `bank-domain/src/estate/aftermath.rs` retired, not
wrapped. Parity evidence for every consumer of the monolith
`operation_aftermath` surface and of the seven-variant
`WorthQueryOperationReversalContract`, then their exact retirement in the same
slice (R8.0, R8.52).

### Gate 8.2 — External-Effect Causality And Indeterminate Posture

**Entry.** 8.1 closed, including R8.57's installation-time rejection.

**This gate builds its own entry condition.** The external boundary does not
exist in the repository (§5 G3) and no Bank World phase schedules it. Rather
than depending on unscheduled work, Gate 8.2 owns constructing it as the first
item of its own slice: a real controllable external service in the Bank world,
in its own process, with the fault repertoire its exit proof requires. Query
owns the contract and posture ladder; the Bank owns the service; no lower
runtime owns dispatch. A gate may not list as an entry condition anything that
no phase is scheduled to produce — see §10.

**Establishes.**

- **R8.22** Provider commit, emitted application causality, dispatch attempt,
  external acknowledgement, external completion, compensation, and
  reconciliation are seven distinct typed postures. No posture is derivable
  from possession of an earlier one.
- **R8.23** Each posture carries a stable exact identity and causal link to its
  predecessor. One new dispatch or causality event derives one identity;
  delivery, acknowledgement, timeout classification, inspection, retry
  resolution, and completion carry it (§8 table).
- **R8.24** Timeout, disconnect, lost response, duplicated acknowledgement, and
  unknown provider outcome are classified without guessing whether the effect
  occurred. "Unknown" is a first-class answer with its own recovery posture.
- **R8.25** The dispatch intent is co-committed with the mutation (D1). This is
  structural, not a cost optimization: under R8.55 every escaping effect is
  anchored by a committed local fact, so an operation with no domain mutation
  still commits its dispatch record and that record is its anchor. Operations
  declaring no external effect pay zero (R8.4).
- **R8.26** `Indeterminate` and `PartialEffect` carry the correlation evidence
  the layer beneath them already produced (C3). The distinction between
  `CommitRecoveryRequired` and `AbortRecoveryRequired` survives to the
  application boundary.
- **R8.27** Foundational completion, provenance, and freshness vocabulary may
  describe a posture only after the Query boundary is known, and cannot upgrade
  an indeterminate effect into a completed one.

**Exit proof.** The real external boundary must be able to: commit then lose
the response; acknowledge without completing; complete after timeout; duplicate
a message; and disappear mid-dispatch. An in-process fake sharing the runtime's
truth source does not close this gate. Each fault produces a distinct typed
posture, and no fault produces `Completed`.

### Gate 8.3 — Recovery Handle And Resolution Lifecycle

**Entry.** 8.2 closed.

**Establishes.**

- **R8.28** A framework-owned handle is minted only for an outcome whose
  installed posture permits recovery work, bound to runtime, schema, typed
  branch, operation, attempt, principal scope, idempotency identity, provider
  posture, correlation evidence, the exact installed aftermath identity and
  operation slot, compatibility generation, and expiry. The operation
  admission carries its compiled aftermath into the committed receipt, and the
  receipt carries that same contract into the handle. Mint accepts no separate
  aftermath argument and performs no lookup, recompilation, or reconstruction.
- **R8.29** The handle is a managed resource in the existing managed-run
  registry (§3), enumerable and terminable by the framework. It is consumed,
  expired, or disposed linearly. The handle's one-terminal law is carried by
  `worth-proof::LinearResource`; the registry owns enumeration, force
  termination, terminal audit, and `Drop` cleanup, not a second `live: bool`
  authority. A second consuming transition on the same resource is
  unrepresentable. A commit receipt remains cloneable historical evidence; it
  is not itself a linear mint permit. The registry therefore atomically claims
  the receipt's authoritative `(provider runtime, typed branch, commit)`
  identity before registering a handle and permanently rejects a second claim,
  including a claim made through a cloned or recovered receipt. That dynamic
  owner check is the only receipt-level dedup authority. It does not weaken the
  compile-time guarantee that the successfully minted handle is move-only.
- **R8.30** Transitions are `inspect`, `resolve`, `safe_retry`, `compensate`,
  `reconcile`, `dispose`, exposed only when admitted by the current outcome and
  installed contract. Receipt facts are read from the handle; installed
  aftermath is read from the handle's exact operation-derived contract. A
  transition signature may not accept either as a caller assertion.
- **R8.31** Every transition that can produce effect authority re-establishes
  current provider truth and current application authority first. A handle
  minted an hour ago authorizes nothing by itself. Current admission is the
  sole caller-presented freshness input. The resulting authority is affine to
  both its runtime owner and the exact handle whose immutable binding matched
  that admission; authority admitted for another handle in the same runtime is
  insufficient. The authority carries an owner-sealed
  `FreshnessScopedBasis<CurrentValidity, _>` artifact. Clock classification is
  necessarily runtime work, but it returns distinct current and expired
  evidence types; a current observation cannot be passed to the expiration
  transition.
- **R8.32** `resolve` reads the graph idempotency record through an admitted
  read and returns the inherited resolution taxonomy (§G5).
- **R8.33** Handle lookup, provider inquiry, and repeated inspection perform
  zero basis preparation, digest derivation, and digest-text comparison (§8).
- **R8.34** The wire boundary carries an opaque recovery identity and
  descriptive posture. A support artifact, opaque identity, or published
  posture cannot be readmitted as a handle. The opaque projection is explicitly
  weakened through `BoundaryBridged<AuthorityRevalidationRequiredBasis<_>>`;
  serialized bytes never retain the current-validity basis of live Query
  authority.
- **R8.35** Unresolved or degraded recovery publishes through Foundational
  support-truth and basis-disclosure vocabulary while the Query handle remains
  the sole next-action authority. Durability posture is published per R8.12.
- **R8.62** This gate repairs **C1** (R8.1): the committed receipt names the
  installed operation, the admitted principal scope, and the idempotency
  binding, and privately retains the compiled aftermath of that exact admitted
  operation. R8.28 binds the handle to exactly those facts, so the handle
  cannot honestly exist until the receipt carries them. Strengthening preserves
  unforgeability under R8.1's construction rule.

**Exit proof.** Lost-response recovery; already-completed recovery; unresolved
external posture; expiry; disposal; copied handle; foreign principal; foreign
runtime; foreign branch with equal version ordinal; duplicate transition;
transition after disposal. Leak detection proves no handle survives its
terminal path.

### Gate 8.4 — Fresh Undo, Inverse Operations, And Compensation (provisional history)

**Product status:** this gate records the current experiment and its regression
evidence. Its undo/compensation product semantics are not accepted Phase 8
closure and must be re-decided by Milestone 9.18.

**Entry.** 8.3 closed, which supplies C1. C3 and C4 supplied by 8.2.

**This gate builds its own entry condition.** G1 resolution (R8.2 consumption)
and G8 typing (R8.9) are not prior deliverables — installation already declares
pre-image demand (R8.18) and a typed lowering-correspondence *slot*, but the
consumption side of Resolution A and Bridge correspondence resolution are this
gate's work. Rather than listing unscheduled work as an entry condition, Gate
8.4 owns both as the first obligations of its own slice, together with **C2**
(R8.1): mutation work must name the touched records before an inverse can be
derived from them. A gate may not list as an entry condition anything that no
phase is scheduled to produce — see §10.

**Establishes.**

- **R8.36** Undo derives an inverse, compensation, or reconciliation request
  from the exact strengthened commit evidence and installed aftermath already
  carried by its recovery handle. Neither object is caller-presented at undo
  admission. It never mutates history and never calls the provider to repair
  state. Admission takes the handle by value. The handle then remains private
  inside the framework-owned undo admission and ordinary-progression handoff;
  application code cannot retain or reuse a borrow after admission. The
  strengthened evidence includes the exact governed operation input retained
  from the original admission, its complete canonical governed-input identity,
  and the commit-derived touched-record identities. Missing retained input or
  missing canonical identity denies correction; a type-erased carriage check
  may recover the application type but cannot substitute for semantic binding.
  Fresh undo authority is re-established from that retained input, not from a
  second caller-authored action. Recorded-inverse and compensation progression
  accept no caller-authored correction target: the inverse target is the exact
  retained-pre-image record among the commit's touched records, and the
  compensation target is derived from the retained original input, its original
  idempotency binding, and commit-owned record evidence. A digest, local slot,
  or newly supplied domain identifier cannot stand in for this carriage.
- **R8.37** Undo re-enters the full current progression: capability, purpose,
  disclosure, conflict, touched-graph, invariant, idempotency, provider, and
  compare-and-commit. It is an ordinary operation with an unusual input. The
  framework derives that input before ordinary admission; a consumer may still
  supply the current principal, request scope, and a fresh idempotency key for
  the new correction commit, but never the original action, inverse target,
  journal-to-reverse, amount, destination, or other correction semantics.
- **R8.38** Money movement produces compensating debit and credit journal
  entries. Both original journals are preserved. Eligible capability changes
  use explicit inverse operations.
- **R8.39** Irreversible legal, audit, approval, released-estate, escaped-effect,
  stale, conflicted, and already-consumed cases deny with a typed cause and no
  fallback mutation.
- **R8.40** Undo derives one new bounded intent identity and carries the
  original committed and aftermath identities. It does not regenerate identity
  per posting, decision fact, or co-committed causal fact (§8).
- **R8.41** Foundational transition, provenance, or committed artifacts may
  describe the completed relationship afterward and cannot substitute for the
  fresh Query admission that produced it.

The committed-undo result separates descriptive evidence from continuation
authority. `WorthQueryProvedUndo` remains descriptive and grants no current
power. A private Query-owned redo-recovery continuation pairs that proof with
the still-linear recovery handle only after the ordinary undo commit succeeds.
If undo does not commit, or the continuation is abandoned, framework ownership
drops and disposes the handle. No caller can reconstruct this pairing.

**Exit proof.** Exactly one compensating transfer with both original journals
intact; an equivalent retry compensates once, not twice. Current-policy denial
after drift. Idempotent retry after lost response. Inverse capability
progression. Rejection of copied, foreign, irreversible, and twice-consumed
receipts. An independent double-entry oracle — not the production accounting
path — agrees on final balances.

### Gate 8.5 — Fresh Redo Intent And Relational-Head-Bound Causality (provisional history)

**Product status:** this gate records the current experiment and its regression
evidence. Redo eligibility, occurrence meaning, current-head policy, and public
DX are not accepted Phase 8 closure.

**Entry.** 8.4 closed.

**Establishes.**

- **R8.42** Redo intent is derived only from a proved undo and is descriptive:
  it binds original operation meaning, undo receipt, an owner-observed
  projection of the exact Relational branch head, principal scope, and
  compatibility generation, and embeds no runtime authority and no replay
  state. `WorthQueryProvedUndo` contains an owner-minted
  `Proof<UndoCompleted, WorthQueryUndoCompletionAuthority>` and exposes no raw-parts
  constructor; the separate co-committed causal fact required to bind that
  completion to the exact Relational child commit is established by R8.44.
- **R8.43** Redo requires fresh capability, policy, conflict, touched-graph,
  invariant, idempotency, provider, and compare-and-commit admission. Redo
  admission consumes the Query-owned redo-recovery continuation by value,
  checks fresh authority against its private handle, and keeps that handle
  private until the ordinary progression handoff terminalizes it. A proved
  undo and an unrelated handle cannot be recombined by a caller, and neither
  undo nor redo can be admitted twice through a retained borrow. Redo reuses
  the exact original governed input privately carried by the handle; no caller
  supplies a replacement action, amount, destination, or other operation
  meaning. Its ordinary idempotency key is the canonical redo-intent identity
  and its idempotency intent is the retained canonical governed-input identity,
  so no caller supplies either raw binding. An operation-specific, move-only
  application continuation owns the exact freshly admitted ordinary operation,
  including its governed input and current authority context, through ordinary
  progression. It accepts no replacement principal, request, action, or second
  admission step.
- **R8.44** The original is represented by its Relational-backed Query commit
  receipt. Every undo or redo co-commits exactly one private typed Query
  `undo-of` or `redo-of` fact with its ordinary mutation. The Relational commit
  result supplies the child commit identity, ordered parents, branch, and
  publication order. No Query-owned chain, head, raw `u64` node, role argument,
  or post-commit append surface exists.
- **R8.45** A divergent operation advancing the current head invalidates the
  redo intent (D7). Both admission and compare-and-commit consume Relational
  current-head evidence so an intervening commit cannot race between a Query
  precheck and publication. No Query branch object, merge placeholder,
  alternate lineage, or mutable head appears. Invalidation-on-divergence is a
  **policy of the linear Query lane over Relational history**, not a property of
  the redo intent type: a future rebasing lane may apply a different policy
  without replacing the causal relation or Relational authority.
- **R8.46** Completed Query aftermath causality lowers into Foundational
  continuity vocabulary only after the Relational commit succeeds and only
  into the lawful linear kinds (§G9). Runtime Bridge may transport that admitted
  projection only for a real cross-runtime consumer and cannot upgrade it into
  Query legality or Relational currentness. Replayed, reconstructed, restored,
  branch-local, partial, and promoted postures may not be relabeled as ordinary
  linear aftermath.

**Exit proof.** Lawful redo; stale redo; newly unauthorized redo; copied intent;
foreign principal; changed operation meaning between undo and redo; duplicate
redo; divergence invalidation. The decisive concurrency case starts two lawful
operations from the same observed Relational head, commits one intervening
operation, and proves the stale redo cannot commit even if its earlier Query
precheck succeeded. A fault injected after Relational commit but before any
Query projection hook still leaves the committed causal fact observable from
the owner state. Destroying any Query-derived causality cache and rebuilding it
from the Relational commit plus co-committed fact changes no decision. Residue
checks reject `WorthQueryLinearLineageChain`, mutable Query heads, public/raw
append APIs, and new ordinary Phase 8 Bridge causality authority. Certification
replay may verify the evidence and must not appear in the ordinary redo path.

### Gate 8.6 — Bank Aftermath Cutover, Publication, And Certification

**Accepted scope:** committed aftermath, external-effect, recovery, exact
retention, and closed publication. Undo/redo journeys in this gate are retained
only as provisional regression evidence.

**Entry.** 8.1-8.5 closed.

**Establishes.**

- **R8.47** Committed outcome, recovery, compensation, reconciliation, undo, and
  redo are reachable through typed public facades with operation-specific legal
  next actions.
- **R8.48** Publication preserves authorization, disclosure, and inherited
  branch affinity across outcome, explanation, recovery posture, and
  receipt-linked aftermath causality. A protected fact that influenced an
  inverse decision does not leak through aftermath explanation, causal-relation
  shape, or next-action availability — Phase 7 noninterference applies
  unchanged to this surface.
- **R8.49** The temporary HTTP boundary stays descriptive and asynchronous,
  deserializes no authority, and makes no route-local recovery decision.
- **R8.50** Superseded monolith, bank-local, and generic rollback paths are
  removed or privatized; destination dependency direction is proved.
- **R8.51** Ordinary commit cost is unchanged when no external or recovery work
  is required; reconstructive inspection and compensation are measured
  separately (§8).

**Exit proof.** The bank transfer and estate aftermath courtroom; the real
external-boundary fault matrix; public consumer compilation with no internal
imports; boundary checks; residue searches; lifecycle and leak probes; and
ordinary-versus-reconstructive measurement — all closed before Bank World
Phase 5 begins.

### Gate 8.7 — Safe-Retry Re-Dispatch (append-only corrective to 8.3 / 8.2)

**Entry.** 8.1-8.6 closed. This gate exists under §9's own rule: a discovery
that strengthens a closed gate becomes an append-only corrective phase, not a
local exception.

**Why this gate exists.** An end-to-end trace of the external-effect subsystem,
run after Phase 8's gates had closed, found that `dispatch_external_effect` has
exactly one call site — in-process, immediately after commit. Nothing ever
re-dispatches. `safe_retry_recovery_handle` returns an *admission* to retry,
performs no dispatch, and is the only one of R8.30's six transitions with no
production consumer; the other five are called from
`bank-server/src/estate_progression/recovery.rs`.

That is a transition admitting an action the runtime cannot perform. No `R8.*`
row required a re-dispatch, which is exactly why every gate passed without one
— the same shape of gap as **Q8.11**, and recorded the same way.

The failure is reachable today and involves no crash: a transport fault on a
live process yields an `Unresolved` posture and a mintable recovery handle over
a committed outbox record whose effect never escaped. The handle admits the
retry. Nothing retries.

**Establishes.**

- **R8.66** `safe_retry` consumes proof of one completed re-dispatch attempt,
  not permission to attempt one. The transition may not return an admission
  whose action no production path performs. This is R8.30 read honestly:
  transitions are exposed only when admitted by the current outcome and
  installed contract, and an unperformable transition is not admitted.
- **R8.67** Re-dispatch goes through `dispatch_external_effect`, which remains
  the **single** site where a transport observation becomes a typed posture. A
  second classification path is the "second, weaker effect lane" this phase
  exists to prevent, and R8.24's fault taxonomy would then have two owners.
- **R8.68** R8.28's *correlation evidence* axis binds the co-committed outbox
  record, not the correlation identity alone. The dispatch request is derived
  from correlation **and** correlation family (§8's dispatch derivation); a
  binding carrying only the correlation cannot re-dispatch, so it was never
  carrying the evidence R8.28 names.
- **R8.69** Re-dispatch is admitted through the same fresh-authority path as
  every other effect-producing transition (R8.31). A handle minted before the
  fault authorizes nothing by itself; the retry re-establishes current provider
  truth and current application authority first.
- **R8.70** Exactly-once is asserted at the **rail**, not at the request layer.
  Re-dispatching an effect the external owner already completed emits nothing,
  and the rail's attempt count is the evidence. A request-layer assertion proves
  only that Query declined to ask.
- **R8.71** The durability limit is typed, not implied. A process-local outbox
  survives only as long as the process, and that lifetime **must be published as
  an explicit posture**, following R8.12's `StoreCapabilityRequired` precedent
  and §5 G11. No Query crate gains a `worth-store` dependency. Scanning
  committed outbox rows after process death is **not a requirement of this
  phase**: with no durable substrate there is no row for a sweep to find, so the
  posture states the runtime's actual guarantee rather than deferring an
  obligation.

**Exit proof.** Commit with the transport faulting; the outbox row commits and
no completion posture is published. Mint a handle from that receipt; safe-retry
through production admission; the effect escapes and the rail records exactly
one attempt. Safe-retry again on a fresh handle over a completed effect; the
rail attempt count is unchanged. A copied, expired, foreign-principal, or
terminal handle denies before any transport call is made — proved by the rail
observing nothing. An operation declaring no external effect leaves the retry
path nothing to find, with the transport live (R8.4). All against the real
`bank-external-rail` process; an in-process fake does not close this gate,
under Gate 8.2's standard.

---

## 10. Self-Support Obligations

A specification is self-supporting when executing it produces everything needed
to prove it. This one was not, and the gaps were found by executing it rather
than by reading it — which is the only way this class of defect surfaces.

The governing rule, and the reason this section exists:

> **No gate may depend on anything that no phase is scheduled to produce.** An
> entry condition naming unscheduled work is not a dependency; it is a wish.
> Either a gate owns building it, or a phase is added that does.

### 10.1 Unowned dependencies — repaired above

Two were found:

- **The external boundary.** Gate 8.2's entry required a real controllable
  external service. §5 G3 correctly identified it as a Bank World deliverable,
  but no Bank World phase scheduled it, so the requirement pointed at nothing.
  Gate 8.2 now owns building it.
- **The carrier repairs C1-C4.** Gate 8.4 listed them as an entry condition and
  no gate's `Establishes` produced them. R8.1 now assigns each to an owning
  gate, and 8.3 gains R8.62 for C1.

Both were papered over during execution — the external boundary by instructing
the implementer to build it anyway, C3/C4 by an implementer who happened to
need them. Neither repair came from the specification, which is precisely the
problem: a spec that only works when the executor patches its gaps is not a
spec, it is a sketch with a diligent reader.

### 10.2 The closure ledger is a deliverable, not a reference

§14 says Phase 8 closes when "a Phase 8 closure ledger records every `R8.*` row
as `PROVED`." Nothing required that ledger to be created, named, or owned — it
was referenced as though it already existed, the same shape of defect as the
unowned dependencies above.

**R8.63** — Phase 8 produces
`plans/WORTH-query/milestone-9.16-runtime-phase-8-closure-ledger.md`, following
the Phase 7 ledger's structure and policy: a row per `R8.*` requirement, a row
per `Q8.*` finding, `PROVED` only when production owner, public consumer
evidence, adversarial evidence, performance posture, and residue posture agree.
The ledger is created when Gate 8.1 opens, not when Phase 8 closes — a ledger
written at the end records outcomes; a ledger written at the start governs
them. Each gate updates it as part of that gate's own closure.

### 10.3 Integration evidence must accumulate with the stack

Gates 8.3, 8.4, and 8.5 each consume the previous gate's product: 8.3 consumes
8.2's postures, 8.4 consumes 8.3's handle, 8.5 consumes 8.4's proved undo. Each
gate's exit proof is local to that gate, and the first cross-cutting
adversarial proof appears only at 8.6.

That is four layers deep before anything tests them together, which is the
condition in which an authority leak survives every local suite. The empty
`bank-courtroom` crate (zero tests at the time of writing) means no net exists
underneath them today.

**R8.64** — Every gate from 8.3 onward contributes at least one scenario to a
named cross-gate integration suite, exercising its product **through** the
products of the gates beneath it, not beside them. A gate whose evidence is
entirely local does not close. This does not build the Bank World Phase 6
courtroom early; it requires each gate to leave behind the thread of it that
its own claim depends on, so integration evidence grows with the stack instead
of arriving after it.

### 10.4 Test-world construction is named infrastructure

Gate 8.1's most serious defect was a fixture constructor on the public
production facade minting contracts from fabricated digests. It existed because
the specification demanded honest installed identities and named no sanctioned
way for a test world to obtain one — so the implementer invented a shortcut and
exported it.

Testing law 9 already requires a narrow, explicitly privileged world compiler
producing the same representation, invariants, and authority relationships as
production. The specification never named one as a deliverable.

**R8.65** — Phase 8 test worlds obtain installed contracts, receipts, handles,
and postures through a named world-construction authority living in test scope,
never through a constructor exported from a production crate. Identities are
derived from declared values through the production derivation path. A
production facade exporting a fixture constructor is a defect regardless of
whether production code calls it.

### 10.5 Standing check

Before any future gate is declared closeable, ask of its entry conditions:

1. Does a scheduled phase produce this, and which one?
2. If the answer is "the implementer will handle it," the spec has a gap here.

---

## 11. Courtroom

Phase 8's courtroom extends the milestone's, and inherits its fixture honesty
rules: dynamic provisioning, causal provenance for every identity, no copied
tokens or literal identifiers standing in for authority.

Scenarios that must exist and must fail closed:

1. Committed transfer, compensating reversal, both journals preserved,
   independent oracle agreement.
2. The same reversal requested twice — one compensation, not two.
3. Undo after the beneficiary conflict became active — denied by current
   policy, not by the policy at commit time.
4. Undo of a released estate — irreversible, with no `undo` method reachable
   from the outcome type.
5. Lost response after commit — resolve by idempotency returns the same
   semantic result and moves no money.
6. External effect acknowledged but never completed; completed after timeout;
   acknowledged twice — three distinct postures, none `Completed` until the
   external authority says so.
7. Redo after intervening divergent operation — invalidated by Relational
   expected-head comparison, with both original journals and all committed
   causal facts intact.
8. Redo after the principal's capability expired — denied on fresh admission.
9. Copied recovery handle identity from another principal, another runtime,
   another branch with an equal version ordinal — three denials, three distinct
   causes.
10. Handle expiry mid-inspection, and disposal followed by a transition
    attempt.
11. A crashed user node mid-recovery — no leaked handle, session, or queue.
12. Fan-out twins: 10 postings vs 1000 postings and narrow vs broad retained
    Relational history — every counter in §8 unchanged.
13. An operation declaring an external effect and `Reversible` aftermath —
    rejected at installation, not at execution (R8.57).
14. An operation with no domain mutation and one external effect — its dispatch
    record commits, a lost response resolves by idempotency, and a retry emits
    the external effect exactly once (R8.55).
15. Two installed operations identical except on the authority axis, and two
    identical except on the mechanism axis — four distinct installed
    identities, no collisions (R8.52).
16. A committed effect whose first dispatch faulted, safe-retried through
    production admission — the effect escapes, and the rail records exactly one
    attempt across both attempts combined (R8.66, R8.70).
17. Safe-retry of an effect the external owner already completed — no second
    emission; the rail's attempt count is unchanged (R8.70).
18. Safe-retry attempted on an expired handle, a terminal handle, and a
    foreign principal's handle — three denials, and the rail observes nothing
    on any of them (R8.69).

Test-form obligations (`testing_laws.md`): the external boundary is a real
process, so gate 8.2 and its courtroom rows are integration or end-to-end and
must be named as such; the double-entry oracle is independent of the production
accounting path; every negative case has a positive twin; and the residue and
import checks are mechanical, not reviewed.

---

## 12. Explicit Non-Goals

- A Query-owned commit graph, lineage store, mutable history head, or raw
  parent-causality append API. Relational history is consumed, never rebuilt.
- Runtime Bridge ownership inside Phase 8 of undo/redo legality, semantic
  target selection, composite current-head admission, or publication. Phase 8
  Bridge transport begins only when an already-admitted causal projection
  crosses runtimes; Milestone 9.17 later adds composite branch correspondence
  and orchestration without giving Bridge correction meaning or component
  currentness.
- Branch-, tree-, or graph-shaped undo/redo navigation and branch-local
  inversion before Milestone 9.18, or merge/rebase and semantic conflict
  resolution before their cross-runtime milestones. They get no placeholder,
  directory, parameter, support posture, or implied authority here. Not
  implementing them is the non-goal; foreclosing them in the causal-relation
  or redo-intent types is a defect (R8.45).
- Populating the deterministic re-derivation mechanism, and amending
  architectural law 14 to admit it. Both wait for the first CAD, EDA, or
  simulation consumer. The **axis** that makes them a leaf addition is built
  here (R8.53).
- Repairing the §13 platform-boundary defects. They are routed to their owner
  phases, not deferred silently and not absorbed here. Phase 8's duty is to
  avoid widening them and to record them (R8.59-R8.61).
- Durable recovery handles, restart-stable undo/redo history, or any
  `worth-store` dependency (§G11).
- A generic rollback API, a provider repair call, or an "admin fix" path.
- Replay in any ordinary lane, under any name, including "redo."
- Compensation logic for domains other than the bank world. Banking semantics
  do not generalize into Query.
- Multi-party or distributed transaction coordination.

---

## 13. Discovered Platform-Boundary Defects

Found while specifying this phase, by auditing what Phases 1-7 built for
meaning that is generic to the platform versus meaning shaped by its first
consumer. Each is classified under the milestone's Discovery Intake rule.

**None of these is in Phase 8's causal closure, and Phase 8 does not fix them.**
Folding them in would violate the same amendment rule this phase is bound by:
the phase that exposes a generic gap does not become its owner. What Phase 8
owes is that it does not make them worse and does not let them go unrecorded.

| ID | Defect | Intake category | Owner |
|---|---|---|---|
| PB1 | A general unit-of-measure slot is named after its finance instance | 2 | rename: Phase 9; widening: 9.17 |
| PB2 | A general magnitude-bound slot is named after its finance instance | 2 | Phase 9 |
| PB3 | Trusted time is authorization-scoped and authorization-named | 2 | **Phase 8** — already G6/R8.7 |
| PB4 | One production ordinary-branch literal; four test-local branch counterfeits | 1 and 5 | Phase 9 |

### PB1 — `Currency` is the platform's unit slot wearing a finance name

`ApplicationFieldRef`
(`worth-query-declaration/src/application_schema/references.rs:178`) takes
eight type parameters; the eighth is `Currency`, defaulting to
`NoApplicationCurrency`, declared through `ApplicationCurrencyMarker`
(`application_schema/capabilities.rs:84`) with its own macro arm in
`application_operation_macro.rs:67`. It threads through capability scope, rule
clauses, request projection, query builders, effect programs, read sets, entity
resolution, projection, disclosure, and invariant aggregation: **244
occurrences in `worth-query-declaration`, 116 in `worth-query-execution`**.

The mechanism is correct and must be kept. Because the marker is part of the
field reference's type, passing a USD-tagged field where an EUR-tagged field is
expected is a compile error — dx law 16 implemented at the strongest
enforcement tier.

The name is the defect. The slot encodes "this field's values carry a unit
marker participating in the field's static identity." Currency is one instance.
Millimetres against inches is another; picoseconds, volts, and femtofarads are
others; milligrams against millilitres, and milligrams against milligrams per
kilogram, are a recurring fatal error class in clinical dosing. A CAD or
simulation schema declaring its units through a type named `Currency` produces
exactly the wrong mental model MENTALITY §12 forbids.

The slot is also under-built against dx law 16, which names units, precision,
tolerance, rounding, **and reference frames**. Finance survives on units alone
because milestone decision 24 already forces exact minor-unit integers.
Geometry does not: a coordinate without a reference frame has no meaning, and a
tolerance without precision is a guess.

**Routing.** The rename belongs in Phase 9, before its contracted declaration
and host facade snapshots exist. After that snapshot the name is a frozen
public contract governed by dx law 20 deprecation and law 21 compatibility
windows, and every consumer domain's generated schema names it — so the rename
converts from a mechanical sweep into a permanent migration surface. The
widening to precision, tolerance, rounding, and reference frame is new generic
capability, so it takes its own phase; Milestone 9.19 is its natural owner
given that milestone's geometry-kernel handoff. The split is safe because the
declared marker can gain those dimensions as marker constants or added
parameters without reshaping the slot.

### PB2 — `amount` is the same error, one line deep

`ApplicationCapabilityAmountDimension` is a type alias for
`ApplicationCapabilityFieldDimension`
(`worth-query-declaration/src/application_capability/scope.rs:165`), and
`ApplicationCapabilityConstraintDefinition` carries a fixed `amount` field. The
mechanism is already generic — a magnitude bound on a declared field — and only
the name is shaped by finance. A dose ceiling, a compute budget, and a
tolerance envelope all file under `amount` today. Milestone decision 13 needs
the same one-word edit. Pure rename, same Phase 9 pre-snapshot slot as PB1.

### PB3 — Authorization-scoped clock

Tracked as §G6 and R8.7. This is the one row Phase 8 does own, because recovery
expiry and dispatch timeout classification force it: two clocks would mean two
answers to "has this expired?"

### PB4 — Ordinary-branch literals

One production inline literal, `bootstrap_publication.rs:158`, duplicating the
existing named constant `PRIMARY_APPLICATION_BRANCH`
(`primary_graph/application_branch.rs:4`). The named constant is the honest
form of the milestone's permitted single-branch implementation limit; the
second literal means Milestone 9.17's composite product-branch and branch-local
MVCC cutover has two sites to find instead of one.

Four further literals construct `BranchId("main")` inside `#[cfg(test)]` blocks
(`managed_run/semantic_basis.rs:150`, `managed_run/truth_read_request.rs:78`,
`application_query/basis/historical_authority.rs:33`,
`provider/idempotency.rs:238`). Branch identity has been authority-bearing
since Milestone 9.16.1, and testing law 10 forbids counterfeiting an
authority-bearing value with a literal instead of having world construction
issue it. These are lesser than the production row and belong to whichever
phase next touches those tests, not to a sweep of their own.

### What the audit did not find

No banking vocabulary in the generic Query crates. A sweep of
`worth-query-{declaration,installation,admission,execution,publication}` for
teller, institution, estate, beneficiary, executor, bank, and account-holder
returned only task-*executor* hits. Phases 1-7 held the domain boundary. PB1
and PB2 entered through the schema and units door rather than the domain door,
which is the door that is harder to watch and the reason this audit was worth
running before Phase 8 rather than after.

### Phase 8's obligations

- **R8.59** Phase 8 does not widen PB1. New aftermath surfaces that reference
  application fields — the inverse contract's pre-image demand above all —
  carry the existing unit slot unchanged and mint no aftermath-local unit,
  measure, amount, or currency vocabulary of their own. When the slot is
  renamed in Phase 9, Phase 8's surfaces must move with it as a mechanical
  sweep and nothing more.
- **R8.60** Phase 8's own new surface repeats none of these patterns: no new
  stringly semantic family beyond those R8.9 already types, no new ordinary-
  branch literal, and no test-local `BranchId` construction — Phase 8 test
  worlds receive branch identity from world construction. If Phase 8 edits a
  file named in PB4 for its own reasons, it fixes that file's literal in
  passing rather than leaving a known defect in a file it touched.
- **R8.61** Phase 8 closes only when PB1, PB2, and PB4 are entered in the gap
  ledger at `workspaces/worth-query-bank-world/docs/front-door-closure-ledger.md`
  under their owner phase, in that ledger's existing column shape, with their
  intake category and a consequence command naming what fails if the defect
  returns. Recording them is the exit obligation; fixing them is explicitly
  not, and a Phase 8 that fixes them instead of routing them has broken the
  same rule it is bound by.

  PB1's row additionally records the deadline that makes it urgent: it must
  close **before** Phase 9 takes its contracted declaration and host facade
  snapshots. A rename that misses that snapshot stops being a sweep and becomes
  a permanent migration surface under dx law 20 and law 21.

---

## 14. Exit Condition

Phase 8 closes when the closure ledger required by R8.63 records every `R8.*`
row above as `PROVED` under the Phase 7 ledger's policy — production owner,
public consumer evidence, adversarial evidence, performance posture, and
residue posture all agreeing — with no unresolved high- or critical-impact
finding, the §13 platform-boundary defects routed rather than carried (R8.61),
the §10 self-support obligations satisfied, and Bank World Phase 5 unblocked.

Gate 8.7 is an append-only corrective under §9 and carries the same bar: its
`R8.66`-`R8.71` rows are `PROVED` on the same evidence policy, and the
process-local outbox lifetime it cannot exceed is published as a typed posture
rather than implied.

The one-sentence test a reviewer should apply to any Phase 8 diff:

> Could a caller holding only serialized data, a status value, a matching
> digest, or a stale handle reach this transition? If yes, the phase is open.
