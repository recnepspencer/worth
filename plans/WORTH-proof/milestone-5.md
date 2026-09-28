# Milestone 5 Engineering Spec: Lowering And Execution Readiness

> **Status:** Closed
>
> **Closeout:** [milestone-5-closeout.md](./milestone-5-closeout.md)
>
> **Roadmap parent:** [worth_proof_roadmap.md](./WORTH_proof_roadmap.md)
>
> **Vision parent:** [worth_proof_vision.md](./WORTH_proof_vision.md)
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Adjacent milestone:** [milestone-4.md](./milestone-4.md)
>
> **Adjacent milestone closeout:** [milestone-4-closeout.md](./milestone-4-closeout.md)
>
> **Impacted later milestones:**
> - `Milestone 6` (`Static Fork And Join Progression`)
> - `Milestone 7` (`Certification And Cross-Crate Migration Closure`)
>
> **Primary architectural driver:** make pure lowering and execution admission
> explicit proof-bearing boundaries now that Milestone 4 has made typed
> transition and non-success outcome law canonical

## Goal

Make lowered forms, execution-ready forms, and optional executed-form proof
states first-class in `worth-proof` so planners can produce canonical lowered
artifacts, executors can consume only proven-ready artifacts, and runtime
admission facts terminate back into static proof-bearing forms instead of
collapsing into ambient executor folklore.

## Why This Milestone Exists

Milestone 3 made proof validity basis-scoped and trust-shift-honest.
Milestone 4 made transition ordering, denial topology, and checked progression
canonical.

That still leaves the next structural gap:

- WORTH repeatedly needs to distinguish "plan has been lowered" from "plan is
  actually executable now"
- many real workflows have runtime-gated readiness based on authority,
  capability, basis freshness, or environmental admission
- executors should consume a proof-bearing ready form, not re-decide legality,
  strategy, or authority that should have been established earlier
- some domains also need an explicit post-execution proof state, while receipts,
  diagnostics, and forensic records must remain outside `worth-proof`

Without Milestone 5:

- domain crates will keep inventing bespoke lowered-versus-ready marker pairs,
  runtime tokens, and post-execution wrappers
- Milestone 4 transitions will terminate too early, forcing executors to
  rediscover readiness law ad hoc
- lowered and executed meaning will drift together under convenience pressure
- runtime admission facts will remain ambient rather than re-encoded into static
  proof-bearing forms

Milestone 5 therefore exists to solve the next hard progression problem:

- what it means for a form to be purely lowered
- what extra proof is required before a lowered form becomes executor-consumable
- how runtime-gated readiness remains typed and basis-honest
- how optional executed-form proof states can exist without turning
  `worth-proof` into a receipt or runtime-orchestration crate

## Hard Part

The hard part is not adding another phase marker named `ExecutionReady`.

The hard part is preserving all of these at once:

- pure lowering as a static proof-bearing phase boundary
- Milestone 4 transition law and denial topology without collapsing readiness
  admission into one boolean or generic runtime error
- Milestone 3 freshness and rebind law so readiness cannot silently ignore
  stale or shifted-basis posture
- executor honesty, where execution consumes already-proven legality rather
  than recomputing plan strategy or authority at the hot path
- optional executed-form proof state without accidentally importing receipts,
  diagnostics, or orchestration concerns into this crate

The design fails if:

- executors accept merely lowered forms and re-decide readiness internally
- runtime-gated readiness produces ambient flags instead of static
  proof-bearing ready forms
- lowered, ready, and executed forms collapse into one convenience type
- basis drift or authority drift can be ignored during readiness admission
- `worth-proof` starts owning receipts, result envelopes, or effect runtime
  policy instead of progression law

## Explicit Assumptions

- Milestone 1 carrier law, Milestone 2 sealing, Milestone 3 freshness law, and
  Milestone 4 transition law remain authoritative.
- `worth-proof` still owns progression law only; it does not become a generic
  executor, receipt schema crate, diagnostics engine, or runtime planner.
- domain crates remain the semantic authorities for what execution means, what
  runtime facts are required for readiness, and whether a post-execution proof
  state is useful.
- Milestone 5 may define lowered, execution-ready, and executed-form markers,
  readiness admission surfaces, and execution-facing transition contracts, but
  must not smuggle in a runtime orchestration engine.
- fork/join family composition remains a later milestone, though Milestone 5
  must leave it an honest lowering/readiness substrate to build on.

## Governing Summaries

- `MENTALITY.md`
  The most important thing it protects here is solving the executor honesty
  problem before more orchestration pressure arrives. Milestone 5 therefore
  hardens lowering-versus-readiness law now instead of letting runtime
  rediscovery spread through executors first.
- `arch_laws.md`
  The most important thing it protects is that rejection must precede
  construction, lowered plans are the only acceptable execution inputs, and
  types must encode what has been proven. Laws 17, 24, 27, 30, 37, 39, and 41
  shape this milestone most strongly.
- `perf_laws.md`
  The most important thing it protects is that planning and control-plane
  decisions belong before execution, and expensive or critical facts must not
  be rediscovered. Milestone 5 must let executors consume narrow, already-lowered,
  already-admitted work.
- `domain_laws.md`
  The most important thing it protects is responsibility clarity. Lowering law,
  readiness admission, executed-state hooks, and hostile certification should
  be distinct responsibilities rather than one broad execution helper bucket.
- `worth_proof_vision.md`
  The most important thing it protects is `worth-proof` as a static progression
  substrate that owns pure lowering, execution readiness, and optional
  executed-form proof hooks, while leaving receipts and runtime behavior to
  domain crates.
- `worth_proof_roadmap.md`
  The most important thing it protects is sequence integrity. Milestone 5 is
  the first milestone that makes lowered-versus-ready distinction canonical,
  and Milestone 6 depends on that distinction before widening into multi-artifact
  progression.
- `worth-proof` test requirements
  The most important thing it protects is that the lowered-versus-ready
  boundary must be closed by a named certification suite with hostile shortcut
  lanes, basis-sensitive readiness pressure, and explicit proof that runtime
  admission terminates back into static proof-bearing forms.
- `milestone-4.md`
  The most important thing it protects is the canonical transition contract and
  category-rich non-success outcomes. Milestone 5 must extend that transition
  algebra into lowering and readiness rather than inventing a parallel runtime
  grammar.
- `milestone-4-closeout.md`
  The most important thing it protects is what later milestones may now safely
  assume: typed success/deny/defer/stale/rebind/failed categories, checked
  progression, pre-construction rejection, and equivalence/divergence proof for
  representative transition lanes.

## Adversarial Constraint

The milestone must survive the following hostile condition:

> Several WORTH subsystems with statically lowered plans, runtime-gated
> readiness admission, freshness-sensitive authority checks, and strict hot-path
> cost requirements must be able to hand work from planner to executor such
> that execution consumes only proven-ready forms, lowered legality is not
> rediscovered inside the executor, and any runtime admission fact returns to a
> static proof-bearing form rather than remaining ambient process state.

The design fails if:

- a lowered form can be consumed by execution-facing APIs without explicit
  readiness admission
- runtime readiness checks produce only transient booleans or callback-local
  state instead of a proof-bearing ready form
- stale, rebind-required, or shifted-basis forms can bypass readiness law by
  flowing straight to executor surfaces
- post-execution proof state requires `worth-proof` to own receipts,
  diagnostics, or effect summaries
- Milestone 6 would need to reinterpret what "lowered" or "ready" means before
  composing several artifacts

## Product Decision Lock

- pure lowering is a first-class proof-bearing phase and is not synonymous with
  execution readiness
- execution-facing APIs consume only execution-ready forms, never merely
  lowered forms
- runtime readiness admission is allowed only if it terminates back into a
  static proof-bearing ready form
- freshness, basis, and authority law from Milestone 3 remain load-bearing
  during readiness admission
- optional executed-form proof state is allowed, but receipts, diagnostics, and
  descriptive execution artifacts remain outside `worth-proof`
- the public facade remains the only public entry surface
- static dispatch and monomorphization-friendly hot paths remain the normative
  posture for representative lowering and readiness lanes

Normative consequence:

- any implementation that lets executors accept lowered forms by convenience is
  out of spec
- any implementation that keeps runtime admission meaning ambient instead of
  re-encoding it into a proof-bearing ready form is out of spec
- any implementation that collapses lowered, ready, and executed into one
  generic execution wrapper is out of spec
- any implementation that makes `worth-proof` own receipts or execution
  reporting is out of spec

## Required Contracts

### Lowered Boundary Rule

Lowering must be a first-class proof-bearing phase boundary distinct from both
symbolic intent and executor-ready posture.

Required vocabulary:

- lowered form
- pre-lowered symbolic or resolved form
- lowering transition
- read-only lowered access posture

Rules:

- lowering must terminate in a canonical proof-bearing lowered type
- lowered forms may carry enough structure for execution planning, but not the
  claim that they are currently executable
- later readiness admission must consume the lowered form rather than
  reconstructing the lowering result procedurally

### Execution Readiness Rule

Execution readiness must be an explicit stronger proof-bearing posture layered
on top of the lowered form.

Required vocabulary:

- execution-ready form
- readiness admission transition
- readiness authority or capability posture where required
- readiness denial or defer posture where readiness is not yet granted

Rules:

- readiness admission must consume a lowered form and produce a stronger ready
  form or typed non-success outcome
- if readiness depends on runtime facts, those facts must be represented as
  explicit admission context or witness posture
- execution-facing APIs must require the ready form structurally

### Runtime Admission Termination Rule

Any runtime-gated readiness decision must terminate back into the static proof
grammar rather than remaining ambient executor state.

Required vocabulary:

- runtime admission context
- admitted ready form
- denied or deferred readiness outcome
- basis-shifted or rebind-sensitive readiness outcome where applicable

Rules:

- runtime checks may participate in readiness, but the result must still be a
  typed proof-bearing output
- executors must not need to retain hidden side state to remember why a plan is
  ready
- basis or freshness-sensitive readiness outcomes must compose with Milestone 3
  and Milestone 4 rather than bypassing them

### Executed-State Hook Rule

Domains that need post-execution proof state must be able to model it without
turning `worth-proof` into a receipt or forensic artifact owner.

Required vocabulary:

- optional executed-form proof state
- execution transition
- execution result boundary

Rules:

- executed-state hooks are allowed only as proof-bearing lifecycle state, not
  as ownership of execution reports or descriptive artifacts
- the crate may encode "executed" or equivalent post-application state, but
  domain crates still own receipts, traces, and descriptive output
- this milestone may ship representative executed-state hooks rather than every
  possible execution lifecycle vocabulary

### Compile-Time Readiness Boundary Rule

The highest-risk lowered-versus-ready shortcuts must be compiler-enforced.

Required compile-time posture:

- lowered forms reject execution-ready-only APIs
- pre-lowered forms reject readiness-only APIs where lowering has not occurred
- stale or rebind-sensitive forms reject readiness shortcuts when freshness law
  requires revalidation first
- facade exposure remains synchronized with intended boundary law

### Performance-Shaping Rule

Lowering and readiness law must keep control-plane decisions upstream and avoid
executor-side rediscovery.

Required performance-shaping surfaces:

- representative pure lowering lane
- representative runtime-admitted readiness lane
- representative ready-to-executed lane where executed hooks are modeled

Rules:

- executors must consume the narrowest ready surface that can be resolved
  honestly upstream
- representative lanes must not require mandatory allocation or virtual
  dispatch
- lowering and readiness proofs must be carried forward rather than rediscovered
  in the execution hot path

## Scope

### In Scope

- explicit lowered-form markers or wrappers
- explicit execution-ready markers or wrappers
- readiness admission transitions from lowered to ready
- runtime-admission context surfaces where readiness depends on runtime facts
- representative execution-ready to executed-form hooks
- basis- or authority-gated readiness posture where representative
- compile-fail support for lowered-versus-ready shortcut misuse
- facade hardening needed to keep internal readiness machinery private
- milestone-local certification notes that map directly onto the crate-level
  `Lowering And Execution Readiness Boundary Test`

### Explicitly Out Of Scope

- generic runtime executor ownership
- receipt, diagnostics, provenance, or forensic execution artifact schemas
- multi-artifact fork/join progression
- same-family symbolic composition programs
- domain-specific execution semantics or scheduling policy
- cross-crate migration closure beyond what is needed to prove the substrate
  shape

## Phases

### Phase 1: Lowered Form Canonicalization

Define the canonical lowered boundary before any readiness or execution hooks
are introduced.

Must ship:

- explicit lowered-form vocabulary for representative proof-bearing artifacts
- representative lowering transition from pre-lowered forms into lowered forms
- clear distinction between resolved-or-symbolic inputs and canonical lowered
  outputs
- explicit rejection of "lowered means probably executable" convenience

Implementation guidance:

- extend the existing recipe/progression surfaces first rather than inventing a
  separate executor-owned carrier family
- keep lowering pure and structural; this phase should not yet decide runtime
  readiness
- prefer one honest lowered grammar that later readiness admission can consume
  directly

This phase is complete only when the crate can honestly say "this artifact is
lowered" without also claiming "this artifact is ready to execute now."

### Phase 2: Execution Readiness Admission

Define the stronger proof-bearing readiness posture that executors may consume.

Must ship:

- explicit execution-ready vocabulary
- readiness admission transition from lowered to ready
- representative readiness authority, capability, or context posture
- typed non-success readiness outcomes for denial, defer, stale, or rebind
  pressure where representative

Implementation guidance:

- build directly on Milestone 4 transition law instead of inventing a separate
  readiness result grammar
- preserve Milestone 3 freshness semantics during readiness admission
- keep executor-facing APIs narrow and obviously stronger than lowered-only APIs

This phase is complete only when a lowered form must explicitly cross
readiness admission before execution-facing APIs become callable.

### Phase 3: Runtime Admission And Executed-State Hooks

Define how runtime facts may participate in readiness while still returning to
static proof-bearing forms, and add representative post-execution hooks where
useful.

Must ship:

- representative runtime admission context for readiness-sensitive transitions
- representative proof-bearing ready output from runtime-gated admission
- representative optional executed-form hook for domains that need
  post-execution state
- explicit structural boundary between proof-bearing executed state and
  out-of-scope receipts or descriptive artifacts

Implementation guidance:

- runtime involvement is allowed only at the admission boundary, not as ambient
  executor folklore afterward
- if executed-form hooks are introduced, keep them narrow and lifecycle-shaped
  rather than report-shaped
- do not force every domain to use executed-state hooks if ready-only surfaces
  are the honest boundary

This phase is complete only when runtime-gated readiness can end in a static
ready form, and optional executed-state proof can exist without dragging
reporting semantics into the crate.

### Phase 4: Hostile Certification And Closure

Prove that lowering, readiness, and optional executed-state hooks are
mechanically distinct and that executors cannot bypass readiness law.

Must ship:

- hostile compile-fail coverage for lowered-versus-ready misuse
- representative runtime-admission hostility for stale, shifted-basis, denied,
  or deferred readiness pressure where applicable
- representative equivalence lane for semantically identical lowered-to-ready
  admission paths
- closure record of what Milestone 6 and Milestone 7 may now assume about
  lowered and ready proof-bearing surfaces

Implementation guidance:

- the named suite should certify that executors consume ready forms rather than
  inferring it from internal helper structure
- certify at least one lane where runtime admission facts return to a static
  ready form
- publish explicit residual debt if first ship keeps executed-state coverage
  representative rather than exhaustive

This phase is complete only when the milestone has machine-checkable evidence
that lowered and ready forms do not silently collapse and runtime admission
does not remain ambient state.

## Must Ship

- one canonical lowered-form progression boundary
- one canonical execution-ready progression boundary
- one canonical readiness admission story from lowered to ready
- representative runtime-admission context surfaces where readiness depends on
  runtime facts
- representative optional executed-form proof hook
- compile-fail coverage proving lowered-versus-ready shortcuts are uncallable
- milestone-local implementation notes that map directly onto the crate-level
  certification bar in `worth-proof/test-requirements.md`

## Must Preserve

- Milestone 1 carrier law
- Milestone 2 sealing and witness authority posture
- Milestone 3 freshness, rebind, and readmission semantics
- Milestone 4 transition and typed non-success outcome law
- plan / lower / ready / execute separation
- zero-cost hot-path posture after monomorphization
- no mandatory heap allocation
- no mandatory dynamic dispatch
- no executor-side strategy rediscovery
- no silent collapse of lowered and ready meaning
- no drift of `worth-proof` into receipts, diagnostics, provenance, or runtime
  orchestration ownership

## Acceptance Evidence

Milestone 5 is complete only when `worth-proof` satisfies the named milestone
suite:

- `Lowering And Execution Readiness Boundary Test`

Required machine-checkable outputs:

- `transition_digest`
- `basis_digest`
- `failure_digest`
- `compile_fail_bundle`

Milestone-specific proof obligations:

- lowered forms cannot silently flow into execution-ready-only APIs
- executors consume only admitted ready forms
- runtime-gated readiness terminates back into static proof-bearing forms
- equivalent lowered-to-ready admission lanes match exactly
- stale, shifted-basis, denied, deferred, or rebind-sensitive readiness lanes
  diverge explicitly where their semantics differ
- representative executed-state hooks, if present, remain distinct from
  receipts and descriptive execution artifacts
- the suite includes hostile lowered-shortcut and runtime-admission lanes

Milestone 5 is not closed by "there is a Lowered marker" or "the executor
calls a helper before it runs" arguments.

## Architectural Notes

- Milestone 5 is execution-boundary law, not an execution runtime.
- Pure lowering and execution readiness are distinct proof-bearing claims even
  when the same payload eventually flows through both.
- Runtime admission is allowed only as a boundary fact that returns to the
  static grammar.
- Executed-state hooks are about lifecycle proof, not about storing receipts or
  explanations.

## Implementation Topology

This milestone should extend the existing `recipe/`, `transition/`, and facade
topology rather than introducing a parallel runtime layer.

Preferred additions:

```text
crates/worth-proof/src/
  facade.rs
  recipe/
    lowering.rs
    readiness.rs
    execution.rs
  transition/
    readiness.rs
```

This is not a forced final topology, but the ownership boundaries are
intentional:

- `recipe/lowering.rs`
  - canonical lowered-form vocabulary and representative lowering transitions
- `recipe/readiness.rs`
  - execution-ready vocabulary and readiness admission progression
- `recipe/execution.rs`
  - representative executed-form hooks and ready-to-executed progression
- `transition/readiness.rs`
  - readiness-shaped transition contracts that extend Milestone 4 rather than
    replacing it

The milestone should avoid:

- one giant mixed `execution_readiness_everything.rs`
- letting executors own the canonical ready type
- treating receipts or diagnostics as proof-bearing execution state
- bypassing the facade by exposing internal readiness helpers publicly

## Sequencing Notes

- This milestone belongs immediately after Milestone 4 because canonical
  transition and non-success law must exist before the crate can distinguish
  lowered versus ready progression honestly.
- Milestone 6 depends on this milestone because multi-artifact fork/join and
  composition-family lowering need a clear definition of when a composed result
  is merely lowered versus ready for execution or publication.
- Milestone 7 depends on this milestone because cross-crate migration cannot be
  honest until the shared substrate proves executors consume proof-bearing ready
  forms instead of rediscovering legality ad hoc.
