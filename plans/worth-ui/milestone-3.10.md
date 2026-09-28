# Milestone 3.10: Mounted Receipts and Host Contract

> Terminology: the allocation ledger named in this specification is a
> product-domain runtime authority, not a QA, proof, closure, or phase ledger.
> The historical-ledger retirement policy does not apply to it.

Status: Complete (2026-07-25). Phases 1 through 10 are implemented and
verified. The human-visible Platform Pulse requirement was adopted after
completion and was closed as product capability by the explicit Milestone
3.10.2 catch-up gate. Milestone 3.10.3 owns the later-discovered
executable-world evidence correction before 3.11.

## Goal

Close the Worth UI host boundary by making one runtime-owned mounted frame the
only ordinary artifact a host may present. The runtime must lower the current
admitted application, graph, measurement, allocation, and execution truth into
identity-backed mounted-node and mounted-frame receipts; bind those receipts to
the exact application, plan, allocation, host session, semantic-surface
manifest, surface-binding generations, and frame generation; assemble every
participating execution lane and surface into one candidate; and publish the
matching runtime tuple atomically after a typed host presentation outcome.

The host owns native mechanics: pixels, windows, device scale, pointer and
keyboard reports, focus reports, scroll reports, clock or tick reports, and
text/IME reports and text-measurement results. It does not own visibility,
enabledness, validity,
layout meaning, semantic focus, hit-test meaning, accessibility meaning,
motion meaning, diagnostics meaning, application lifecycle, or Query meaning.

This milestone replaces Milestone 3.9's minimal host-output envelope and the
direct preview-paint path. It does not add a second output abstraction, prebuild
visual snapshots, admit host observations into UI meaning, broaden Query
projection products, route intent, introduce general runtime services, or
define the appearance system.

## Why This Milestone Exists

Milestones 3.9 through 3.9.2 established hot application replacement, canonical
execution planning, allocation, lane execution, and a Query-native input
boundary. The remaining runtime-to-host seam is intentionally narrow but not
yet strong enough to carry the platform:

1. the active framework turn presents lane outputs independently rather than
   assembling one cross-lane frame;
2. the minimal host-output envelope carries summary identity and counts rather
   than the complete mechanical projection a renderer needs;
3. mounted identity still follows a graph-node-shaped one-to-one model even
   though one semantic node may have zero, one, or many mounted instances;
4. preview painting can bypass the canonical host-output path;
5. the current `UiHostObservation` name describes solicited measurement
   evidence, while Milestone 3.12 needs a distinct general observation
   admission artifact;
6. replacement can publish application, plan, and allocation truth before a
   matching mounted frame has been admitted and presented; and
7. public digest-shaped construction is too weak to prove exact host-session,
   surface, generation, and publication authority.

If those seams survive into snapshots, hot rebind, Query projection, intent,
services, and appearance, every later subsystem will either depend on partial
host truth or invent a compensating side path. Milestone 3.10 therefore closes
the projection boundary first.

## Governing Summaries

- `MENTALITY.md` protects causal closure under adversarial pressure. The
  milestone must replace the weak host boundary as a foundation, not place
  richer receipts beside predecessor scaffolding.
- `arch_laws.md` requires typed, owner-minted progression and compiler-visible
  authority. Mounting must culminate in one proof-bound frame artifact with
  typed preparation, presentation, publication, denial, and indeterminate
  states.
- `composition_laws.md` requires one named semantic responsibility per file and
  visible control transfer. Identity, projection, frame assembly, publication,
  retention, host translation, observation reporting, and inspection must not
  collapse into a generic host or receipt module.
- `domain_structure_laws.md` requires physical topology to reveal truth source,
  authority, lifecycle, and future insertion. The destination tree must admit
  Milestones 3.11 through 3.16 without moving the public facade or letting
  diagnostic containers become operational owners.
- `perf_laws.md` requires honest hot-path costs and carried proof. Initial mount
  may scale with the mounted closure; steady mount may scale only with changed
  semantic instances plus the honest batch granule; unchanged presentation
  may claim constant time only through a carried exact reuse witness.
- `worth_ui_roadmap.md` requires mounted and host truth to close before visual
  snapshots, observation admission, hot rebind, Query projection, intent,
  services, and appearance build on it.

## Adversarial Constraint

One 240 Hz framework turn may contain ordinary, virtualized, canvas, and
realtime output across multiple native surfaces with different capability and
device-scale profiles. During that turn, a source edit may replace the active
application; a virtualized collection may reorder, unmount, and remount
instances; one host surface may lag or fail after another has accepted work;
and duplicate, reordered, stale, or foreign observation reports may arrive.

A successful outcome must establish one completely presented and published
mounted generation. A rejected-before-effects outcome must leave the prior
published runtime truth and prior known host presentation truth unchanged. The
runtime must never classify a lane-by-lane, surface-by-surface,
application-by-application, or generation-mixed presentation as success.
Before native effects begin, every surface must be admitted against the exact
host session, capability profile, application generation, graph world,
execution plan, allocation generation, mounted frame generation, and surface
identity. If native effects become indeterminate, the runtime must surface a
third, blocked outcome explicitly and must not invent rollback or silently
publish a partial current frame.

Equal printable IDs, digests, visible values, geometry, primitive batches, or
diagnostic text from different runtimes, worlds, sessions, surfaces,
applications, plans, allocations, or frame generations must never alias.
Reorder without unmount must preserve a semantic mounted instance; actual
unmount and remount must create a new mount incarnation even when the graph
node and repeated-instance basis are unchanged.

Initial projection may perform work proportional to the mounted semantic
closure plus its compact render batches. A steady delta may perform work
proportional only to changed semantic instances, affected indexes, and the
honest batch and affected-surface granules. An unchanged frame may claim
constant reuse only through carried exact equivalence proof. Work must not
scale with total authored declarations, total Query
projection width, all graph nodes, `surfaces * nodes` when surfaces share
projection, or one generic receipt allocation per canvas primitive. Retained
frames, in-flight presentations, observation reports, diagnostic projections,
and inspection evidence must all have named bounds.

The 240 Hz condition is an arrival and correctness stress model, not a
hardware-independent latency promise. This milestone must remain semantically
correct and memory-bounded under that demand, using explicit presentation,
backpressure, coalescing, or rejection outcomes when capacity is exceeded. Any
throughput or percentile claim must additionally name workload distribution,
surface and batch scales, hardware, runtime profile, cold/warm posture,
repetitions, variance, utilization, queue bounds, and reported percentiles.

## Product Decision Lock

- One `UiMountedFrameReceipt` is the canonical ordinary runtime-to-host
  artifact. It contains or references every participating lane and native
  surface for one frame generation. It is the canonical boundary projection,
  not authoritative application truth and not proof that presentation or
  publication occurred.
- `UiMountedNodeReceipt` identifies a mounted semantic instance. It is not a
  graph-node alias, an authored declaration, a renderer widget ID, or a paint
  primitive.
- One graph node may map to zero, one, or many mounted instances. Mounted
  indexes must represent that cardinality explicitly.
- Stable mounted identity uses semantic repeated-instance basis and an opaque
  mount incarnation. Positional indexes, visible order, geometry, digests, and
  host widget IDs are not identity.
- Runtime semantic surface identity, host-native surface registration identity,
  and their generation-bound association are separate types. A mounted instance
  belongs to one semantic surface; a surface receipt belongs to one exact
  semantic-to-native binding generation.
- A presentation attempt uses a frozen admitted surface-binding set. Milestone
  3.10 registers, validates, resets, and deregisters host mechanics, but does not
  invent application window creation or destruction meaning. A candidate whose
  required semantic surface lacks the necessary binding returns typed
  surface-rebind-required posture before effects; Milestone 4 owns broader
  multi-window shell lifecycle.
- A mounted frame may be empty, diagnostic-only, bulk-only, multi-lane, or
  multi-surface. “Frame” never means “one ordinary-lane payload.”
- Paint, clip, layer, allocation, input participation, focus participation,
  hit-test participation, accessibility projection, motion projection, and
  diagnostic projection are separate facts or compact tables. A generic
  per-primitive receipt abstraction is forbidden.
- Input participation is eligibility and routing evidence, not a callback,
  intent, command, or mutation result. Focus, scroll, portal, motion,
  selection, and command-related fields are projections, not host permission
  to implement service policy.
- Paint projections carry current admitted mechanical visual output or typed
  opaque references without inventing Milestone 3.16's theme roles or state
  axes. The host may translate that output but may not choose semantic colors
  or visual posture.
- Hosts receive only sealed mechanical projections and the minimum concrete
  platform authority required to consume them. Public callers cannot mint a
  valid current mounted generation from raw IDs or digests.
- Mounted-frame views, presentation outcomes, observation reports, and
  measurement exchanges carry explicit host-protocol identity and version.
  Producer and adapter compatibility is negotiated before effects; unsupported
  versions are never reinterpreted as current bytes.
- The mounted-frame canonical core carries protocol and schema identity, exact
  frame and publication basis, required manifest, integrity evidence,
  contractual cost reference, recovery posture, and diagnostic disposition.
  Optional traces or rich explanations are policy-bounded sidecars with typed
  omission; they cannot be required to interpret correctness or authority.
- Self-description grants no disclosure authority. Each surface receipt is
  audience-scoped to the exact local host binding and carries only presentation,
  accessibility, resource, and diagnostic facts required by that contract.
  Redaction, omission, retention, and deletion posture remain typed; credentials
  and unrelated Query or causal evidence are never mounted.
- The runtime requests presentation; it does not call host rollback. Outcomes
  distinguish rejection before effects from an indeterminate result after
  effects may have begun.
- Native presentation is classified as a reconcilable external effect, not a
  reversible transaction. Re-presenting the current published frame on a fresh
  or reset binding is reconciliation, never inverse rollback.
- `Unchanged` means the existing published frame remains exactly reusable under
  the same host session, required surface manifest, surface-binding
  generations, and complete adapter-visible projection. It mints no new frame
  identity and performs no host presentation.
- Application replacement is complete only when application, graph, execution
  plan, allocation, mounted indexes, and current mounted frame agree on one
  admitted generation. A failed candidate preserves the predecessor truth.
- General host observation reports are untrusted, non-reentrant, bounded, and
  generation-aware. The host may report mechanics; Milestone 3.12 owns
  semantic observation admission and rebind planning.
- Solicited text-measurement evidence is renamed and remains a distinct
  request-response exchange. It is not the general host observation stream.
- The direct preview-paint path and lane-local host presentation path are
  removed after cutover. Compatibility wrappers may not preserve either route.
- Query artifacts, Query identities, Query rows, Query patches, and Query
  diagnostics never cross the host boundary. The mounted projection contains
  only UI-owned meaning.
- Opaque frame-basis references let the contract validate exact application,
  plan, allocation, and publication-candidate affinity without exposing those
  upstream artifacts or enough representation to reconstruct their authority.
- New proof belongs in existing consolidated certification owners. The
  milestone creates no nested Cargo workspace, per-phase integration crate,
  generated fixture workspace, or mechanical “old symbol is absent” migration
  test.
- Per-phase test bullets are proof obligations, not a mandate for one test
  function or binary apiece. Implement the smallest coherent set of real
  lifecycle scenarios that independently covers them, sharing compiled owners
  without hiding which oracle proved each obligation.
- Search discovers migration candidates; human review adjudicates the
  subsystem and boundary CSVs. The CSVs are design evidence, not runtime
  manifests or checked-in sentinel tests.

### Phases 1-8 Authority Retrofit Lock

- Runtime owners mint validity, adapters own native effects, and one
  runtime-owned host-truth coordinator classifies every effectful transition.
  Matching raw contract fields or receipts never recreates authority.
- Surface registration and deregistration have three exhaustive outcomes:
  rejected before effects, verified applied, and effects indeterminate. A
  malformed success receipt is indeterminate because native effects may already
  have occurred.
- Presentation authority is an adapter-issued, non-cloneable active-session
  lease. Runtime may hold the one live lease and borrow a mechanical
  consumption view from it, but cannot mint or publicly bind adapter authority.
  Asynchronous completion tokens are move-only and `Pending` returns ownership
  of the token.
- Registration, deregistration, and presentation uncertainty share one
  host-truth owner. Identity indexes do not call adapters or maintain a second
  blocked-state map.
- Solicited measurements begin from caller intent. The lifecycle owner issues
  the request identity and private dependency basis from current session,
  binding, allocation, and adapter-environment truth. Completion never accepts
  a caller-supplied basis or identity.
- Observation validation progresses through sealed structural, exact-coverage,
  basis, and retention phases. Coalescing and overflow remain distinct loss
  semantics, and exhausted `u64` sequence sources cannot silently continue.
- The manually adjudicated
  `milestone-3.10-phases-1-8-authority-effect-inventory.csv` is the migration
  reference for constructors, tokens, revisions, adapter calls, and success
  receipts. Search seeds discovery, but unresolved rows block implementation;
  no mechanical symbol-sentinel test substitutes for adjudication.

## Proof Architecture

Milestone 3.10 extends the existing consolidated certification program; it
does not grow a parallel test architecture. Proof is divided by the production
boundary each observer can honestly certify:

- deterministic model evidence certifies mounted-identity continuity,
  presentation-state progression, publication eligibility, report sequencing,
  coalescing, overflow, and retention invariants across bounded operation
  traces;
- a scripted host implements the exact production host contract and exposes
  controlled completion, rejection, cancellation, and lost-result mechanics.
  It certifies caller-side protocol and state-machine behavior only; it cannot
  certify egui, native effects, filesystem behavior, or timing;
- the production headless adapter emits a canonical mechanical transcript
  observed after adapter translation rather than echoing the input receipt;
- real egui evidence observes the native consequences of every effect family
  the mounted projection completely defines. For an incomplete or unsupported
  family, it observes typed denial before effects and independently confirms
  that no native effect occurred; a non-empty frame alone is insufficient;
- real lifecycle certification begins with actual `.wui` bytes on disk, uses
  production filesystem acquisition and operating-system watcher delivery,
  crosses the public mounted facade and real adapter boundary, and observes the
  published runtime tuple independently.

The model oracle shares public vocabulary and declared transition inputs with
production, but it may not call the production classifier, comparator,
canonicalizer, mounted projector, presentation coordinator, or report
coalescer whose semantics it judges. Adapter-neutral expected facts are stated
from the authored scenario contract. Adapter consequences are observed after
translation. Runtime publication is observed through the public current-tuple
inspection surface. No one observer is allowed to certify both its own input
and its own correctness.

Fixture infrastructure is a small portfolio of immutable, causally complete
worlds: a registered known-empty surface world, an adapter-projection world
reached through real mounted projection and recording its exact admitted,
deferred, withheld, and omitted mechanics, a published mounted world, and an
in-flight presentation world reached through the real host contract.
Tests receive production-minted semantic handles for applications, graph
nodes, mounted instances, sessions, surfaces, bindings, frames, and
publications. They derive named isolated deltas such as reorder,
unmount/remount, surface rebind, application replacement, lost completion,
report overflow, and shutdown. Indeterminate and corruption worlds are reached
only through named fault or corruption fixtures. Setup, action, observation,
and teardown remain separately diagnosable and record scenario identity plus
the deterministic operation trace or seed needed to reproduce a failure.
Here, immutable world means a reusable source blueprint, compiled artifact, or
checkpoint whose production postconditions are proven; it never means one live
mutable runtime shared by tests. Every case receives a fresh session, isolated
namespace, cloned snapshot, or transactional delta with no ordering, clock,
identifier, cleanup, or teardown dependence on another case.

All fixtures use ordinary production composition roots, contracts, clocks,
schedulers, and external ports. Replaceable ports may be driven
deterministically, but test-only runtime branches, hidden constructors,
weakened validation, alternate publication paths, and counterfeit authority
are forbidden. Every important proof family includes a positive control and a
named stale, foreign, inverted, bypassed, or misrouted control showing that its
oracle is sensitive to the defect it claims to catch.

Proof remains in the existing compiled owners:

- the ordinary fast lane retains library and facade evidence and performs no
  operating-system watcher wait;
- the existing `application_contracts` and other certification targets own
  bounded model traces, real filesystem/watcher work, scripted host-protocol
  scenarios, real adapters, replacement, recovery, and structural cost proof;
- the existing two-session compile-contract owner proves only irreducible
  public unrepresentability;
- scheduled stress may broaden seeds, scale, and saturation, but no correctness
  claim depends solely on a rarely run stress lane.

Phase 1 records opening clean, warm, link, fixture, execution, and external
startup evidence for the affected lanes. Phase 10 records comparable closing
evidence and updates the mechanically checked topology budget. New integration
targets, compiler sessions, fixture workspaces, retries, ignored tests, or
ordinary-lane external waits require an explicit reviewed budget amendment.
The flake-retry budget is zero.

Each phase's proof setup is part of that phase's deliverable. The phase must
name the production claim and plausible defect, authority boundary, canonical
world or causally complete construction path, isolated scenario delta,
independent oracle, intended failure cause, consequential observations,
mutation-sensitive control, isolation and teardown mechanism, unique evidence,
compiled test owner, and total compile plus execution cost before adding cases.
These facts stay visible in the fixture API and test module topology; they do
not become a second machine-consumed test manifest.
If a phase cannot obtain a valid world through production authority, that is a
production-boundary defect to fix, not permission for a test-only constructor.
In particular, launch's truthful zero-row allocation commit and a count-only
paint batch are not native-paint setup. A phase may claim a native effect only
when `adapter_projection_world` proves that the production frame owns every
input needed to execute that effect. Otherwise the required result is a typed
unsupported or omitted posture before effects, with an independent zero-effect
oracle, not synthetic fixture state.

## Semantic Vocabulary

- A **semantic surface** is runtime-owned UI placement meaning such as an
  application window, preview world, or portal target. It is not a native
  window handle.
- A **host surface** is one host-session-owned native registration. Its identity
  dies with deregistration or its host session. Registration establishes a
  typed known-empty baseline or denies; an unclassified preexisting native
  surface cannot enter ordinary presentation.
- A **surface binding generation** is the exact admitted association between
  one semantic surface and one host surface together with the capability,
  device-scale, coordinate, and resource posture used for presentation.
- A **presentation mode** is part of that binding: for example,
  `NativeDisplay` or `RecordOnly`. “Presented” means every required binding
  completed its declared mode; record-only completion never claims pixels,
  focus, accessibility delivery, or another native effect it did not perform.
- A **mounted instance** is one graph node plus admitted repeated-instance basis,
  semantic surface, operating world, and mount incarnation. Its identity may
  survive frames and reorder, but never actual unmount.
- A **mounted node receipt** is one frame-scoped immutable projection of a
  mounted instance. Its identity is the mounted-instance identity plus the exact
  mounted-frame identity; it is not the stable instance identity.
- A **mounted frame identity** is owner-minted lifecycle identity scoped to the
  runtime world and host session. Separate assembly attempts never share it
  because content matches; exact unchanged reuse returns the existing identity
  rather than minting another.
- A **prepared mounted frame** is complete, sealed, and validated runtime output
  with no host effects and no current-publication authority.
- A **presented frame** has typed host evidence that every required surface
  completed presentation. Presentation evidence does not itself make runtime
  application, plan, allocation, or mounted indexes current.
- A **published frame** is the frame named by the atomically current runtime
  tuple. Publication is permitted only after complete presentation or exact
  unchanged reuse.
- **Known host presentation truth** is the last fully presented surface set the
  runtime can prove, including the registration baseline before the first
  frame. It is distinct from desired output and from current runtime truth.
- **Presentation indeterminate** means at least one required native effect may
  have occurred but complete presentation cannot be proven. Runtime current
  truth remains the predecessor, known host presentation truth becomes unknown
  for the affected bindings, normal presentation and report admission are
  blocked there, and explicit reconciliation is required.
- **Rejected before effects** means the adapter proves that no required target
  surface received candidate effects. It is the only host rejection that may
  claim the predecessor presentation remained untouched.
- **Unchanged reuse** is a carried equivalence proof for the exact published
  frame and exact current surface bindings. It is not a digest comparison,
  semantic similarity judgment, or newly minted empty frame.

## Boundary Contract

The forward authority chain is:

```text
admitted application + graph world + measurements + allocation + plan
-> runtime-owned mounted-instance projection
-> complete cross-lane, multi-surface prepared frame
-> exact host-session and capability admission
-> host presentation attempt
-> typed rejected-before-effects / presented / in-flight / indeterminate outcome
-> infallible atomic current application / plan / allocation / mount publication
   after complete presentation
-> bounded retained frame and inspection evidence
```

The return path is deliberately weaker:

```text
native host mechanics
-> bounded report batch carrying session + surface + presented-frame basis
-> shape, provenance, sequence, and bound validation
-> validated but still semantically unadmitted report for the Milestone 3.12
   intake owner
```

There is no reverse authority edge. A host report, pixel, geometry value,
widget ID, receipt digest, screenshot, diagnostic label, or mounted identity
cannot authorize UI meaning, Query work, application replacement, or frame
publication.

After native presentation succeeds, current-tuple publication contains no
allocation, host call, fallible derivation, policy decision, or external
effect. Every publication input and index replacement is prepared beforehand.
If that total commit cannot be guaranteed, presentation admission must deny
before effects rather than create a success-without-publication state.

## Target Developer Experience

The ordinary application path requests one frame for semantic surfaces; it
does not select execution lanes or construct host payloads:

```rust
let outcome = session.execute_mounted_frame(
    UiMountedFrameRequest::for_semantic_surface(main_surface),
    |turn| {
        turn.run(application_target)?;
        Ok(())
    },
)?;

match outcome {
    UiMountedFrameOutcome::Published(frame) => {
        inspection.observe(frame.identity());
    }
    UiMountedFrameOutcome::Unchanged(frame) => {
        inspection.observe(frame.identity());
    }
    UiMountedFrameOutcome::Reconciled(frame) => {
        inspection.observe_reconciled(frame.identity());
    }
    UiMountedFrameOutcome::PreparationDenied(denial) => {
        diagnostics.record(denial);
    }
    UiMountedFrameOutcome::PresentationRejected(rejection) => {
        diagnostics.record(rejection);
    }
    UiMountedFrameOutcome::PresentationPending(pending) => {
        inspection.observe(pending.identity());
    }
    UiMountedFrameOutcome::RetentionDenied(denial) => {
        diagnostics.record(denial);
    }
    UiMountedFrameOutcome::PresentationIndeterminate(report) => {
        recovery.require_host_reconciliation(report);
    }
}
```

Advanced callers may request multiple admitted semantic surfaces, but the
candidate or current admitted plan chooses lanes and the runtime resolves each
semantic surface through its exact current native binding. Lane-specific
execution remains an internal or certification surface unless a later
milestone establishes a distinct public product need.

## Destination Topology

The exact file split may refine during implementation, but ownership must land
under these semantic containers:

```text
worth-ui-host-contract/src/
  mounted_frame/
    node_receipt.rs
    frame_receipt.rs
    surface_receipt.rs
    identity/
      semantic_surface.rs
      host_surface.rs
      surface_binding.rs
      mounted_instance.rs
      mounted_node_receipt.rs
      mounted_frame.rs
    participation/
      paint.rs
      input.rs
      focus.rs
      hit_test.rs
      accessibility.rs
      motion.rs
      diagnostics.rs
    compact_tables/
    presentation_outcome.rs
  observation_report/
    report.rs
    batch.rs
    family.rs
    sequence.rs
    time_basis.rs
  measurement_exchange/
  capability_report/
  protocol/
    identity.rs
    version.rs
    negotiation.rs
    mounted_frame_schema.rs
    observation_schema.rs
    measurement_schema.rs
  headless/
    recorder.rs

worth-ui-runtime/src/
  mounting/
    identity/
    node_projection/
    frame_assembly/
    publication/
    presentation_reconciliation/
    retention/
    indexes/
    counters/
  host_exchange/
    presentation_admission/
    observation_report_validation/
    measurement_admission/
  replacement/
    mounted_publication/
  inspection/
    mounted_frame/

worth-ui-host-egui/src/
  adapter.rs
  frame_runner.rs
  mounted_translation/
  observation_translation/

worth-ui-certification/tests/application_contracts/
  mounted_protocol_model/
    identity_transition.rs
    presentation_transition.rs
    report_sequence_transition.rs
  mounted_application_lifecycle/
    known_empty_surface_world.rs
    adapter_projection_world.rs
    published_mounted_world.rs
    in_flight_presentation_world.rs
    authored_expectation.rs
    scenario_delta.rs
  mounted_host_protocol/
    scripted_host.rs
  mounted_presentation_model_trace.rs
  mounted_convergence_lifecycle.rs
  mounted_presentation_recovery.rs
  mounted_bounded_trace_certification.rs
```

`worth-ui-host-contract` owns inert cross-crate contract values. The runtime
owns their authoritative construction, admission, lifecycle, currentness,
publication, retention, and semantic indexes. Adapters own translation to and
from native mechanics. Inspection owns projections over retained evidence and
never becomes an operational truth source.

Canonical worlds, authored expectations, scenario deltas, and scripted-host
mechanics live under responsibility-named child modules of the existing
`application_contracts` target. That placement keeps real filesystem, launch,
adapter, and teardown setup out of unrelated certification targets while
creating no new compiler session. The independent transition models are also
target-local because 3.10 has one compiled consumer; moving one into the
certification library requires a demonstrated second existing consumer and a
reviewed compile-cost change. None of this infrastructure enters runtime,
host-contract, or adapter production topology. The three application-contract
files are child modules of the existing compiled suite, not new Cargo
integration targets. Shared certification infrastructure may remove setup
ceremony but may not combine world construction, transition modeling, adapter
observation, and assertions into one universal fixture.

Fixture delivery follows the production lifecycle rather than preceding it
with counterfeit state:

- Phase 2 establishes `known_empty_surface_world` through real `.wui`
  acquisition, application launch, semantic-surface creation, and successful
  production host registration. Identity tests consume its production-minted
  handles rather than rebuilding registration with local IDs.
- Phase 3 establishes `adapter_projection_world` by extending
  `known_empty_surface_world` through the public production projection and
  assembly entry points. It begins with actual `.wui` source and
  production-minted application, graph, allocation, surface, and frame
  authority. Its construction transcript records the exact participation,
  allocation, compact-table, resource, and omission posture the runtime
  produced; a fixture may not insert receipts or table rows, change
  participation, assign geometry, or upgrade omission into admission.
- Phases 3 and 4 extend the appropriate known-empty or adapter-projection world
  through public projection and assembly entry points. Their fixture support
  may remove repeated ceremony, but it may not mint a prepared, presented, or
  published frame directly.
- Phase 5 establishes `in_flight_presentation_world` only after a scripted
  host has accepted a real prepared frame through the production presentation
  contract. Controlled completion, rejection, cancellation, and lost-result
  mechanics begin at that port; they are not test-only runtime branches.
- Phase 6 establishes `published_mounted_world` only from an admitted
  production presentation outcome and the real publication transition. Later
  routing, recovery, replacement, observation, and end-to-end tests derive
  named deltas from that immutable world instead of assembling partial runtime
  state independently.
- Every canonical world owns a causally complete construction transcript and
  teardown. A world helper may return production-minted handles and independent
  observations, but it may not return production-generated expected values or
  combine setup, action, oracle, and assertion into one opaque helper.

The phase that first makes a canonical world representable cannot close until
that world has a positive control and a named stale, foreign, or bypass control
showing that the setup fails for the defect its consumers rely on it to expose.
No later phase may keep a parallel ad hoc fixture once the corresponding
canonical world exists.

The existing graph identity owner retains semantic node and repeated-instance
basis. The mounting owner adds mount incarnation and surface/frame binding; it
does not move graph identity into the host contract. The existing
`runtime/host_observation` container is split by responsibility rather than
renamed wholesale: measurement exchange, general report validation, and
inspection evidence receive distinct owners. The headless recorder remains a
named adapter responsibility inside the existing host-contract crate; this
milestone does not add a production crate merely to move those files.

## Phase Plan

### Phase 1: Adjudicated Host-Boundary Inventory and Topology

Freeze the migration surface and destination ownership before production edits.
Repository search seeds an inventory of subsystems and an edge matrix, but each
row is manually classified by semantic responsibility and disposition.

**Relevant subsystems**

- minimal sealed host-output envelopes and their generation values
- lane-local framework-turn completion and host presentation
- graph-mounted identity and mounted-receipt indexes
- direct preview paint and preview host traits
- solicited measurement observations and general host-shaped inputs
- application replacement, allocation publication, and current-plan activation
- egui and headless adapters, diagnostics, inspection, and certification

**Relevant APIs**

- `WorthUiHostOutputEnvelope`
- current lane-specific `execute_*_frame` and completion surfaces
- `UiMountedReceiptIdentity` and `UiMountedReceiptIndex`
- preview-paint facade and host implementation
- current `UiHostObservation`
- application replacement and current-frame publication facades

**Required artifacts**

- `milestone-3.10-host-boundary-inventory.csv`
- `milestone-3.10-boundary-edge-matrix.csv`
- `milestone-3.10-phases-1-8-authority-effect-inventory.csv`
- `milestone-3.10-test-cost-evidence.json`

The inventory classifies each subsystem as retained, refined, moved, replaced,
deleted, diagnostic-only, certification-only, or unrelated. The edge matrix
records source owner, artifact, authority carried, destination owner,
validation, weakened output, lifecycle, denial, cost, and eventual disposition.

**Warnings**

- Symbol search is discovery, not adjudication. Similar names may represent
  different authority; different names may implement the same bypass.
- Do not infer ownership from the current directory tree. This phase exists
  partly because current `graph/mounted_receipt`, `runtime/host_observation`,
  and preview paths obscure responsibility.
- Do not add tests that pass because a predecessor symbol or file path is
  absent. Runtime behavior and compiler-visible authority prove the cutover.
- Do not change production topology until all ambiguous inventory rows and
  edges have an explicit human disposition.

**Review evidence**

- Human adjudication must show that direct preview paint, lane-local
  presentation, and mounted-index construction form one boundary migration
  rather than three unrelated renames.
- Every existing supported host capability and every 3.9/3.9.2 publication
  truth maps to a destination owner or an explicit deletion; no behavior may
  disappear because search missed an alias.
- Equal digest-shaped generation values do not allow two edges with different
  session or lifecycle authority to receive the same disposition.
- Semantic surface, host surface, surface binding, prepared frame,
  presentation evidence, publication evidence, raw report, validated report,
  and solicited measurement evidence retain separate rows and owners even when
  their current representations are similar.
- The existing fast, hostile-certification, compile-contract, dependency,
  topology, boundary, and agent-context gates pass before production edits.
  Opening cost evidence records the current target/session counts and the
  clean, warm, link, fixture, execution, and external-startup posture used for
  the closing comparison.

**Test setup and proof ownership**

- Phase 1 captures the immutable opening source digest, topology counts, and
  lane measurements in `milestone-3.10-test-cost-evidence.json`; later phases
  compare against this exact baseline rather than rerunning an undocumented
  approximation.
- Inventory and edge adjudication are human review evidence, not a synthetic
  runtime world. Existing production and certification gates remain the
  independent control that the pre-migration repository is coherent.
- This phase adds no test target, fixture workspace, compiler session,
  external startup, retry, ignored case, or test-only production path.

**Engineering decisions**

- The two CSVs are implementation references and review evidence, not generated
  manifests consumed by production or tests.
- Phase 1 deliberately has review evidence rather than migration sentinel
  tests. Later runtime, model, adapter, and compiler evidence proves the
  adjudicated cutover at the boundaries that can actually falsify it.
- Boundary-check configuration and generated agent contexts remain the machine
  authority for crate dependency legality; the CSVs do not duplicate them.
- Phase 1 may correct later phase boundaries when adjudication reveals a
  misplaced owner, but may not hide unresolved rows under “follow-up.”

**Open questions**

- None.

### Phase 2: Mounted Instance, Surface, and Frame Identity

Introduce identities that distinguish semantic graph identity from mounted
incarnation and bind every receipt to its exact presentation world. Replace the
one-graph-node/one-mounted-receipt assumption with explicit zero-to-many
cardinality.

**Relevant subsystems**

- graph node identity and semantic repeated-instance basis
- mounted-instance lifecycle and indexes
- host session and native surface registration
- application, graph-world, plan, allocation, and frame generations
- replacement, reorder, virtualization, unmount, and remount

**Relevant APIs**

- `UiMountedInstanceIdentity`
- `UiMountIncarnation`
- `UiMountedNodeReceiptIdentity`
- `UiMountedFrameIdentity`
- `UiSemanticSurfaceIdentity`
- `UiHostSurfaceIdentity`
- `UiHostSurfaceBaselineReceipt`
- `UiSurfaceBindingGeneration`
- `UiHostSurfacePresentationMode`
- `UiMountedIdentityBasis`
- graph-node-to-mounted-instances and mounted-instance-to-current-receipt
  indexes

**Warnings**

- A graph node is semantic identity, not mounted identity. Reusing it directly
  aliases concurrent repeated instances and remounts.
- Repeated-item position, visible order, allocation coordinates, paint order,
  host widget ID, hash, and digest are forbidden identity inputs.
- Reorder without unmount must not mint a new incarnation. Actual unmount and
  later remount must not recover the old incarnation merely because the
  semantic repeated-instance basis matches.
- Printable identity is diagnostic representation only. Constructors accepting
  raw parts must not open current-frame or host-presentation authority.
- Semantic surface identity may outlive a native registration. Host surface
  identity may not outlive its host session. Rebinding the same semantic surface
  after native recreation, capability change, or device-scale change mints a
  new surface-binding generation.
- Mounted-instance identity is stable across frame generations and includes the
  semantic surface plus mount incarnation. Mounted-node-receipt identity is
  frame-scoped. Neither identity is the host surface identity.

**Test requirements**

- A convergence test must prove that two executions with equivalent semantic
  instance bases and uninterrupted mount lifetimes produce equivalent mounted
  identity behavior despite different visible order and allocation geometry.
- A denial test must prove that identical graph node IDs, repeated keys,
  digests, and visible output from different runtime worlds, semantic surfaces,
  or mount incarnations cannot satisfy each other's mounted-instance indexes.
  Different host sessions or surface-binding generations may preserve eligible
  semantic instance continuity, but cannot satisfy each other's frame-scoped
  receipt or publication checks.
- A virtualization test must prove zero, one, and multiple mounted instances
  for one graph node; reorder preserves live instances, while unmount/remount
  produces a distinct incarnation and retires the old receipt mapping.
- A surface-lifecycle test must deregister and recreate the host-surface
  registration for the same semantic surface through the production host
  contract. Semantic surface identity must remain eligible for continuity,
  while host surface identity, binding generation, frame identity, and every
  frame-scoped node receipt must change. Milestone 3.10 does not require an
  operating-system window lifecycle that belongs to the Milestone 4 shell.
- A baseline test must deny registration when a native surface cannot establish
  typed known-empty presentation truth. A successfully registered surface must
  provide the exact baseline used if first-frame presentation later becomes
  indeterminate.
- A bounded model trace must permute reorder, mount, unmount, remount, surface
  rebind, frame advance, and retirement operations. The independent identity
  model and production indexes must agree after every step, and the recorded
  trace must reproduce any disagreement without depending on generated raw
  identifiers.

**Test setup and proof ownership**

- The reusable `known_empty_surface_world` blueprint is established through
  real `.wui` acquisition, public application launch, semantic-surface creation,
  and successful registration through the production host contract. Each case
  obtains fresh session and binding state, receives only production-minted
  handles, and derives isolated reorder, mount, remount, and rebind deltas.
- The authored scenario states semantic continuity facts; the independent
  identity-transition model predicts trace outcomes without calling production
  identity, equality, index, or retirement logic.
- Cases remain child modules of the existing `application_contracts` owner.
  The ordinary fast lane may use the immutable constructed world without an
  operating-system watcher wait; real registration lifecycle evidence stays in
  the hostile-certification lane.
- Positive continuity controls are paired with foreign-world, stale-binding,
  and remount controls. A setup failure must be reported as world construction,
  never accepted as the expected identity denial.

**Engineering decisions**

- Semantic repeated-instance basis remains graph-owned. Mounting composes it
  with runtime-owned semantic surface, operating world, and mount incarnation.
  Frame-scoped receipt identity then composes the stable mounted instance with
  the exact mounted-frame identity.
- The canonical forward index is graph node to a bounded set of mounted
  instances in the current mounted closure. Reverse indexes name the exact graph
  node, semantic surface, current frame-scoped receipt, and frame membership;
  surface receipts separately name the native binding generation.
- Semantic-to-native surface bindings are generation-bearing runtime records.
  Host capability, device scale, coordinate posture, and native resource epoch
  belong to the binding generation rather than either stable surface identity.
- Identity retirement is explicit. Bounded historical inspection may retain
  terminal identity records, but retired identities cannot authorize current
  work.
- Identity certification consumes production-minted semantic handles from the
  canonical worlds. Literal IDs and digests appear only in explicitly invalid
  or corruption cases and never establish a valid mounted world.
- Identity state prepares move-only registration and deregistration candidates
  but performs no adapter effects. The host-truth coordinator reserves the
  attempt, classifies the adapter's three-way outcome, and only verified
  success commits identity and known-empty host truth.
- If an adapter reports success with a foreign or malformed receipt, the exact
  attempt becomes blocked indeterminate host truth. Registration cannot appear
  absent and deregistration cannot remain ordinarily live merely because
  receipt verification failed after the native call.

**Open questions**

- None.

### Phase 3: Mounted Node Projection and Compact Frame Storage

Lower UI-owned meaning into complete mounted-node receipts and specialized
compact tables. Keep semantic instance receipts distinct from high-volume paint
or spatial primitives.

**Relevant subsystems**

- execution-plan outputs from ordinary, virtualized, canvas, and realtime lanes
- allocation and coordinate-space evidence
- paint, clip, layer, input, focus, hit-test, accessibility, motion, and
  diagnostic projections
- compact frame storage, reuse, and inspection references
- Query-free and Query-backed UI-owned presentation consequences

**Relevant APIs**

- `UiMountedNodeReceipt`
- `UiMountedParticipation`
- `UiMountedAllocationProjection`
- `UiMountedPaintProjection`
- `UiMountedAccessibilityProjection`
- `UiMountedMotionProjection`
- `UiMountedDiagnosticProjection`
- `UiMountedResourceReference`
- `UiMountedResourceTable`
- specialized clip, layer, paint-batch, spatial-batch, and realtime-batch
  tables

**Warnings**

- A node receipt names semantic mounted participation. Do not allocate one
  receipt per canvas primitive, glyph, vertex, hit cell, or diagnostic row.
- A paint-batch count is cost evidence, not renderable paint meaning. Milestone
  3.10 carries only complete paint facts the current runtime already owns. If
  execution has not established exact runtime-owned geometry and the admitted
  appearance or immutable resource basis, projection carries typed omission
  and the adapter cannot advertise or report `NativePaint` for that work.
- Do not flatten distinct participation axes into one `interactive` or
  `visible` boolean. Rendering, input, focus, hit testing, accessibility,
  motion, and diagnostics have different meaning and denial.
- No participation axis is inferred from another at the host. Painted does not
  imply hit-testable; hit-testable does not imply focusable; focusable does not
  imply pointer participation; clipped does not imply unmounted; and diagnostic
  substitution does not inherit the denied node's interaction posture.
- Do not copy Query artifacts or host-native objects into mounted storage.
  Upstream Query inputs must already have become UI-owned consequences; native
  handles remain adapter-owned.
- Mounting may retain compact UI-owned provenance references, but it may not
  reopen `.wui` source, evaluate expressions, relower declarations, or inspect
  the active Query projection to reconstruct missing meaning.
- Do not predefine Milestone 3.15 service semantics or Milestone 3.16 appearance
  semantics. This phase carries their admitted projections only where current
  execution meaning already exists.
- Closing the missing primitive seam does not authorize a default color,
  role-derived style, synthetic unit rectangle, debug label, or adapter-local
  widget choice. Phase 3 may carry an existing resolved token/resource fact or
  explicitly omit appearance; Milestone 3.16 remains the owner of new
  appearance semantics.
- Coordinate spaces, transforms, clipping, and allocation bases must be
  explicit. Geometry without its basis is not a usable receipt.
- Milestone 3.10 projects allocation geometry exactly as the allocation owner
  established it: known, portal-relative, or typed omitted/unknown. It does not
  become an allocation solver. Viewport or portal-anchor observation is input
  evidence, not solved node paint bounds, and mounting, fixtures, headless
  translation, and egui may not solve, default, or invent layout.
- NaN, infinity, negative extent, empty box, offscreen allocation, DPI
  rounding, transformed-canvas coordinates, and portal coordinates require
  explicit canonicalization or typed denial before the host receives them.
- Clip and layer order are runtime output. Host call order, native widget
  nesting, or adapter-local maps cannot decide stacking, modal, overlay, or
  portal meaning.
- Resource references are frame-contract identities for UI-owned immutable
  resource entries, never native texture, font, widget, or GPU handles. An
  adapter may maintain a derived native-resource cache keyed by resource
  content identity plus surface-binding generation, and must be able to discard
  it without losing UI truth.
- Preview, diagnostic, authoritative, replacement-candidate, and certification
  worlds remain distinct identity bases; they are not flags that make otherwise
  interchangeable receipts safe.

**Test requirements**

- A parity test must show that equivalent admitted application, plan, and
  allocation truth produces equivalent mounted-node facts across the real
  Query-free path and the real Query-backed path after Query meaning has been
  translated into the same UI consequence. Expected facts are stated from the
  authored scenario contract; neither production path may generate the oracle.
- A leakage test must prove that Query keys, settlements, rows, patches,
  operational identities, and host-native widget or texture handles cannot be
  recovered from mounted receipts or compact tables.
- An audience test must mount two surfaces with different accessibility and
  diagnostic disclosure posture. Each adapter view must contain only its
  admitted surface facts, typed omissions must remain interpretable, and
  credentials or unrelated causal evidence must never enter either frame.
- A hostile canvas test must project a large primitive batch through a bounded
  number of semantic node receipts and specialized batch storage; receipt
  count must not grow one-for-one with primitive count.
- A denial test must reject a node projection whose allocation, coordinate
  basis, graph world, plan, or mount incarnation does not match the prepared
  frame basis while preserving predecessor mounted truth.
- A geometry test must cover non-finite and negative extents, empty and
  offscreen boxes, nested clips, overlays, portals, and differing device scale;
  adapters may round native pixels differently but must consume equivalent
  canonical coordinate and layer meaning.
- A participation test must deliberately disagree paint, clip, input, focus,
  hit-test, accessibility, motion, and diagnostic posture and prove that each
  axis reaches both adapters without host inference or accidental coupling.
- A resource test must recreate an adapter-native texture or font cache under a
  new surface-binding generation from the same immutable mounted resource entry;
  native handles must change while UI resource meaning remains equivalent.

**Test setup and proof ownership**

- Projection cases derive named application, Query-binding, geometry,
  participation, disclosure, and resource deltas from
  `known_empty_surface_world`; no fixture may mint a prepared node receipt or
  compact-table row directly.
- Projection cases derive `adapter_projection_world` through a real production
  framework turn and separately assert its exact admitted, deferred, withheld,
  and omitted mechanics before judging projection. Missing or unknown
  allocation is valid evidence for a typed omission or unsupported outcome; it
  is never evidence of successful native geometry.
- Geometry value-law cases may use public value constructors to test
  canonicalization and denial, but they may not claim that a production
  lifecycle emitted native-ready geometry. A positive native-geometry case is
  permitted only when the production world itself owns exact canonical bounds.
- No positive native-paint case is required until a production mounted
  projection owns complete geometry and appearance/resource meaning. Batch
  counts, touched rows, node roles, viewport extent, portal-anchor input,
  adapter surface size, or adapter defaults cannot satisfy that precondition.
- `authored_expectation` states adapter-neutral UI consequences from the
  scenario declaration. Query-free and Query-backed production paths are
  observations under test and may not contribute expected receipt contents.
- Deterministic canvas and geometry generators record their seed and semantic
  regime. Generated volume varies primitive topology without manufacturing
  mounted authority or turning one golden frame into the oracle.
- Evidence remains in the existing `application_contracts` compiled owner.
  Phase 3 observes public projected facts; native egui and headless translation
  claims remain reserved for Phase 7.

**Engineering decisions**

- Node receipts contain compact value facts or typed references into immutable
  frame-owned tables. References never outlive their frame generation.
- Solved allocation geometry remains allocation-owned committed truth.
  Milestone 3.10 preserves known, portal-relative, and omitted/unknown posture
  through mounted projection; it does not add layout policy or broaden egui
  into a solver.
- Specialized bulk tables expose only the operations their adapters need.
  There is no generic “payload” bag or downcast extension route.
- Resource entries are interned once per immutable content identity within a
  bounded retained frame family. Node and batch projections carry compact typed
  references; they do not clone strings, style blobs, accessibility subtrees,
  texture bytes, or causal evidence per node.
- Diagnostic projection is bounded and relevance-indexed. The complete
  diagnostic evidence graph remains owned by diagnostics and inspection.
- Provenance and future Query traceability use compact UI-owned evidence
  references. They do not retain Query operational artifacts or clone causal
  reports into every node.
- Receipt leakage is proven by the public sealed view, audience-specific
  translation, and compiler-visible access boundaries. A byte search, debug
  rendering, or assertion that a private field name disappeared is not proof
  that authority or confidential meaning cannot cross the boundary.

**Open questions**

- None.

### Phase 4: Atomic Cross-Lane, Multi-Surface Frame Assembly

Assemble every participating execution lane and native surface into one
prepared frame before any adapter call. Complete preparation is a prerequisite
for presentation, not a promise that later native effects are transactional.

**Relevant subsystems**

- active framework-turn execution and lane completion
- ordinary, virtualized, canvas, and realtime lane outputs
- semantic surface targeting and native surface registration
- cross-surface capability and device-scale admission
- empty, diagnostic-only, and bulk-only frames

**Relevant APIs**

- `UiMountedFrameRequest`
- `UiMountedFrameManifest`
- `UiMountedFrameCanonicalCore`
- `UiMountedFrameIntegrity`
- `UiRequiredLaneContribution`
- `UiMountedSurfaceBindingRequirement`
- `UiPreparedMountedFrame`
- `UiMountedSurfaceReceipt`
- `UiMountedFrameReceipt`
- `UiMountedLaneParticipation`
- `UiMountedFramePreparationDenial`
- internal lane contribution and frame assembler contracts

**Warnings**

- A lane completion is not a host payload. No lane may call the host or publish
  current mounted truth independently.
- The frame manifest is frozen from the admitted request and candidate plan
  before lane execution. “Complete” means every required lane/surface cell has
  either one admitted contribution or an explicit empty contribution, and every
  requested semantic surface resolves to one exact current surface-binding
  generation. Missing is not empty.
- Surface assembly may share immutable graph-derived facts, resources, and
  batch content across receipts, but mounted-instance identity and
  surface-specific geometry remain scoped to their semantic surface. Native
  binding multiplicity must not clone surface-independent content.
- Empty, diagnostic-only, canvas-only, and realtime-only frames are legitimate.
  Do not require an ordinary-lane node as a sentinel.
- A lane failure, stale contribution, or unsupported surface capability before
  presentation must preserve the complete predecessor frame on every surface.
- Capability preflight is generation-bound and must be revalidated by the
  adapter at the last effect-free boundary. A capability or device-scale change
  before effects returns rejection-before-effects; a change discovered only
  after effects begin is indeterminate.
- The application requests semantic surfaces and work; it does not manually
  choose lanes to make the frame complete.
- A missing or retired native binding is not an empty surface contribution.
  Preparation returns typed surface-rebind-required posture before effects and
  preserves the candidate for a later attempt after the mechanical binding
  owner resolves it.

**Test requirements**

- A parity test must prove that the same admitted meaning assembled from
  different legal lane completion orders produces canonically equivalent
  manifest membership and adapter-visible projection. Independently minted
  frame identities must remain distinct even when content is equivalent. Legal
  orders come from the deterministic assembly model rather than a hand-picked
  pair that leaves most cells untested.
- An atomicity test must fail one late lane or one surface capability admission
  and prove that no adapter receives any candidate contribution and every
  surface retains the predecessor complete frame.
- A mixed-frame test must assemble ordinary, virtualized, canvas, realtime,
  diagnostic-only, and empty surface contributions in one turn without
  inventing placeholder nodes or presenting any lane separately.
- A drift test must reject a contribution from a predecessor plan, allocation,
  application, graph world, surface registration, or frame request before host
  effects begin.
- An integrity test must corrupt manifest membership, a compact-table range,
  surface-binding basis, or required canonical-core field. Admission must deny
  before adapter access; omitting an optional sidecar must not change the
  operational outcome.
- A capability-race test must change one surface binding after preparation and
  before the effect boundary, then during presentation. The first case must
  reject before effects; the second must become indeterminate and block the
  affected binding.
- A surface-requirement test must add and remove semantic surface requirements
  across replacement while native bindings remain unchanged. Missing or
  surplus binding posture must be explicit; the frame may not silently omit,
  invent, close, or reinterpret a surface to obtain completeness.

**Test setup and proof ownership**

- Assembly tests extend `known_empty_surface_world` only through public
  projection and frame-assembly entry points. The fixture may carry prepared
  contributions returned by production, but it may not construct, repair, or
  publish a frame.
- The independent assembly model enumerates legal completion orders and
  authored manifest membership. Production canonicalization is the subject of
  comparison and cannot order the oracle.
- Late-lane, drift, corruption, and capability-race deltas begin from the same
  valid prepared basis. Assertions separately observe the typed denial,
  predecessor current tuple, and empty adapter-invocation transcript so an
  earlier setup failure cannot satisfy atomicity.
- Cases remain modules of the existing `application_contracts` owner and reuse
  one compiled world; they add neither an integration target nor a per-order
  fixture rebuild.

**Engineering decisions**

- Frame identity is minted only after all lane and surface contributions are
  complete and admitted against one basis. It is owner-generated lifecycle
  identity, never a content digest or canonicalization result.
- The manifest fixes the required surface set, lane cells, surface-binding
  generations, and capability generations. A prepared frame cannot silently
  shrink its manifest to turn missing or unsupported work into success.
- Integrity evidence covers the canonical core, manifest, and referenced table
  ranges. It detects corruption or mismatched assembly but grants no authority
  and makes no cryptographic claim unless a later transport contract explicitly
  requires one.
- Canonical ordering is semantic and deterministic where ordering affects
  rendering or evidence. Completion scheduling order is never observable
  identity.
- Surface receipts carry only surface-specific projections and references to
  shared frame-owned tables. They do not become independent current frames.
- The assembly model is an independent transition table over declared manifest
  cells and terminal classifications. It may share enum vocabulary with
  production but may not call the production assembler, completeness
  classifier, canonicalizer, or integrity checker.

**Open questions**

- None.

### Phase 5: Host Presentation State Machine and Reconciliation

Close the external-effect lifecycle from prepared frame through a typed host
outcome. This phase establishes what the runtime can know about native
presentation and how it recovers when that knowledge becomes indeterminate; it
does not publish candidate application truth.

**Relevant subsystems**

- host-session and surface-capability admission
- prepared, presenting, in-flight, presented, rejected-before-effects, and
  indeterminate states
- known host presentation truth and affected-binding reconciliation
- host lag, in-flight presentation, shutdown, and recovery

**Relevant APIs**

- `UiMountedPresentationAdmission`
- `UiMountedPresentationAttempt`
- `UiMountedPresentationInFlight`
- `UiMountedPresentationWitness`
- `UiMountedPresentationReceipt`
- `UiMountedPresentationOutcome`
- `UiPresentationDeadline`
- `UiPresentationIndeterminateReport`
- `UiHostPresentationReconciliation`
- `UiMountedSurfaceReconciliationBinding`
- `WorthUiActiveApplicationSession::present_current_mounted_frame_for_reconciliation`

**Warnings**

- Presentation does not mutate current application, plan, allocation, mounted
  indexes, or published frame. It emits external-effect evidence for the next
  authority owner.
- Phase 5 defines and certifies the presentation state machine, but no
  production caller may begin candidate native effects until Phase 6 has made
  the matching post-presentation publication transition total and infallible.
- Do not claim rollback after native effects may have begun. A partial or
  unknown multi-surface effect is `PresentationIndeterminate`, blocks ordinary
  publication, and requires explicit host reconciliation.
- Host callbacks may not reenter execution, replacement, mounting, or
  observation admission. Reports are returned or queued after presentation.
- An in-flight attempt owns a bounded presentation lease, exact surface set,
  deadline, and completion channel. Cancellation is rejection only while the
  host proves no effects began; otherwise cancellation or deadline expiry is
  indeterminate.
- Do not acknowledge a frame merely because an adapter method returned; the
  outcome must name which surfaces accepted, rejected before effects, or became
  indeterminate.
- The aggregate outcome is `RejectedBeforeEffects` only when every required
  binding proves zero candidate effects. After any required binding may have
  begun effects, any missing, rejected, cancelled, expired, or unknown sibling
  completion makes the aggregate attempt indeterminate.
- Indeterminate bindings admit no new normal presentations and no normal
  observation reports. Reconciliation must establish a fresh known
  presentation by fully presenting the current published frame—or the typed
  known-empty registration baseline when no frame has ever published—on a new
  or reset binding, deregister an affected candidate-only host surface that the
  current published manifest does not require, or terminate the host session.
  Pixel inspection and desired-output comparison cannot reconcile authority.
- Current-frame recovery accepts one complete typed set of affected-to-
  replacement binding generations. It must rebind and present the whole current
  frame through ordinary host admission, not acknowledge individual surfaces
  or mutate blocked state directly. Every blocked binding required by the
  current manifest must be covered exactly once; incomplete, duplicate,
  unrelated, or stale replacement sets deny before effects.

**Test requirements**

- A convergence test must complete the same required surfaces in different
  legal orders, including synchronous and in-flight completion, and produce the
  same aggregate presentation classification and canonical per-surface evidence
  while preserving distinct attempt identity.
- A denial test must force host rejection before effects and prove that every
  predecessor runtime artifact remains current and prior known host
  presentation truth remains unchanged while the candidate is retained only as
  bounded diagnostic evidence.
- An indeterminate-effects test must accept one native surface and lose the
  result of another; the runtime must publish no candidate current truth, must
  expose exact affected surfaces, and must require reconciliation rather than
  calling rollback or retrying invisibly.
- An in-flight test must exercise completion, effect-free cancellation,
  deadline-before-effects, and deadline-after-possible-effects. Only complete
  presentation may publish; the last case must become indeterminate. The test
  uses a deterministic clock and controlled completion through the production
  host contract; wall-clock sleeps, polling races, and widened timeouts are
  forbidden.
- A reconciliation test must block successor presentation and quarantine
  reports on affected bindings until a full current-frame re-presentation on a
  fresh binding establishes known host truth. Comparing pixels or replaying the
  failed candidate must open no recovery authority.
- A multi-surface reconciliation test must prove that one complete replacement
  set re-presents one complete current frame and atomically refreshes all
  covered binding generations while preserving its frame identity and
  application, graph, plan, and allocation generation.
- A candidate-only reconciliation test must prove that verified deregistration
  closes an affected binding absent from the current manifest without changing
  the predecessor publication tuple.
- An initial-frame reconciliation test must lose the first presentation before
  any frame has published and recover only by re-establishing the typed
  registration baseline on a reset or recreated binding before retrying.

**Test setup and proof ownership**

- `in_flight_presentation_world` is obtainable only after a real production
  frame has crossed presentation admission and the scripted host contract has
  accepted it. The fixture cannot assign coordinator state, attempt identity,
  or known-host truth directly.
- The scripted host controls only declared host outcomes and completion order;
  a deterministic clock and scheduler drive deadlines and cancellation.
  Runtime transition expectations come from the independent presentation
  model, not from coordinator predicates or adapter-return classifiers.
- The target-local `mounted_protocol_model/presentation_transition.rs` uses its
  own authored surface-start, completion, and cancellation vocabulary. It
  predicts pending cardinality, terminal presentation class, blocked posture,
  and publication eligibility without importing runtime outcome enums or
  calling production admission, completion, cancellation, or publication
  classifiers.
- Each rejection, cancellation, lost-result, and reconciliation case derives
  one isolated delta from the admitted world and observes typed outcome,
  consequential runtime state, known host state, and the exact effect
  transcript separately.
- These integration cases remain in `application_contracts`. They prove
  caller-side protocol and state-machine behavior only; Phase 7 retains the
  separate burden for real headless and egui translation.

**Engineering decisions**

- Presentation outcomes are exhaustive:
  `RejectedBeforeEffects`, `InFlight`, `Presented`, and
  `PresentationIndeterminate`. A presentation receipt is evidence for the next
  publication authority; it is not publication itself.
- Per-surface completions are subordinate evidence. Only the presentation
  coordinator may mint `UiMountedPresentationReceipt`, and only after every
  manifest-required binding reports complete success for the same attempt.
  Partial completion never escapes as a smaller successful frame.
- An indeterminate presentation is an operationally blocked state, not a
  successful frame with a warning. Recovery may re-establish host truth, but
  cannot infer that truth from desired output.
- Presentation admission requires a pre-reserved bounded in-flight lease. The
  lease pins the immutable prepared frame until complete presentation,
  rejection, or reconciliation closes the attempt.
- The runtime registers and owns every in-flight attempt before exposing a
  pending observation handle. Dropping that handle neither cancels nor abandons
  the attempt; cancellation is an explicit runtime request with the same
  partial-effect rules.
- Shutdown rejects new presentation and drains or cancels in-flight attempts.
  Any attempt lacking proof of zero effects or complete presentation is recorded
  indeterminate before the host session is invalidated; disposal never
  reclassifies or readmits its observations.
- Scripted-host scenarios implement the same versioned contract, admission,
  token, and completion obligations as real adapters and are checked by one
  shared contract suite. They prove coordinator behavior against controlled
  native outcomes, not that egui or another platform performed real effects.

**Open questions**

- None.

### Phase 6: Atomic Mounted Publication and Replacement

Consume complete-presentation evidence or exact unchanged-reuse evidence and
publish one coherent current runtime tuple. Ordinary execution and hot
replacement converge on this same commit owner.

**Relevant subsystems**

- complete presentation receipt consumption
- application, graph-world, plan, allocation, and mounted-frame publication
- current mounted indexes and predecessor retirement
- ordinary execution and hot application replacement
- exact unchanged-frame reuse

**Relevant APIs**

- `UiMountedFramePublicationCandidate`
- `UiMountedFramePublicationReceipt`
- `UiMountedFrameReuseWitness`
- `UiMountedFrameOutcome`
- `UiApplicationMountedReplacement`
- current mounted frame and index publication facade

**Warnings**

- Before the first native effect, reserve every allocation and capacity slot
  and prepare the complete publication tuple and index replacement. After a
  complete-presentation receipt exists, no allocation, validation, derivation,
  callback, policy branch, host call, or other fallible work may precede atomic
  publication.
- Presentation evidence and publication evidence are different artifacts.
  Holding one cannot reconstruct or counterfeit the other.
- A replacement candidate is not current merely because it was prepared or
  presented. Only the publication receipt names the new current application,
  graph world, plan, allocation, mounted indexes, and frame.
- `Unchanged` cannot mean “equal digest” or “nothing obvious changed.” It
  requires the exact runtime-minted reuse witness for the current published
  frame and exact current surface bindings.
- No replacement-only commit, mounted-index swap, or facade wrapper may publish
  a subset of the current tuple.

**Test requirements**

- A convergence test must prove that ordinary execution and hot replacement
  both publish through the same owner and end in one coherent tuple of
  application, graph world, plan, allocation, mounted indexes, and current
  frame.
- Publication-totality evidence must combine an infallible post-presentation
  API, exhaustive move-only phase types, controlled failure at every real
  pre-effect preparation boundary, and an independent publication-stage
  observer. Each injected failure must stop before adapter effects; after a
  complete presentation receipt exists, publication must perform no allocation,
  capacity check, validation, scheduler handoff, adapter call, or other
  fallible work. No behavioral test alone may claim to prove the absence of
  hidden post-presentation branches.
- A swapped-evidence denial test must exchange presentation receipts,
  publication candidates, reuse witnesses, frame identities, or surface-binding
  generations between otherwise equal worlds. Every mismatch must deny before
  effects or publication and preserve predecessor current truth.
- An unchanged-reuse test must prove that exact reuse mints no frame identity,
  performs no adapter call, and returns the existing publication receipt. A
  capability, device-scale, manifest, resource, or adapter-visible change must
  make the witness unavailable rather than turn into a late runtime denial.

**Test setup and proof ownership**

- `published_mounted_world` is constructed only by taking
  `in_flight_presentation_world` through a complete production presentation
  outcome and the real mounted-publication transition. No fixture writes the
  current application, plan, allocation, mounted index, or frame slots.
- Ordinary and replacement cases instantiate isolated successor deltas from
  the same proven published-world blueprint or checkpoint. The independent
  observer reads the public current tuple and presentation transcript; it does
  not reuse the publication candidate or replacement comparator as its oracle.
- Publication-totality evidence combines exhaustive phase types and reachable
  call-graph review with allocation observation scoped to the prepared
  post-presentation tail. Fixture allocation, logging, assertion formatting,
  and adapter work occur outside that measured tail; an allocation observed
  inside it is a product failure, not a threshold to widen.
- Positive publication and exact-reuse controls are paired with swapped,
  stale, and forced-preparation-failure controls in the existing
  `application_contracts` owner. They add no new target or alternate
  composition root.

**Engineering decisions**

- The production sequence is prepare all publication storage, preflight,
  present, consume complete-presentation evidence, then perform one infallible
  current-tuple swap. Process failure after native success but before the
  in-memory swap terminates the host session; no live session may continue with
  an unclassified gap.
- Publication outcomes are `Published` and `Unchanged`. Preparation denial,
  presentation rejection, in-flight work, retention denial, presentation
  indeterminacy, and `Reconciled` current-frame presentation remain distinct
  outer `UiMountedFrameOutcome` variants. `Reconciled` refreshes the current
  frame's host-binding and presentation evidence after full re-presentation; it
  does not mint a successor frame or change application, graph, plan, or
  allocation truth.
- The publication candidate is move-only and exact-basis-bound. A presentation
  receipt is consumed once, and duplicate publication is unrepresentable.
- Application, graph, plan, allocation, mounted indexes, and current frame
  change under one runtime publication authority. Inspection observes the
  resulting receipt after the swap and cannot participate in it.
- Failure controls use ordinary production preparation ports and allocation
  observation rather than a test-only publication branch. The final publication
  operation has no injectable fault point because its contract is total.

**Open questions**

- None.

### Phase 7: Host Contract and Adapter Cutover

Replace the minimal envelope and preview bypass with one sealed mounted-frame
contract. Make egui a real mechanical translator and make headless behavior
honest about whether pixels were presented or only frame evidence was recorded.

**Relevant subsystems**

- `worth-ui-host-contract`
- egui frame runner and rendering translation
- headless host and certification recording
- capability and surface reports
- direct preview paint and predecessor host-output envelopes
- public framework-turn facade

**Relevant APIs**

- sealed mounted-frame consumption view
- concrete `UiMountedPresentationWitness` carried through `worth-proof`
  transition outcomes
- `WorthUiHostAdapter`
- `WorthUiEguiAdapter`
- `WorthUiHeadlessRecorder`
- `UiHostProtocolIdentity`
- `UiHostProtocolVersion`
- `UiHostProtocolNegotiation`
- `UiMountedFrameSchemaVersion`
- `UiHostObservationSchemaVersion`
- `UiHostMeasurementSchemaVersion`
- `UiHostCapabilityReport`
- `UiHostCapabilityGeneration`
- `UiMountedPresentationOutcome`

**Warnings**

- The adapter receives no authored declaration, graph node object, Query
  artifact, active plan, allocation authority, or mutable runtime owner.
- Egui must render receipt facts rather than a summary label describing frame
  counts or digests. A debug summary is inspection, not presentation.
- Egui must not turn batch counts, mechanical roles, missing allocations, or
  omitted appearance into synthetic shapes. It advertises `NativePaint` only
  for the receipt schema it can execute completely and reports that effect only
  after actual native shapes were emitted.
- A 3.10 adapter may truthfully omit `NativePaint` and deny count-only or
  otherwise incomplete work. That does not permit it to classify an empty egui
  frame as painted.
- Headless recording must not report that pixels, native focus, or unsupported
  canvas/realtime effects were consumed. Capability and outcome must tell the
  truth.
- Presentation outcomes carry the binding's declared presentation mode and
  exact completed effect families. `RecordOnly` success can satisfy a
  record-only manifest cell but can never be substituted for a required
  `NativeDisplay` cell.
- Public raw-ID or digest constructors must not permit a caller to forge current
  frame, session, surface, or presentation authority.
- An adapter must revalidate the exact capability and surface-binding
  generations at its last effect-free boundary. A generic “egui supports this”
  or host-wide capability boolean cannot stand in for per-surface admission.
- Protocol compatibility is separate from feature capability. Unknown protocol
  identity, unsupported version, or incompatible receipt/report schema denies
  before effects with migration or upgrade posture; adapters may not best-effort
  decode or reinterpret fields.
- Preview and certification must traverse the same mounted projection and host
  contract as ordinary execution. Do not retain a convenience bypass.
- Adapter translation may optimize native mechanics but may not reinterpret
  visibility, disabledness, validity, layout, focus meaning, hit-test meaning,
  accessibility, motion, or diagnostics.

**Test requirements**

- A parity test must send equivalent real `.wui` applications through the
  ordinary egui adapter and the headless recorder. Expected adapter-neutral
  meaning is stated independently from the authored scenario, the headless
  oracle observes its post-translation mechanical transcript, and the egui
  oracle observes the exact native consequence of each effect family that the
  scenario admits. For an incomplete family, the egui oracle instead observes
  typed denial plus zero native effects. The test compares those consequences
  by semantic instance basis; comparing each adapter's input receipt to the
  other input receipt is self-certification. Host session, surface-binding,
  frame, presentation, and frame-scoped receipt identities must remain distinct
  rather than compare equal.
- A compiler-visible denial test must prove that application code and adapters
  cannot construct an admitted current frame, open internal mounted tables, or
  invoke presentation from raw IDs, digests, declarations, graph nodes, Query
  values, or predecessor envelopes.
- A cutover test must exercise preview, ordinary execution, and replacement
  through the same host contract and show no second paint callback or lane-local
  presentation in the runtime trace.
- A capability test must request unsupported headless canvas or realtime native
  effects and receive typed unsupported-before-effects evidence while a
  supported record-only mode names exactly what it recorded.
- A protocol test must cross current, explicitly compatible, too-old, too-new,
  and foreign protocol identities for mounted frames, presentation outcomes,
  observations, and measurement responses. Only declared compatibility may
  proceed, and incompatibility must deny before native effects or semantic
  intake.
- One shared host-contract conformance family applies every adapter-neutral
  admission, version, attempt-identity, terminal-outcome, and report-shape
  obligation to the scripted host, headless recorder, and egui adapter.
  Adapter-specific native effects retain separate real-boundary oracles.

**Test setup and proof ownership**

- Adapter cases begin with real `.wui` bytes and production-minted mounted
  frames derived from `known_empty_surface_world` or
  `published_mounted_world`; no adapter test constructs a privileged frame or
  imports runtime internals.
- Egui cases derive frames from `adapter_projection_world` (or a
  `published_mounted_world` proven to descend from it) and assert before
  `egui::Context::run` exactly which effects have complete frame-owned inputs.
  For a complete effect, the case inspects `FullOutput` or the relevant native
  access/resource output independently after translation. For an incomplete
  effect, it asserts typed unsupported-before-effects and unchanged native
  output. Adapter-owned trace records may diagnose the crossing but cannot
  replace either oracle.
- A native-paint setup must prove that every primitive under test resolves exact
  frame-owned geometry and appearance/resource meaning. Until such a production
  frame exists, Phase 7 proves the adapter's honest refusal and absence of
  synthetic shapes; an empty `FullOutput`, debug shape, or unsupported result
  can never be classified as paint success.
- The shared conformance family owns only adapter-neutral contract obligations.
  Headless correctness is observed from its post-translation mechanical
  transcript, while egui correctness is observed from actual admitted native
  consequences or, for unsupported mechanics, typed denial plus an independent
  zero-effect observation.
- `authored_expectation` supplies semantic consequences independently of both
  adapters. Neither the incoming frame receipt nor one adapter's transcript may
  be reformatted into the other's expected value.
- Conformance, real headless, and real egui cases are modules of existing
  compiled owners. Any platform-specific native startup or retained artifact is
  charged to its named hostile-certification cost lane, never the ordinary
  fast lane.
- Adapter-projection construction is one named fixture-cost component in that
  lane, with acquisition, launch, mounted preparation, adapter execution, and
  teardown timed separately. Measurement exchange and allocation commit are
  separately charged only when the scenario actually exercises them. Cases may
  reuse an immutable source blueprint or compiled artifact, but each live
  session and allocation authority is fresh; no shared mutable world or hidden
  per-case relaunch may distort either isolation or cost evidence.
- A Phase 7 preview case may establish splitter allocation authority through a
  real production replacement commit. That proves the host-path cutover only;
  it must not claim that synchronous launch minted allocation rows. Launch
  remains a proven zero-row commit until mounting and host measurement
  authority exist. Freshly mounted allocation establishment is certified by
  Phase 10 and may not be simulated with a no-op source edit or a forged
  `Preserve` transition.

**Engineering decisions**

- Contract values live in `worth-ui-host-contract`; authoritative construction,
  currentness, and publication remain runtime-owned and require concrete
  platform proof.
- Host protocol negotiation is one shared contract responsibility, but each
  artifact family retains its own schema version and compatibility window.
- Adapters are supplied a narrow borrowed or sealed view whose lifetime cannot
  escape synchronous translation. An asynchronous native completion may retain
  only a move-only in-flight token and adapter-owned native work; the runtime
  retains the immutable frame under the token's bounded lease.
- Host-session activation negotiates protocol and capability truth before
  acquiring one adapter-issued presentation lease. Every adapter call receives
  a view borrowed through that lease. Shutdown drains or classifies in-flight
  attempts before releasing it; deny-only adapters state that posture
  explicitly rather than inheriting permissive binding defaults.
- Polling a still-pending attempt returns its move-only completion token, while
  every terminal completion consumes it. No raw token constructor is part of
  the public adapter surface.
- The headless transcript is a canonical mechanical observation contract, not
  a serialized `UiMountedFrameReceipt`, debug dump, or copy of private mounted
  storage. It records exactly the mechanics the adapter consumed or explicitly
  declined.
- The old envelope, its forgeable generation constructor, direct preview host
  trait, and public lane-to-host completion methods are deleted once all real
  callers cross the canonical facade.

**Open questions**

- None.

### Phase 8: Bounded, Non-Reentrant Observation Reporting

Define the untrusted mechanical report boundary without prematurely admitting
reports into semantic UI meaning. Separate solicited measurement exchange from
general viewport, pointer, keyboard, focus, scroll, time, and tick reports.

**Relevant subsystems**

- current text-measurement request and observation evidence
- viewport, device scale, pointer, keyboard, focus, scroll, time, and tick
  mechanics, including text and IME composition
- host session, native surface, presented frame, mounted instance, sequence,
  and time bases
- report batching, coalescing, overflow, retention, and shutdown
- Milestone 3.12 observation intake seam

**Relevant APIs**

- `UiHostMeasurementRequest`
- `UiHostMeasurementObservation`
- `UiHostMeasurementDeadline`
- `UiHostMeasurementOutcome`
- `UiHostObservationReport`
- `UiHostObservationBatch`
- `UiHostObservationCanonicalCore`
- `UiHostObservationIntegrity`
- `UiValidatedHostObservationBatch`
- `UiHostObservationFamily`
- `UiHostObservationSequence`
- `UiHostObservationTimeBasis`
- `UiHostObservationFrameRelation`
- `UiHostObservationDisposition`
- `UiHostObservationReportValidation`

**Warnings**

- A report is not a semantic observation and cannot directly change focus,
  hover, scroll ownership, input state, allocation, application state, or
  rebind. Milestone 3.12 owns that admission.
- Observation delivery may not reenter an adapter call or framework turn. Hosts
  return a bounded batch after presentation or enqueue it through the named
  non-reentrant boundary.
- Coalescing policy differs by family and declared source contract. Pointer
  position, viewport, and ticks may admit latest-value coalescing only when
  their exact pointer/capture or tick contract permits it; keys, buttons, text
  composition, focus transitions, and discrete scroll deltas must not be
  silently dropped.
- Host wall-clock timestamps do not establish runtime order. Sequence and time
  bases must identify their owning session, surface, and presented-frame basis.
- Sequence is monotonic within one exact host-session/surface-binding
  observation source. Cross-family discrete transitions retain that source
  order; coalescing records the complete replaced sequence range rather than
  renumbering survivors.
- Overflow is typed evidence. Dropping, replacing, or backpressuring reports
  without naming the affected family and range is forbidden.
- Every batch canonical core carries protocol/schema identity, session and
  surface-binding basis, frame relation input, sequence range, integrity,
  overflow/coalescing disposition, and byte/count cost. Optional native detail
  cannot be necessary to interpret ordering or loss.
- Measurement results remain bound to exact requests and allocation or font
  basis; moving them into the general stream would weaken request-response
  authority.
- Measurement requests have independent count/byte budgets, deadlines,
  cancellation, and cleanup. Late or cancelled results cannot be relabeled as
  general observations or admitted against a successor allocation/font/surface
  basis.
- “Superseded” and “never presented” are not the same. A report based on a
  fully presented retained predecessor may be structurally valid and carries a
  `SupersededPresented` relation for Milestone 3.12 to interpret. A report for
  an unknown, rejected, expired, or never-fully-presented frame is denied at
  this boundary. Reports from indeterminate bindings are quarantined until
  reconciliation and never enter ordinary semantic admission.
- A referenced mounted instance or node receipt is validated against the exact
  retained presented frame index named by the report, never against the current
  mounted index. Retirement from the current frame does not rewrite historical
  mechanical provenance.

**Test requirements**

- An equivalence test must prove that legal batching and legal latest-value
  coalescing produces the same terminal mechanical state as the equivalent
  individual reports for coalescible families while preserving explicit
  replaced sequence ranges. Expected terminal state and replaced ranges come
  from the independent report-sequence model, not the production coalescer.
- A basis-classification test must pass current and retained fully presented
  predecessors with distinct frame relations, deny unknown, expired,
  rejected, and never-presented frames, and quarantine indeterminate bindings.
  A predecessor report may reference an instance retired from the current frame
  when it existed in the exact retained basis. None may schedule semantic work
  in Phase 8.
- A denial test must reject foreign-session, foreign-binding,
  out-of-range-instance, and post-shutdown reports and idempotently suppress an
  exact duplicate with typed duplicate evidence before semantic work or rebind.
- An integrity test must corrupt a sequence range, family payload length,
  surface-binding basis, frame basis, or required canonical-core field and deny
  the batch before coalescing or semantic scheduling.
- A loss test must overflow both a coalescible family and a lossless discrete
  family; the first must report the replaced range, while the second must
  backpressure or fail explicitly without silently losing transitions.
- An isolation test must flood one session, surface, and coalescible family and
  prove that its local quota cannot evict another surface's lossless transitions
  or consume the reserve required to classify them.
- A pointer-policy test may coalesce motion only for the same pointer identity,
  capture epoch, button posture, and presented-frame relation. Button, capture,
  gesture, text-composition, focus, and discrete scroll transitions must cut the
  coalescing range.
- A reentrancy test must attempt to report pointer, focus, and measurement data
  from inside adapter presentation and prove that no execution, replacement,
  publication, or semantic observation handler can run until the adapter call
  has completed.
- A measurement-lifecycle test must complete, cancel, expire, reorder, and
  duplicate real requests across font, allocation, host-session, and
  surface-binding changes. Only the exact live request may produce solicited
  measurement evidence, and all retained request state must remain bounded.

**Test setup and proof ownership**

- Report tests derive current, retained-predecessor, indeterminate, rebind,
  overflow, and shutdown deltas from `published_mounted_world`; fixtures may
  submit raw contract reports but may not insert validated reports or mutate
  semantic observation state.
- Measurement tests derive live requests from an active production session
  whose host capabilities, surface-binding generation, allocation revision,
  font revision, deadline, and request identity were established through the
  ordinary public lifecycle. A fixture may author measurement content and
  invalid foreign or stale controls, but it may not insert pending requests,
  terminal identities, observations, or budget accounting into the lifecycle
  owner.
- The independent report-sequence model owns batching, coalescing, replacement
  ranges, duplicate posture, and terminal mechanical state. It consumes the
  authored trace and public protocol vocabulary without calling production
  validation, sequencing, or coalescing.
- The reentrancy fixture gives an adapter only the production enqueue
  capabilities available during presentation. It records the in-call pointer,
  focus, and solicited-measurement return attempts, proves that the runtime
  session and semantic handlers are unreachable from those capabilities, and
  drains and classifies the queued inputs only after the adapter call has
  exited. An unchanged value, lack of panic, or elapsed-time assertion is not
  the oracle.
- Deterministic clocks, schedulers, queue capacities, and recorded seeds make
  action and observation phases reproducible. Reentrancy is observed through
  execution, replacement, publication, and semantic-handler transcripts rather
  than timing or absence of a panic.
- Each report and measurement scenario asserts its admitted world and exact
  live basis before applying one named duplicate, reorder, stale, foreign,
  overflow, cancellation, expiry, shutdown, or integrity delta, then observes
  both the typed outcome and retained-state consequences. Positive controls
  share the world but not the production classifier used as their oracle.
- Bounded trace families remain in existing certification targets. The
  ordinary lane uses small fixed traces; broader seeds and saturation occupy
  the scheduled lane without owning any otherwise-unproven correctness claim.
- Phase 8 evidence remains in child modules of the existing
  `application_contracts` compiled owner. Shared real-world construction and
  independent sequence modeling are reused through responsibility-named
  modules; the phase adds no Cargo target, fixture workspace, compiler session,
  external startup, retry, or retained artifact.

**Engineering decisions**

- `UiHostMeasurementObservation` is the renamed solicited evidence type.
  `UiHostObservationReport` is raw host input. The future admitted semantic type
  may use the concise `UiHostObservation` name under Milestone 3.12.
- Measurement admission owns a private dependency basis derived from the exact
  request family. It reuses ledger-issued allocation truth, active-session and
  binding truth, and admitted monotonic adapter-environment generations.
  Callers submit intent and later only the host observation plus current time;
  malformed or foreign input cannot consume the legitimate pending request.
- Real deadline, binding retirement, allocation drift, or adapter-environment
  drift terminalizes a pending measurement. Success yields sealed solicited
  measurement evidence and schedules no semantic work in this phase.
- Phase 8 validates structural shape, exact mechanical provenance, sequence,
  presentation-basis relation, retention eligibility, and bounds. It emits a
  validated but still semantically unadmitted batch and deliberately does not
  decide semantic consequence or rebind scope.
- Structural admission, exact sequence coverage, basis admission, and retention
  admission are distinct sealed stages. Exact coverage is linear in report
  count, consumes already ordered singleton report segments plus at most one
  declared loss interval, and uses checked successor arithmetic.
- Coalescing names the exact survivor identity and replaced range; overflow is
  retained as overflow rather than relabeled coalescing. Lossless-family
  overflow rejects, and reaching `u64::MAX` closes the source with
  `SequenceExhausted`.
- Report queues are partitioned by host session and surface with explicit
  family policies, local count/byte quotas, and a global count/byte budget.
  Global pressure is attributed and cannot silently transfer loss between
  partitions.
- Normal validated reports and indeterminate-binding quarantine evidence have
  separate named retention classes. Quarantine cannot consume the ordinary
  report budget or become semantic input after reconciliation.
- Bounded deterministic traces permute report family, sequence, duplication,
  reordering, partition pressure, frame relation, reconciliation, and shutdown.
  The recorded trace and seed are part of failure output; a larger scheduled
  trace corpus supplements but never replaces the bounded presubmit model
  proof.

**Open questions**

- None.

### Phase 9: Delta-Bounded Cost, Retention, and Inspection Evidence

Make initial, changed, unchanged, retained, and rejected work separately
observable. Prove that mounted truth remains compact and that inspection does
not become a second full frame store.

The cost vocabulary is:

- `m`: semantic mounted instances in an initial manifest;
- `c`: semantic mounted instances whose admitted projection changed;
- `i`: mounted and relevance index entries actually touched;
- `b`: rows and bytes in the smallest honestly replaced specialized batches;
- `u`: affected surface-instance pairs requiring distinct geometry, capability,
  or resource projection;
- `s`: surface-binding generations added, removed, or changed;
- `a`: adapter-visible rows and bytes translated for surfaces actually
  presented; and
- `r`: raw, validated, coalesced, denied, or quarantined report entries handled.

Initial mounting is bounded by named `m + i + b + u + s` work. Steady mounting
is bounded by named `c + i + b + u + s` work. Adapter and report work are
reported independently as `a` and `r`.

**Relevant subsystems**

- mounted projection reuse and invalidation
- specialized batch reuse and replacement granules
- reverse indexes and relevance indexes
- frame, table, diagnostic, and observation retention
- runtime counters, diagnostics, and inspection projections
- unchanged-frame suppression and host presentation reuse

**Relevant APIs**

- `UiMountCostReport`
- `UiMountWorkClass`
- `UiMountedFrameRetentionBudget`
- `UiMountedRetentionLease`
- `UiMountedRetentionReport`
- `UiMountedInspectionReceipt`
- `UiMountedFrameDelta`
- `UiMountedFrameReuse`
- `UiMountedFrameReuseContract`
- named counters for considered, minted, reused, retired, rejected, retained,
  presented, coalesced, and overflowed work

**Warnings**

- One aggregate “nodes processed” counter cannot prove delta-bounded work.
  Counters must distinguish graph consideration, semantic projection, batch
  work, index changes, adapter translation, and retention.
- Do not claim O(changed nodes) when a compact paint or spatial batch has a
  larger honest replacement granule. Name both `c` and `b`.
- Do not hide per-surface work inside `c`. Shared semantic projection is minted
  once; genuinely surface-specific projection is counted as `u`, and binding
  lifecycle work is counted as `s`.
- Inspection references retained evidence; it must not clone full receipts,
  batches, diagnostics, or observation queues per selected node.
- Retention is not “keep the last few” folklore. Current, in-flight,
  observation-basis, predecessor-inspection, diagnostic, quarantine, and future
  snapshot pins are separate classes with distinct eviction eligibility,
  leases, count limits, and byte limits.
- An unchanged semantic frame may still require host work because viewport,
  device scale, or native surface capability changed. Reuse classification must
  name which basis changed.
- The O(1) unchanged lane exists only when upstream phases carry an exact
  `UiMountedFrameReuseWitness`. Computing sameness by scanning manifests,
  receipts, tables, graph nodes, or surfaces is comparison work and may not be
  reported as unchanged reuse.
- Counter collection must not introduce allocations or broad scans on the hot
  path it measures.

**Test requirements**

- A cost-parity test must execute semantically equivalent Query-free and
  Query-backed UI frames and prove that mounting counters depend on UI-owned
  changed instances and batch granules rather than Query projection width or
  unrelated graph size.
- A bounded-delta test must change `c` mounted semantic instances in a graph of
  size `n`, touch `i` indexes, affect batch granule `b`, and reproject `u`
  surface-instance pairs across `s` changed bindings; after warmup, work must be
  bounded by those named terms rather than `n` or all `surfaces * nodes`.
- An unchanged test must execute an identical current frame and prove constant
  reuse through the carried witness, no manifest, surface, graph, declaration,
  receipt, or table scan, no receipt remint, and no adapter presentation. A
  control case without the witness must be classified and counted as comparison
  or reconstruction rather than unchanged.
- A retention-leak test must drive long-running frames, host lag, diagnostic
  pressure, and observation pressure past every configured budget and prove
  bounded memory ownership, typed backpressure or retirement, and no dangling
  current index.
- A retention-priority test must prove current and in-flight interpretation
  evidence cannot be evicted. Budget pressure must expire eligible inspection
  or observation leases, emit typed richness omission, or deny the next frame
  before effects rather than make an outstanding outcome uninterpretable.
- A 240 Hz arrival test must declare frame, surface, instance, batch, resource,
  and report distributions plus burst and host-service posture. It must prove
  exact structural counters and bounded queues under saturation; any latency or
  throughput conclusion must also report the required environment and
  percentile metadata. Correctness and boundedness use a deterministic arrival
  and service schedule, not a wall-clock sleep loop. A separate benchmark may
  measure elapsed latency but cannot become the sole correctness oracle.

**Test setup and proof ownership**

- Cost cases clone a proven checkpoint or reconstruct fresh isolated deltas
  from the `published_mounted_world` blueprint; warmup, baseline construction,
  delta construction, adapter translation, and assertion work are measured as
  separate layers.
- Workload declarations name every independent scale axis and semantic regime.
  Expected complexity is expressed through exact owner-emitted counters and
  hand-authored delta cardinalities, never elapsed time or production traversal
  output.
- Small deterministic boundary cases stay in the existing ordinary or
  hostile-certification owners. Environment-qualified percentile benchmarks
  and broad saturation traces occupy scheduled lanes and retain their workload,
  seed, hardware, runtime, cold-or-warm posture, and variance metadata.
- Closing cost evidence must distinguish compile, link, immutable-world,
  isolated-delta, execution, external-startup, retained-artifact, and retry
  cost. A regression in one layer cannot be hidden by a faster unrelated layer.

**Engineering decisions**

- Cost receipts distinguish initial mount, semantic delta, batch delta,
  surface-only reprojection, unchanged reuse, rejected preparation, rejected
  presentation, and indeterminate presentation.
- The reuse contract declares identity basis, canonical dependency order,
  comparator semantics, every invalidation source, and the stage that minted
  the witness. Adding an adapter-visible field or surface-binding input must
  break exhaustive reuse construction until its invalidation behavior is
  supplied.
- Adapter translation reports `a` separately from runtime mounting, including
  surface count, rows, bytes, native resource cache hits/misses, and any
  asynchronous handoff. Host-native effects are never inferred from runtime
  receipt counts.
- Counters are emitted by owning stages and composed into one frame cost report;
  no outer timer or inferred count substitutes for owner-reported work.
- Inspection uses identity- and relevance-indexed views over bounded retained
  artifacts. Milestone 3.11 may pin frames for snapshots through the reserved
  retention class without changing mounting ownership.
- Capacity for non-evictable current and in-flight classes is reserved before
  presentation. Inspection, observation, diagnostic, quarantine, and future
  snapshot leases cannot borrow that reserve.
- Cost evidence names compilation, linking, immutable-world construction,
  isolated delta construction, production external startup, execution,
  retained artifacts, and retries separately enough to explain a regression.
  Aggregate lane time remains useful but cannot hide an OS wait, repeated world
  compilation, or new compiler session.

**Open questions**

- None.

### Phase 10: Facade Closure, Documentation, and Real Certification

Close the public path and certify the actual lifecycle through existing
compiled test owners. Remove predecessor routes only after every ordinary,
preview, replacement, adapter, and inspection caller uses the mounted-frame
facade.

**Post-completion platform-pulse obligation**

Phase 10 completed the real-filesystem, watcher, mounted-publication, headless,
and egui mechanical lifecycle before the Platform Pulse requirement existed.
That evidence remains valid, but it does not prove a human-visible page: the
closed egui path deliberately rejects appearance-incomplete native paint and
the admitted no-effect case produces no native shapes.

Milestone 3.10.2 therefore owns the retroactive Platform Pulse seed. It must
extend—not replace—the completed Phase 10 lifecycle so one continuing
file-authored page visibly renders through the canonical host contract and
independent evidence binds the exact filesystem snapshot, mounted-frame
publication, complete static-paint mechanics, and post-translation adapter
observation. This is a new successor obligation, not a rewritten claim about
what Phase 10 had already shipped.

**Relevant subsystems**

- public Worth UI session and framework-turn facade
- real filesystem application loading and file-watcher replacement
- Query-free and Query-backed execution
- egui translation and headless recording
- mixed-lane and multi-surface presentation
- diagnostics, inspection, AI guidance, and lifecycle documentation
- existing application-contract and compile-contract certification targets

**Relevant APIs**

- `UiSession::execute_mounted_frame`
- `UiMountedFrameRequest`
- `UiMountedFrameOutcome`
- mounted inspection facade
- existing filesystem application lifecycle scenarios
- existing consolidated application and topology contract suites

**Warnings**

- Do not certify a filesystem lifecycle with in-memory strings, fake change
  notifications, or a host stub that never performs real file or adapter work.
  A deterministic scripted host is valid for coordinator fault injection, but
  that scenario is named integration evidence and cannot satisfy the real
  adapter or real lifecycle claim.
- Do not create one integration binary per phase or one compile-test crate per
  denial. Consolidate coherent evidence into existing owners and use compiler
  tests only where runtime proof cannot establish authority closure.
- Do not assert internal struct layout, private helper calls, debug text, or
  symbol absence as product evidence.
- Do not leave deprecated public wrappers that preserve lane selection,
  predecessor envelopes, preview paint, graph-node receipt identity, or
  synchronous observation reentry.
- Documentation must teach the final mounted mental model, not narrate the
  migration sequence.

**Test requirements**

- The completed real lifecycle test writes its `.wui` application to disk,
  loads it through the production filesystem path, executes and presents a
  multi-lane frame through a real adapter, edits the file, receives a real
  watcher event, replaces the application, and proves one coherent successor
  mounted generation. Milestone 3.10.2 must reuse this production path and add
  the missing complete native-paint and human-visible proof.
- Before that first edit, the same lifecycle must register and mount a
  file-authored splitter surface, establish its first allocation catalog only
  after mounted and host-measurement authority exist, and publish a preview
  through the mounted host contract. A dummy replacement, counterfeit
  predecessor, or capability-only resize marker cannot establish this
  pre-edit control.
- A cross-seam parity test must run equivalent Rust-authored and file-authored,
  Query-free and Query-backed applications through the same public mounted
  facade and compare UI-owned mounted facts by semantic basis, identity
  continuity behavior, publication transitions, and cost classes against the
  independently stated authored scenario contract rather than private
  representation, production-generated expectations, or cross-world identity
  equality.
- A hostile end-to-end denial test must combine reorder/remount, stale
  allocation evidence, one unsupported surface, duplicate/foreign observation
  reports, and a failed source edit; it must preserve the predecessor complete
  frame and provide causally ordered typed evidence at each rejecting owner.
  The journey resolves each denial before introducing the next named delta so
  an early failure cannot make later assertions green without reaching their
  intended boundaries.
- A real adapter test must render representative ordinary, virtualized, canvas,
  realtime, accessibility, motion, and diagnostic projections with egui or
  explicitly deny unsupported native effects; the headless recorder must
  independently verify its post-translation mechanical transcript against the
  authored expectation rather than echo the frame contract it received.
- A real-boundary recovery test must script a partial or lost multi-surface
  completion through the production host contract, prove the runtime/host truth
  split becomes explicit and blocked, and restore known presentation only by
  fully presenting the current published frame on fresh bindings.
- A compile-boundary test must prove only the smallest irreducible set of
  authority denials: downstream application code cannot mint a current mounted
  frame, and host code cannot obtain application, graph, Query, or publication
  authority from its sealed view.

**Test setup and proof ownership**

- The real lifecycle family creates an isolated temporary directory, writes
  actual `.wui` bytes, enters through production filesystem acquisition,
  receives operating-system watcher delivery, and tears the directory down
  independently. File writes, watcher startup, settlement, replacement, and
  teardown remain separately diagnosable.
- The lifecycle uses the production headless adapter for deterministic
  end-to-end mechanical observation and separate real egui cases for native
  translation. The scripted host appears only in recovery fault injection and
  cannot satisfy either real-adapter claim.
- Authored scenario expectations, public current-tuple inspection,
  post-translation adapter observations, and filesystem/watcher evidence are
  distinct oracles. The journey asserts each named precondition before applying
  its next delta so wrong-reason green outcomes are impossible.
- All runtime scenarios remain child modules of existing consolidated
  certification targets; irreducible public type denials remain in the existing
  two-session compile-contract owner. Phase 10 records closing topology and cost
  evidence before any increase is accepted.

**Engineering decisions**

- Real lifecycle evidence extends
  `worth-ui-certification/tests/suites/application_contracts.rs` and its
  reusable helpers. It does not create a new nested workspace or per-phase
  executable.
- Final certification is composed from three coherent compiled scenario
  families rather than one test per bullet: canonical authored-to-mounted
  convergence and replacement, hostile multi-surface denial and reconciliation,
  and bounded identity/report/cost traces. Each family may contain narrowly
  named cases, but its setup, independent oracle, mutation controls, and cost
  lane remain visible.
- Automated Platform Pulse proof remains a named scenario within that existing
  canonical authored-to-mounted family. Milestone 3.10.2 may add exactly one
  permanent downstream pulse executable because no human-runnable composition
  root currently exists; successor milestones must extend that same executable
  and the same compiled proof owner instead of adding milestone-specific
  binaries, targets, or fixtures.
- Compile-denial cases share one consolidated target and dependency build.
  Runtime denials remain runtime tests.
- Closing evidence must preserve the existing integration-target and
  two-session compiler budgets, keep zero flake retries, and explain any
  reviewed increase in link, fixture, execution, external-startup, or retained
  artifact cost rather than hiding it in aggregate wall time.
- `AI_README.md`, `docs/application-lifecycle.md`, host integration guidance,
  and inspection guidance are updated in the same phase.
- `docs/application-lifecycle.md` owns the continuing `Platform Pulse` run
  section. Milestone 3.10.2 must add the exact launch command, visible mounted
  result, source-edit action, and independently inspectable receipt/adapter
  evidence; successor milestones revise that same section rather than creating
  competing demo instructions.
- Phase 10 closes only after the inventory and edge matrix have no unresolved
  or compatibility-lane dispositions.

**Open questions**

- None.

## Must Ship

- `UiMountedNodeReceipt` and `UiMountedFrameReceipt`
- distinct semantic surface, host surface, surface-binding generation, mounted
  instance, mount incarnation, node-receipt, and frame identity
- typed known-empty host-surface registration baselines
- zero-to-many graph-node-to-mounted-instance indexes and exact reverse indexes
- specialized paint, clip, layer, allocation, input, focus, hit-test,
  accessibility, motion, diagnostic, canvas, and realtime projections
- one cross-lane, multi-surface frame assembler
- typed preparation, preflight, presentation, publication, rejection, and
  indeterminate outcomes
- explicit presentation receipts, publication receipts, exact unchanged-reuse
  witnesses, bounded in-flight presentation, and host reconciliation
- replacement integrated with mounted publication
- bounded frame, batch, diagnostic, inspection, and report retention
- one sealed proof-bound host contract
- real egui mechanical translation and honest headless recording
- a distinct solicited measurement exchange
- bounded generation-aware raw and structurally validated host observation
  reports for viewport, device scale, pointer, keyboard, focus, scroll, time,
  tick, text, and IME mechanics
- immutable UI resource tables and adapter-derived native resource caches
- named initial, delta, unchanged, surface-only, rejected, retained, and
  observation cost evidence
- one narrow public `execute_mounted_frame` path
- adjudicated subsystem inventory and boundary edge matrix
- canonical mounted-world fixtures, independent authored/model/adapter
  oracles, deterministic identity/presentation/report traces, and explicit
  proof-lane cost evidence in existing compiled owners
- real filesystem, watcher, adapter, replacement, mixed-lane, multi-surface,
  identity, denial, and cost certification in existing compiled owners

## Post-Completion Platform-Pulse Catch-Up

The Must Ship list above records what the completed milestone actually
delivered. The later-adopted human-visible requirement was specified and closed
as product capability by
[Milestone 3.10.2](./milestone-3.10.2.md). Its automated evidence was later
adjudicated honestly as in-process integration, so
[Milestone 3.10.3](./milestone-3.10.3.md) is the remaining mandatory
executable-world successor gate:

- one permanent human-visible Platform Pulse Page seeded by the completed real
  filesystem, watcher, mounted-publication, and adapter lifecycle;
- one complete runtime-owned static filled-rectangle paint path translated by
  egui without adapter-owned appearance meaning; and
- one durable downstream composition root, one consolidated automated
  integration path, and one exact-product executable-world path that later
  Milestones 3.11 through 3.23 extend cumulatively.

## Must Preserve

- the closed 3.9 through 3.9.2 application, graph, plan, allocation, Query
  binding, replacement, cleanup, and frame-cost truths
- `worth-ui-query-binding` as the sole production Query importer and the
  permanent exclusion of Query authority from hosts
- authored declaration and graph identity as upstream semantic truth rather
  than host or mounted identity
- predecessor complete truth on every pre-effect denial
- separate current runtime truth and known host presentation truth, with
  explicit blocked reconciliation on every indeterminate native effect
- Query-free applications without dummy Query ceremony or Query-derived cost
- semantic repeated-instance identity across reorder and distinct incarnation
  across actual remount
- host ownership of native mechanics without host ownership of UI meaning
- real filesystem and adapter boundaries in end-to-end claims
- consolidated compile and integration targets and existing iteration budgets
- zero flake retries, no wall-clock correctness oracle, no test-only production
  branch, and no scripted-host claim masquerading as real adapter evidence
- Milestone 3.11 ownership of visual snapshots and pixel-to-mounted identity
  bridges
- Milestone 3.12 ownership of semantic observation admission and hot rebind
- Milestone 3.13 ownership of broad Query projection-product consumption
- Milestone 3.14 ownership of interaction intent, 3.15 ownership of services,
  and 3.16 ownership of appearance meaning

## Acceptance Evidence

- one real file-authored application and one Rust-authored application reach the
  same canonical mounted facade and produce equivalent UI-owned receipt meaning
- one framework turn containing all four execution lanes and multiple native
  surfaces is classified only as complete presentation, rejection before any
  effects, bounded in-flight work, or explicit indeterminate presentation
- egui renders only the sealed mounted frame, and the headless adapter states
  exactly which mechanics it records rather than pretending to render; both
  are judged by post-translation observations independent of the runtime
  projector
- the host receives no authored declarations, graph authority, Query artifacts,
  active-plan authority, allocation authority, or mutable runtime owner
- equal printable values from foreign worlds, sessions, surfaces,
  applications, plans, allocations, frames, and incarnations cannot alias
- semantic surface continuity remains distinct from native surface recreation,
  capability generations, and frame-scoped receipt identity
- virtualization proves zero-to-many mounted cardinality, reorder stability,
  and remount distinction
- replacement publishes one coherent application/plan/allocation/mount tuple,
  and a failed candidate preserves predecessor truth
- complete host presentation is followed only by an infallible allocation-free
  runtime publication step
- pre-effect rejection preserves every surface; post-effect uncertainty becomes
  an explicit blocked indeterminate state
- host reports are bounded, non-reentrant, generation-aware, and incapable of
  semantic mutation before the Milestone 3.12 owner admits them; valid retained
  predecessor reports remain distinguishable from never-presented or
  indeterminate bases
- deterministic model traces agree with production identity, presentation,
  publication, and report invariants across recorded hostile operation
  sequences, while real-boundary scenarios separately prove filesystem,
  watcher, and adapter mechanics
- initial, changed, unchanged, surface-specific, adapter, report, retained, and
  rejected work is separately counter-backed; steady work scales with the named
  `c + i + b + u + s` axes rather than total graph or Query width
- preview, ordinary execution, replacement, certification, and inspection use
  no parallel paint or host-output route
- boundary, agent-context, line-cap, format, clippy, workspace, consolidated
  compile-contract, and certification gates pass without new build topology;
  comparable opening and closing evidence accounts for compilation, linking,
  fixture, execution, external-startup, retained-artifact, and retry costs

## Sequencing Notes

- Phase 1 precedes edits because current names and directories are not reliable
  authority maps.
- Phase 2 closes identity before receipts so projection cannot fossilize the
  current one-to-one graph-node model.
- Phase 3 defines semantic receipt granularity before frame assembly so bulk
  lanes do not force a generic per-primitive abstraction.
- Phase 4 closes cross-lane and cross-surface frame completeness before native
  effects.
- Phase 5 closes host presentation outcomes and reconciliation without claiming
  native transactional atomicity.
- Phase 6 makes complete presentation feed one total runtime publication and
  unifies ordinary execution with replacement.
- Phase 7 cuts over real hosts only after the final contract and publication
  authority are stable.
- Phase 8 closes the mechanical return boundary while deliberately leaving
  semantic admission to Milestone 3.12.
- Phase 9 proves that the completed lifecycle is delta-bounded and retained
  within named budgets.
- Phase 10 closes the public facade, documentation, predecessor routes, and real
  end-to-end evidence after the underlying authority is no longer in motion.

Milestone 3.10 is complete only when there is exactly one ordinary
runtime-to-host presentation path, exactly one bounded unsolicited mechanical
host-report path, and one distinct solicited measurement-response exchange,
with stronger semantic admission remaining on the runtime side of every return
boundary.
