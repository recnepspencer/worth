# Milestone 9.17.6.6: Composable Actions And Retained Operation Ownership

> **Status:** Complete — public API, covered consumer cutover and scoped qualification.
> Qualified source checkpoint: `4a93d8b3c59edf7dda3d6d0dd2963120ed1cde72`.
> Grounded at public master `0e2aae4233e0803ad8f86a630bff79f94662c03a`.
> Consumes the implemented owning
> operation capabilities of [9.17.6.4](./milestone-9.17.6.4.md), without claiming
> that predecessor's entire acceptance portfolio is complete. May proceed beside
> [9.17.6.5](./milestone-9.17.6.5.md); workflow attribution, disclosure and command
> extensions remain there. Proprietary adoption remains the separate successor
> gate below; this qualification does not close that work.

## Goal And Roadmap Placement

An application declares one complete action and retains one genuine operation
owner across commands. It must not choose between delivery and its other action
contracts, reconstruct native custody, or reorganize its whole state around a
borrow of the installed runtime merely to retain unfinished work.

Consumer experience is a design input to the public platform. The destination
is a clean replacement API, not a compatibility patch for a private application.
The public neutral consumer proves the capability; downstream application
cutovers demonstrate adoption separately and cannot stand in for public proof.

The baseline blocker was declaration composition. Feature authoring methods
constructed independent actions for locality/change, correspondence,
requirements, external inputs or delivery. Their capabilities could not all be
attached to the same occurrence through the supported audience API. The complete
action builder now supplies that supported composition.

The second change completes retained runtime ownership. Fixed and discovered
operations now share SourceOnly's authentic installed-runtime retention while
preserving each lane's distinct operation semantics and native failure topology.

This milestone does not add a second execution, recovery, workflow, permission,
currentness or topology owner. It does not close unfinished CAD geometry, native
UI or other predecessor portfolios. It does not wait for 9.17.6.5's unrelated
workflow features, reopen completed milestones, or retroactively gate 9.17.7.

## Baseline Boundary

Paths below are relative to `workspaces/worth-query/crates/`.

| Owner | Baseline fact | Destination |
| --- | --- | --- |
| Declaration `application_program/feature/authoring/capsule.rs` | `mutation_with_*`, `repeated_optional_member_*`, `fixed_output_action` and `operation_action` each create a fresh action. Feature fields are private and actions are read-only. | One typed action-authoring progression configures and appends the complete declaration once. |
| Declaration `application_program/action.rs`, `action/delivery_attachment.rs` | Canonical action meaning already includes delivery, correspondence, evaluated requirement, external input, locality/change and binding-derived workflow requirements. | Reuse this meaning and its private attachment owners; no second descriptor or registry. |
| Installation `application_program/action_resolution.rs` | Typed occurrence resolution checks schema binding, composition instance, feature and delivery; later resolution compares the complete installed declaration. | Preserve complete installed meaning and fresh resolution, including every composed contract. |
| Execution `application_installation/program/program_runtime.rs` under `domain_computation/primary_graph/` | `ProgramRuntime::clone` retains the same private Arc allocation without new installation or admission. Conditional-runtime close refuses retained keepalives. | Use this existing runtime ownership capability, without weakening close or retirement. |
| Publication `application_entry/operation/source_admission.rs` | Source-only admission retains `runtime.clone()` and the complete actual installed action. | The authentic keepalive pattern also governs fixed/discovered owners. |
| Publication `application_entry/operation/{fixed,settled,terminal,discovered}.rs` and children | Fixed/discovered operation families retain borrowed runtime/action fields, including through settled and terminal custody. | Retain runtime and installed meaning by value; native obligations remain move-only. |
| Publication `application_entry/operation/mode_construction.rs`, existing runtime configuration | Mode selection exposes runtime-borrow lifetimes; consuming keepalive refusal can drop configuration custody. | The construction borrow ends at return; dependent public contracts cut over together. |
| `worth-query-decl`, `worth-query-host`, feature macro and consumer fixtures | Public authoring recipes and operation types repeat displaced combinations or borrowing assumptions. | One supported current API, compiled examples, and required caller deletion. |

## Adversarial Courtroom

Use existing public consumer composition roots through `worth-query-decl` and
`worth-query-host`. Reuse production-issued programs, requests, source reads,
publication failures and cleanup owners. A hand-built result, copied receipt or
mock operation cannot prove this milestone.

| Claim | Hostile sequence and independent observation | Defect the proof must expose |
| --- | --- | --- |
| Complete action composition | Declare a fixed action with locality/change and meaningful correspondence or external-input/requirement contracts. Install it; inspect actual installed metadata and execute the owning path. Change a metadata contract in a real successor and attempt to reuse the retained old action. | A delivery builder silently drops another contract; an action reference caches only its binding/delivery; advancement accepts changed installed meaning. |
| Exact occurrence | Install two composition occurrences using the same binding and distinct legal fixed delivery/projection or locality meanings. Select each genuinely; try an incorrect occurrence/delivery and a duplicated occurrence. Rule/provider capability lookup is not claimed to be occurrence-isolated. | First-binding lookup, duplicate registration, or a selector that loses composition identity. |
| Storage beyond construction | Return a fixed/discovered owner from a function whose runtime borrow has ended. Retain a separate authentic runtime handle for fresh request construction; drop the constructing handle and advance the genuine owner through that same installed allocation. | Borrowed installed-action/runtime fields hidden behind a new return type, counterfeit static lifetimes, replacement runtime construction, or an operation runtime accessor used as admission. |
| Fixed partial effects | Interrupt at an existing post-source publication/retention fault, retain the returned operation, leave the initiating request's scope, then retry under fresh permission. Independently read source history and outputs. | Cloned obligations, repeated handler execution, loss of original preconditions/source/key, or receipt-to-owner reconstruction. |
| Discovered partial effects | Fail original discovered-output retention after a genuine source effect, then retry the actual returned carrier; independently check original membership and outputs. | Rediscovery replaces the original aggregate, a missing member becomes a green partial result, or the source effect repeats. |
| Authority and currentness | Supply a foreign runtime/branch request, revoke authority, change an upstream dependency, and attempt current delivery. Retry lawfully where native semantics permit. | Runtime keepalive promoted into permission, wrapper-address identity, stale current output, or lost custody on refusal. |
| Cleanup and runtime lifetime | Attempt runtime close and workflow-spec retention/support while operation custody exists; retain all refused inputs. Cause authentic cleanup refusal, then close lawfully, retry vocabulary binding and finally close the runtime. | Premature close, leaked keepalive, consuming configuration refusal dropping runtime/spec/signing custody, or cleanup refusal losing its usable runtime. |
| API construction | Try incompatible binding/delivery/provider/rule types, repeat a single-valued contract, call feature finish/provides on an unfinished action, obtain a full declaration from a selector, and import removed API families. Valid counterparts compile. | Mutable declaration escape hatches, silent overwrites, partial declarations presented as installed meaning, retired aliases or permissive generic marker authority. |

Use representative sequences that distinguish these failures. No Cartesian
product of every metadata combination, fault and output mode is required. Cost
claims cover actual added declaration work and runtime retention: attaching a
fixed number of contracts is local; retaining a runtime shares its existing
allocation and does not copy the graph or native obligations. Do not add new
meters or exact-count cost tests without a demonstrated need.

## Product Decisions

### One Complete Typed Action

The feature authoring parent owns Schema, Instance and Feature. An ordinary
`.action::<Binding, Delivery>()` selects the binding and delivery once and
returns a subordinate action builder. It does not append an incomplete action.
The builder supports:

- `.locality_and_change::<Scope, Shape>()` as the existing paired contract;
- `.correspondence::<Correspondence>()` with the current typed binding bound;
- `.evaluated_requirement::<Rule>()` with the current operation bound;
- `.external_input::<Provider>()` with the current operation bound;
- `.finish_action()` consuming the builder, appending exactly one complete
  declaration and returning its feature parent.

Each single-valued contract has a consuming, compiler-visible unconfigured to
configured transition. It cannot be set twice or silently overwritten. Keep
state mechanics sealed and inferable; callers must not manually name state
flags or supply runtime type identities. Do not generate every possible
combination or expose public mutable action fields.

Delivery mode derives the action's required-output-source flag. Ordinary callers
cannot override that derivation. A source-completion action for a binding with
required-output obligations is refused by actual owning construction through the
existing required-binding roster; it is not claimed to be compiler-denied.
Existing installed root/source validation remains mandatory. Binding-derived workflow requirements are preserved; this
milestone does not redefine 9.17.6.5's guarded requirement semantics.

The five-part `ApplicationActionReference` becomes only a typed selector. Replace
its `declaration()` method with `selector()` returning a sealed
`ApplicationActionSelector`: composition instance, feature, binding identity/type
and delivery meaning, without a public constructor or conversion to a complete
`ApplicationActionDeclaration`. Remove the lane-specific selector twins `ApplicationFixedAction` and
`ApplicationDiscoveredAction`, plus `resolve_fixed_action` and
`resolve_discovered_action`. Both use `ApplicationAction`; fixed/discovered
constructors constrain `Action::Delivery` by their actual respective delivery
trait and retain all mode/root/source checks. One selector resolution path
returns the complete actual installed declaration, and fresh progression compares
that complete meaning through `resolve_declared_action`. Never add metadata type
parameters to the reference or teach a partial declaration to pass full equality.

Existing `DuplicateAction` validation already refuses duplicate binding TypeIds
within an occurrence. Extend the same owner to also refuse two binding types with
the same portable `(instance, feature, binding identity)` resolution key, which
currently pass validation and later become ambiguous. Distinct legal composition
occurrences remain supported. Pure authoring creates no execution or authorization
proof. The subordinate action builder is `must_use`; it exposes neither feature
`finish()` nor `provides()` until `finish_action()` returns its parent.

### Legal Combinations And Scope

All subsets of the four optional metadata clauses are legal when their existing
trait bounds and program root/source validation admit them. In particular,
requirement-without-provider and provider-without-requirement are legal: the rule
has its own typed Context and does not intrinsically require external input.
Delivery may compose with correspondence, requirement and provider; installing
metadata alone does not evaluate a rule or admit an external revision.

Preserve the current lookup meanings explicitly. Delivery/locality/change belong
to the exact action occurrence. Correspondence capability lookup is binding-type
scoped. Evaluated-rule and provider capability lookup is operation-type plus
rule/provider-type scoped across declared actions; it is not occurrence-isolated
permission. Do not silently change these registries or promise a new automatic
policy gate. Each action's full metadata still participates in its canonical
meaning and in fresh full-action comparison.

The first combined consumer uses a real installed additional capability, not only
metadata presence: resolve external input, change its revision and observe its
actual admission refusal, then freshly resolve/admit it and use the accepted
value in an owning source action. Rule coverage uses the installed evaluator and
submission enforcement, accurately labeled as that existing capability rather
than an unbypassable native authorization rule. Source-only required-output
misclassification is the existing typed construction refusal. Wrong trait types
are compiler errors; invalid program root/occurrence combinations are validation
refusals. Tests must distinguish those boundaries.

### Explicit Native Declaration

Genuinely specialized native mutation declarations use
`.native_action::<Binding>()` with the same applicable metadata progression and
`.finish_action()`, without a delivery declaration or output-source toggle. Native
required-output membership is derived from the actual declared output roots. The
covered public baseline has no real caller of the old flag-only authoring helper;
downstream ordinary replacements acquire genuine delivery, whose output meaning
already sets the action flag. Do not create a new boolean/marker escape hatch.

Classify by the occurrence's actual consumers. An occurrence used by an owning
operation/adapter, a normal product mutation path, or ordinary front-door/guide
example must use `.action::<Binding, Delivery>()`. A legacy product caller that
manually drives source/recovery is still ordinary and must cut over. Native-only
classification requires a present specialized responsibility matching 9.17.6.4's
advanced table, such as native staging, authority/phase certification or explicit
branch/host control; absence of `retain_operation` is not justification. If both
ordinary and deliberate native callers use an occurrence, declare its real
ordinary delivery; the native caller keeps only its justified current controls.
Mechanical native translations preserve identical canonical meaning. Delete
`.mutation()` rather than retaining a second native spelling. Root-only
`.operation::<Operation>()` and conditional operation declarations are unchanged.

### Retained Runtime, Move-Only Operations

Fixed and discovered pending, refused, settled and terminal owners retain an
owned clone of the authentic `WorthQueryProgramApplicationRuntime` and the
complete resolved installed action declaration. This is shared keepalive custody
of the same installed allocation, not a second runtime, copied graph, new grant
or cloned operation. Keep the distinct native source, output, recovery and
cleanup carriers move-only.

Remove the construction-runtime lifetime from these returned operation types,
mode factories and their actual operation consumers. No baseline workflow
operation adapter names these fixed/discovered owners. Short-lived request, phase, observation
and borrowed result lifetimes remain meaningful and must not be erased.

Every advance, delivery and close still supplies fresh authorization. Compare
underlying runtime allocation identity and the actual original branch, not the
address of a cloned wrapper. Preserve freshly selected program support, complete
action comparison, original source/preconditions/key, complete-set currentness,
retirement, cancellation and resource refusals.

After verifying underlying runtime/branch affinity, open the phase through
`request.application`, whose borrow remains valid while `self` moves through the
closure. Do not add an Arc clone per advance/close when that verified borrow
already exists. A temporary genuine keepalive is allowed only at a boundary with
no verified request borrow and a demonstrated ownership need. Do not solve
borrow errors with a leak, fabricated static reference, unsafe code, copied
native obligation or substitute runtime. Runtime/action retention must survive
all refusal handoffs and actual operation consumers; no unrelated workflow
adapter work is assigned.

Operations expose no borrowed runtime accessor or `Deref` to runtime. Applications
bind initial workflow vocabulary and other unique-runtime configuration before
cloning runtime handles or retaining operations. They then retain their own
genuine runtime handle for fresh request construction; operation
keepalive is custody, not an admission entrance. Existing explicit consuming
SourceOnly cleanup handoff stays distinct from a general runtime accessor.
Arrange fields/destruction so runtime retention outlives bounded native-carrier
release, without adding cleanup-on-Drop semantics.

Runtime close continues to report `RuntimeKeepaliveRetained` while a real
operation retains the allocation. Successful operation close releases its own
keepalive; refused close returns the complete still-usable owner. Releasing
cleanup does not settle uncertain source/output work. Drop remains abandonment,
not successful external cleanup or crash durability. No new serialization,
cross-thread progression or durable restart claim is introduced.

Ordinary fixed terminal `close` also bridges the existing native negative
completion of a Required fixed leaf. The sealed output factory carries actual
leaf eligibility when an output refusal becomes terminal; applications do not
reconstruct root kind or choose per-tool disposal policy. Native
`finish_unavailable` remains the sole authority: original source validation,
terminal producer domain refusal, and absence of settlement, pending authority,
dependents and unresolved native cleanup are required. Close opens one phase
through the fresh verified request and preserves the same native output handle
on refusal. Once native negative completion succeeds, its actual unavailable
evidence is retained across any later cleanup refusal and returned on successful
close, without repeating native finish or source/producer execution. This means
source saved with no current output; it is never positive required-output
settlement. Preparation, partial, settled and nonleaf/discovered states gain no
negative-completion permission. The neutral ordinary assessment court covers
real producer rejection, fresh authority refusals, recovery-inspector cleanup
refusal/retry, no result publication/replay and lawful runtime close. Its new
execution and subsequent private adoption/Claude qualification are separate
from the previously accepted public evidence.

### Clean Replacement And Canonical Meaning

Consuming keepalive-sensitive configuration must also preserve all inputs on
refusal. `retain_workflow_spec(self, workflow, authentication)` returns a typed
`WorthQueryWorkflowRuntimeBindingRefusal<Schema, Spec, Program>` with `denial()`
and consuming `into_parts()` restoring the original runtime, installed spec,
signing owner and denial. `support_workflow_spec(&mut self, workflow)` returns
the original workflow spec with its typed denial; the borrowed runtime remains
owned by the caller. Apply this to every refusal variant, before or after the
keepalive check, without leaking signing material through Debug. This is repair
of existing runtime configuration custody, not new workflow-adapter/9.17.6.5 work.

No backwards compatibility is required. Remove covered mutation authoring
permutations, `fixed_output_action`, `operation_action`, their obsolete macro
arms, outdated lifetime signatures, aliases, wrappers and false examples after
their callers cut over. The deletion scope is executable/current consumer
surfaces; historical milestone descriptions and intentional negative misuse
fixtures may name retired APIs without supporting them. Ordinary product create/edit/retry cannot be preserved as manual
prepare/execute/start/recover loops moved under `advanced()`; retain only genuinely
specialized current native controls listed by the predecessor contract.

The existing feature macro has real public callers and downstream feature capsules;
its action/native-action blocks replace existing permutation arms rather than
creating a second DSL. It exposes action/native-action blocks with the same metadata
vocabulary and delegates to the typed builders. It must not implement a second
contract merger, independent validation policy or registry.

The block grammar is fixed for the implementation lanes: `action(Binding,
Delivery) { ... };` and `native_action(Binding) { ... };`. Block clauses are
`locality_and_change(Scope, Shape);`, `correspondence(Correspondence);`,
`evaluated_requirement(Rule);` and `external_input(Provider);`; native blocks
have no output-source override. An empty block is valid when no
optional contract applies. Metadata order does not change canonical meaning.
Ordinary delivery is never inferred from omission. Each block lowers through
one builder and one `finish_action()`; unsupported or repeated clauses cannot
silently disappear. Existing non-mutation feature members keep their semantics.

An authoring-only representation change preserves equivalent canonical action
meaning and program identity. Adding/removing/changing actual delivery or other
contracts changes canonical program meaning through the existing identity owner.
Do not bump unrelated formats to hide a caller break. If the owning encoded
contract genuinely changes, use current-format-only typed refusal before effects;
never reinterpret, erase or automatically convert the displaced bytes.

## Intended Caller Experience

The [application guide](../../docs/build-an-application.md) now binds compiled
examples for composed actions and retained fixed, discovered, SourceOnly and
workflow owners. The sketches below remain illustrative architectural examples;
`CreateGeometryDelivery` stands for real fixed-root/projection meaning.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. Target authoring API.
ApplicationFeatureSpec::root::<Schema, GeometrySourceFeature>()
    .action::<CreateGeometryBinding<Schema>, CreateGeometryDelivery>()
        .locality_and_change::<GeometryOccurrence, CreateGeometry>()
        .external_input::<GeometryInputProvider>()
        .finish_action()
    .finish()
```

Ordinary execution keeps the existing semantic entrance. The returned operation
can be stored after this construction borrow ends, while every attempt receives
a fresh request and every refusal returns actual custody.

<!-- illustrative -->
```rust,ignore
// Illustrative: not compiled. Existing entrance; retained result no longer borrows runtime.
let description = request.mutate(intent)
    .expect_source(&observed_source)
    .preconditions(original_conditions)
    .idempotency_owned(key)?;
let operation = description.retain_operation::<Program, CreateGeometryAction>(&runtime)?;
let progress = operation.advance(&fresh_request);
// Match typed progress/refusal/settlement; retain the actual returned owner.
```

No new host executor, polling authority, request clone per pump or manually
reconstructed native continuation belongs in the ordinary consumer.

## Destination Topology

`[existing]`, `[new]`, `[changed]`, `[removed]` describe implementation obligations,
not permission to create unpopulated placeholders.

```text
workspaces/worth-query/crates/
  worth-query-declaration/src/
    application_program/
      feature/authoring/
        capsule.rs                     [changed] feature membership, action entry
      action/
        authoring.rs                   [new] typed subordinate action progression
        authoring/
          contracts.rs                 [new] consuming contract attachments/states
        delivery_contract.rs           [existing] distinct delivery semantics
        reference.rs                   [changed] sealed selector; declaration/lane-specific selector twins removed
    application_program/program/definition_validation.rs
                                       [changed] existing TypeId + portable-key duplicate validation
    feature_spec_macro.rs              [changed] typed action blocks, old arms removed
  worth-query-installation/src/application_program/
    action_resolution.rs               [changed] sealed selector resolution; lane-specific selector twins removed
  worth-query-execution/src/domain_computation/primary_graph/
    application_installation/program/program_runtime.rs
                                       [existing] authentic keepalive/close owner
    application_installation/program/program_owner/selected_owner.rs
                                       [changed] unified selector resolution delegates
  worth-query-publication/src/application_entry/
    operation/
      fixed.rs, settled.rs, terminal.rs [changed] retained fixed custody
      discovered/custody.rs            [changed] retained discovered custody
      construction.rs, mode_construction.rs and mode_construction/discovered.rs
                                       [changed] no escaping construction borrow
      fixed_advance.rs, fixed_close.rs, close.rs, discovered/*
                                       [changed] fresh admission/delivery/cleanup handoffs
  worth-query-execution/src/domain_computation/primary_graph/
    application_installation/program/workflow_runtime.rs and workflow_runtime/support.rs
                                       [changed] preserve configuration inputs on refusal
  worth-query-decl/src/                [changed] audience exports and compiled authoring contract
  worth-query-host/src/                [changed] audience ownership/compiler contract
  worth-query-certification/fixtures/consumer_entry/
    consumer_root/src/application_program/ [changed] real combined action fixture
    consumer_root/src/application_invariant_acceptance/proof/application_program/
                                       [changed] existing owning lifecycle courts
workspaces/worth-query-bank-world/      [affected consumer cutover, no new owner]
docs/build-an-application.md, docs/capability-index.md
                                       [changed] continuing consumer guidance
```

Declaration owns meaning, Installation owns accepted program meaning, Execution
owns installed runtime lifetime, and Publication owns operation composition over
native carriers. Audience facades expose capabilities but implement no policy.
Contract states live with action authoring; runtime handles do not enter a pure
meaning substrate. New ordinary metadata clauses extend this stable action axis;
fixed/discovered/source-only successors stay in their distinct lifecycle owners.
No catch-all helper module, consumer metadata registry or private recovery engine
is permitted. Keep code/test files within the repository cap; generated context
files are regenerated only through their owning tools.

## Ordered Phases

### Phase 0: Freeze Contracts And Caller Ownership

Lock action/native-action signatures, consuming metadata transitions, macro
block syntax, required-output derivation, retained runtime semantics and owner
lifetimes before fan-out. Inventory actual public callers and classify ordinary
versus deliberate native responsibilities, including Bank and fixture roots.
Identify exact removed APIs and keepalive-sensitive configuration callers. Record this in the
implementation plan and code changes, not a new proof ledger or phase README.
The next phases may trust these signatures; changes to them coordinate once.

### Phase 1: Complete Action Authoring And First Installed Journey

Implement subordinate action authoring, constrained contracts and macro lowering;
cut over the first real fixed action and affected declaration callers. Phase 1
does not require the selector replacement: current fixed resolution already
returns the complete actual installed action after delivery checks. Reuse
existing installation and exact action-resolution owners. Prove installed
locality/change plus delivery, one further meaningful metadata contract, exact
occurrence and owning execution/current delivery/close. Initially retain the
existing borrowed fixed owner internally until Phase 2 lands; do not weaken its
native contracts to accelerate this journey. Old authoring helpers are not a
supported parallel API; mechanical remaining caller edits form the same cohort.

### Phase 2: Retained Fixed And Discovered Lifecycles

Retain the actual runtime and installed declaration through all fixed/discovered
states, mode construction, delivery, terminal and cleanup handoffs. The Lifecycle
lane also owns sealed selector definitions/resolution and removal of both
lane-specific selector traits/resolvers in the same signature edit, including
keepalive-sensitive runtime configuration. Remove displaced runtime lifetimes from returned
owners. Prove construction-borrow escape, post-source fixed recovery, discovered
retention retry, foreign-runtime/currentness refusal and close/cleanup lifetime.
This phase may proceed beside Phase 1 after Phase 0 signatures freeze.

### Phase 3: Joined Public Consumer Cutover

Integrate declaration and lifecycle batches, then convert all covered Query,
Bank, fixture, example, facade and documentation callers to the joined API.
Delete the displaced helper methods/macro arms and old lifetime forms. Verify
all meaningful metadata clauses through the actual same-action path and native
exceptions through their explicit declaration/entry. No old ordinary caller is
made green by automatically inserting `advanced()` or `native_action()`.
Public closure requires the supported consumer roots and mandatory enforcement,
not merely an owner crate check. Do not run unrelated suites after every edit.

### Phase 4: Certification And Downstream Handoff

Run the affected public courts and final required boundary, agent-context,
formatting, composition, facade and documentation checks on the joined result.
The public acceptance claim names its exact scope; old evidence remains tied to
its original contract. Publish a coherent certified revision through the normal
repository workflow and hand the current action/operation contracts downstream.

The associated proprietary adoption batch pins one real public revision, declares
owning delivery alongside existing action contracts, stores the actual returned
operation and removes displaced manual original-request/recovery composition.
Use genuine source-only/fixed/discovered modes and retain justified no-source
native output demands. Prove one ordinary author/edit/refusal/repair journey,
current result and current-format save/fresh-process reopen/further edit through
the actual application/UI owners. This is a named successor adoption gate
required by the user task, not part of the public milestone's certification gate
and not a claim that a public fixture certifies private CAD or closes unrelated
geometry milestones. Do not use local dependency overrides, copied authority,
receipt reconstruction or all-native fallback to avoid the cutover.

## Fan-Out And Integration

Use persistent implementers and independent reviewers. Start only the independent
Declaration and Lifecycle implementation lanes after Phase 0. Caller inventory
is preparation, not a separately staffed cutover lane while the APIs are absent.
Use the spare worker slot for scoped review when an owner has a concrete delta.
Resume the Consumer implementer only after both owner batches are implemented
and integrated; agreed signatures alone do not make that dependency ready.

| Lane | Owned production responsibility | Shared dependency / handoff |
| --- | --- | --- |
| Declaration | Action authoring, typed states, feature entry, macro, action-module/audience export wiring and declaration compiler evidence. | Phase 0 signatures; hands exact API and canonical meaning to consumers. Owns facade wiring exclusively, applying Lifecycle's supplied selector export delta. |
| Lifecycle | Selector definitions in `action/reference.rs`, resolution in Installation `action_resolution.rs` and Execution `selected_owner.rs`, both selector-twin removals; fixed/discovered owner bounds/lifetimes, mode factory, move/phase handoffs and keepalive-sensitive runtime configuration. | Lands selector and lifetime edits together in Phase 2. Supplies exact export changes to Declaration; does not edit that lane's facade wiring. Hands final owner signatures to consumers. |
| Consumer (dependency-gated) | After the concrete owner batches are integrated, Query/Bank fixtures, examples and authoritative docs. | Reuses the prepared caller inventory and actual implemented APIs; remains parked beforehand. Does not invent shims or change owner semantics. |
| Coordinator / review | Integrates coherent commits, owns all Cargo/native/oracle runs and routes scoped independent review. | One dependency-ready integration batch; no per-edit merge/rebuild loop. |

Respect available global agent slots; reviewers replace idle implementer slots
rather than silently exceeding the limit. No nested workers beyond that limit.
Authors run source/static checks, commit coherent batches and freeze affected
inputs before machine work. Keep sccache enabled, Cargo jobs uncapped and at
least 100 GiB free. Do not share a live target between simultaneous Cargo runs.

## Documentation Deliverables

- `docs/capability-index.md`: complete action composition, authentic retained
  operation lifetime, ordinary/native distinction, and current supported task rows.
- `docs/build-an-application.md`: compiled authoring example retaining metadata;
  operation storage beyond construction; fresh-request progression and lawful close.
- Query `docs/foundations/ordinary-application-front-door.md`: retained original
  meaning, authentic runtime, source/output/currentness/cleanup distinctions and
  explicit native controls; remove displaced authoring/lifetime recipes.
- Query host/decl READMEs, facade rustdoc and affected runtime/SourceOnly guidance:
  consistent current owner signatures and lifetime guarantees; no false implication
  that retain extends permission, makes Drop cleanup, or enables serialization.
- Existing compiled-example/doc tooling: bind new examples to real facade consumers;
  illustrative plan snippets remain visibly illustrative until implementation.
- `WORTH_query_roadmap.md`: sibling relationship to 9.17.6.5 and downstream adoption.

## Acceptance, QA And Test Cost

Architecture review must establish one canonical declaration and native lifecycle,
no overwritten/lost metadata, no copied recovery authority, and consistent runtime
retention through every owning handoff. Composition review checks the destination
axis and covered caller deletion. Evidence review checks real installed worlds,
independent source/history/output observations and intended-cause refusal.

Use focused owner/compiler checks for construction and sealed boundaries. Use
existing installed integration fixtures for metadata, exact action resolution,
partial publication, retained discovery, currentness and cleanup. Reuse a small
number of intentional test binaries: first a production/compiler frontier, then
one focused consumer build and exact discovered test filters. Build a selected
binary once per coherent dependency batch and run it directly. Add broader
Query/Bank checks only for affected shared contracts or mandatory closure gates.
No whole-workspace campaign after each edit, quota relaxation, budget-meter
project, generated proof ledger, source-authentication layer or recursive tests
for tests belongs in this milestone.

Required positive installed fixed/discovered/source-only cases must succeed;
refusal alone cannot satisfy them. Negative cases must exercise the intended
owner after valid setup and preserve exact returned custody. Report failed,
cancelled and unrun results honestly. Public compiler/runtime qualification,
private adoption and native/UI evidence remain separate claims.

Closure requires the joined public API, covered public callers, deletions,
documentation and scoped independent review with adequate executed evidence.
The downstream application consumes that result; future 9.17.6.5 workflow work
extends the same declarations and owners without reintroducing a second authority.

## Preview refusal custody prerequisite

The covered public preview lifecycle must return original custody on every
readmission, discard and promotion refusal. Bridge's execution-binding,
nonterminal-state, residue, proof and authority-basis checks precede any terminal
transition; errors therefore return the same active native session. Its owning
handle preserves the original runtime, execution record and liveness owner.
Query also retains one authentic installed ProgramRuntime clone, keeping its
conditional resources alive through native carrier destruction; no runtime
accessor or per-attempt clone is introduced. There is no partial cleanup phase to reconstruct, and no new cleanup engine.

Query readmission returns `(original_session, typed_denial)` without automatic
discard. A readmitted preview retains the complete original source/session;
promotion finalizes that original native session/execution proof without claiming
current Query source or publication. An authentic source action may commit after
readmission and before this separate preview evidence transition. Discard refusal returns the same phase owner. Stale, foreign and stopped
admission cannot refresh the source; explicit discard remains available. Drop
is abandonment and does not certify discard or promotion. Direct returning
certification errors also preserve actual preview custody.

Focused proof uses existing Bridge residue/proof/liveness courts and Query's
installed preview request-stop/fault courts: exact denial, unchanged original
owner/source/execution, lawful retry and exactly one real terminal posting.
These new hunks require Root compiler/runtime checks, scoped independent review
and final Claude qualification; historical public qualification below does not
qualify this added prerequisite or private adoption.

## Post-Cleanup Delivery Adoption Prerequisite

This bounded adoption prerequisite does not reopen or extend the historical
qualification checkpoint above. Its source is implemented; focused native and
compiler proof passed at their scoped revisions through `b196481079`. Final
review, publication/CI and private adoption remain separate qualification steps.
Root's actual existing consumer-root binary compiled (294); the original Fixed
and complete no-source Stable/no-receipt close/read journey passed (295), and the
pre-effect governed-artifact constructor denial passed (296). Earlier failed
289 target discovery and 290 fixture compilation remain failed attempts.
The final consumer binary compiled (301); the source-change and constructor
guards passed (302-source/302-guard), as did the Host family's four valid and nine
compile-fail contracts (303). The first selected-meaning attempt (302-meaning)
failed on an unavailable native retained basis; its corrected ordering and exact
refusal expectations compiled (308) and passed (309) at `b196481079`.

A settled Fixed parent can `close_for_delivery(&fresh_request)` into sealed
`WorthQueryClosedFixedDelivery`; a settled complete no-source parent can do the
same into `WorthQueryClosedProgramOutputDelivery`. Native cleanup and ALL original
read interests close first. The closed stage preserves the authentic original
complete settlement, root correspondence, installed meaning and SAME runtime,
plus actual native close evidence. It cannot advance business work, replay a
source/producer, reopen cleanup, clone custody or be minted from receipts/DTOs.
Plain `close` remains valid cleanup-only intent.

Each family has ONE owned delivery core. Existing live and closed methods use
identical native complete-output/discovery currentness and actual selected
program/action/root checks. Fixed declared query/batch selectors remain mandatory;
no-query delivery requires `ApplicationNoProjection`. A current check and its
read share the native basis and phase. Any refusal borrows the genuine CLOSED
stage; it never reconstructs a live parent or claims an old prepared view fresh.

Close checks original allocation/branch and fresh execution, independently of
selected-current source/program meaning: adoption or supersession cannot strand
cleanup. Fixed close refusal returns the SAME settled parent. No-source positive
close refusal retains a NON-OPTIONAL full settlement plus the actual remaining
native custody and any released prefix; only fresh `retry_close` is legal.
Native performed-source/aggregate completion and unavailable-leaf eligibility
are unchanged. No new cleanup engine, fault counter or disposal policy is added.

The no-source closed core retains only settlement/branch/runtime; original
extra basis/demand/controls are released after complete native close. Runtime is
last in the ENTIRE owning destruction order, after native custody/evidence in
outer parents/refusals and inside the shared core. Readonly carriers/observations
pin history and runtime; release promptly after final publication and before
whole-runtime close or OS join. Their presence is not uncleared cleanup and their
root closure does not certify other cohorts or the whole application closed.

The SDK's currentness claim is as of the actual fresh native observation and
covers the original complete tree plus installed context. Applications must
re-read additional source/model/membership/pose/context facts at that observation;
old frozen projections or descriptive commit equality are insufficient. CAD
meaning stays private and does not create an SDK projection-equivalence engine.

Fixed construction now invokes the EXISTING native
`validate_program_root_artifact_source` after installed action/delivery/root
checks and before cloning the runtime or minting an effect-capable owner. The
same immutable installed artifact/source guard already used for discovered
construction checks actual locality/change/succession/dependencies. Refusal returns
the original description with `ConstructionDenial::OutputSource`; the borrowed
runtime remains caller-owned and no source effect runs. Later preparation retains
its own authentic-context check; Legacy artifact posture and static definition
policy are unchanged. The actual native missing-locality/no-effects court guards
this timing repair without a new API, metadata mirror or checker family.

The Host compiler family covers construction/phase/shape and valid counterparts.
The new rostered-successor closed-delivery court covers action meaning
drift/removal and compatible meaning selected before capturing the operation.
Program adoption after settlement may make its retained basis terminally
unavailable (`RetainedBasisUnavailable`); restoring compatible meaning does not
resurrect it, and the application must run a fresh operation. The original settlement
moves whole, including discovery artifacts; its SAME native complete-membership
guard retains the existing discovery-court scope. No new membership schema/world,
root-kind coercion or repeated whole-milestone campaign is required absent a
concrete finding. Root alone executes compiler/native/enforcement/doc checks,
regenerates facade/context artifacts, and publishes after the SAME independent
Sol source/receipt review and separate Claude final qualification.

### Exact-observation closed delivery

The approved private cutover presents several closed carriers at ONE native
observation. Bare `request.at(&O).require_current_program_output` validates
lineage only. It omits Fixed's original installed-action comparison and
no-source selected-root eligibility. Each closed carrier's `deliver_current`
chose its own fresh observation, so no public path authorized a common `O`.

Both authentic closed carriers therefore gain `deliver_current_at(&retained,
policy) -> Result<(), WorthQueryOperationDeliveryDenial>`:

- closed no-projection Fixed;
- closed no-source.

It reuses the existing core gate at the caller's observation, which covers:

- runtime and branch affinity;
- the fresh scope;
- selection, with a foreign branch refused before installed meaning is read;
- the original action or selected typed root;
- the installed ceiling;
- complete original lineage.

The existing `deliver_current` delegates to the same split gate unchanged. There
is no new token, accessor, denial variant or replay.

A declared-query Fixed owner does not gain it. That owner delivers only through
its declared projection, and the existing Host E0599 fence already refuses its
no-query `deliver_current`. A unit gate at a caller `O` followed by an ordinary
query would let it replace its declared selector. A projected exact-observation
read would be new API with no consumer, so it is a known gap.

Proof runs in a genuine `NoProjectionProgram`-selected fixture world. The
no-projection Fixed owner and a no-source tree are settled and closed under the
same selected program. The pre-existing projected courts keep their
`read_current` assertions unchanged.

- **Exact-observation court:** a cancelled scope, a foreign-branch observation,
  and a common positive with a read at that same observation. After an actual
  consumed source edit through the selected program, typed Superseded.
- **Selected-action court:** rostered no-projection successors with identical
  meaning, changed declared contract, and removed action. It proves:
  - a compatible positive at `O`;
  - changed and removed actions are denied at `O`, and the owner's own
    `deliver_current` gives the same verdict;
  - an older `O` keeps its own changed-action verdict;
  - `RetainedBasisUnavailable` is reported at the restored `O` for both
    carriers.
- **Host fences:** the existing positive fences call it. A new `E0599` fence
  refuses it on a declared-query Fixed owner.

The existing successors all share one root, so a removed no-source root is not
constructible. Its eligibility at `O` is the same `validate_selected_output_root`
call that `deliver_current` makes. Root executes the focused proofs and gates.

## Public Qualification

The joined public API and covered Query, Bank, fixture, example, facade and
PlatformPulse consumers are qualified through focused compiler and installed
lifecycle courts, compiled documentation and real workflow/consumer execution.
Evidence retains its original source scope; this does not claim a freshly rerun
whole-workspace campaign. The final source passed constitution and agent-context
checks, formatting and the file cap, documentation debt and lockfile alignment.
Independent Claude CLI Opus 5.5 QA found no material defects at the checkpoint above.

The real WGPU offscreen Pulse journey resumes interrupted source and action work,
completes the same Intent request once, checks actual Query commit increments,
publishes the mounted consequence and closes with empty resource counts. It does
not establish native OS window/file-chooser or pixel-appearance qualification,
runtime before-effect Intent cancellation, or an executed `CloseIncomplete.retry`.
Those runtime paths were source-reviewed. CPU/binding evidence covers the
corresponding operation ownership and close contracts, rather than those entire
native journeys.

Query, Bank and binding owner Clippy checks passed. Whole-app scoped Clippy still
exits nonzero solely for the unchanged `native_application/theme.rs` collapsible-if
warning; broader UI dependency lint also retains untouched DSL debt. Neither is
reported as a passing command or absorbed into this milestone. Normal public
publication verifies the applicable repository CI. Private CAD/UI adoption and
the sibling or predecessor acceptance portfolios remain separate.
