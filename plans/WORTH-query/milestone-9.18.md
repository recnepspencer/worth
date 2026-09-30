# Milestone 9.18: Tree-Based Semantic Undo And Redo

> **Status:** Not started. Design reconciled with the current application entry,
> program adoption, workflow and owner-publication code on 2026-09-29.
> **Dependencies:** 9.17.4, 9.17.5, 9.17.6 and [9.17.7](./milestone-9.17.7.md)
> are completed. Phase 1 may begin
> independently; inbound completion and custody
> courts gate external-effect integration and final acceptance, not Phase 1 entry.
> This document specifies destination
> APIs; it does not claim that tree correction ships today.

## Goal And Roadmap Placement

Accept semantic correction as a new, freshly admitted operation on an exact
product branch. Reversal applies an installed inverse, compensation or reconciliation
of one exact committed occurrence. Reapplication executes its retained governed
meaning again under current authority. Each successful operation appends a new
single-parent composite commit. Original commits, intervening edits and alternative
correction lineages remain observable under retention and disclosure policy.

The source is historical evidence. The target is the current product world against
which effects are admitted. Neither a receipt, old approval, undo handle, derived
value nor program digest grants current execution authority. Domain meaning decides
what can be corrected; Query admits and coordinates it; Relational and Signal prepare
their own effects; Runtime World alone publishes product currentness.

Consume the exact bases and branch isolation of [9.17.1](./milestone-9.17.1.md),
[9.17.2](./milestone-9.17.2.md)'s composite publication/custody and
[9.17.3](./milestone-9.17.3.md)'s Query carriage. Extend
[9.17.4](./milestone-9.17.4.md)'s ordinary application entry,
[9.17.5](./milestone-9.17.5.md)'s branch-local supported programs and
[9.17.6](./milestone-9.17.6.md)'s workflow ownership. Preserve the accepted
[9.16 aftermath foundation](./milestone-9.16.md); replace its provisional correction
lane. Do not reopen completed predecessor portfolios.

[Cross-runtime merging and branching](../cross-runtime/merging-and-branching-roadmap.md)
adds merge, rebase, multiple parents, collaboration and distributed recovery later.
Correction here never grafts a historical component into a target world or silently
rebases a stale attempt. Process-local custody remains the acceptance boundary;
Store-backed reconstruction of correction history/custody is a successor contract.

## Current Boundary And Required Change

Paths below are repository-relative evidence of today's implementation.

| Existing boundary | What 9.18 consumes and changes |
| --- | --- |
| `worth-query-publication/src/application_entry/request.rs` under `workspaces/worth-query/crates/` | `request(&principal, &scope).on_branch(branch)`, typed mutation/query requests, retained observations and bounded `at_commit` already exist. Add correction here. Retained requests remain read/demand-only; `at_commit` is not correction admission. |
| `application_entry/mutation/{request,outcome,performed}.rs` | Typed source expectations, idempotency, `Committed`/`AlreadyCommitted`, uncommitted terminals and separate required-output custody exist. Corrections use the same ordinary admission, handler, invariant, publication and output return paths. |
| Declaration and installation `src/application_aftermath/` | Authority and mechanism are already separate axes; recorded inverse/pre-image, compensation, reconciliation and irreversibility have meaning. Extend these contracts with typed executable bindings and exact occurrence/program compatibility; do not create a second declaration registry. |
| Execution `domain_computation/application_aftermath/{undo_*,redo_*}.rs` | Provisional admission/progression and descriptive intent types exist. Redo binds a linear head and rejects divergence. Replace this lane and its policy-dependent tests; retain useful pre-image, custody, authority and bounded canonicalization guarantees. Source comments promising unchanged 9.18 reuse do not govern the new contract. |
| Execution `primary_graph` and `crates/worth-runtime-world/src/publication/` | Query already carries selected products; World already lowers explicit component plans and returns performed, no-effect or unpublished owner-effect custody. Extend these owners only where a required correction mechanism is absent. Do not build `worth-runtime-world/src/correction/` as a second publication engine. |
| `worth-query-host/src/facade.rs` | `provisional_aftermath` remains exported; it is not stable acceptance. Remove its executable undo/redo surface at cutover. Host/decl stay exports-only audience facades. |

Bank already routes ordinary operations through `bank-server`'s application definition
and installed mutation handlers. `bank-domain/src/schema/operations/reverse_journal_binding.rs`
and `bank-server/src/mutation_handlers/reverse_journal.rs` implement the business
operation `ReverseJournal`; this creates a linked compensating journal. It does not
establish a generic historical correction facade. Estate aftermath declarations
already distinguish freeze pre-images, disbursement compensation and external death
notice effects. Bank's generic provisional undo/redo HTTP routes were removed;
`bank-http-adapter/tests/protocol_boundary.rs` protects that cutover. Install new
accepted correction commands through the real server, HTTP adapter and user node.

The separate WORTH Proprietary repository contains actual CAD/House consumers through
`worth-query-decl` and `worth-query-host`: `HouseApplication`, `worth-cad-entry`,
`worthy-house-declarations` and `worthy-house-certification`. Its
`docs/house/query-platform.md` still labels `CorrectAuthoredChange` and
`ReapplyAuthoredChange` planned and requires capture/generation/identity gates before
shipping them. These are domain bindings for this platform correction contract, not
proof that an accepted correction API exists. Its WORTH dependency is pinned to an
older revision; cross-repository acceptance must build against the exact candidate
facade through a reviewed pin update or explicit local dependency override.

## Adversarial Courtroom

The plausible defective implementation reads an old value, runs today's mutation,
advances only Relational, and stores one redo item in a session. It passes a simple
undo/redo example while losing occurrence identity, external custody, branch meaning
and alternatives. The following courts must convict it.

### Composite source, target and concurrent publication

Use the existing public `application_graph` certification composition with real
Relational, Bridge, Signal and World owners. Install a small numeric source and a
required derived output with independently calculable P0/P1 formulas. Publish source
occurrence S, fork A/B from its exact composite basis and retain S for inspection.

1. Make a disjoint edit on A and an overlapping edit on B. Independently change B's
   supported program/Signal definition through ordinary program adoption. Keep A on
   P0. Use separately issued branches with equal-looking versions as negative twins.
2. Select S and reverse it on A. Preserve A's intervening edit and exact unchanged
   Signal definition basis; publish a new child of A's selected head. Required derived
   output must reconcile from the corrected source, not reuse S's cached value.
3. On B require typed conflict or incompatibility before effects. Also present foreign
   source/target/owner evidence, insufficient disclosure and revoked capability. Each
   denial has a lawful twin which reaches the disputed boundary and performs.
4. Prepare two valid corrections against the same A head. Pause one immediately
   before World's product comparison, publish the other and release the pause. Only
   one prepared attempt can advance that reference. The loser reports stale/no-effect
   or exact unpublished owner effects according to what actually occurred.
5. Cancel before owner preparation, after the first owner effect, and after World
   performs but before required-output delivery. Observe each boundary independently:
   no effects; unchanged product head plus retained owner custody; performed source
   plus pending delivery. Cancellation never converts the latter two into rollback.
6. Reapply from retained S meaning, reverse again, make another ordinary edit, then
   list both correction alternatives. Explicitly select an older eligible alternative;
   new input, old authority or a session redo cursor cannot substitute for it.
7. Drop history pages, correction handles and derived output caches. Reopen bounded
   inspection from canonical retained history; rebuild derived outputs. Original
   commits, correction links, input/pre-image provenance and mandatory custody survive.
   Expired optional correction material returns unavailable; it never guesses an inverse.

Observe actual source fields, computed numeric values, component bases, composite
parents, owner effects, outbox contacts and recovery catalog state. Expected values
come from a small independent semantic model, not production inverse/redo code.
Mutation of exact occurrence binding, fresh admission, target comparison, component
retention, output reconciliation or alternative retention must fail a named scenario.
Do not create a second proof ledger or tests for the tests.

### Bank: authenticated process, accounting and escaped effects

Extend `bank-courtroom/tests/transport_process_courtroom.rs` and its scenario modules.
Use `TransportProcessWorld`, separately authenticated user nodes, the actual TCP/HTTP
adapter/server and the rail process where external behavior is under test. A direct
runtime test cannot substitute for this product journey.

- Perform one lawful money movement, retain its occurrence, then perform an unrelated
  movement. Reverse the first through the new correction command bound to the installed
  journal compensation. Observe balances and linked original/compensating journals
  through ordinary account-summary/activity queries; compare with independent posting
  arithmetic. No journal is deleted, no unrelated movement is undone and balanced
  postings remain an invariant of each new journal.
- Repeat the exact correction after response loss with the same idempotency key:
  return its existing occurrence/custody and emit no second journal. Reuse the key
  with a different source, action or target branch: typed intent drift. Copy the wire
  selector to another authenticated principal: fresh denial, no protected disclosure.
- Add an overlapping change or revoke correction capability after inspection and
  before execution. The valid twin reaches ordinary correction admission; hostile
  twins return conflict/authority denial before new owner effects.
- Correct an estate freeze using its recorded pre-image and current eligibility;
  revoke or change the required authority independently. This distinguishes an exact
  local inverse from journal compensation and security restoration by copied data.
- After 9.17.7 closes, consume a real external callback for a dispatched effect, lose
  the response and then request correction. The performed effect remains completed.
  Only a separately installed compensation/reconciliation may proceed. Duplicate or
  late callbacks cannot undo completion, authorize redispatch or resurrect a cancelled
  workflow. Observe rail logical-effect counts and external-owner posture separately
  from local journal and product publication.

Keep `ReverseJournal` available as its own business operation. Historical correction
uses its installed meaning where applicable and adds exact source/correction linkage;
a user-supplied journal identifier alone cannot select or authorize a historical inverse.

### WORTH Proprietary: real authored source and regenerated geometry

Extend `worthy-house-certification/tests/journeys.rs` with `journeys/correction/` in
that repository. Start from real `HouseApplication` installation and ordinary CAD
source commands, using the existing extrusion/split, workflow and program-adoption
journeys for composition. Do not certify this with a substitute geometry provider or
`prepare_extrusion_source_for_certification` as the product entry.

- Create a rectangular extrusion and dependent split with actual units/tolerance.
  Change one source dimension; retain that exact edit occurrence. Make an unrelated
  feature edit. Reverse the dimension change through House's correction command,
  observe source parameters, actual solid volume and split volume conservation, and
  verify the unrelated edit survives. The oracle computes the rectangular volume from
  independent dimensions; it does not reuse production geometry or inverse helpers.
- Reapply the original dimension change and verify new source/geometry occurrences.
  Change the dimension again before reversal to expose the exact applicability conflict.
  P0/P1 branch adoption and a retained sibling exercise source interpretation versus
  current target rules. No old geometry receipt makes unsupported code callable.
- Evict derived geometry/scene views through their owner lifecycle and rebuild from
  the corrected source. Assert stable semantic correspondence and current output
  lineage, not equality of regenerated raw entity IDs or cached mesh bytes. A pending
  required output remains visibly pending; source success is not geometry completion.
- Use actual source retirement/membership changes for the deletion case. Physical
  identity resurrection is supported only by an owner-admitted restoration or explicit
  new-identity correspondence contract. If that binding is absent, installation denies
  reversibility for that family. It must not fail for the first time after a user edits.
- A local UI redo continuation may clear on a new edit. Reopen the explicit bounded
  history chooser and recover both retained alternatives. This reconciles the planned
  House session UX with Query's canonical tree: clearing selection never deletes history
  or correction evidence. House must consume the shared correction owner rather than
  co-commit a competing inverse/history system.

The first proof is a small real extrusion/split world, followed by a House population
with 1,000 unrelated source features while correcting one dependency closure. Count
producer contacts and source/history visits; unrelated population must not enter the
correction or output work. A native Studio shortcut/menu journey is required when that
UI enables Undo/Redo; the House command integration gate does not claim native UI or
restart-durable correction support. Existing Save/Open tests alone cannot certify that
new correction records and live recovery rights survive restart.

## Product Decision Lock

### Source, target and installed meaning

The exact source occurrence binds its composite publication, operation identity and
version, source program, governed input, pre-image capture, component meaning and
external-effect lineage. A copied receipt may locate that occurrence through fresh
owner-backed lookup; possession proves neither permission nor retained availability.
There is no deserialize-proof, public constructor from IDs or caller-supplied
`already_corrected`, `applicable` or `reversible` boolean.

This milestone reverses an occurrence in the selected target's retained ancestry,
including an ancestor inherited through a fork. It reapplies retained meaning selected
from that ancestry's correction lineage. A foreign sibling occurrence which is not in
that lineage requires future transplant/merge meaning and is denied. Tree alternatives
are explicit lineage selections, not permission to run any historical operation anywhere.
A repeated reversal of the same active correction edge is already-corrected, not a
second inverse; after a performed reapplication a new reversal names that new occurrence.
An alternative is consumed only in its exact target lineage, never process-globally.

Each correction appends to the selected target head; its source/correction relationship
is canonical causal metadata, not a second ancestry parent. Program compatibility is
checked between exact retained source meaning and the target's admitted current program.
Same-version-looking digests cannot establish owner compatibility. Unsupported old input
or inverse meaning returns a typed support/migration requirement before effects. No
implicit program rollback, new input substitution or API resurrection is permitted.

One operation declaration owns the capture and correction binding. Extend existing
`DeclaredApplicationAftermathContract` and installed aftermath resolution with typed
inverse/compensation/reconciliation/reapplication bindings to installed domain handlers,
input codecs, source expectations, pre-image and output contracts. Names currently used
as descriptive inverse references cannot become runtime dispatch by string. Installation
checks handler membership, schema/version support, bounded capture and legal external
posture; execution checks live target applicability and authority. Domain packages author
these bindings at entry-band composition; pure schema meaning remains Query-agnostic.

Required pre-images and retained governed inputs are captured with the original admitted
candidate from its actual changed footprint, co-retained with the performed occurrence.
A later read cannot recreate a missing pre-image. Uncorrectable operations remain valid
ordinary operations with explicit posture. There is no automatic field-diff inverse.
Historical capture stays truthful about unsupported create/delete/identity mechanisms.

### Component and derived-state posture

Every plan has an explicit Relational and Signal disposition: retain exact target basis,
prepare an owner inverse/compensation/reapplication successor, prepare an owner definition
successor, or deny. Retaining the historical source basis is allowed for inspection;
it does not switch a current component back to that basis. Signal values are derived
and never restored as authority. A definition inverse uses Signal owner authority and
compatible Bridge correspondence, just as ordinary definition/program publication does.

Authoritative source/component correction and derived settlement are distinct phases.
A source edit may retain the exact Signal definition basis while invalidating its derived
outputs. The performed-publication return path then reconciles the declared dependency
closure through existing required-output demand/settlement owners. A simultaneous
Relational and Signal definition correction must use one World component publication.
Signal recomputation is not an extra authoritative definition change.

World owns exact target reference/incarnation/head comparison, immutable composite
history and owner-effect custody. Query fixes semantic component dispositions before
owner effects and consumes existing prepared component/publication types. Bridge owns
correspondence, Relational owns truth/inverse application, Signal owns definitions and
derived lifecycle. No new head registry, general undo manager or correction scheduler.

### Compiler-visible progression and typed outcomes

Names in this table describe required new semantic products; constructors remain private
to their owners. Existing concrete proof, admission and publication carriers are reused.
Legality witnesses belong in `worth-proof`; portable descriptive vocabulary belongs in
`worth-foundational`; live handles/counters/Drop and issuance stay in their owning runtime.

| Product | Issuer, proof and consumer |
| --- | --- |
| Selected correction occurrence | Query fresh disclosed lookup over World history and retained owner material; proves exact available source meaning, grants no effects; consumed by correction preparation. |
| Inspected applicability | Bounded report of available mechanisms, conflict, compatibility and missing material; observation only; never accepted as a prepare/publish permit. |
| Prepared correction | Query ordinary fresh admission plus component owners; seals exact source, target head, current program, full observed dependency footprint, invariants, idempotency and reserved custody; consumed once by publication. |
| Performed correction | Constructed only from World's performed publication; exposes canonical source/target/causal receipt and separately held required-output/external obligations. |
| Unpublished correction | Existing World/Query custody records actual owner effects with no product movement; only existing owner recovery can settle/clean them. |
| Reapplication selection | Retained original governed meaning plus exact performed correction lineage; fresh preparation required; cannot hold replacement input or previous authority. |

Applicability reports distinguish ready-for-preparation, revalidation required,
component reconciliation required, conflicting divergence, incompatible program,
retained material unavailable and non-correctable/irreversible. These are inspection
facts. Preparation discharges every validation/reconciliation requirement or denies;
no effectful executor accepts unresolved applicability.

| Runtime event | Required result and effects |
| --- | --- |
| Undisclosed/foreign source or denied current authority | Typed selection/admission denial before effects; protected source facts remain omitted. |
| Relevant overlapping change or failed target invariant | Typed conflict/invariant denial; current state unchanged. Disjointness comes from installed dependency contracts and owner observations, including negative reads. |
| Target head moves after preparation | Stale prepared attempt; no automatic retry, rebase or target substitution. Any performed owner effects retain unpublished custody. |
| Budget/deadline/cancellation before effects | Typed no-effect terminal and released temporary reservations. |
| Owner effects exist; World did not perform | `ProductUnpublishedOwnerEffects` through the existing public recovery wrapper. No committed correction edge or product success. |
| World performs; output delivery/settlement interrupts | Performed source receipt plus exact pending/recovery obligation; never rerun the inverse. |
| Remote compensation result is unknown | Indeterminate external posture and owner recovery; no declaration that the external effect was reversed. |
| Same key, same source/action/target intent | Existing result/custody; no repeated inverse, dispatch or component publication. |
| Same key with changed source/action/branch or governed meaning | Typed idempotency intent drift before new effects. |

Use existing terminal families instead of flattening `ProductUnpublished`,
`SettlementDeferred`, denied, cancelled and committed states into a correction boolean.
A correction-specific enum may add meaning but must carry the original typed terminals.
Exact target head is preparation evidence. Retrying the same immutable intent after
response loss first resolves its idempotency result under fresh admission; it does not
turn a changed current head into changed command meaning or silently prepare again.

### Workflow, external effects and lifecycle

Correcting a workflow-produced operation corrects that occurrence only. It does not
rewind instance status, reuse approvals, rerun an entire workflow or erase completed
inbound consumption. The workflow owner derives the resulting continuation disposition
from performed publication and current contracts. Any new effectful workflow step needs
fresh transition/approval evidence. Missing continuation or mandatory-custody disposition
denies preparation. Current program adoption may remove ordinary APIs while retaining
exact-occurrence owner recovery for already-performed obligations.

External effects remain at their actual performed/unknown posture. Local journal
compensation is not reversal of a remote transfer. A separately admitted compensating
effect has its own occurrence and idempotency identity, linked to the original. Query
cannot promise remote exactly-once behavior beyond the installed external owner contract.

History pages and selections use existing bounded owner pins. Prepared corrections own
bounded reservations; dropping an uneffected preparation releases them. Dropping an
observer cannot abandon performed or unpublished mandatory custody. Optional input and
pre-image retention obey installed byte/age/disclosure/deletion policy. Expiry returns
unavailable and preserves permitted causal metadata/typed omissions; alternatives are
not promised to retain protected payloads forever. Branch/program retirement inventories
these users through existing owners. No session-local map is canonical history.

## Public Developer Experience

The stable entry remains `worth_query_host::facade::application_entry` and the existing
`WorthQueryApplicationRequestExt`. Declaration bindings remain behind `worth-query-decl`.
Add `corrections()` to an ordinary branch-bound request; do not add mutation to a retained
read request. Principal, purpose, deadlines and cancellation come from the existing typed
request scope. Domain intent, exact source, idempotency and bounded work are explicit.

This is the required new call shape, not currently compiling API. Convert it into a
compiled public example in Phase 1 and keep the example as the compatibility contract:

```rust,ignore
let request = application.request(&principal, &scope).on_branch(branch);
let selected = request.corrections()
    .select(&source_receipt, history_limits)?;
let preparation = request.corrections()
    .reverse(&selected)
    .idempotency(&correction_key)
    .limits(correction_limits)
    .prepare()?;
let outcome = preparation.publish();
// Exhaustively match performed, no-effect/denial and unpublished custody.
// A performed source may still own required-output or external recovery work.
```

The selected receipt is re-resolved under disclosure and owner provenance, not cast
into a proof. Preparation pins and compares the exact current target head internally;
callers cannot fabricate component plans. The default `reverse` chooses the single
installed mechanism. If a domain has semantically different corrective actions, they
are separate typed installed bindings, not a free-form mechanism flag.

```rust,ignore
let request = application.request(&fresh_principal, &fresh_scope).on_branch(branch);
let page = request.corrections().alternatives(&selected, page_limits)?;
let alternative = page.select(&alternative_ref)?; // descriptive exact selection
let outcome = request.corrections()
    .reapply(&alternative)
    .idempotency(&reapplication_key)
    .limits(correction_limits)
    .prepare()?
    .publish();
```

Reapplication has no input setter, principal-copy helper or implicit redo top.
A bounded history continuation is owner-bound, freshly disclosed and invalidated with
an explicit reason when retention/program support changes. Exhaustion returns a typed
stop/continuation; it is not evidence that no alternative exists. The ordinary path
exposes legal next actions and required output/recovery handles; richer explanation
remains opt-in. Transport DTOs carry selectors/outcomes and require re-admission; they
never serialize live authority.

## Destination Topology And Enforcement

Prefixes Q = `workspaces/worth-query/crates`, W = repository `crates`,
B = `workspaces/worth-query-bank-world/crates`. E extend existing, N create,
R replace/remove, S committed successor destination (no empty placeholder).

```text
Q/worth-query-declaration/src/application_aftermath/       E semantic authoring owner
  correction_mechanism/{recorded_inverse,compensation}.rs E existing mechanism meaning
  correction_binding/{mod,inverse,reapplication}.rs       N typed handler/input binding
  reconciliation.rs                                     E reconciliation declaration
Q/worth-query-installation/src/application_aftermath/      E installed meaning owner
  correction_mechanism/                                  E validated mechanisms
  correction_binding/{mod,validation,compatibility}.rs    N support/version resolution
Q/worth-query-execution/src/domain_computation/application_aftermath/
  correction/
    mod.rs                                              N private facade, exports only
    selection/{mod,occurrence,retention}.rs               N exact source and custody
    admission/{mod,applicability,program,authority}.rs    N fresh complete admission
    planning/{mod,components,dependencies}.rs            N fixed semantic plan
    reversal/{mod,preparation,progression}.rs             N inverse/compensation execution
    reapplication/{mod,preparation,progression}.rs        N retained meaning execution
    history/{mod,causality,alternatives}.rs               N owner-backed projection
  retained_preimage.rs, governed_input.rs                E original capture foundation
  recovery_handle/, recovery_progression/, external_effect/ E existing custody owners
  undo_*.rs, redo_*.rs                                   R provisional policy/progression
Q/worth-query-execution/src/domain_computation/primary_graph/
  product_operation/                                    E ordinary owner coordination
  workflow/                                             E performed correction disposition
Q/worth-query-publication/src/application_entry/
  request.rs                                            E corrections() entry
  correction/{mod,selection,preparation,outcome,alternatives}.rs N caller progression
Q/worth-query-publication/src/application_aftermath/       E disclosed correction projection
Q/worth-query-host/src/facade.rs                          E exports; remove provisional lane
Q/worth-query-decl/src/facade.rs                          E declaration exports only
W/worth-runtime-world/src/{history,publication,recovery,retention}/ E existing owners
  publication/component_plan/                            E exact retain/successor plans
  history/                                               S multi-parent history in cross-runtime
W/worth-relational/src/                                  E existing truth/candidate owners
W/worth-signal/src/                                      E existing branch/derived owners
Q/worth-query-certification/tests/application_graph/
  correction/{mod,branches,programs,custody,cost}.rs       N grouped public proof
Q/worth-query-certification/examples/application_correction.rs N compiling caller example
B/bank-domain/src/schema/                                E installed correction bindings
B/bank-server/src/correction/{mod,commands,outcome}.rs     N product orchestration
B/bank-http-adapter/src/http/{protocol,server}/           E correction DTOs/routes
B/bank-user-node/src/{session,server/routes}/             E authenticated correction entry
B/bank-courtroom/tests/transport_process_courtroom/
  correction.rs                                         N actual process journey

WORTH Proprietary repository, coordinated consumer phase only:
crates/worth-cad-entry/src/application/correction/         N CAD semantic bindings
crates/worthy-house-application/src/commands/correction.rs N House command mapping
crates/worthy-house-certification/tests/journeys/correction/ N source/geometry courts
```

Declaration classifies stable semantic meaning; installation resolves supported
implementations without mutable branch activation. Runtime selection/admission,
effect progression and history projection have different lifecycle/truth ownership
and remain separate. Query correction history reads canonical owner records and may
maintain discardable indexes; it cannot own another branch or correction truth table.
Canonical causal metadata is recorded with the performed operation in owner history;
a projection rebuild may not change applicability or lose a committed edge.

World's publication/history/recovery directories remain the cross-domain authority
axis. Any missing retain/successor or exact lineage access goes there through its
existing facade. Domain-specific inverse logic cannot enter World. Mechanism-neutral
component preparation stays reusable for future merge/rebase; those successors add
strategies/parent forms in their owners without moving correction's public facade.
9.19 adds managed access consumers beside correction and must preserve its footprints.
No generic `helpers`, flat correction mega-file, second runtime, per-domain authority
wrapper or product-owned component publication is permitted.

Enforce private constructors, existing audience/tier dependencies, no ordinary replay
imports, and exports-only facades. Compile-fail public cases cover forged selection,
report-as-permit, reused/mutated preparation, replacement reapplication input and
retained-read mutation, with lawful counterparts in the same grouped compile lane.
Runtime cases cover fresh but foreign/stale owner-issued evidence. Keep code/test files
within 400 lines; this specification grants no line-cap exemptions.

## Ordered Phases

### Phase 1: One exact source correction through the ordinary entry

Consume completed predecessor publication and custody contracts. Install typed inverse
bindings and original pre-image capture, implement exact ancestry selection, fresh
ordinary correction admission and one Relational successor with exact Signal retention.
Reach a new World commit and actual readback through the public request facade and
compiled example. Prove disjoint versus overlapping edits, copied/foreign selectors,
revoked authority and idempotency. Retained receipts alone cannot execute. This phase
establishes the smallest real vertical path; later phases trust its occurrence and
current-authority binding, not a future facade or private host plumbing.

### Phase 2: Composite compatibility, effects and recovery

Extend that path to owner definition successors, program compatibility, required-output
reconciliation, compensation and workflow/inbound dispositions. Use existing World
preparation/publication/recovery and existing output demand. Prove two-owner partial
effects, same-head races, cancellation at each boundary and no mixed world. Complete
the Bank accounting and external process court. No missing 9.17.7 capability may be
simulated and called completed. Later phases trust honest performed/unpublished/pending
posture and complete mandatory custody.

### Phase 3: Explicit tree reapplication and bounded history

Add retained-meaning reapplication, canonical causal edges, exact alternative selection,
branch-local consumed-edge rules, bounded pagination and lawful retention expiry. Prove
undo/edit/reapply divergence and handle/index destruction; remove linear-head policy
and migrate useful provisional tests together. Ordinary edits cannot clear alternatives.
This phase establishes the accepted platform product and enables full consumer adoption.

### Phase 4: House integration, cutover and public closure

Build Proprietary against the exact new facade, install CAD/House correction bindings,
and complete real source/geometry and 1,000-feature work courts. Revise House's planned
commands and session-continuation wording to match canonical history. Complete Bank
transport/user-node behavior and DTO compatibility. Remove executable provisional
exports, callers, linear-stack tests and misleading documentation together; retain
accepted aftermath/recovery. Ship documentation and run the required enforcement and
focused/integration gates below. No parallel product-owned correction engine remains.

Each phase adds its adversarial proof as the boundary becomes real; there is no final
phase where all correctness testing begins. Implementation plans choose private edit
order within these decisions, not new authority or product semantics.

## Cost And Resource Contract

- Ordinary operations incur zero correction selection, applicability, alternative
  traversal, inverse preparation or retry work. Existing installed pre-image/input
  capture and canonical causal metadata remain separately counted ordinary obligations;
  “zero correction work” cannot hide mandatory capture cost.
- Admission rejects invalid identity, missing support, authority and resource ceilings
  before expensive history/candidate construction. Correction work is bounded by selected
  source material, relevant intervening changes and the declared dependency closure.
  Required owner indexes belong to owner history/dependency access; a full-history scan
  is not an acceptable hidden fallback. Index absence/exhaustion yields an explicit
  bounded stop. Diagnostic reconstruction has a separately admitted cost lane.
- Bound selected history work, page entries, pre-image/input bytes, prepared candidates,
  live pins and pending output/recovery resources. No internal unbounded queue, implicit
  publication retry or silent truncation. Deadlines and cancellation are checked before
  effects and at existing owner progression safe points.
- Retained component bases cause no inverse, branch advance or rebuild by themselves.
  Derived work scales with semantic delta and declared dependencies; a dense fallback
  is explicit and budgeted. Count history visits, dependency checks, capture bytes,
  inverse construction, owner prepares, World comparisons, output contacts and dispatches.
- Compare 10 and 1,000 unrelated history/feature entries while holding the affected
  closure fixed. Assert unchanged relevant-work counts and exact zero untouched-owner
  mutation/dispatch contacts. Grow relevant ancestry separately to verify the admitted
  bound, continuation and exhaustion. Do not confuse output fan-out with identity work.

## Documentation Deliverables

Use existing authoritative feature guides under
`workspaces/worth-query/crates/worth-query/docs/`; do not create a competing milestone
closeout guide. The implementation phases must deliver:

| Audience | Document and required content |
| --- | --- |
| Application authors/operators | `execution/application-aftermath-and-recovery.md`: exact source/target selection, typed correction bindings, compensation, irreversibility, retained material expiry, custody and provisional removal. |
| Ordinary callers | `foundations/ordinary-application-front-door.md` and `foundations/branches-and-previews.md`: compiled request/correction examples, explicit alternatives, idempotency, bounded history, denial, source-versus-output completion. |
| Program/workflow authors | `foundations/programs-and-adoption.md` and `foundations/workflows.md`: source interpretation, target support, fresh approvals and non-rewindable performed effects. |
| Bank maintainers/users | `workspaces/worth-query-bank-world/docs/public-consumer-contract.md` and `docs/process-transport.md` in that workspace: new commands, authentication, response loss, journal compensation and external recovery. |
| House/Studio maintainers | Proprietary `docs/house/query-platform.md` and `docs/house/milestones.md`: M3B bindings to accepted Query correction, local selection versus canonical alternatives, identity restoration gates, real integration evidence and native UI scope. |

Compile snippets through the public certification example; check transport examples
against real process journeys. Keep planned syntax explicitly marked until it compiles.
Document mandatory recovery even when optional diagnostics/payload disclosure is omitted.
Cross-repository documentation and pin changes belong to Phase 4, not this design-only edit.

## Acceptance Evidence, QA And Handoff

9.18 closes only after all four phases, the composite public court, real Bank process
court and real Proprietary source/geometry court pass against the candidate APIs.
Platform certification without the private product build is platform evidence only;
missing access or a mismatched dependency pin leaves the product integration gate open.
Do not replace that gate with a generic CAD mock or claim existing House tests prove
new correction semantics.

Verification uses the existing grouped `application_graph` target in
`workspaces/worth-query/Cargo.toml`, affected owner tests, public compile tests/example,
Bank's grouped process target in `workspaces/worth-query-bank-world/Cargo.toml` and
Proprietary's grouped `worthy-house-certification` journeys target. Confirm target/test
names by discovery before expensive runs. Focused semantic/compile checks belong in CI;
process and population lanes use explicit bounded workload/resource budgets. Record
executed commands, candidate revisions and unavailable environments in normal CI/review
output, not a new source-controlled proof ledger.

Run `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`,
`cargo run --manifest-path tools/agent-context/Cargo.toml -- check`, affected formatting
and `scripts/ci/check_workspace_rust_line_caps.sh dirty` for implementation. A document-only
revision checks links, path claims and diff integrity; it does not claim runtime proof.

QA focuses on counterfeit source authority, stale target/program reuse, omitted negative
read conflicts, partial effects misreported as rollback, automatic external redispatch,
provisional residue, incomplete cross-repository adoption and correction work leaking
into ordinary requests. Review the independent observations and lawful negative twins
before expanding test volume. Accepted 9.16/9.17 recovery, disclosure, branch isolation,
workflow custody, cert-only replay and derived-state rebuildability remain mandatory.

[9.19](./milestone-9.19.md) receives exact source/target correction, complete dependency
footprints, new-history causality and honest resource/custody outcomes for its advanced
access consumers. Cross-runtime successors retain these semantics while adding merge,
rebase and durable recovery. Multi-parent correction, automatic conflict resolution,
remote/distributed rollback and restart serialization of live authority are not supplied
by this milestone.
