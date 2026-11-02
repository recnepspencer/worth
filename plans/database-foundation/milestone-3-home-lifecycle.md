# Application home and host lifecycle implementation

Specification: [Milestone 3](milestone-3.md), Phase 3, D1–D17; the Phase 4
handoff and shared roadmap requirements also bind the work.

## Required companion registration retirement prerequisite

An authentic PublicationCompanionRegistrationPort may consume its exact issued
registration through try_retire_required_for_request(registration, request).
This releases callback routing only; it grants no service, publication, read or
runtime-close authority and does not establish D11. The owner need not stop:
ordinary publication close closes settlements, not the companion registry.

Under the existing companion try_write gate, verify the original publication
owner identity and current active generation, then consult the original request
before replacing RequiredActive with RequiredRebind at the SAME generation.
The gate excludes actual publisher epochs and begin/activate. Contention,
foreign/superseded registrations and stopped requests return the same original
registration, without effects. Poison recovery preserves existing typed-state
semantics. Do not enter ordinary owner.admit(): last-operation release may lock
the close waiter and is not a bounded cleanup path.

The returned opaque RetiredPublicationCompanionRegistration keeps the removed
original participant/client and registration/port, with no callback or data
extraction surface. No removed payload, last registry/lifecycle Arc or client
control allocation is destroyed in the controlled step. Final Drop/reclaim is
explicitly unbudgeted. Existing remove_required remains its ordinary contract;
Bridge's later consuming port must use this authentic bounded result.

Place the production release and its local typed denial/retired owner under
mvcc/publication/companion/registration/retirement.rs. Its tests.rs child drives
actual owner-issued registration, original lock contention, request cancellation,
foreign owner, superseded generation and service stop. A participant Drop probe
must survive successful detach until retired-owner reclamation; the registry
must retain RequiredRebind at its original generation and refuse delivery.
Only that exact owner-private leaf may construct SDK ExecutionRequest and
SerialRequest roots. It uses the existing genuine Native owner fixture; it does
not add a runtime audience, fake participant authority or Home-close claim.

Implementation order: add the consuming release and explicit facade exports;
prove refusal/retry and deferred destruction against actual registry state;
run the focused owner tests, boundary/context guards, formatting and line caps;
then independently review the exact causal diff before integration. Full Bridge
observation/branch-head retirement, Native Directory close, Complete capture and
Host D11 remain separate unfinished obligations.

The Bridge canonical subscription adds a consuming
try_retire_for_request(self, request) adapter over that SAME Native registration
release. Success retains the returned Native retired owner and original Bridge
port in opaque RetiredRelationalBridgeCanonicalSubscription; refusal reconstructs
the SAME subscription from the returned original registration and its original
port. No callback, source/runtime keepalive, replacement allocation or authority
extraction is added. Preserve ordinary close(). The existing private
canonical_subscription/head_cell_tests.rs audience adds only
canonical_subscription_retirement_retries_original_publication_epoch_and_defers_consumer_drop
and canonical_subscription_retirement_after_owner_stop_defers_consumer_drop.
Drive actual Native publication contention and original owner stop; retain the
original consumer until explicit unbudgeted reclaim. Use the existing fixture
and SDK audience without new runtime roots. Place the adapter in the existing
canonical_subscription/retirement.rs child and export only its opaque retired
owner through the explicit Bridge facade chain. Query's later consuming source
cleanup must use this port; this alone does not retire branch-head or observation
registrations or establish full D11.

## Retained Native observation custody retirement prerequisite

RelationalRetainedObservation::into_retirement consumes the actual issued value
into a non-Clone RelationalRetainedObservationRetirement. advance(&mut self,
&ExecutionRequest) reports the owner-specific Advanced, Busy,
Stopped(WorkCeilingDenial), Refused(typed cause), or Complete progress. Refusal
causes describe actual owner unavailability or registration/accounting mismatch;
no generic cleanup registry is introduced. into_retired returns either an
opaque RetiredRelationalRetainedObservation or the same unfinished task.
Final reclaim/abandonment is unbudgeted. Completed tasks report Complete without
another effect or request poll; live steps consult the original request before
effects, without retaining the request or issuing a replacement.

The actual value owns an ExternalComponentBasis lease and TWO observation
shares. Those share an admitted inner with its own Observation obligation and
basis-registry lease. Removing only the external lease is insufficient. Retire
under the original owner's external_cut then original lane try gates, matching
existing acquisition/capture order. Validate actual obligation identity, kind,
roots and accounting, update existing multiplicities/counters once, disarm the
original guard and retain the removed original record before unlocking. Preserve
its root_ids, original guard roots, lane and upgraded original owner Arc.

Consume the two original observation shares with Arc::into_inner. A nonfinal
share releases only its local custody; an independent alias remains operational
and continues vetoing owner close. For the unique actual admitted inner, retire
its own Observation obligation with the same retained-record guarantee, then
its exact basis-registry registration. Keep the original removed registry key,
entry and upgraded registry/metrics Arcs; mark the original registration retired
at removal. Moving the unique inner makes its registry Weak non-upgradeable:
concurrent readmission may prune that dead entry and decrement metrics, or a new
registration may replace it before this stage. Absence or a different original
registration ID completes this lease idempotently, without removing a successor
or decrementing again. Preserve the ordinary exact-ID conditional accounting.
When ordinary registration replaces an expired original entry, its existing
metrics must record that actual removal and insertion, so the live-entry count
stays equal to real rows. This is causal to the newly exposed serving-owner
handoff; use the existing counters, with no extra accounting framework.
No root-vector clear, descriptor clone, replacement ownership
allocation, ordinary runtime admission or last upgraded-owner destruction occurs
inside the controlled step, including Busy/stop/refusal returns. Each committed
stage is recorded before the next poll, so retry never repeats terminal effects.

Place the public task under change_source/retained_observation/retirement.rs;
keep retaining lane and basis-registry mechanics under their existing owners.
Reuse or strengthen the existing private observation retirement where lawful;
do not add an ownership mode or general lifecycle framework. Keep all original
payloads and grants in opaque inert custody until unbudgeted reclaim. This
retires these registrations only: retained root/publication-cell ownership still
needs the full Native/physical close proof, and SnapshotHandle registry/cache
obligations remain separate.

The exact owner-private retained_observation/retirement/tests.rs child may
construct only SDK ExecutionRequest/SerialRequest roots. Use genuine Native
owner-issued bases and the actual gates for external-cut Busy, lane Busy,
original stop after external removal, registry Busy after Observation removal,
once-only terminal accounting, concurrent original-entry pruning or successor
registration after unique-inner handoff, and an independent readable observation alias.
The alias must preserve its genuine close veto. The exact private cfg(test)
assertion children history/retention/lane/retained_release/tests.rs and
branch/basis_registry/retirement/tests.rs borrow that caller's original request,
runtime, basis and task, and hold/assert their actual owner gates locally. They
construct no SDK request or runtime and return no guard/Arc/authority. Register
only their mechanically observed borrowed-type classes: ExecutionRequest for
the lane assertion leaf, ExecutionRequest/RelationalRuntime for the registry
assertion leaf. No SerialRequest or constructor/factory audience is added.
Record the four exact helper/caller pairs in src-court-private-logic-audit.json;
no family-wide exception or production test door is permitted.
Review the exact causal source,
run these controls plus relevant existing retirement evidence, then boundary,
context, formatting and line-cap gates before integration. No full D11, Bridge
source, Directory Native-close or physical inertness claim follows from this
prerequisite alone.

## Bridge observation and branch-head lease retirement prerequisite

Consume the actual RelationalBridgeObservationLease or
RelationalBridgeBranchHeadLease into its non-Clone retirement task. advance
borrows the original ExecutionRequest and concrete RuntimeBridgeRelationalSource;
into_retired returns opaque original custody or the same unfinished task. Use
one owner-local RelationalBridgeBasisRetirementProgress contract with Advanced,
Busy, Stopped, Refused and Complete; refusal distinguishes foreign source,
registration/commit-index mismatch and original Native refusal. Final reclaim or
abandonment is unbudgeted. Completed tasks perform no further poll or effect.

Every live advance compares the original bindings Arcs with the borrowed source
before effects. Head tasks verify BOTH head and observation issuers before their
first effect and on later live advances. Match head binding ID AND its original
snapshot identity: a missing or different tuple is locally detached, preserves
its successor, and still retires this lease's own observation. Abandonment after
a partial/foreign/Busy refusal must also remove only this original tuple, never
a successor; do not accidentally route that task's cleanup through scalar-only
matching. Existing ordinary public release retains its unbudgeted contract.

Observation removal uses typed original snapshot/version parts without parser
error allocation. Validate its actual retained Native subject and matching
by-commit membership under the original observation try-lock, then consult the
original request before removal. A missing original observation row before this
task's recorded removal is RegistrationMismatch: authentic non-Clone issuance
has no supported concurrent prune or replacement path. Shared runtime handles
cache their original Native runtime ID, and the sole observation insertion caller
checks the actual runtime against it before issuance; a swapped fresh runtime
cannot insert reused IDs into the same source. Add no duplicate runtime-ID field,
identity token, counter, allocator, registry or ownership-policy mode.

Use remove_entry to move original branch keys, bindings and snapshot rows;
BTreeSet::take removes the actual snapshot ID. Retain a removed empty commit
set, original metadata payloads and registry Arcs in the same task. Disarm the
ordinary lease cleanup and record the committed stage before another poll.
Release EVERY Bridge guard before entering the existing Native retained-
observation task. No replacement ownership allocation, key/descriptor clone,
clear, drain or whole-index scan is permitted. Tree point removal is O(log n)
and may free fixed control nodes while rebalancing; it is neither O(1) nor zero
deallocation. Preserve all actual payload/grant custody until unbudgeted reclaim.

Place tasks under runtime_source/observation_bindings/retirement.rs and
branch_head_bindings/retirement.rs, with a narrow basis-retirement progress
contract and existing explicit facade exports. Use an exact retirement child
of the existing bridge_snapshot_reader_tests court for genuine source/native
fixtures and original request inputs; owner-private cfg(test) assertions may
hold actual gates/indexes without exposing a production gate or constructing
hidden SDK/runtime roots. Root records only mechanically observed exact test
classes and named private helper/caller pairs, with no family-wide exemption.

The exact new court child bridge_snapshot_reader_tests/retirement.rs and two
owner assertion children branch_head_bindings/retirement/tests.rs and
observation_bindings/retirement/tests.rs receive only their scanner-observed
ExecutionRequest/RuntimeBridgeRelationalSource classes. Assertion children only
borrow original fixtures and issue no runtime or SDK requests. Bind the three
exported helper/caller pairs in src-court-private-logic-audit.json; the head-gate
assertion is private inside its wrapped-successor owner helper. The exact
negative wrap fixture may set the ORIGINAL next_binding_id to MAX under cfg(test)
and issue max/wrapped leases via the PUBLIC source over a genuine admitted basis;
it creates no fake lease or production counter change. All captured gates and
original index values remain private, with no authority or guard returned.

Prove actual lock Busy/retry, original stop between committed stages, foreign
sources sharing one Native runtime, superseded-head preservation, commit-index
consistency and deferred reclaim. An already-open reader must still read its
old packet after registration retirement while new opens refuse. Separately,
a genuine observation alias must cause Native ReadCustodyRetained AFTER original
Runtime Arc unwrap succeeds and the same refused runtime must remain usable;
outer Arc sharing is not that oracle. Retained-snapshot reader retirement remains
unsupported because its RuntimePublicationSnapshotReader owns an operational
RelationalRuntimeHandle. These lease ports do not retire that handle, prove full
Native root/publication-cell inertness, or establish whole Host D11.

## Immutable Native alias and retained-reader decomposition prerequisite

RelationalRuntimeHandle adds a pure validate_immutable_owner_alias(&Runtime)
validator and consuming try_retire_immutable_alias(&ExecutionRequest, &Runtime).
Reject Shared representation before effects, require the borrowed runtime's
actual Owner tenure, and compare the immutable Arc's live pointee with that
borrowed owner. A cached instance ID or reference-count observation is not the
identity proof. The consuming port repeats validation and consults before
relinquishing only its original Arc share; the matching owner borrow guarantees
another strong owner remains, so no Runtime destructor runs in that step.
Failure returns the SAME original handle. RetiredRelationalRuntimeHandleAlias
retains only original cached metadata and lock-observation slot until unbudgeted
reclaim, with no Runtime/RuntimeHandle or serving accessor. The slot contains a
Weak listener, not its callback payload. Add no Weak<Runtime>, lifecycle binding,
owner counter, ownership mode or close witness. The SAME Native owner stays
serving for Complete capture/select/ack; this is local alias discharge only.

The actual non-Clone RelationalBridgeRetainedSnapshot has one closed production
constructor: open_retained_snapshot consumes its non-Clone lease, resolves the
SAME issuer row, and clones that row's observation into its private concrete
RuntimePublicationSnapshotReader. While this RAW lease remains untouched, its
registry Arc anchors that row and Native's TWO original observation shares.
Only consuming that lease removes the row; the cached-runtime insertion fence
prevents supported replacement. Therefore dropping the reader's ONE observation
share FIRST is provably nonfinal by construction. Use no count proxy, existing
Memory root-clearing helper, new Native one-share port or fake replacement reader.

A pure retained-snapshot try_into_retirement validation failure returns the SAME
usable snapshot. The resulting non-Clone RelationalBridgeRetainedSnapshotRetirement
borrows original request, actual source and matching Native owner on advance;
each live call rechecks original source issuer and authentic owner. Decompose the
private reader into takeable ORIGINAL fields, then use structural order:
1. Discharge its immutable runtime-handle alias.
2. Consult and relinquish its original observation share while the RAW lease
   still anchors Native's two shares; do not advance or detach that lease first.
3. Convert the original lease into the qualified Bridge observation task.
4. Delegate until that authentic retired lease result is obtained.

Busy/stop/refusal retains the same original parts and remaining work. Completed
calls perform no further poll/effect. Abandonment field order releases reader
observation BEFORE the raw lease; lease conversion is legal only after that
observation field is empty. RetiredRelationalBridgeRetainedSnapshot keeps original
snapshot/partition metadata, Native alias data and retired lease custody for
unbudgeted reclaim, with no source or operational handle. Original independent
readers retain their genuine Native alias/read vetoes.

Place Native alias handling under change_source/runtime_handle/alias_retirement.rs,
private reader parts under relational_source/snapshot_reading/retirement.rs, and
public composite under runtime_source/retained_snapshot/retirement.rs. Use existing
facade chains and owner-specific progress/denial wrapping actual alias/binding
causes; no general retirement framework or new registry is introduced. Test exact
original snapshot restoration on foreign/unsupported entry, stop at each stage,
nonfinal observation-before-lease release and abandonment, unchanged Native release
counters at that stage, existing-reader protection, and genuine Native work AFTER
alias discharge. Source/caller tests use existing genuine owner fixtures; Root
records only exact observed SDK classes and named owner-private assertions.

Only Native runtime_handle/alias_retirement/tests.rs receives the observed
ExecutionRequest/RelationalRuntime/SerialRequest classes, and only the existing
bridge_snapshot_reader_tests/retained_snapshot_retirement.rs court child receives
ExecutionRequest/RelationalRuntime/RuntimeBridgeRelationalSource. The Native
three named units inspect original private handle/slot identity and real admitted
tenure, including a genuine owner-issued operation; record those exact private
placements in src-court-private-logic-audit.json. The four named closed-reader
construction/phase/abandonment/independent-reader cases reuse their existing
owner court and genuine Native fixture; no family-wide fixture or factory
permission is added. Runtime state equality/cached identity never substitutes for
actual owner pointee identity, and no counterfeit positive owner is constructed.

Shared support and actual Directory Native business/graph retirement remain
unimplemented. Current Memory source-affinity refusal and ordinary prepare_close/
finish cannot substitute. Later coupled proof must still capture/select/ack
while Native serves, then retire genuine original graph/provider/root/member
obligations and issue actual business-complete allocation custody. These local
ports establish neither that witness nor whole Host D11.

## Cold member custody retirement prerequisite

This prerequisite retires one actual source-issued registration, or one local
share of it. It does not establish cold graph retirement, NativeClosed,
ApplicationHome close, physical close, or completion of D11. The accepted
Memory retirement path remains separate from cold provider obligations.

The adversary is the original Store guard retaining selected checkpoint
members while another independent cold reader remains live. Hold its actual
registry lock, stop after a partial member release, switch sources, clone the
original token, and finally abandon an incomplete task. Bypassing issuance
identity must fail when an unrelated provider falsely returns Complete;
neither callback on that provider may execute. A source with the same Serving
runtime but a different registry is also foreign. Successful local release
must preserve the other reader, actual cold reads and checkpoint generation.

The Native facade keeps `RelationalDurableMemberCustody` opaque. Its existing
`new(owner)` constructor remains supported but cannot issue bounded retirement
for an unknown provider. Add `new_from_source(owner, Weak<dyn
RelationalDurableColdSource>)`; the original owner and weak issuing-source
identity reside together in one sized Arc cell containing the boxed original
payload. Acquisition pays one additional allocation and indirection. Cloning
custody retains the same actual registration, rather than registering again.

Consuming `into_retirement()` returns a non-Clone
`RelationalDurableMemberRetirement`. `advance(&mut self, &ExecutionRequest,
&dyn RelationalDurableColdSource)` returns the typed observations Advanced,
Busy, Stopped(original cause), Refused(typed cause), or Complete. The public
refusal distinguishes UnsupportedProvider, ForeignSource and ForeignCustody;
no callback may translate these into successful release. An already recorded
Complete is observed without further callbacks or effects. Otherwise check the
original request, then compare the borrowed source's data address against the
retained issuing Weak's data address before any callback or shared-token
consumption. Never upgrade the Weak: its retained allocation prevents address
reuse without introducing a temporary last strong source owner into advance.

The existing cold-source trait gains default-Unsupported provider callbacks:
`validate_member_custody_retirement(&self, &(dyn Any + Send + Sync))` returns
the typed refusal or success; `advance_member_custody_retirement(&self,
&ExecutionRequest, &mut (dyn Any + Send + Sync))` returns the same typed
advancement observations. Only the provider sees the borrowed opaque payload;
there is no caller getter or downcast door. Store downcasts its original
private guard and validates BOTH registry and Serving Arc identity before
effects. Immutable validation performs no IO and acquires no locks; every
advancement lock is nonwaiting. Every effectful call repeats issuance validation, preventing a source
switch after partial progress.

Use `Arc::into_inner` for atomic local-share release or unique-cell transfer,
never `strong_count` to decide ownership. Complete for a nonlast share means
only that this local token relinquished custody. For a unique guard, retain
the original member Vec, registry, cursor and optional Serving Arc. A step
uses `try_write` on the actual registry, decrements/removes one original member
and advances the cursor under that same lock. Busy, stop and refusal retain
the authentic remaining payload. On the final member detach the guard's extra
Serving Arc while the matching borrowed Store source still owns Serving.
Detach whenever the cursor reaches the original members.len(), including an
originally empty registration.
Keep the disarmed guard, original Vec and registry opaque until explicit
unbudgeted reclamation. Ordinary Drop retires only the remaining suffix;
incomplete task Drop is unbudgeted abandonment and proves no successful close.
The composite Native `RegionMemberExport` remains UnsupportedProvider here.

`into_retired()` returns the opaque retired custody only after Complete,
otherwise returns the original task. `retired.reclaim()` is explicitly
unbudgeted. Advancing performs no new allocation, reservation, rehash or
shrink, arbitrary payload destruction or last physical-provider destruction.
Each actual member step is O(log distinct registered members); BTreeMap
removal may rebalance and deallocate nodes, and Arc share release may reclaim
a small control allocation. There is no hard O(1), no-deallocation, or
destructor wall-clock claim.

Destination topology (existing parent modules remain canonical):

```text
worth-relational/src/durability/medium/
  member_custody.rs                [revise: opaque acquisition/clone]
  member_custody/retirement.rs     [create: consuming retirement protocol]
  cold_source.rs                  [revise: provider callbacks]
worth-store-relational/src/
  cold_source.rs                  [revise: cyclic source construction]
  cold_source/member_custody.rs    [revise: original guard/cursor]
  cold_source/member_custody/retirement.rs [create: actual registry effects]
  cold_source/member_custody/retirement_tests.rs [create: private proof]
  cold_source/member_custody/retirement_tests/directory.rs [create: genuine FS fixture]
```

Store's private `new(serving) -> Arc<Self>` uses `Arc::new_cyclic` and retains
weak_self. The existing certification observer is supplied by private
`new_with_commit_read_observer` before construction; remove its mutable
postconstruction setter. Cut over shared and unshared production attachment,
both corresponding certification observer attachments and the two existing
private test modules. The unshared Home attachments use the same cyclic source
constructor and preconstruction observer injection. No public source extraction, new control, registry, worker,
counter, scheduler, or ownership mode belongs in this prerequisite.

The following exact three private tests are a narrow specification exception
to Iteration Law 2's foreign-runtime construction prohibition. They live only
in `worth-store-relational/src/cold_source/member_custody/retirement_tests.rs`:

- `cold_source::member_custody::retirement_tests::selected_directory_guard_retirement_preserves_partial_custody_and_cold_reads`
- `cold_source::member_custody::retirement_tests::issued_source_identity_refuses_fake_and_switched_sources_before_callbacks`
- `cold_source::member_custody::retirement_tests::shared_local_retirement_preserves_other_owners_and_unbudgeted_drop_retires_only_remaining_suffix`

These tests alone may construct genuine foreign Native/Store runtimes because
the original guard, registry lock, cursor and detached Serving field cannot be
observed through the public court without an extraction door. This grants no
family exemption and no new public test access. Their private `directory.rs`
child may hold only their shared concrete filesystem producer/recovery fixture
and cold-read/generation oracle; this exact helper is included in the same
construction exception, introduces no additional tests, and is used only by
the three named tests. Keep its Native builder/schema/ordinary commits,
StoreJournalMedium, production external-member capture/preparation, anchored
publication/read and recovery calls explicit in the review inventory. Split
this fixture to obey the ordinary 400-line cap; do not add a test family.
The registry's private production acquisition core may return its genuine
registered guard to the source-local wrapper before opaque Native issuance;
the same core supplies the named private negative affinity test. No public
payload access is added. Same-Serving/different-registry validation must be
tested independently of the earlier issuance check. A negative-only test may
temporarily substitute another actual filesystem Serving Arc in that genuine
guard, require ForeignCustody, then restore the original before Drop. This
does not fabricate successful custody, counts or publication evidence.
Use the existing
`cold_source::test_store::initialize`: its `physical_record_journeys::media`
qualifies actual filesystem media. Produce Native schema and ordinary
transactions, production section capture/encoding, actual anchored publication
and selected membership, then production recovery with that same Store source.
Read actual cold record/history data. Fabricated member bytes, locators,
snapshots, authorities and registry-count-only success are insufficient.

Cover registry Busy, partial stop/rejoin, fake-source and source-switch refusal
before callbacks, same-Serving foreign registry, local shared release,
remaining-suffix Drop exactly once, independent reader protection, retained
physical protection after source loss and unchanged checkpoint generation.
This is a filesystem provider integration proof, not an ApplicationHome proof.
Update `crates/worth-relational/README.md`, `docs/capability-index.md` and
`workspaces/worth-store/crates/worth-store-relational/README.md` with
the facade, issuer affinity, local-share meaning, unsupported composite case,
cost and unbudgeted reclamation. Verify those documents against actual exports.

Implement acquisition/trait/progress first, then Store guard/construction
cutover and honest fixtures, then run focused provider tests, relevant Native
regressions, formatting, scoped line caps, boundary and context checks. Fresh
independent source review must approve the coherent batch before integration.

## Installed call close-admission prerequisite

All four installed Query opening sites borrow placement through World publication
port `with_execution_placement`. That whole callback now retains a borrowed
admission on the EXISTING World operation ledger, including its original
completion scope. A cloned ProductRuntime cannot bypass close admission merely
because it does not retain the outer Query program allocation.

Admission try-locks bootstrap, operation and close in the existing order,
requires performed bootstrap, a present owner and Open, then increments active
with checked arithmetic. Release every lock before the callback. Return or
unwind decrements that SAME entry once with the existing reservation Drop and
poison contract; it allocates no attempt, ledger, registry or per-call Arc.
World close checks this ledger before sealing and refuses InFlightCriticalSection.
The refusal preserves the actual serving Home and permits its real resume.

The decisive existing-filesystem Host court pauses a genuine ProductRuntime
advancement, observes World(Denied(InFlightCriticalSection)) rather than a later
Native/Bridge share refusal, resumes the same Home while the call remains held,
then releases it, reads the original balance and completes owning close. An
owner-local unwind court protects once-only admission release.

Bootstrap, standalone owner placement and stopped-owned D12 capture use separate
ports. D12 capture still checks its actual three durable flights and adapter
slots, not unrelated active installed calls: mandatory maintenance runs inside
this very admission. This prerequisite does not qualify full Host D11; arbitrary
installed payload destruction and actual World/Bridge/Signal/Native source
retirement remain the consuming owner work described below.

## Installed handler retirement boundary

This is a consuming Query prerequisite, not implemented full Host D11. The
actual registration moves each Handler into a fresh Arc inside its existing
TypedHandlerSlot, itself inside the erased entry Arc. Sealing clones that
outer slot; active/prepared work may clone the inner handler Arc. The public
installation API does not expose Query's newly allocated handler Arc. A
supplier can independently share operational state nested inside its Handler;
exclusivity of the outer allocation alone proves nothing about that state.

Preserve those SAME allocations. At registration, retain a monomorphized
function pointer beside the existing entry, sufficient to recover its original
Schema/Binding. The non-consuming preflight takes a mutable borrow of the
original erased Arc, uses Arc::get_mut, downcasts to the original typed slot,
then uses Arc::get_mut on its original handler Arc. Either uniqueness refusal
is Busy with the original payload untouched. These atomic uniqueness checks
also exclude Weak aliases; no strong_count probe, second typed Arc anchor,
alias destruction, replacement allocation or drop-and-restore is permitted.

The existing OperationHandler contract gains family-local object-safe
preflight_home_retirement and advance_home_retirement hooks, both defaulting
to Unsupported. They borrow the original invoking ExecutionRequest and the
original mutable handler. Preflight preserves business state. Only an audited
supported handler can establish how its ACTUAL operational fields retire; a
declared durable-family roster or empty capability list is insufficient.
Known supported nested operational sharing refuses Busy before seal. Unknown
wrappers, opaque closures and externally mutable state without an authentic
owner admission contract remain Unsupported.
The first admitted family is the actual internal compiled-body registration
origin. Opaque supplier registrations remain Unsupported even if they override
the hooks and assert Complete. Only the concrete Host close consumer may invoke
`begin_after_world_seal`; its exact caller is mechanically sealed as
`query-home-retirement`. The prepared owner exposes no live clone between the
unique original-owner preflight and that irreversible handoff.

Run every actual handler/inventory preflight before World seal. A refusal
preserves the same usable Query and its genuine pre-seal resume. Once sealed,
the existing close phase retains the original registry, slots, handler Arcs,
payloads and advancement position through Busy, stop and retry; it never
recreates Query service. Retire only their actual operational capabilities.
Keep arbitrary inert supplier payloads until the SAME retired Home allocation's
explicit unbudgeted reclaim. A handler's Complete observation does not certify
Native binding exclusion, source retirement or physical Store Closed; those
actual owner gates remain independent prerequisites.

The adversarial proof covers both erased-slot and active-handler aliases,
nested external operational sharing, unchanged execution after pre-seal
refusal, stopped post-seal retry retaining the original handler, and actual
filesystem Store Closed while a user Drop probe remains alive until reclaim.
Owner-private tests may create otherwise inaccessible internal aliases; those
are not public external-handler handles. Passing only those private courts
cannot qualify physical close.

Committed successor topology stays with the existing owners:

```text
worth-query-execution/src/domain_computation/
  primary_graph/application_entry/mutation/ [extend original handler contract/refusals]
  primary_graph/handler/registry/registration.rs [issue typed entry thunk]
  primary_graph/handler/registry/retirement.rs [consume original entry owners]
  primary_graph/application_runtime/ [preflight actual installed inventories]
worth-query-host/src/application_home/close/ [retain original post-seal Query task]
```

No empty placeholder or general retirement registry belongs here. Query's
runtime Drop extraction and actual World/Bridge/Signal/Native consuming ports
must be qualified together with their real close consumer before SourceGO or
a full bounded Host-close claim. The current drop(query) remains an explicit
gap until that cutover.

## Initial-empty Signal stop and composite close preparation

This narrow prerequisite stops the actual first supported Signal owner and
retains its original allocation. It does not perform general Signal cleanup or
report Signal Closed. Unknown/nonempty configurations remain unsupported; the
actual Native and physical Store close gates still decide their own completion.

SignalRuntime consumes its original owning value into an opaque
PreparedInitialEmptySignalStop, or returns that same value with a typed refusal.
It takes only nonwaiting guards on the existing transition gate, registry,
original witness cell, metadata, retention, conditional-retention and temporal
state. Require the actual same initial witness/branch/incarnation, sole selected
live cell, generation zero, next branch one, no snapshots/restoration/history/
retirement/reservations and the graph's genuine producer-empty state with no
nodes, dependencies or authorization identities. Compare borrowed owner fields;
reconstructing an observation allocates strings and is unnecessary here.

The pre-seal BranchState initial-origin predicate requires absent installed
definition custody, so it cannot be reused verbatim after sealing. The real
baseline has authentic sealed definition/service/basis custody. Retain it;
requiring absent conditional services or zero retention counts would reject the
supported baseline. Bridge separately checks actual empty lowerings, indexes,
reservations, targets, async declarations, clock lanes, execution-basis
reservations and matching baseline dependency state through their owning locks.
Descriptive count methods or durable-family declarations are not that proof.

No Signal operation admission may exist in this preparation. With all required
guards held, poll the original request and CAS the EXISTING packed OPEN|0 to
CLOSING|0 once. A competing admission makes this Busy with no state change.
Keep the original SignalOwnerRoot in the prepared value. Do not call
request_close, claim_cleanup, a cleanup coordinator or the root's Drop. Count
zero excludes a later last-admission Drop initiating cleanup. Defer retention
acquisition-close flags until commit. Before commitment the genuine owning
prepared value alone may resume the SAME CLOSING|0 owner to OPEN|0, with no
cleanup claim. It cannot mint a replacement root or resume another owner.

Borrowed preflight followed by World seal leaves a real race: externally cloned
Signal service ports can change the baseline between those calls. Use reversible
Signal preparation inside World's ACTUAL existing final close admission cut.
Expose an opaque RuntimeWorldPreCloseAdmission borrowing the authentic World
owner. It retains the existing bootstrap -> operation -> close guards and the
same preallocated close task, after all existing frontier, owner, custody,
durable-publication and original-request checks have passed. Active or recovery
operations refuse as before. The same World gates exclude installed calls.

Store orchestrates above the owners: obtain that World guard, perform only the
narrow try-only Signal preparation, then consume the World guard with an
INFALLIBLE begin that installs the SAME close task and existing Closing state.
No poll, allocation, supplier callback, I/O or other fallible operation may occur
between Signal's successful CAS and World begin. Signal refusal drops the World
guard without changing either owner. Release all World guards immediately after
begin, then commit Signal's existing ledger flags and drive existing World
drain outside admission locks. A later Busy/stop retains the same owning close
posture; it never resumes service. Any future pre-begin refusal must restore
the authentic Signal owner before reopening World admission.

World product-head retirement updates World-local component claims while
retaining the actual owner leases; it does not need Signal admission. Its
history drain is World-local and original-result cleanup calls Native. Preserve
that direction. World must not own a Signal callback or a second admission
ledger, counter, mode or scheduler.

Signal commit closes actual acquisition flags under their nonwaiting guards and
returns the original stopped allocation for explicit unbudgeted reclaim. It
never reports cleanup Closed. Bridge's current owning Drop blocks/scans and can
allocate. A later consuming Bridge cutover must take the SAME original private
state from a takeable inner slot, transfer RuntimeBridge's actual Native/source
capabilities to their owner retirement, and retain Signal/root plus arbitrary
inert payloads. Ordinary Drop still cleans its actual Some state. No replacement
runtime, ManuallyDrop or whole-Bridge retention may conceal those capabilities.

Proof covers admission-versus-CAS races, each actual lock Busy, original stop,
nonempty refusal, external mutation before preparation, unchanged execution
after resume, denied service after commit and no root/payload destruction until
reclaim. World tests prove guard exclusion and refusal preserves Open. The
eventual Store consumer must prove the joint cut and actual physical Closed
while retired allocations remain alive; owner-local tests alone cannot certify
full Host D11.

```text
worth-signal/src/branch/owner_services/owner/initial_empty_stop.rs [create owner cut]
worth-signal/src/branch/owner_services/lifecycle_state/ [extend original phase gate]
worth-signal/src/logic/transaction/runtime/state/runtime_state/
  initial_empty_stop.rs [create consuming original-runtime API]
worth-runtime-world/src/lifecycle/close/
  admission.rs [extract existing final admission; no competing checks]
worth-runtime-bridge/src/conditional_execution/contract.rs
  [later consuming original inner-state extraction]
worth-store-application/src/world_section/close.rs
  [later joint owner orchestration and retained stop task]
```

No empty placeholders are required. Qualify the narrow owner primitives and
their coherent ordinary-path cutover before integrating the consuming Store
composition. Native/Bridge/Query retirement remains independently required.

The following exact private Signal leaf is a narrow Iteration Law 2 exception:
`crates/worth-signal/src/branch/owner_services/owner/initial_empty_stop/tests.rs`.
Its seven tests alone may construct SignalGraph and original SDK SerialRequest/
ExecutionRequest roots to exercise the owning crate's inaccessible proof locks,
admission CAS, original-root lifetime and poison posture. An external court
cannot hold those private gates without adding a production extraction/control
door. Register exactly those three roots for that file in
`tools/boundary-check/config/src-court-baseline.json`; this grants no family-wide
exception or additional runtime/testkit/helper constructor.

- preparation_resume_preserves_actual_basis_and_service_execution
- every_actual_proof_lock_busy_preserves_the_same_open_runtime
- competing_real_admission_defeats_the_final_zero_count_cas
- commit_retains_real_sealed_service_custody_until_explicit_reclaim
- original_stop_preserves_open_or_prepared_custody_without_cleanup
- nonempty_or_reconstructed_graphs_and_multiple_branches_refuse_before_stop
- poisoned_owner_metadata_refuses_without_stopping_or_losing_the_root

## Native and Store source-close prerequisite

This is the next source-close destination, not implemented completion. It
extends the qualified single-registration retirement into an actual Native and
Store physical close consumer. It does NOT close full Host D11: Query's
installed handlers/resources/providers and Bridge's assemblies/source/sink
owners still need consuming retirement before their arbitrary destruction can
be kept off the Host request boundary. Retaining the whole Query object would
retain operational capabilities and obstruct Native close; it is not a fix.

The decisive court holds the retired allocations while obtaining actual Store
physical Closed. Include cold and materialized region/history, BOTH nested
dictionary owners, empty lineage registrations, deferred index detached history
and selected canonical backing when CHK18 lands. Interrupt a provider callback
handoff and hold its return-slot lock; resume the SAME task without another
callback. A live fork/port refuses before seal. A source-capable external plan
keeps its own physical protection and prevents Closed, while an inert metadata
pin keeps readable resident values and its normal once-only lease accounting.
No count, a region flag, empty container or dropped owner substitutes for those
observations.

### Authentic preparation and irreversible seal

The entry consumes the original outer runtime Arc through authentic ownership
transfer. A reversible refusal preserves the SAME actual runtime/state/source
authority and usable legitimate observers; the outer allocation address is not
authority. Preserve the inner runtime-state Arc and its Weak services. Do not
use recovery's `unshared_state_mut`/`Arc::get_mut(state)` as a close gate: that
would reject or invalidate legitimate Weak bindings.

Apply the existing stop/in-flight handshake, actual publication/candidate/
settlement/read fences and incremental complete source/provider validation
before destructive cleanup. Require an actual original medium binding in this
supported Store composition. Source-only/no-binding, moved medium, unsupported
provider, foreign source, unresolved publication or pending codec transition
refuses before seal. A cold-source-only fork has no retained medium binding,
so absence of that binding cannot prove exclusion. Memory's strict
source-affinity refusal stays unchanged.

Move the actual medium, cold-source and transition-history options under the
existing durability ownership lock. Construct bindings/provider attachments
outside its guards; release guards before binding locks or callbacks. Replace
automatic state cloning with explicit fork snapshot construction: no publisher,
the original transition reader and the retained cold source. Never clone a
publisher into a speculative runtime.

After complete preflight, take the ORIGINAL binding under a nonwaiting guard
and use `Arc::try_unwrap`. Failure reinstalls that exact Arc before admission
resumes. Success is the irreversible cut: retain the extracted original binding
and boxed provider, permanently disallow serving restoration. Dormant Weak
checkpoint observers do not veto this cut; a callback that upgraded its Weak,
a real fork or an owned port retains a strong binding and therefore refuses it.
Do not use reference counts as an authority probe.

### Actual physical slots and retained callback work

Leave immutable root/radix/record/page allocations intact through unbudgeted
reclamation. Retire only actual issued physical obligations in their existing
owners. Inline slots retain optional original source/custody/task and completed
inert payload; a missing temporary slot is never a completion witness.

| Owner | Mutable placement |
| --- | --- |
| Region backing and readmission | Existing region state mutex; retain descriptors and resident contents |
| Symbol residency | Existing pages state mutex; retain dictionary/page data |
| Lineage node residency | One inline custody mutex in its existing shared residency allocation |
| Lineage event residency | One inline custody mutex in its existing shared residency allocation |
| History prefixes | One shared concrete custody cell per ORIGINAL load-head history registration, shared by prefix clones |
| Deferred index image | Existing admitted-state mutex plus local directory-edge retirement; traverse its detached history owner |
| Canonical envelope residency | Existing state mutex, including CHK18 selected member/directory backing when implemented |
| Directory | Consume only the Native-local Arc edge; keep any last finite directory payload in that original holder slot |

Node/event aggregate registrations require retirement even if their page
containers are empty. This adds mutex/progress metadata inside existing shared
allocations, not another Arc per node/page. Prefix issuance pays one additional
allocation and indirection per original history registration, never per row.
Document exact inline/task/directory footprint rather than calling it free.

Visit top-level and cold-backing dictionaries AND
`cell.head_readmission.symbols` AND mirror Head readmission symbols. Detached
snapshots may own distinct physical residencies; ordinary aliases of one
residency observe the same original retirement progress.

Each owner-specific cursor holds the actual current owner Arc/root and exact
slot identity. Under its lock, convert the original token once or take its
existing task, placing that task in the close phase BEFORE unlocking. Invoke
the provider outside Native locks. Retain the callback observation in the phase
before attempting nonwaiting slot return. If return is Busy, retain task AND
observation, keep the cursor fixed, and return the task first on the next call.
Do not repeat an already performed callback, recreate a task, or classify an
empty handoff slot as discharged. Use constant current-handoff storage, not a
graph-sized queue, shadow registry, scheduler or ownership-mode framework.

Retire each Native-owned issued region lease exactly once and disarm that
lease's later Drop decrement, retaining its cell/resident allocations. Preserve
external metadata-pin lease counts and their ordinary clone/Drop behavior.
Their actual public methods only read resident metadata and a copied interner;
they may survive as inert data once physical slots retire. Never clear shared
counters or require such external pin counts to become zero.

### Reversible inventory traversal and ordinary concurrency

Replace the primary retired-root DashMap with an existing-owner
`RwLock<BTreeMap<usize, RelationalRetiredBranchRoot>>`. Close uses a nonwaiting
excluded-key successor lookup, retains at most one actual root, and releases
the map guard before traversal/callbacks. It never pops/requeues the ordinary
SegQueue and never invokes diagnostic inventory or count-based GC to certify
close. Root locator maps hold only key/generation data and need no physical
owner traversal.

This introduces REAL global write serialization formerly distributed across
shards; `external_cut` did not already serialize retirement writes. Keep reserve,
retire and release to individual entries. For sweeps, snapshot one entry's
generation/root/binding handles, inspect lane liveness outside the primary
guard, then reacquire and validate generation and actual binding identity
before pruning. Locator cleanup and removed-payload destruction occur outside
the primary guard. Preserve existing locator-before-primary guard release
order; never hold one global write guard through a sweep or lane wait.

After authentic stop and custody fences no new retirement entry can be
admitted. Reservation Drop may remove an unretired entry, but its unchanged
live head still holds that root. Retired entries remain because close invokes
no ordinary reclaimer. Preserve these premises so dropping a temporary cursor
Arc cannot destroy the final root. Selection costs O(log retained roots),
member steps O(log distinct registered members); there is no hard destructor
wall-clock or no-deallocation claim.

### Actual Store consumer and explicit reclaim

Replace synchronous `ClosingRelationalSection::finish` with original-request
advancement and consuming retired extraction. The section privately lends its
actual source and retains actual Serving/selection custody. No new source
getter or public unfinished close helper is introduced. Unknown medium
providers refuse in preflight; their trait-object Drop is never assumed inert.

Retire Native-owned slots, then original medium physical shares, then Store's
local source share. A shared external source or RecoveryPlan retains its own
semantic basis/protection; never globally detach it. Keep the actual close
stage across Busy/stop/refusal and drive the existing physical close operation.
The same actual registry and Serving identity checks continue to apply.

The selected physical read membership adds authentic obligations to that SAME
source traversal. Its private shared backing holds an original reader and
session, two references to one C.10 acquisition, lifecycle leases, an operation
grant, and possibly the last inline/extent frame pin even after verified EOF.
Ordinary final Drop/cancel is unbudgeted and may block or drain closed-pool
frames; it cannot prove bounded source retirement. A consuming physical-owner
task must validate original Serving affinity before discharging its local Arc
share. A unique backing moves its original data/readers/grants into the task
without another Box; fixed Arc control storage may be deallocated. Shared
backings retain all remaining owners' protection.

Retire the actual frame pin, operation grant, C.10 references and lifecycle
leases through their original nonwaiting owner gates, recording each completed
stage before release so Busy/stop cannot repeat an effect. Frame discharge must
retain a matching resident owner, use original generation/bytes identity and
avoid closed-pool whole drains. The existing allocation-event recorder has its
own lock and potentially growing certification trace: acquire that SAME lock
nonwaitingly and establish lawful trace capacity before pool effects. No new
ledger or dropped trace substitutes for those obligations. Retired table/data
allocations and execution grants remain inert until explicit reclaim.
The same recorder's certification trace uses owner-private fixed-size boxed
pages rather than a history-sized growable vector. A controlled release funds
one concrete page from its original request before acquiring the pool gate;
the page retains that grant until its actual events are reclaimed. With the
pool gate and SAME recorder gate held nonwaitingly, counter arithmetic,
sequence space and page capacity are established before physical effects.
Installing a prepared page links the original pages without copying history.
Ordinary recording uses that same storage; explicit trace observation still
copies the event history outside the bounded retirement claim. This is trace
storage inside the existing owner, not another release ledger or scheduler.
The committed destination is physical_runtime/anchored_records/read_membership/
retirement.rs plus private release mechanics at existing session, C.10,
lifecycle and pool owners. No empty placeholder is needed. Require shared and
independent-session proofs, every Busy/stop boundary, once-only partial retry,
inline/extent EOF pins and foreign affinity. Store shutdown may report retained
readers/residency even with a ClosedRuntime value; require its authentic clean
classification and discharged physical obligations while inert data stays alive.

Keep Native/provider retired allocations in existing
`ApplicationHomeRecoveryCustody` through physical close. Actual Closed moves
them and remaining inert Home allocations into an opaque
`RetiredApplicationHomeAllocation` in the existing
`ApplicationHomeCloseOutcome::Closed`. Only explicit `reclaim(self)` performs
unbudgeted destruction. Incomplete Drop remains unbudgeted abandonment and
proves neither discharge nor close. The existing
`WorthQueryApplicationHomeClosed` already retains the lower outcome; extend its
reclaim surface instead of introducing another top-level wrapper or mapping
success to unit.

The Native+Store court must prove physical Closed WHILE all those inert
allocations remain retained. Test Busy during task return, original stop and
rejoin, source switching, empty registrations, both nested dictionary paths,
deferred history, original provider release, forks/ports/readers, external plans
and metadata pins. Private exact-slot tests complement genuine filesystem
observation; they do not qualify physical close alone. Caller/control additions
require Root's exact ruling, never a family-wide test exemption.

Current and committed-successor topology remains owner-directed:

```text
worth-relational/src/runtime/state/runtime_state/source_close/
  [create consuming preparation/seal/traversal; retain actual state identity]
worth-relational/src/storage/residency/ [revise actual region/lease/prefix slots]
worth-relational/src/runtime/state/subsystems/services/
  [revise actual symbol/node/event residency owners]
worth-relational/src/history/retention/ [replace primary map; keep original authority]
worth-relational/src/history/commit/envelope_residency/
  [extend source retirement for actual journal/selected checkpoint backing]
worth-store-relational/src/section/close/ [revise actual consumer and provider handoff]
worth-store-application/src/close/ [retain same task and retired allocation]
worth-query-host/src/application_home/close.rs [extend existing Closed reclaim]
Query/Bridge consuming ownership retirement [committed successor; full Host D11 open]
```

No empty placeholders are required. Query/Bridge must later disassemble their
actual physical capabilities and retain arbitrary installed payloads before a
full Host request-boundary claim is made. Their current `drop(query)` and
composition destruction remain explicit gaps; Native+Store success cannot
silently certify them.

## Fresh Directory bootstrap completion publication

This producer-only prerequisite may run in an isolated lane alongside Native
catalog implementation after its contract is frozen. It uses existing ordinary
Native publication APIs and does not implement or claim existing-Home service.
The latter remains gated on qualified catalog recovery. Preserve the already
qualified NoPendingOwnerAdmission and actual Host ownership-close paths.

Implement the completion schema/record described below, its producer codec and
the additional commit ONLY in Directory's publish_home_query_seed. Ordinary
Memory bootstrap receives no extra commit. Place lowering after projected,
derived and aggregate lowering, allocating from the actual completed registry
and declared contracts; preserve existing platform reservations and IDs.
Factor the existing admitted binding construction into one private operation,
without a second binding authority or copied Store descriptor codec.

Freeze the binding String grammar now. It starts with ASCII `WQHB1\n` (the
final character is a newline). Every following item is one netstring:
canonical unsigned decimal UTF-8 BYTE length, colon, exactly those payload
bytes, comma. Decimal lengths/counts have no leading zero except zero itself.
Items occur in this order: package digest, schema digest, selected identity,
selected revision, roster count, then identity/revision pairs in the actual
ApplicationHomeBinding roster order (strict revision order). Digests are
exactly 64 lowercase ASCII hexadecimal characters. Roster count is itself a
canonical decimal string inside its netstring. Identity payloads preserve
their exact UTF-8 bytes, including delimiters; no build UUID, process ordinal
or trailing item belongs in the encoding. Use checked total byte length and
original-request allocation/work accounting. A pure length/render core may
have literal known-value codec tests without constructing a foreign runtime.
The bounded decoder and installed readmission remain in the dependent slice.

Publish only after genuine bootstrap rows and index construction succeed.
The existing seed producer consumes activation before its first real row
commit; use its actual terminal bootstrap receipt. Require the new completion
commit's parent to match that receipt. Set PreparedHomeQueryRuntime.publication
only after actual completion success, and return the cached success on repeat
without another commit. Fence a failed completion attempt with the existing
bootstrap failure fact before its fallible preparation/allocation/publication:
no automatic replay of accepted seeds or uncertain completion. Preserve actual
prepared Query, Home and cause in existing SeedPublication refusal. Do not add
an ownership mode or pretend an indeterminate record is absent.

The existing Host publishes before select_query_root; actual World genesis
must therefore bind the completion-containing Native commit. Prove this on the
same genuine Home, using existing sealed preparation ports in the external
Store Application court:

```text
process/application/query_readmission/mod.rs
process/application/query_readmission/bootstrap_publication.rs
process/application/query_readmission/bootstrap_publication/inspection.rs
```

Only bootstrap_publication.rs may gain the existing query-home-preparation,
native-home-query-preparation and physical_serving_for_test caller audiences.
Keep factory/control references there; inspection.rs receives borrowed genuine
Native/root data. Root coordinates the required leaf dependencies and locks
under the existing Store-court D1 exception. Add no getter, control or raw
authority factory.

Real multi-batch cases with all rows flushed and with a final pending batch
must show exactly one completion entity, literal expected aggregate counts,
actual activation, original terminal receipt/parentage, absence at the terminal
seed basis and presence at the World genesis basis. A matching Memory case
retains its commit count. Arm the existing pre-effect candidate-retention
rejection after a genuine all-flushed initializer so completion consumes it:
assert exact failure, unchanged accepted seed history, no World genesis/service,
same Home/lease and no callback replay. Inspect actual owning close posture;
do not require Closed for indeterminate physical custody. Use real registry
contracts and immutable receipts, never offsets or a last-commit census.

## Existing Query publication and service readmission

This batch follows actual Native index-catalog durability/readmission and the
real Store NoPendingOwnerAdmission gate specified in
[Durable owner contracts](milestone-3-durable-owner-contracts.md). It reopens
the same initialized filesystem Home without initializer, seed-supplier or
business-handler replay. It supports zero actual retained World recovery
custody and explicitly admits only the supported persisted Query families.
Full recovered unpublished custody, original-receipt retry, automatic
checkpointing, transitions and full D8/D9/D11 certification remain successors.

Native currently registers index definitions only in its live catalog and
restores them only from Native checkpoints. Actual canonical Store artifacts
use the authoritative projection, omitting derived index generations; their
tails do not restore the full definition inventory. World-backed close may
lawfully have no checkpoint and does not currently create a combined manifest.
Thus RequireRecovered cannot yet admit a genuinely initialized tail-only Home.
Before this batch's Source GO, freeze and implement Native-owned persistence
and readmission of the existing catalog through its canonical durability path,
including checkpoint-plus-tail recovery. Reuse the actual catalog owner and
existing generation build/demand after definition admission. Neither Query
Register on reopen, an independent definition vector/registry, nor checkpoint
on close alone closes this crash/recovery boundary.

The adversary has genuine Native history beyond the selected World root.
Business reads must use the selected World Native basis; the latest-Native
scan independently restores durable outcome identity frontiers. Native 9 /
World 8 with unpublished custody remains an owning refusal before service.
An empty new Query cache or completion counts cannot erase that custody.

Freeze the following construction boundary:

- `PreparedHomeNativeMeaning::finish_existing_home_native_readmission` consumes
  the prepared schema/program meaning, original phase, actual Native runtime,
  recovered authority and recovery outcome. It refuses pending codec state,
  performs real `readmit_recovered_initial_schema_installation`, and uses
  `IndexInstallationPosture::RequireRecovered`. Its result is
  `ReadmittedExistingHomeNativeMeaning`, never a fresh initializer capability.
- The existing-only source attachment returns
  `PreparedExistingHomeQueryRuntime`, with no initializer or seed-publication
  methods. Share private product-equivalent mechanics where appropriate,
  without converting existing preparation to the public fresh preparation
  type or introducing an ownership mode. Only fresh attachment creates
  `WorthQueryProgramActivationSeed`.
- Existing attachment consumes an actual selected `ProductBranchObservation`.
  Both Home preflight and source preparation use its admitted relational
  basis through the existing owner-proven basis bind. Add only the source-owner
  `prepare_product_source_at_basis(execution, &AdmittedRelationalBranchBasis)`
  operation. Remove current-head lookup from Directory Home attachment; keep
  existing unrelated Memory construction behavior.

The source-at-basis operation validates and retains the supplied observation
through the existing Native snapshot operation; it MUST NOT rebind Bridge's
truth head. Otherwise it could mask a preflight that still binds latest Native.
The existing Store native_artifacts court independently retains the actual
source from query_composition_parts and calls the existing Bridge
TruthBranchHeadSource::load_branch_head_patch. Compare its actual envelope
version with the selected World's admitted observation and immutable receipt;
separately assert later unassociated Native truth has value 9 while World
service reads 8. Those are values, not hardcoded version IDs. Existing
attachment issued_from checks independently convict a wrong source basis.
Root may add worth-runtime-bridge only to this existing Store-court leaf to
name the existing trait; no reexport, raw getter or test port is introduced.

The one new Native Query entity is `worth-query-bootstrap-publication`, aspect
`bootstrap-publication`, contract revision 1. Required fields are
`format-version: UInt64` (exactly 1), `binding: String`,
`terminal-bootstrap-commit: UInt64`, and UInt64 counts for principal bindings, identity
indexes, application equality indexes, policy entities and policy relations.
The binding is exactly the admitted portable ApplicationHomeBinding described
in the shared contract, encoded by Query's closed versioned codec with checked
lengths, canonical digest rendering, canonical roster order and no trailing
data. Bound decoding/allocation by the actual row and original execution
resources. Reconstruct through ApplicationHomeBinding's validator and readmit
against the actual installed meaning; do not copy Store's private descriptor
codec or serialize live binding ordinals/generations/build UUIDs.

Append this entity after the maximum actually lowered platform kind and its
aspect after all existing platform, projected and aggregate reservations.
Preserve existing IDs; workflow extends beyond inbound completion through
workflow-first + 40, so inbound+1 is not free. Derive allocation from actual
lowering/registry state rather than duplicating offset arithmetic. Missing or
unknown-format records and incompatible recovered schemas return owned refusal;
recordless Homes never regain initialization opportunity or migrate silently.

After all genuine seed batches and index construction, the fresh Directory
`publish_home_query_seed` path commits the completion record in one additional
ordinary Native transaction BEFORE World genesis. Its final-seed locator is
the actual terminal bootstrap commit; the record transaction is a distinct successor
and World genesis binds that record-containing basis. Do not impose this
extra publication on ordinary Memory bootstrap. A failed completion publication
retains the actual prepared state and causes, without retrying accepted seed
work or exposing a service/root missing completion.

The existing seed producer consumes initial activation before its first real
row commit. Thus genuine all-rows-flushed multi-batch publication already has
no pending activation seed, and its accepted last-seed receipt is terminal.
Preserve and verify that actual producer invariant; a synthetic reinstalled
activation seed is not a lawful recovery fixture. Use the issuer-derived
publication.bootstrap_commit_id, build/admit indexes at that terminal basis,
and require the completion transaction's actual parent to match. Do not guess
a current head or alter ordinary Memory publication without a demonstrated
causal need. Include genuine all-rows-flushed and still-pending multi-batch
cases in the filesystem proof. No activation-only-tail support is claimed.

At the selected World basis, require exactly one completion entity. Admit its
format/binding and both its creation and terminal-bootstrap immutable Native receipts;
the completion transaction's sole parent must be that terminal bootstrap commit on
the expected branch. Read actual program activation at the SAME selected
basis, admit its revision and install its real entity identity in the fresh
activation cell. Then construct the Query publication with the NEW installed
binding identity, original terminal bootstrap commit and descriptive aggregate counts.
Do not use checkpoint `recover_publication`, whose one-commit census cannot
recover multi-batch aggregate publication. No index registration or bootstrap
publication occurs on reopen.

Definitions alone do not make recovered indexes usable. Before QueryReadmitted
or public service escapes, invoke the existing ensure_primary_indexes_for_basis
at the SAME selected World relational basis and original request. It probes
the actual layout-derived primary inventory and uses Native refresh_for_basis
with its existing cold reconstruction budget and before=None for missing
generations. Do not add another index list/build algorithm or substitute the
latest Native head. Ordinary principal selectors prohibit population rebuild,
so deferring this step to first authentication can expose unusable service.
Any denial retains Query/Home before service. This is derived reconstruction,
without another business commit; the existing wrapper's small missing-ID Vec
is not evidence that all wrapper allocations are already request-reserved.
Preserve genuine Native equivalent-definition selection: matching_definition
can select the first actual recovered semantic match among duplicate IDs.
Build the IDs actually installed into the layout; do not re-register or claim
the historical Query-assigned ID is unique when equivalent definitions exist.

Before service, read the actual latest-Native Query idempotency rows with the
existing bounded entity scan and cumulative original request. Validate exact
field/intent shape, unique keys, unique nonzero outcome identities, checked
effect counts and actual immutable creation receipts. Extend the existing
`application_attempt/idempotency/recorded_intent.rs` owner decoder rather than
add another grammar. The existing intent
owner's shape validator must require its actual lowercase root digest and
ordered source, operation, scope, precondition, input, proposal and
conditional-definition slots. The root is always 64 lowercase hex; mandatory
nullable digest values are '-' or 64 lowercase hex;
scope is '-', 192 hex for the current encoding, or the existing legacy
240-hex form. Optional suffixes remain in encoder order: workflow-definition,
workflow-instance, workflow-proposal-context, workflow-captured-inputs,
workflow-transition, workflow-support, workflow-client-key, workflow-approval,
mutation-binding, correction. Present optional suffixes contain exactly 64 lowercase hex,
never '-'; absent optional meanings omit the slot as the actual encoder does.
Reject duplicates, unknown keys, reordered or
trailing data. Reuse the same borrowed validation in comparison and recovery,
without reserialization. Legacy shape admission does not grant verifiable
operation equality; preserve its existing RecordedIntentUnverifiable behavior.
Restore each actual outcome through the existing identity
frontier operation before any new outcome may be minted. Zero and MAX refuse;
MAX-1 leaves the next mint honestly exhausted. A partially advanced allocator
frontier grants no service. This is a bounded full recovery scan, with typed
owned exhaustion, not resumable paging. Leave completed receipt caches empty;
an old exact retry may return CommittedReceiptNotRetained without handler replay.

Also scan and refuse persisted unsupported Native families: actual outbox,
inbound completion and aftermath entity kinds, and all workflow entity/relation
kinds. Derive the thirteen workflow entity and twenty-seven relation kinds
from exhaustive actual `WorkflowLayoutFields` and nested output/input layouts,
without `..`, offsets, name matching or a second registry. This includes
persisted workflow definition rows because their publication lifecycle has not
been readmitted. Compiled workflow declarations remain allowed. Existing live
outbox/aftermath support is unchanged; this is a stricter cold-readmission
posture. Ordinary application principal/capability truth, activation,
completion and admitted idempotency rows remain supported.

Destination topology, extending existing owners:

```text
primary_graph/schema_layout/bootstrap_publication.rs [create: completion schema]
primary_graph/bootstrap/home_publication/
  record.rs       [create: descriptive stored record]
  codec.rs        [create: closed portable binding grammar]
  publication.rs  [create: fresh ordinary Native completion commit]
  readmission.rs  [create: selected-basis record/activation admission]
primary_graph/application_installation/home_preparation/
  native.rs, source.rs [revise: fresh-only construction]
  existing.rs         [create: existing-only concrete carriers/orchestration]
primary_graph/workflow/schema/home_readmission.rs [create: owner-derived kind inventory]
worth-query-host/src/application_home/opening/
  fresh.rs       [move: same fresh recipe]
  existing.rs    [create: actual Store-origin existing recipe]
  service.rs     [move: common real custody/transition completion]
```

Keep latest-Native idempotency admission under its existing Query attempt
owner; do not mix it into the selected-basis completion codec. Add semantic
children there as needed to preserve the 400-line limit. Commit known
successor recovery populations under their actual Query owners, without
creating empty placeholders now.

Host branches only on actual Store origin/refusal and consumes its same Home,
incarnation and lease. ProvenAbsent keeps the one fresh opportunity;
ExistingArtifacts invokes the existing-only recipe; consumed initialization
remains refused. Extend the existing owning refusal and failure-close routes
for each real preparation state. Common completion requires zero actual World
custody and consumes the Store no-pending carrier into the concrete existing
CustodyReadmitted/TransitionSettled progression before exposing public service.
Seal new intermediate factories to exact Host orchestration paths and named
certification controls; ordinary applications still import only Decl and Host.

The external Decl/Host court and existing application_home testkit must prove:
real multi-batch seed/commit/close/same-directory reopen/read/new commit; no
initializer or supplier replay; separate writer/reader processes and restored
identity frontiers; old-key exact retry with unchanged value and truthful
receipt-not-retained refusal; unchanged meaning with changed build UUID;
owned missing/unsupported completion, foreign binding, missing recovered index,
activation mismatch, original-request stop/exhaustion and retained-World
refusals. Require
mutation-sensitive observations for both selected-basis attachment consumers
and the D17 scan. No process-local cache or fabricated authority can stand in
for recovery. Keep all failure logs; run scoped courts, formatter, line caps,
boundary/context and affected compilation once the coherent patch is ready.
Update `docs/build-an-application.md`, `docs/capability-index.md` and the Host
README with existing-open semantics, ownership on refusal, stricter persisted
family posture and receipt limitation, checking examples against real exports.

Persisted Native-family negative journeys belong in the existing Store
Application court, at
`process/application/query_readmission/native_artifacts.rs`, rather than
adding Store dependencies to any other Query package. This exact file may
use the existing query-home-preparation and native-home-query-preparation
caller classes. Keep their actual configuration entries file-exact when the
fixture is implemented; no new getter, control or factory is authorized.
The Store court may add the required Host/testkit/execution leaf dependencies
under its existing D1 exception, coordinated by Root with shared lockfiles.

Create and close a genuine public Host Home, then prepare the SAME admitted
meaning and recover that directory through actual Store recovery, attachment,
schema installation and Home readmission. Borrow its actual Native through
query_finalization_parts, publish ordinary unassociated administrative Native
records under the installed registry's actual contracts, release their actual
commit snapshot, drop preparation and close the same unshared Home. Reopen
through the public Host and assert the exact family/kind/record refusal, with
an untouched successful counterpart. Independent unassociated Native history
beyond World roots is supported by the existing world_occurrence_readmission
court; do not invent World occurrences or custody to stage these negatives.

These are contract-valid Native presence adversaries, not positive Query
workflow/inbound/correction lifecycle authority. Directory policy still
refuses the unavailable workflow producer. Exercise distinct workflow-lineage,
outbox, inbound and aftermath presence checks with real rows. The exhaustive
owner-layout projection establishes the full thirteen-entity/twenty-seven-
relation roster. Do not claim independently reachable deletion-sensitive
twins for all forty workflow kinds: required endpoints/relations can make an
earlier entity refusal mask a removed relation check. Further per-kind
workflow lifecycle proof requires a legitimate owner producer design; no
synthetic policy override, binding-only Store fixture, fabricated authority,
foreign-runtime unit exception or newly exposed production door is assumed.

Baseline: `083ab6f50f`. The branch is `codex/m3-phase3-home`. The prior
`codex/m3-production-host` branch and its requirements-tracing commit
`1111d78663` remain preserved. This plan adopts that commit's distinction:
requirement identifiers belong in the requirement map, while test names state
behavior. Phase 3 and Phase 4 land together, after coordinator certification.

## Boundary review

The adversarial constraint is an owner effect acknowledged by its durable
medium while live publication remains incomplete. Neither a refused or
indeterminate answer, a memory-only operation, nor an unrelated committed
subject may authorize that subject's durable live cutover.

Current authoritative paths:

- `worth-query-host` is an audience facade. Its application-installation
  export names Execution's constructors; it currently has no home composition
  module. Applications must continue to import only Decl and Host.
- Execution's `application_installation/construction.rs` creates its own
  Relational runtime with `builder.build()`. The future host must inject the
  one readmitted runtime through an owner-defined construction port. Holding
  another home beside the current constructor would create two runtimes.
- `worth-store-relational/src/home.rs` currently owns physical opening,
  descriptor admission, incarnation advancement, recovery and the runtime.
  Descriptor binding is derived from a constructed Relational runtime.
  Physical media are qualified under the mutation lease before newness is
  tested; recovery moves the held lease into serving. Preserve that seam.
- `checkpoint_record.rs` selects `relational.checkpoint`, prepares native
  members, holds the selection fence, publishes the root and advances member
  selection. `cold_source.rs` also names this anchor. The application owner
  must ultimately select one `home.checkpoint`; adding another selector is
  not a cutover. Keep the unchanged Relational native section format.
- Current close consumes the home and may return `StillShared` after dropping
  its runtime. That is not D11's usable refused-close posture.
- Relational owns canonical commit/transition meaning, its medium trait,
  canonical encoding and artifact readmission. Store implements this port
  over its physical journals and never interprets owner payloads.

The earliest missing contract is in `durability/medium/binding.rs`:
`BindingState::settle` reduces `Committed` to `Ok(())`. A commit returns
placement/readmission data, but not a consumed durability proof. A transition
returns only unit success. `performed_stream.rs` and
`branch_transition_stream.rs` then perform live cutover. D17 requires a
concrete carried proof at this boundary, before the new home builds on it.

Search of `worth-proof/src` and `worth-foundational/src` at this baseline
found no `DurablyCommitted`, `DurablePublicationFate`, `DurableIdentity`,
`HomeIncarnation` or recovery-stage proof names from D17. Both substrate
crates are already Relational dependencies. Existing sealed owner markers and
witness minting are available; a public unit marker would not be a seal.

Independent Sol-high boundary review inspected these paths. It found that
home extraction must transfer actual lifecycle/checkpoint authority rather
than rename the old home, and accepted the publication-proof prerequisite as
a coherent first vertical slice. The coordinator confirmed this scope is
disjoint from the landed B3 data and B4's tooling work.

## First slice: publication fate to consumed live cutover

This slice establishes the existing Store-to-Relational publication contract.
It does not introduce a directory Query host or claim Phase 3 completion.

### Result and type placement

- Add one closed `DurablePublicationFate<Detail>` in `worth-foundational`,
  exported through its facade. Its exhaustive variants are `Committed`,
  `ProvenNoEffect` and `Indeterminate`. Preserve existing owner detail and
  failure ordering. Remove the separate Relational fate enum and update its
  callers; keep no compatibility alias or dead variants.
- Add plain, move-only `DurablyCommitted<OwnerPublication>` in `worth-proof`.
  Its single shallow parameter retains the actual owner-sealed publication,
  not a boolean, unit success, or independently selectable subject.
- Concrete Relational committed carriers stay in Relational: they retain live
  cold-source/readmission custody and the exact positioned subject. They have
  private construction, no `Clone`/`Copy`, and no public conversion from an
  artifact or source. Only the medium binding's `Committed` branch creates
  them. Reuse existing owner-marker sealing; add no generic authority door,
  public unbound issuer, macro or blanket implementation.
- Durable cutover signatures require the concrete Relational specialization
  and apply the subject it carries. Proof/carrier authority is consumed and
  cannot escape after cutover. Subject data may remain ordinary history.
- Memory-only publication remains an explicit, separate admission path. An
  absent medium never mints `DurablyCommitted`. Existing replay/readmission
  remains distinct from a new durable publication and never fabricates its
  witness either.

### Ordered implementation

1. Add shared fate and proof vocabulary with facade exports. Include valuable
   negative compile proofs for fabrication, a foreign specialization and
   repeated consumption, with a valid counterpart.
2. Make Relational's medium binding preserve the actual committed subject in
   the proof. No-effect still returns the same refusal; indeterminate retains
   its original unresolved subject and blocks later publication as before.
3. Thread the proof through canonical commit publication and consume it at
   the first live cutover. Bind journal readmission from that same subject.
   The stream publishes its own positioned subject through the installed
   medium and original allocation policy; it accepts no callback that could
   substitute another committed subject for the prepared route.
4. Thread positioned branch transitions through live publication and consume
   their proof at append/cutover. Keep explicit memory-only and replay paths,
   the existing reservation, canonical ordering and pre-effect checks.
   Seal the step to its live and fidelity-checked replay implementations;
   publication variants stay private to that owner. The concurrency fixture
   parks the real live step through its owned unit-test control.
5. Update the Store medium and exact existing test/testkit consumers to the
   shared fate. The initial exact identifier search found 27 Rust files:
   Relational port/binding/facade and support, two World refusal fixtures,
   one Relational UI fixture, and Store's journal adapter. This is a type
   cutover, not a fixture redesign or A2 migration.
6. Verify the coherent batch once, fix actual failures, commit with the agreed
   coauthor footer, and obtain a fresh bounded Sol-high independent review.

### Proof and verification

| Requirement | Distinguishing evidence |
|---|---|
| D2/D17 shared, exhaustive fates | Owner port and Store adapter compile with the single shared enum; no old enum/alias remains. |
| D4/D17 actual durable proof before live | Sealed concrete committed payload reaches commit and transition cutover; negative compile cases reject fabrication/foreign proof/reuse. |
| No effect and indeterminate grant no cutover | Existing durable-before-live, unresolved-fate and branch-transition controls observe original heads, journals, sequence reuse and typed denials. |
| D13 memory-only semantics | Existing memory-only publication and branch lifecycle controls succeed without a medium or durable witness. |
| World handoff unchanged | The two affected World unresolved-publication fixtures compile and their relevant custody tests pass. |
| Real Store port | A focused existing filesystem Relational-home journey commits, reopens and continues through the actual journal adapter. |

The requirement map's concrete behavior tests are:

- D4/D17 committed subject:
  `every_acknowledged_commit_is_the_artifact_the_medium_holds_at_its_position`.
- D4/D17 proven-no-effect:
  `a_refused_publication_was_offered_to_the_medium_and_moved_nothing_live` and
  `a_refused_position_is_not_consumed`.
- D4/D17 indeterminate:
  `an_unresolved_publication_installs_nothing_and_refuses_every_later_commit_alike`,
  `reopening_resolves_an_absent_unresolved_commit_as_never_committed` and
  `reopening_resolves_a_present_unresolved_commit_as_committed_exactly_once`.
- D4/D17 transitions:
  `every_transition_fate_preserves_the_durable_before_live_boundary` and
  `an_unresolved_commit_also_blocks_branch_transitions_with_its_original_fate`.
- D13 memory/replay:
  `a_memory_only_runtime_publishes_without_a_medium`,
  `memory_commit_waits_behind_a_transition_without_consuming_its_candidate` and
  `partial_replay_includes_an_intermediate_fork_without_its_own_commit`.
- Medium ownership:
  `a_forked_runtime_never_inherits_the_medium`,
  `recovery_moves_the_medium_into_the_rebuilt_runtime` and
  `an_attachment_the_medium_moved_out_of_publishes_nothing`.
- World handoff:
  `unresolved_relational_durability_retains_a_possible_effect` and
  `an_unresolved_durable_fork_retains_its_attempt_until_reopen`.
- Store port:
  `reopen_keeps_regions_cold_and_a_point_read_and_commit_demand_only_their_partition`.

The fabrication, foreign-specialization and reuse compiler fences, with their
valid counterpart, are executable `DurablyCommitted` API examples in
`crates/worth-proof/src/durable_publication.rs`.

Use the existing runners and scope discovery. The planned `loop` executable
is not present at this baseline (scoped filename search under scripts/tools
found no developer-loop entry); do not implement a substitute framework.
Tier 1 uses focused touched-owner compilation and named affected tests, not
the full Store ordinary suite or cumulative oracle. Run full boundary,
generated-context, formatting and dirty/range line-cap checks explicitly at
completion of this planned slice, not after individual writes. Record exact
commands, head, clean/dirty state, times, exit and test counts in lane logs.
Use pinned Rust 1.99.1 and committed profiles, preserve MSRV 1.94, and export
no artifact-changing settings. Never run ignored, Query-scale or worth-ui
tests. Ordinary focused process correctness runs may overlap under Root
coordination. Controlled timing comparisons require an exclusive agreed window.

### Next ownership transfer and reserved handoff

The next concrete batch is specified in
[Application home ownership](milestone-3-application-home.md). Its shared
identity, issuer, frame and recovery contracts are in
[Durable owner contracts](milestone-3-durable-owner-contracts.md).

After this prerequisite, `worth-store-application` takes physical lifecycle,
descriptor/registry and single checkpoint selection. `worth-store-relational`
retains only its medium, cold source and narrow section preparation/readmission
contracts; delete `RelationalStoreHome` rather than wrap it. Keep private cold
internals private. The future descriptor admits declared program/schema/roster
binding before runtime construction and advances incarnation before service.

D1's exact future config edits are confined to `road1.toml` dependency denials
and source/dependency allowlists: only the Host production composition module
names Store Application; other Query crates cannot depend on Store; Store
Application implements owner ports without depending on Host; the application
oracle is a leaf allowed to depend on Host and Application. This prerequisite
does not need those new dependencies or config edits.

The host then composes the one supplied, readmitted runtime, Store-proven
new/existing disposition, program/roster, trusted services and original
execution authority. Only genuine newness runs initial state. D15 refusals,
the frozen interim checkpoint door, typed transitions/discard/fencing, honest
close/checkpoints and public docs remain explicit Phase 3 work.

Phase 4 owns World durable identities, occurrences, attempt associations,
custody reconciliation and original-result retry. Reserve its owner sections
and ordering, but do not implement its catalogs or mint recovered application
history from graph/checkpoint presence in this slice.

B3's measured catalogs, compiler goldens and test-cost plan, B4's tooling,
worth-ui, broad A2 migrations and full phase/landing certification remain
outside this slice. Any shared manifest, testkit or gate overlap is coordinated
before editing; the coordinator owns integration and final landing.


## Live-original Native Complete preparation into Section

The Native producer now captures original configuration, schema, record identity,
branch transitions and component obligations under the same configuration and
settled publication epochs. Physical source protection follows their release.
It verifies the original full canonical prefix, folds the sealed resolved inputs
for every covered commit, and emits complete schema and required component
images directly into the existing CHK18 grammar. The strong Complete result and
its weak preparation service feed the actual Store Section preparation, selection
and acknowledgement engine. Ordinary checkpoint values have no promotion door.

Positioned agreement compares the format's complete authoritative projection,
including the original semantic result. Live diagnostic entries and derived
caches follow their existing separate contracts. Both resident agreement passes
and cold-entry digesting use the invoking request, with 64 KiB work/stop checks;
no comparison regenerates the original frame. Intrinsic output backing and its
actual shared controls retain their admissions through their final aliases.

The verified scope comprises 90 Native controls, 16 Store controls and four
external checkpoint regressions, all passing with zero ignored cases. These
include genuine partial updates, resolved merge/cache twins, snapshot/replay
pinning with an actual historical external component lease, unused full schema
constraints, source-forgery refusal, request limits and same-selection ACK retry.
Boundary/context, full five-workspace metadata, formatting, line caps and scoped
function dispositions accompany this source batch.

Serving recovery and Directory close remain required successors. Recovered
histories without original producer schema/configuration/birth DATA still refuse
this live-original preparation path; deferred cache capture also refuses until
its real owner can supply the original generations. Region ACK applies to the
same rebuilt cells held by the output, while canonical ACK retains the original
canonical cells. The original serving region/source/business graph is still
owned. This preparation qualification does not establish physical `Closed` or
D11. Complete Home source admission and mandatory Query/Host maintenance wiring
must consume genuine owner evidence before the production reopen/close courts
can close Phase 3.

## Fresh Home source admission before first service

This is a prerequisite of the existing Phase 3 checkpoint and lifecycle claim.
It does not change the roadmap's joint Phases 3–4 landing or qualify Complete
checkpoint reopen. The physical Store owner already distinguishes actual
`ProvenAbsent` initialization from `ExistingArtifacts`; Native must consume that
closed route without replacing the runtime whose original birth and initial
installation it is required to retain.

### Decisive adversarial proof

The proof composes three real, distinguishing owner boundaries. Native's
original-source court installs a genuinely unused schema constraint and checks
the SAME runtime/root/schema pointers plus retained birth/install inputs before
its first Complete capture. Store's actual filesystem Home court checkpoints
at configured C0, then makes a real write and lawful forks to exercise the SAME
manual/count/byte checkpoint engine and acknowledgements. The production Query
Host court opens the actual authored schema with an unused physical relation,
performs its genuine bootstrap/index registration, makes a manual Complete
checkpoint, then advances real business state across the automatic threshold.
Observe actual reads, selected generation/frontiers and settled cadence; an
ordinary-capture substitute, dropped original data or missing callback must
turn these courts red. Native's schema-image oracle checks the otherwise unused
constraint independently of canonical bodies.

The public Host opener necessarily commits program activation and principal
seed before returning, so its checkpoint is post-bootstrap and must be named
honestly. It exposes no pre-bootstrap Home checkpoint port to the initializer.
Do not introduce a test-only opening callback, observer authority or extra
production checkpoint merely to describe that public journey as C0. Native
identity and Store C0 proofs establish the earlier boundary, and the real Host
post-bootstrap Complete fold must still require that preserved original data.
Physical close/reopen remains separately qualified, with existing close and
recovery oracles unchanged.

Contrast a real existing empty Home with neither selected checkpoint nor
canonical/transition tail. Reopen it through physical `ExistingArtifacts`:
the initializer must not run and generic recovery must still replace Native.
Missing original producer data must retain its existing typed refusal; empty
coverage is never evidence of newness. Keep the existing recovered-Empty
missing-original negative control. A genuine foreign runtime receipt, foreign
protected source, selected checkpoint or nonempty source presented at the
fresh seam must refuse before fresh admission or World construction.

Consult the original request before source work and again before publishing
admission. Cancellation, deadline and allocation refusal retain the same
actual unshared owner or already-admitted phase; they create no replacement
request, allowance, runtime, initialization opportunity or fallback recovery.
After source refusal, no World attachment or initializer effect is permitted.
Receipt storage is fixed-size and borrows/shares the actual producer data;
source validation follows the existing protected read protocol. It must not
copy the history, schema registry or serving graph. Ordinary advancement gains
no history traversal. Exact factory-caller enforcement must reject an outside
fresh-source caller, alongside a valid real Store counterpart.

### Product and authority decision

`RecoveredRelationalRuntimeAuthority` keeps its current meaning: actual images
reconstructed through an admitted recovery plan. Do not widen it, convert a
fresh receipt into it, fabricate `RecoveryOutcome`, or change generic rebuild's
discard of isolated construction birth.

Native owns a distinct, non-serializable `FreshRelationalSourceAdmission`.
Only the closed `admit_fresh_home_source` Native door constructs it, against
the actual exclusively borrowed attached runtime and its actual protected
source/medium. It verifies source binding, no selected image or durable tail,
no unresolved/accepted durable operation, the original live initial root and
the owner's retained birth/installation inputs. Source emptiness is a
consistency condition, not the authority to choose this door. Store's private
physical `ProvenAbsent` custody and the exact caller seal choose the door.
An origin enum, scalar coverage, caller boolean or descriptive digest grants
no such capability.

The receipt carries the real runtime identity, initial axes and actual native
schema/installation data needed to validate this admission. It may share the
existing schema authority Arc and downgrade the actual live root, without
allocating or retaining a second operational graph. The live initial component
must remain the SAME root Arc, not another Empty root with equal descriptors.
No recovered-root inventory is needed by the fresh World route: subsequent
query-root selection observes that same live branch and uses the existing
durable-bootstrap operation. Historical Empty reconstruction and general
recovered original-data availability remain separate owner obligations.

The receipt proves only admission of that exact original source/runtime. It
does not authorize initialization, arbitrary recovery, World history,
publication, Complete promotion, physical close or D11. Query's fresh
finalizer validates its SAME-runtime initial installation against this receipt
and the real retained inventory, then registers indexes by its existing
initial-installation contract. It retains its original installation receipt;
there was no checkpoint restore, so restore-work is absent directly.

Store Section holds one private concrete sum of actual owned data:
`ReadmittedRelationalSource::Fresh(FreshRelationalSourceAdmission)` or
`Recovered { authority: RecoveredRelationalRuntimeAuthority,
outcome: RecoveryOutcome }`. This is mutually exclusive source custody, not
a strategy/mode, generic authority wrapper or a new phase. The unshared
readmitted section always owns one variant; shared attached sections may
have no admission yet. Replace independent recovered/outcome options with
this coherent payload. Share, close, close refusal and resume move the SAME
payload intact. The existing `worth-proof::RelationalReadmitted` phase stays.

The Section fresh entry is `admit_fresh_home_from_source`, distinct from the
Native issuer's name so name-based reach cannot authorize an earlier layer.
Section getters are `home_fresh_query_finalization_parts` and
`home_recovered_query_finalization_parts`; Application getters are
`fresh_query_finalization_parts` and `recovered_query_finalization_parts`.
These named getters return typed source-mismatch
denials (`FreshSourceAdmissionRequired` or
`RecoveredSourceAdmissionRequired`), never panic or unwrap an assumed payload.
The Host's getter refusal owns both the unchanged prepared Query and actual
unshared Home, before Query finalization has consumed either. It is distinct
from an already-consumed Query finalization refusal.

Both the exclusive Query opening recipe and direct Store convenience opening
route through the private physical origin. Fresh custody enters the distinct
Section/Native admission and then shares the same owner. Existing custody
uses current source admission/recovery, including when all coverage is empty.
The existing initializer opportunity is consumed at its present point, after
Native admission; this work never moves the initializer earlier.

The direct convenience `readmit_relational` reuses `attach_relational` and that
same private Home source-routing core. Its result becomes an owning
`RelationalHomeOpenRefused::{Attachment(RelationalAttachmentRefused),
Readmission(RelationalHomeReadmissionRefused)}`. The attachment refusal already
owns the real before-construction or failed-construction phase; no redundant
before-attachment marker is needed. Testkit and oracle callers retain that
actual refusal in their own concrete error result rather than extracting a
cause and dropping the Home. This is a coherent caller cutover, with no second
opening algorithm or legacy cause-only fallback.

The common Home phase must not carry the close-only failure payload inline.
The default-stack filesystem trial measured a 10,288-byte `HomeContext` and
9,872-byte checkpoint denial, with large by-value phase/result frames. Both
Store and Host fresh journeys overflowed before Native admission; a private
stage diagnostic placed the Store failure inside `admit_home`, after physical
recovery. This establishes the layout problem, while the unchanged-stack
filesystem rerun must establish that the repair closes the observed failure.

Use one concrete private close-checkpoint result backing, allocated once as a
Box under the ORIGINAL opening request before descriptor effects. Quote its
actual layout, including its intrinsic LAST reservation field, before allocation.
Its result starts absent; the same backing moves through Home phases and
recovery custody and is filled or replaced without another allocation admission.
Do not allocate it lazily at close: the existing release-after-memory/work-
exhaustion contract must remain true. This backing owns only its fixed storage;
cached results retain their own child owners/admissions. It is not a generic
Context wrapper, shared service, error ledger, new phase or replacement result.
Preserve borrowed checkpoint-result observation and existing actual public
Closed/refusal data. Any final move out releases the backing's reservation only
when that physical Box is gone; it does not release still-owned child backing.

Keep ordinary fresh phases small without raising thread stacks, changing build
profiles or weakening a runtime oracle. Retain the diagnostic evidence and
remove every temporary marker from the final source. Require the same Store
and authored Host filesystem cases under unchanged defaults, plus affected
checkpoint/close request and exhaustion controls, before qualifying this repair.

The unchanged-default follow-up also reaches a distinct checkpoint stack
failure after Home admission. Actual debugger unwinding identifies repeated
by-value preparation results in Execution scope carriers, and a 572,792-byte
selected-read frame combining open, payload copy, EOF transfer and admission.
Fix this at those phase boundaries: retain the actual preparation result in
the invoking caller's output slot while a named private refusal carries its
status through Execution; preserve the original domain error and final scope
controls. Only after preparation succeeds, move the same Native/World owners
into the existing funded Home attempt and return its actual driver/input.
Do not introduce another allocation, allowance or checkpoint owner.

Separate World capture, Native capture/previous-selection validation and frame
assembly so later-phase locals do not occupy the stack during earlier source
reads. Likewise, selected reads separate opening, bounded copying, consuming
verified-EOF handoff and manifest admission. Preserve the same reader, table,
payload grants and refusal custody through these calls. The unchanged C0 and
authored Host controls remain the runtime oracles. Original cancellation now
refuses at prepaid-slot admission before descriptor encoding; retain both
serial and leased no-descriptor/no-commit assertions. The additional
`prepaid_close_storage_refuses_before_descriptor_effects_when_opening_memory_is_unavailable`
control protects that same pre-effect boundary under actual zero memory.

The physical append refusal carried by anchored-record and ordered-journal
failures must exclude successful preparation. Their shared append producer
projects the five non-success categories into the concrete
`PhysicalMutationPreparationRefusal`, preserving each original typed payload
with an exhaustive match. No refusal can hold `PreparedPhysicalMutation` or a
successful terminal fact. This compiler-enforced cutover removes a 9,856-byte
success-capable payload from ordinary Home failures without allocating error
storage, changing persisted formats, or weakening physical preparation itself.

Host opening likewise keeps the actual initialized result beside Store's
callback, which carries completion only. Fresh opening returns from actual
source preparation before invoking initialization, and completes World/service
attachment after initialization. These steps consume the existing concrete
Query stages; they introduce no initializer opportunity or substitute owner.

Boundary ruling for the existing Store Query courts: migrate the exact
`query_readmission/bootstrap_publication.rs` caller from its former generic
source-finalization getter to `application-home-fresh-finalization`, and the
exact `query_readmission/native_artifacts.rs` caller to
`application-home-recovered-finalization`. Both files already belong to the
closed Native/Home Query preparation audience. This preserves those same
callers with narrower source provenance; it admits no new file, family or raw
source constructor. Their actual Home still supplies the sealed fresh receipt
or recovered authority. Observational existing-World attachment receives the
same Home's real funded checkpoint operation through Query's maintenance port;
it does not gain a no-op advancement path or qualify checkpoint reopen/close.

World receives a distinct closed `admit_fresh_world_home` entry with the actual
Native runtime and its fresh receipt. It validates that affinity before any
World construction, reads the real protected World source and requires no
selected checkpoint, branch, open, occurrence, disposition or other durable
World record. It shares only the private execution/input/owner-construction
algorithm with recovered admission. It never invokes recovered associated
effect discovery or recorded-component reconciliation on the fresh route.
Recovered World admission continues to require its real recovered authority.
Neither path fabricates the other's result or falls back to the other.

The existing no-pending-owner service gate also consumes this actual source
distinction. Its fresh branch keeps the SAME physical transition-journal
inspection and typed pending/unsupported/refused outcomes; its recovered
branch additionally retains the recovered codec-transition check. The receipt
describes initial source admission, so do not revalidate initial-root currentness
after lawful bootstrap publication has advanced it. Neither receipt presence
nor a source-kind flag substitutes for the actual current journal inspection.

### Destination and implementation order

The stable axis is source provenance at its owning runtime, with cross-owner
sequencing in Store Application. Existing facades remain stable; the new
receipt is exported only through Native's durability facade. Its operations
and the Section/World fresh doors are closed by the existing named-factory
caller mechanism, not exported through consumer declaration/Host facades.
Root owns exact caller metadata changes and their negative enforcement proof.

```text
crates/worth-relational/src/
  durability/authority.rs                         existing; export fresh receipt
  durability/authority/fresh_source.rs            new; real Native source admission
  runtime/initial_schema_installation/
    fresh_admission.rs                           new; SAME installation validation
  facade/durability.rs                           existing; explicit owner export
crates/worth-runtime-world/src/lifecycle/builder/
  home_readmission.rs                            existing; recovered entry
  home_readmission/fresh_source.rs                new; empty protected World admission
  home_readmission/construction.rs                new; private actual owner construction
workspaces/worth-store/crates/worth-store-relational/src/section/
  unshared.rs                                    existing; first-owner consuming routes
  readmission.rs                                 existing; recovered source route
  readmission/fresh_source.rs                     new; closed original-source adapter
  readmission/source.rs                           new; mutually exclusive source DATA
  close.rs                                       existing; move actual source payload
workspaces/worth-store/crates/worth-store-application/src/
  open/mod.rs                                    existing; direct origin routing
  close/checkpoint_result.rs                     new; prepaid fixed cached-result storage
  open/readmission/transition.rs                 existing; actual journal admission
  relational_preparation/{unrecovered,unshared}.rs existing; origin/getters/owning denials
  world_section/readmission.rs                   existing; exact owner-source dispatch
  world_section/readmission/query.rs             existing; same Query composition route
  world_section/readmission/transition.rs        existing; source-aware service gate
workspaces/worth-query/crates/
  worth-query-execution/src/domain_computation/primary_graph/
    application_installation/home_preparation/native.rs
                                                  existing; real fresh finalizer
  worth-query-host/src/application_home/
    opening/{fresh,existing}.rs                   existing; typed getter custody
    refusal.rs                                   existing; exact pre-finalization denial
    cleanup/opening.rs                           existing; consuming refusal cleanup
tools/boundary-check/
  src/seal_rules.rs                              existing; fresh factory names
  src/seal_rules/factory_ownership.rs             new; owner/class taxonomy
  config/certification-construction-callers.txt   existing; exact participating callers
```

The fresh Native owner has source admission and receipt validation children
as that family grows; recovered source reconstruction stays beside it, not
inside it. World fresh source admission owns its empty-source failure and
recovered admission owns reconciliation; their private construction child
contains only shared actual construction. Section source DATA is separate
from I/O admission and close effects. No generic helpers bucket, registry,
history-policy switch, trait-based authority tower or placeholder module is
part of this destination. Complete Home source admission and actual serving
graph retirement are committed successor responsibilities at their existing
Native/Section source owners and consuming Query/Host close owners; neither
requires reclassifying this fresh receipt or moving these facades.

Implement in dependency order: (1) Native real original-source admission and
same-installation validation, (2) Section data custody and private Store origin
routing plus Query fresh finalization, (3) distinct World fresh entry and both
real Store composition callers, (4) exact seals and genuine fresh/old-empty
runtime evidence. These form one vertical slice and remain unqualified until
the production callers compile together and the distinguishing courts pass.
Each owner first records its finite boundary-derived edit plan. Preserve all
current local work; integrate the already-qualified lower baseline before
building the Upper composition. Do not remove `CompleteNativeRequired` at
selected Home source admission as a shortcut to this proof.

### Continuing documentation and acceptance

Revise this lifecycle plan and the durable-owner contracts for platform
contributors: distinguish original fresh admission from recovered source
reconstruction, document the fixed DATA and explicit refusals, and keep the
remaining reopen/retirement obligations visible. Revise `docs/capability-index.md`
for application developers to describe actual Host behavior and its qualified
limits; compare every capability statement with the real facade and runtime
evidence. Revise Native and World owner README contracts if their admission
APIs are documented there. Do not add a competing handoff or closeout guide.

Run focused affected owner/production filesystem courts, source-affinity and
factory-seal negatives, the existing recovered-Empty negative, and the changed
manual/automatic checkpoint controls. Run pinned/configured owner compilation,
formatting, scoped line caps, boundary/context checks and full affected metadata
graphs under the existing build policy. Reuse unchanged closed evidence and
record source hashes and actual results. Fresh admission does not close
selected Complete Home reconstruction, original serving graph retirement,
callback/provider release, D11, the joint landing oracle or the milestone.


### Default-stack qualification and selected-source stop

Fresh-Home admission qualification includes running the existing filesystem C0
and authored Host checkpoint/automatic-maintenance controls directly on the
normal libtest thread stack, in addition to the owned nextest lane. The
configured nextest stack alone concealed an overflow. Native candidate
preparation transfers its existing history proof and the same diagnostic,
request-control and retention custody into a distinct registration frame.
Early admission/draft frames return before later publication locals exist;
this requires no new allocation, authority or memory allowance.

Selected Complete reopening remains refused by `CompleteNativeRequired`.
The selected-source boundary reviews in
`target/complete-home-selected-source-boundary/{brief,original-input-refinement}.json`
identify the missing persisted original-input chronology: zero-write
birth/installation, transition schema attribution and per-canonical original
mutation configuration. CHK18's full schema union does not establish those
roles. Canonical format 4 and transition format 1 also lack these original
inputs. Removing the gate requires an explicit durable-owner and versioned
wire-placement ruling. Fresh authority cannot substitute for reconstructed
original evidence. This is a stop on reopening, not acceptance of Phase 3;
registered-source retirement and the remaining audited lifecycle obligations
are still outstanding.

### Phase 3 master integration boundary ruling (2026-11-02)

The existing Store application courts retain their exact construction authority
when migrating from `query_finalization_parts` to the split, typed owner doors.
`query_readmission/bootstrap_publication.rs` is an exact caller of
`application-home-fresh-finalization`; it consumes the actual fresh Native and
admission from its unshared Directory Home before Query seeding and World genesis.
`query_readmission/native_artifacts.rs` is an exact caller of
`application-home-recovered-finalization`; its existing artifact/basis courts
consume that same Home's actual recovered Native, authority and outcome. Both
paths are under `worth-store-court-application/tests/worth_store_court_application/
process/application/`. These are migrations of the already-ruled courts, not a
new factory, wrapper, production audience, wildcard caller or forged authority.
The sealed calls remain in their original files; no child module inherits them.

Query 9.19 integration preserves one managed Source registry through installation
and Home attachment, and the original predicate owner across Memory and Directory
publication. A returned preparation-pending terminal boxes its original checkout
guard so ordinary result enums do not embed the full retained Native/World
preparation on every stack frame. The existing recorded-correction consumer's
explicit 8 MiB thread is the reproducer; its stack size and the repository's
configured stack size are unchanged. Guard handback, recovery and destructor
ordering retain their existing semantics.

### Merged default-stack candidate execution boundary

On merged source `224c00ae83`, raw libtest with `RUST_MIN_STACK` absent
completes fresh initialization and the first authored checkpoint, then overflows
on the ordinary business mutation. Windows x64 unwind metadata at the actual
overflow attributes 1,596,504 bytes to Publication's `prepare_staged_candidate`
and 273,480 bytes to its `prepare_in_program_using` caller. The captured stack
and complete symbol names are in
`target/lane-evidence/integration-baseline-stack.log`.

Candidate preparation separates actual handler execution from completed-program
binding. `preparation/handler_execution.rs` owns authorization, idempotency,
pending-source consumption and the handler. Its move-only execution result
carries the actual completed program, original identities, extension, work and
workflow authority. `preparation/completion.rs` consumes those exact values for
original-input retention, workflow binding and signed creation. Handler and
Native preparation frames return before completion's program-move temporaries
exist. The phase split adds no allocation, authority, execution allowance, alternate
preparation algorithm or changed publication order. Keep all existing terminal
outcomes and validate ordinary and external custody through the production
courts. Raw default-stack qualification covers fresh Host startup, the authored
Host checkpoint/business/automatic-maintenance journey and Store's C0/manual/
automatic checkpoint control. This integration does not qualify Complete
reopening, retirement, transitions, discard, receipts or the joint landing.

The retained-payload measurement identifies the owning cause behind ordinary
carriage: a complete read set is 30,288 bytes, including a 25,416-byte operation
admission. Its authorization-basis enumeration embeds capability lifecycle
predecessors even for Conventional operations. Approval, close and mandatory
review bindings occupy 7,032, 12,776 and 18,416 bytes respectively. Compiler
layout diagnostics are retained in
`target/lane-evidence/integration-{carriage,admission}-layout.json`; the compile-only
probes were removed and produced no runnable artifact.

Only these three large lifecycle variants retain their original binding in
unique backing. Admission reserves that backing against the original request
before allocation and retains the same grant beside it until consumption or
drop. The binding allocation drops before its grant. An unfunded or stopped
admission returns the original requested/approved/mandatory custody with the
actual typed stop; replay preserves that stop as well. Conventional operations
allocate nothing for this lifecycle. This is a runtime-custody representation
change, not a receipt-format, replay-authority or capability-lifecycle cutover.
Handler-plan preparation also uses an Infallible completed value on its stop
report, making a completed candidate impossible before the handler runs.
