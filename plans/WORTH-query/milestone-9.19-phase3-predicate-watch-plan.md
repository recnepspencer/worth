# 9.19 Phase3: retained predicate watches and selected-image publication

## Boundary brief (completed before implementation plan)

HEADac5892cabb is clean. All six static performance repairs plus current-live
churn are qualified, but no retained predicate axis exists. ManagedSelectionLease
retains the actual installed Parent, scope, Source/root custody, original session
proof and received coverage. The Native Source service has genuine before/after
rows, changed occurrences and same_installation; it runs before invariants and
publication and must not wake readers. Source dependencies remain conservative.

The existing derived_view/DependencyIndex indexes entity/aspect/field/adjacency
facts, and registry.prepare_publication scans legacy Weak views. Managed indexes
are separately registered in index_directory and publish material through Native
companion cells; they are not those Weak views. Current managed index lookup keys
serve a known runtime/incarnation/principal/scope/Parent, not a Source->predicate
query. No current inverse probe/typed changed-row influence contract exists.

The existing Live entrance admits only a scope identity initially and delivers
one business-emission target. It cannot represent a complete Parent query watch
or supply complete initial coverage. LiveDeliverySource does already own bounded
batches, real ProductBranchIdentity+incarnation partitions, observed product
leases and reservation/ACK/overflow/close. Reuse that owner with an internal
whole-Parent cause; do not invent an ApplicationEffectRef or business payload.
PreparedViewPublication is already retained across managed_unpublished and
applied after authentic World publication/adoption. Carry prepared watch effects
through that same custody, never publish from Native preparation.

An exact race exists: a preparation can snapshot postings before a new opening
read registers, then publish afterward. Rechecking the old Source twice does not
close it. Arming the World-owner/before-commit fence must happen before Native
proposal validation: proposal_round prepares Sources during transaction.validate,
not the later prepare_validated_proposal door. The actual Query entrance is
provider/invariant_execution/relational_validation::validate_relational_candidate,
before entering the runtime borrow. Keep the unique fence owner through later
receipt/touch admission and retain_validated_candidate, which can still refuse
without effects. Opening reads register atomically against that fence after a full
admitted Parent read: they register before preparation or refuse/retry; a read
already underway when arming occurs cannot slip through. Native-effect/World-
pending retains the actual fence; domain/cancelled no-effect preparation releases
it. Wrong owner/before/after or untracked external adoption yields explicit
resync-unavailable, never a retargeted or complete-empty success.

Native currently receives no World identity. Use an explicit per-preparation
context port and the existing actual NativeSourceInterpretationIdentity, not TLS,
branch-name inference or canonical RootPayload attachments. A Native-issued
one-preparation identity binds a Query-private child context; Scope loans carry
that same identity. Symbol admission may retry proposal rounds: an explicit
per-validation factory creates an independently bound child for each real Native
interpretation. Native retains only Weak receipts, never arbitrary strong payload Drop under its
runtime borrow. The caller retains strong children through the entire invocation,
promotes the exact successful child after both retry owners finish, and retires
abandoned children outside the runtime borrow. Each child has separate funded
metadata; the outer owner never accumulates their wakes into a publication batch. The unique outer
Query fence owner survives rounds and is never delegated to Native Arc lifetime.
A captured child cannot be rebound to a new preparation. The
context is metadata, not READ, Source or publication authority.

ExecutionReductionTree has fixed PartitionIdentity order, not arbitrary probe
order. A Query-owned generic AVL is the inverse index; ordinary allocation and
record ownership stay at the Query storage owner. One WatchSlot->Node record map
embodies the authoritative active set, with secondary Source/resource routing
axes. Domain hooks may order immutable admitted probes, derive own summaries and
prune exact child enclosures; they cannot choose which watches exist or emit IDs.
Prefix uses framework-derived ordered extrema and exact UTF-8 prefix reasoning;
box uses the exact numerical union. Actual before OR after row matching validates
surviving predicates, including unchanged-membership projection/order changes.

Persistent SDK-map removal allocates copied paths, so it cannot be hidden in
requestless Drop. Use the existing funded mutable BTree owner pattern (as in
ManagedIndexDirectory) for the narrow record backing. By-slot parent links permit
allocation-free deletion/rotations without comparisons or Probe loans. Preserve
WatchSlot->actual immutable Probe identity on two-child deletion by transplanting
the actual successor slot/node. Summary repair is a restricted, explicitly
qualified Copy algebra: no allocation, dynamic provider state, Source/Probe loans,
Query/SDK reentry, I/O or panic. Audit the actual prefix-unit and fixed Box2 union
implementations. Copy/infallibility alone is insufficient. Demand Drop has no
request budget or transferable/prepaid work claim; it performs structurally
machine-bounded release. Move retired payloads/grants out and drop outside the
predicate gate; retire empty backing before its last grant.

## Intended API and contract

Add a binding-aware managed-watch entrance beside open_index in Publication and
Host. A declared managed Parent permits the Live lane without a business cause;
ordinary business Live validation stays separate. Initial opening executes the
full admitted Parent and retains its received complete coverage. Return that
initial complete OneShot result together with a managed watch lease. Subsequent
next calls require fresh phase/principal/request and read/disclosure admission
before inspecting cause/availability metadata, then deliver the full Parent
through the same original-request bounded read/projection and Live ACK owners.
No cached result becomes current READ or mutation authority.

The first qualified matching profile is a complete, Native-maintained Parent
with the same canonical Source projection. Actual prefix_definition additionally
uses enabled as an admitted root equality parameter; Source projects enabled.
Compile an explicit ConservativeProjectedRefinement posture when every extra
parameter is consumed by a root predicate and each predicate's exact physical
field/scalar meaning is projected at Source root. Retain the whole parameter set,
route provider-candidate before OR after for every actual projected-row edit,
and apply full fresh Parent refinement/order/projection at delivery. False-positive
unchanged full-Parent deliveries are explicit; locality applies to disjoint
candidate probes, not differently refined queries sharing a candidate prefix.
Box has DirectSourceProjection. Unused parameters, relation predicates and other
untracked inputs refuse. Exact refinement routing remains later Phase3 work. Installation
derives a matching contract from the existing actual Source/Parent/probe/provider
version binding. Optional typed inverse-provider configuration binds to that same
installed object; unsupported/candidate-only/public-cache-only profiles refuse
explicitly. This is a supported-profile first vertical, not Phase3 completion;
protected/refinement/continuation expansion remains required afterward.

Representative DX (exact facade type details follow the existing bound request):

    let (initial, mut watch) = request.query(find).watch_index::<Index>()?;
    // A fresh request/phase/principal is supplied for each next delivery.
    // Pending, complete full-Parent delivery, resync-unavailable and close are
    // distinct outcomes; a business effect binding is not required.

## Ordered implementation plan

1. Installation/definition boundary: compile the exact matching correspondence,
   Source projection support and existing provider version into the installed
   access contract. Allow the managed whole-Parent Live meaning only for a managed
   selection; retain normal business-cause checks. Add installation hostile/
   lawful twins for wrong probe/Source/Parent and explicit unsupported profiles.
2. Provider/record owner: add the optional typed predicate algebra and generic
   framework AVL under managed_index/predicate. One actual immutable raw admitted
   probe record owns original-funded storage; controlled decode sessions lend
   typed values during comparison/routing. Do not retain a parse grant through a
   copied value or clone a move-only SDK array. Order equal probes by full genuine
   WatchSlot. Framework maintains children/parent/height/extrema/derived summaries
   and emits only reachable active slots. Actual capacities/headers/BTree cells
   are funded before insertion. No geometry/string switch in Query.
3. Implement both inverse algebras in the existing query-agnostic value/adapters:
   prefix exact ordered-interval/prefix comparison (all UTF-8 byte boundaries),
   box exact enclosure union/overlap and existing spatial ordering. Restricted
   summary combination is a qualified fixed-data release algebra. Add fast pure
   independent membership cases before a framework build; no second storage
   fixture or synthetic Source factory.
4. Registry and Native handoff: add Source/program/scope/World owner posting axes
   under the existing registry/dependency owner; do not enumerate index resources
   or principals to discover watches. Arm the World/before fence before actual
   Native proposal validation (before transaction.validate, outside runtime
   borrow), issue each child through the explicit validation factory and bind it
   to the genuine Native interpretation identity, route genuine changed before/after rows and explicit availability/
   retirement dispositions. Prepare only metadata receivers/slots and retain the
   exact expected after binding in PreparedViewPublication/unpublished. Context
   ownership, not arbitrary Arc lifetime, releases no-effect fences.
5. Live/public cutover: add internal whole-Parent cause IDs to real Live batches
   without faking business emissions or changing emitted-effect counts. Actual
   after-World settlement transfers prepared IDs into the existing reservation;
   no recomputation after World succeeds. Initial full read and registration join
   the fence atomically; construct/close real Live basis/subscription with failure
   cleanup. Next uses fresh admission, full Parent projection, interruption and
   actual ACK. Close and final runtime disposal remove exact owned postings and
   release all grants outside gates. No unused production scaffold is accepted.
6. Real public finite evidence on the existing maintained Catalog/geometry world:
   empty future entrant, removal, actual enabled-toggle entrant/removal twins,
   boundary/equal-value/order changes for BOTH
   providers; retained old initial result; unrelated probes independently varied
   and never evaluated/woken; same-image prepare-before-register deterministic
   race; domain/cancelled preparation and Native-performed/World-pending do not
   deliver; matching adoption does; close and foreign runtime/incarnation/alias
   isolation. Use existing authoring and genuine SDK/World receipts. Reuse
   unaffected lifecycle/authorization tests rather than a combinatorial matrix.
7. Run the smallest value/installation tests, then the finite public watch courts
   and affected existing provider/Live families. No large50k build is required
   for this first watch contract. Default Rust1.99.1/sccache/shared target only, no
   restarted builds. Mandatory boundary/context, fmt/whitespace, dirty400-line
   caps, function scrutiny, compiled docs and independent qa-loop/qa-tests/
   code-quality-qa gate the coherent slice before commit. Update application
   guide/capability index with complete versus unavailable outcomes and limits.

Keep all files <=400 lines and separate predicate topology, raw probe retention,
registration fence, Native context, prepared publication, cause transport and
public opening/delivery/cleanup by responsibility. Preserve existing conservative
Source dependencies and ordinary consumers; Phase4 alone may issue narrowed
footprint authority. No private SDK adoption or Phase3/4/5/6 completion is claimed.


## Working implementation checkpoint (not a completed slice)

HEAD remains ac5892cabb. Phase3 work is deliberately uncommitted until the real
watch vertical clears independent review. No build is running at this checkpoint.

Implemented and compiled:
- Installation managed_index/matching compiles DirectSourceProjection versus
  ConservativeProjectedRefinement from actual compiled physical routes and
  tracked root predicates/parameters. Access meaning carries the descriptive
  result. Source remains complete/conservative. Candidates and unsupported shapes
  refuse. Real maintained Catalog prefix and box entries qualify their respective
  postures; the public membership court asserts both actual installed entries.
- Declaration admits managed whole-Parent Live without inventing a business cause;
  the ordinary cause-less Live denial remains. No public watch entrance yet.
- Execution managed_index/predicate owns the optional typed pure inverse trait,
  restricted static summary algebra, one funded mutable AVL record map, exact
  successor-slot transplantation, extrema and aggregate repair, requestless exact
  removal returning payload/grant custody, and final-backing-before-final-grant
  disposal. Ordinary paid insert/visit call the same locate/attach/walk mechanics
  checked by pure owner tests. Failed insert arguments and whole-tree disposal
  still require caller-side ownership outside gates at integration.
- Existing query-free value owner has exact inverse UTF-8 interval reasoning and
  exports existing exact box Morton order. Both existing entry providers implement
  the new algebra. Unit prefix summary and fixed Box2 union are release-qualified.
- Native optional validate_branch_transaction_with_source_context borrows the
  factory through both retry owners. Immediately after actual interpretation
  issuance, bind_context obtains a Weak child receipt. Native stores Weak only;
  Source scope and successful ValidatedRelationalProposal lend metadata plus the
  genuine NativeSourceInterpretationIdentity. Query has not wired a factory yet.

Evidence (logs outside repository, C:/forge_workspace):
- query919-predicate-tree-pure.log: current pure AVL3/3,0.04s; no SDK/runtime roots.
- query919-predicate-prefix-value.log: inverse interval2/2, independent UTF-8
  prefix enumeration and original callback stop.
- query919-predicate-live-declaration.log: managed/ordinary Live twins1/1.
- query919-predicate-installed-profile-recheck.log: Installation managed_index4/4,
  including direct/conservative, unused-parameter and unprojected-field twins.
- query919-predicate-public-profile.log: actual maintained membership court1/1,
  0.34s; both adapters compile through Host and both actual profiles asserted.
- mandatory boundary-check and agent-context pass; dirty caps36Rustfiles pass;
  function scrutiny16advisories/0scanerrors; diff whitespace clean; existing
  compiled_example_drift::serial::compiled_examples_match_consumer_documents1/1.
  The initial new source-court construction was corrected, not exempted: private
  AVL tests now use only algorithm data. Request/authority evidence is still owed
  by the real watch courts. Default Rust1.99.1/sccache/shared target only; no live
  build killed/restarted. Do not repeat these builds until integration changes
  justify them; no large Source court is required for this vertical.

Independent reviewer audited the profile correction, weak receipt custody,
actual AVL/transplant/disposal mechanics and both pure provider algebras. No new
mechanics defect remains. This is not watch/World or Phase3 completion clearance.

Immediate remaining implementation boundary:
1. Optional inverse configuration must share the actual provider object and exact
   installed association. Generic typed root adapter retains the raw admitted probe
   and complete Parent parameter context with their real original grants; decode
   loans never escape their parse-grant invocation. No provider/domain switch.
2. Add actual Source/program/scope/World owner posting axes under existing registry.
   Lookup cannot enumerate index resources/principals or all retained probes.
3. Arm unique Query fence owner before validate_relational_candidate enters the
   runtime borrow. Factory retains separately bound strong children through both
   retry owners; promote only final accepted proposal metadata after runtime borrow.
   Keep fence owner through Query receipt/touch checks, approved attempt and commit;
   those can still reject without effect. Retire abandoned children outside gates.
4. Route genuine changed before OR after rows during Source preparation, including
   unchanged candidate membership with enabled/projection/order edits. Native
   prepares metadata IDs only. Keep exact after binding and unique fence owner in
   existing PreparedViewPublication/ManagedUnpublishedAttempt. Actual World apply
   joins the existing reservation/batch; Native-only never delivers. Missing or
   expired context/untracked movement with active watches is explicit resync.
5. Existing managed receiver already selects full actual Source/coverage, runs
   admitted bounded full Parent and finalizes one-shot. Reuse that path with an
   internal capture before final consuming terminal, while genuine selection lease
   and plan still exist. Initial full read+registration must atomically join fence;
   failure cleanup outside gates. Managed Live uses internal whole-Parent IDs, not
   fake effects. Fresh bound request/phase/principal admission precedes next metadata;
   full Parent delivery and original ACK/overflow/close owners remain authoritative.
6. Finite public watch courts for BOTH actual providers: empty entrants, deletion,
   boundary/ties/order, actual enabled-toggle entrant/removal, unrelated candidate
   M, prepare/open race, cancelled/domain and Native-only pending, exact adoption,
   close/disposal/foreign World/incarnation/alias. Preserve old full result. Then
   appropriate affected old families and required gates, reviewer, commit.

Further exact-refinement/protected/continuation Phase3 work and Phases4–6 remain
open. Do not revise the goal down to this checkpoint or label a scaffold complete.

## Verified Live partition boundary correction

The existing publication reservation is acquired by product_publication after
Native validation. It cannot fence Source preparation. The existing Live
partition, however, already owns the exact World branch occurrence and bounded
publication history. Reuse that owner instead of adding a World fence table.
The independent reviewer approved the following protocol before implementation:

1. Arm a unique early preparation guard in that actual partition before Native
   validation. Multiple preparations remain legal. Each prevents a managed
   opening join; an opening permit prevents both new preparation and the existing
   late publication reservation. Keep this guard through accepted attempt,
   unpublished custody and actual World settlement; retire outside runtime gates.
2. After fresh Parent admission, open a genuine subscription BEFORE the full
   Parent read. It pins the actual partition/history. Re-observe current World
   through its real occurrence admission and require the admitted selected image.
   Snapshot sequence, completed commit and resync epoch from the pinned partition.
3. After the full read, atomically join that exact subscription and stamp under
   the existing Source state gate: same occurrence, same commit/sequence/epoch,
   no preparation, no late reservation, no retirement/close. Increment the opening
   permit count. Hold it through exact predicate insertion/finalization or rollback.
4. Every removal path respects subscribers, preparations, opening permits and
   publication reservation. Record completed commit/sequence even with no retained
   subscriber batch. Preserve existing bounded history and overflow. New resync
   state represents unsupported/untracked World movement, never empty completeness.
5. Removed partitions/batches leave the gate before final custody Drop. Retain
   original allocation grants through backing retirement; new managed allocations
   must be priced by the original execution, including queue/map backing. Resolve
   the concrete backing bound before editing that allocation boundary.

This changes no acceptance claim: the full watch vertical is still required
before the slice commit. The current execution check passed after exact inserted
slot rollback and Native preparation-denial family preservation. Real Source
routing, accepted-child promotion, full Parent delivery/ACK and public race courts
remain implementation/evidence obligations.

Concrete backing ruling: convert this existing partition directory from HashMap
to BTreeMap (the authentic key already has Ord). HashMap retained capacity after
mass removal invalidates a fixed per-live-entry charge; do not add a parallel
capacity ledger. A managed-created partition owns the fixed-node maximum plus its
actual bounded queue backing in original SourceFactsAllocation custody. Legacy
unfunded partitions receive original backing before joining managed operations;
changed configured capacities must be rechecked rather than underquoted.
Thread the existing original execution through the late reservation door so each
new batch cell and preallocated eviction buffer owns original funding. Carry
allocation stops through the existing relational execution-stop projection, not
the legacy capacity string. Eviction buffers are reserved before effects, populated
without allocation during publish, and dropped after the state gate. Removed
last-empty BTree backing closes before the final partition grant. Verify existing
bounded queue/close/count behavior alongside the real watch race courts.

Managed delivery contract ruling (independently checked against 9.19): choose a
private whole-Parent invalidation/current-refresh API. The notification identifies
actual publication B; the delivered full OneShot result and receipt independently
identify freshly selected current image C in the SAME World occurrence, with C
not earlier than B. Do not use the business LiveUpdate type or imply result-at-B.
Fresh current principal/scope/purpose/governance precede cause/availability access.
ACK the single authentic notification only after successful full read/projection
and original completion; refusal retains it or explicitly terminates. No implicit
ACK-through-C, skipped-batch coalescing or weakened one-shot currentness. A B→C
backlog court must distinguish cause B from selected result C. Preserve a completed
delivery if a late enclosing poll sees stop after its ACK has taken effect, using
the existing completed-effect handoff pattern. Existing business Live remains
batch-bound and unchanged. Historical managed Live/pinned support is not claimed.

Actual Native binding correction: retain the real AdmittedRelationalBranchBasis
and compare its Eq (owner identity, root and admission identity included), rather
than compare a copied descriptor. Both validation retry owners clone the same
transaction.basis; no re-admission is intended. Its Clone shares the existing Arc,
so the new Descriptor header is funded but no copied string backing is invented.
Its private retention issues no public READ door. The accepted owner follows the
typed approved candidate and exact unpublished custody. Pending Query publication
may also fail after World movement; install the owner into that exact pending
slot before resuming it. A must-use retirement carrier created outside Native and
registry gates receives the owner only after actual causality cutover, including
recovery and branch-retirement callers. Rejected incoming candidates and discarded
phases/aborted entries return their custody beyond the attempt gate before Drop.

Publication and locality correction: the receiving owner checks the authentic
performed receipt's old head (World branch/incarnation/generation/commit and
actual Native admission) as well as its original attempt, before labeling notices
Matched. A mismatch is Untracked. Known Source/scope retirement or unavailability
point-routes that actual axis/root and emits its active slots as per-watch
Unavailable notices; it never advances unrelated scopes' World epoch. Only an
unsupported/untracked World transition advances the partition resync epoch.
Notice deduplication uses coordinates issued from the existing registry-wide
checked next_id, shared by all resources/providers and never reset on disposal;
failed openings consume their coordinate. No per-root counters are introduced.

Receiver/ACK boundary review: lower_admission confirms the existing Live helper
uses a genuine graph_work query basis to admit an INDEPENDENT Bridge queue basis
(fresh request lane, its own selected Native observation and terminal reservation).
Its empty read packet conveys queue bookkeeping only. The reviewer approved
reusing it before the initial full Parent one-shot finalizer, retaining no completed
graph_work or proof in the watch. Move the helper into the existing Live owner
and cut its old private placement over; keep legacy Live's graph_work completion
path unchanged. Retain actual maintained selection/coverage derived from the
genuine selection loan before dropping that loan, and share the full Parent read
body through one runtime capture mode rather than duplicate generic read engines.

The managed queue has one pending notification. It polls the same bounded Live
history, performs a paid binary lookup among the batch's actual affected IDs,
and acquires the existing BridgeManagedQueueOccupancy. It ACKs exactly that B
sequence after the fresh C result/receipt has completed and is retained for the
outer completed-effect handoff. Failed ACK restores its exact occupancy; no
implicit cursor-through-C. Close releases occupancy before finalizing its exact
queue basis, restoring both on refusal. Source subscription/record/coverage retire
outside gates. Next validates fresh full Parent admission, actual audience,
World occurrence, purpose/association, all parameter values and query limit before
availability/cause access. Re-observe actual selected Source availability before
Pending so untracked physical suspension is never complete-empty or silent Pending.

### Public watch court resource boundary

The first 259-watch public locality journey compiled, then hit the fixture's
explicit 32 active World observation budget while opening disjoint watches.
Each watch legitimately retains its independently admitted Live queue basis
and real product observation. Preserve the normal fixture budget; parameterize
the existing fixture resource installer and reuse its existing host constructor
with an explicit 320-observation budget for the locality court. No platform
capacity or authority changes. Preserve only Delivered refresh outcomes over
a late outer stop: an already closed token is not proof of a completed effect.

### Reviewer World-occurrence correction

A sibling request can carry the same principal/scope IDs. The existing Live
subscription already owns the actual Source pointer and World branch/incarnation
key. Expose its exact occurrence comparison without lifecycle lookup and use it
after fresh Parent admission, before active/progress checks. A foreign request
refuses without terminating or acknowledging the original watch. Reuse the same
comparison within progress validation, and prove sibling refusal plus subsequent
lawful delivery in the actual geometry watch journey.

### Spatial backlog evidence

Reuse the real existing geometry fixture and its authored-coordinate oracle.
Open A probes 5/25/30 and disjoint B50, plus the configured prefix provider's
empty A probe. Perform actual shared endpoint motion to20 then30 before poll.
Assert first notification names20 while whole Parent and receipt name current30;
a second delivery names30, then Pending. Assert closed boundary30 enters, B50
and prefix stay Pending, sibling request refuses without consuming A5, old
result stays immutable, and allocator ownership balances on close. No new root
or alternate query implementation is introduced.

### Deterministic registration and recovery courts

The maintained adapter captures the real Source image under the existing native
read owner, then invokes the pure provider outside the Native runtime borrow.
Reuse its existing after_second_match observer to hold a genuine opening query
while an independently admitted mutation publishes. The stale opening must refuse
before registration; a fresh opening must see the successor. No private posting
fabrication or receiver hook is needed. Reuse the actual World operation pause
before product comparison for Native-performed/World-pending mutation. Cancel the
original request there, require ProductUnpublished, keep the existing watch
Pending on the selected predecessor and reject a new opening while its early
preparation guard lives. Recover through the existing authenticated mutation
request, then require exactly one full delivered refresh and Pending.

## Supported watch vertical verification (2026-10-31)

The actual public `watch_index`/`next_watch` path now consumes the inverse tree,
Native context, publication join and queue substrate. It returns full initial
and refreshed Parent results through the ordinary shared receiver/finalizer.
There is no unused scaffold or business-emission stand-in. The original full
Source dependencies remain conservative.

Current behavioral evidence:
- Public filter `predicate_watch_`: 2/2 pass, final run 6.99s. The prefix court
  varies 256 unrelated probes and checks actual inverse-region work, admitted
  enabled refinement, deletion/recreation, alias/parameter rejection, observed
  original cancellation during full refresh without lost notification, no-effect
  domain/cancellation payer release, deterministic obsolete opening, genuine
  Native-performed/World-pending and matching authenticated adoption. The box
  court checks independent authored closed bounds, exact sibling refusal, and
  first B notification/current C receipt followed by the still-deliverable C
  notification. Both run warmed allocator journeys and balance all ownership.
- Ten focused existing public regressions: 10/10 pass. Covers ordinary public
  cache Native-ahead/World-predecessor, unopened membership, maintained prefix
  and indirect geometry, retirement/generation, malformed structure, rebuild,
  generated suspension and pending restoration, and interrupted completeness.
- Final real Live partition unit filter: 2/2 pass in0.24s (one default unit build,
  4m11s; no restart or feature/profile change). Covers overlapping preparation
  counts, opening exclusion, late reservations, last-subscriber guard retention,
  final removal and invalidated stamps. Current compiled inverse tree pure
  filter: 3/3 pass in0.04s; independent membership/pruning and removal identity.
- Foundation evidence retained from this slice: matching compiler4/4,
  managed Live declaration1/1, UTF-8 inverse pure2/2, actual Native funded
  share/weak receipt court2/2. No new SDK/runtime test roots were introduced.
- Compiled consumer documentation drift1/1; mandatory boundary-check and
  agent-context check pass. Exact closed control callers name only the real
  pending watch recovery court and the two original control owners.
- rustfmt, git diff --check, dirty133-file400-line cap pass. Function scrutiny
  finds83 advisories and zero scan errors; independent review confirms the
  read/open/next, Native validation and publication coordinators should retain
  their visible protocol ordering. No semantic split is needed for line counts.

The independent reviewer identified and rechecked two receiver corrections:
foreign World occurrences refuse before progress/cleanup, leaving the original
watch usable; only Delivered overrides a late outer stop. It also required an
explicit trigger flag for the cancellation observer, now asserted before the
lawful retry. The larger320-observation policy belongs only to the259-watch
fixture; the default fixture32 limit and platform capacity remain unchanged.
Three unused imports and a capture-field visibility warning were removed at the
end; a final publication cargo check verifies that mechanical cleanup.

No broad workspace or large8192/50k growth lane was repeated: this vertical adds
retained-query routing, and its finite public locality court directly rejects
scan-all watches. The existing large forward-provider evidence remains scoped
to its prior commits. Phase3 exact domain refinement, protected influence and
continuation qualification remain required; Phases4–6 remain open.

Independent qa-loop, qa-tests and code-quality-qa commit gate: cleared for this
supported slice. Final publication cargo check passed59.67s. The reviewer caught
one Windows-encoded dash in the new milestone sentence; it was replaced with
ASCII3-6. Strict UTF-8 audit over all138 staged text files and staged whitespace
checks pass. This review does not close the remaining milestone phases.
