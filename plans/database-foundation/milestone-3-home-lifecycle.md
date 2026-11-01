# Application home and host lifecycle implementation

Specification: [Milestone 3](milestone-3.md), Phase 3, D1–D17; the Phase 4
handoff and shared roadmap requirements also bind the work.

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
