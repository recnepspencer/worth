# Milestone 9.18.2: Consumer History Trees And Snapshot Navigation

**Status:** specified; all seven phases below remain open.
**Public boundary:** process-local Decl/Host consumers.
**Prerequisites:** qualified branch/correction owners from
[9.18](./milestone-9.18.md) and retained-read, guarded adoption and history
contracts from [9.18.1](./milestone-9.18.1.md). This adds no dependency on 9.19.
**Scope decision:** physical reclamation, catalog pruning, compaction and durable
history reopening follow [Store integration](../database-foundation/roadmap.md).
This milestone implements navigation, not a total-storage cap.

Each phase contains its own product contract, authority and failure rules,
implementation boundary, test requirements, documentation and closure criteria.
Dependencies name completed capabilities, not requirements deferred to a later
phase. New API names describe the required implementation, not existing APIs.

## Phase 1: Install Document History And Record An Ordinary Edit

**Depends on:** the named 9.18/9.18.1 public contracts.
**Outcome:** a consumer declares document history, attaches its actual model
head, executes one ordinary domain edit and receives one exact history step.

### Required behavior and public contract

Declare ApplicationHistoryBinding in the existing application contribution.
The binding identifies the model type, full model-instance identity, participating
ordinary operations, summary/label meaning and current admission requirements.
Decl owns authored meaning; Host installs the binding and exposes HistoryDocument.
No consumer World/Native imports, receipt constructors or parallel undo stack
are required.

Attaching HistoryDocument transfers exclusive management of the application's
actual model-head slot. A UI selector or copied branch ID cannot become another
writable head. Independently owned documents must not share selection state.
Retiring/recreating a model under the same name creates a different history
identity. No global model-name registry may decide ownership.

A participating edit uses its existing declared operation, current authorization,
installed invariant, candidate/publication and delivery contracts. The framework
enrolls its actual performed result; the consumer must not manually append an
arbitrary receipt or depend on a post-success callback.

Reserve recording capacity before effects. Execute on owned candidate custody,
then expose the new selected model head, completed history-step binding and
process-local attempt outcome in one document-owner guarded transition. Before
that transition the old document remains selected. Published candidate effects
that precede it remain actual effects owned by the original continuation.

HistoryNodeId is sealed and document/runtime/incarnation scoped. A step binds its
genuine composite endpoint, preceding user step and authored label. Equal values
do not collapse distinct performed occurrences. The journal owns labels,
grouping and completed bindings; canonical model changes and composite ancestry
remain owned by Relational/Signal/World. Tree/layout caches are derived.

Use the original operation outcome and recovery owners. Same attempt/input
resolves the original step; changed input under the same key refuses. Losing
delivery after adoption returns the selected posture and retained delivery/
cleanup owner, never a generic no-effect result. Closing an attached document
must enumerate and release its managed custody, preserving any refused cleanup.

Out-of-band model-head changes must use this same owner or produce a typed
head-diverged result on its next operation. They cannot be silently invented as
recorded steps. Ordinary operations outside history keep their existing meaning.

### Implementation boundary

Extend existing application declaration/installation and ordinary owning-operation
entry points. Put runtime document-slot, step-binding and candidate progression
under Query's existing primary_graph/product_operation owner. Audience facades
aggregate exports only. Keep the binding's pure meaning out of live runtime
registries; keep live head custody out of the UI and schema substrate. Do not
create a second publication engine or a history manager containing all concerns.

### Tests required before this phase closes

Use a real installed Decl/Host application with documents A and B and
independently authored expected scalar values.

- Edit A through the new entry. Independently read its selected state, actual
  composite correspondence and one completed user step; B remains unchanged.
- Repeat delivery/attempt lookup and supply changed input under the same key.
  The original lawful retry succeeds without another edit or node; drift refuses.
- Supply B's owner, a foreign runtime and a recreated model with the same name.
  Each refuses at the intended identity boundary, with a lawful twin.
- Refuse recording allocation before effects. Separately lose delivery after
  real adoption and resume its actual result owner. Observe the selected head
  and outstanding custody, not a fixture flag or handler-call counter.
- Move the actual head outside the attached owner and verify typed divergence.
- Public compiler tests must reject fabricated completed steps and mutation
  through a read-only owner; valid counterpart code must compile.

These tests belong to the existing public document/correction court topology.
No test binary per scenario and no alternate test-only composition root.
Compare short and longer retained histories while recording the same local edit.
Enrollment may visit its changed step data and owner-required persistent paths;
it must not copy the model or scan all old steps to append the new one.

### Laws, documentation and closure

Apply [Architecture](../../docs/coding-guidelines/arch_laws.md) 1-7, 9-11, 13,
16 and 19-22; [Composition](../../docs/coding-guidelines/composition_laws.md)
1-8 and 13-14; and [Testing](../../docs/coding-guidelines/testing_laws.md)
2-4, 6-12 and 15. The named types must enforce the ownership distinctions.

Add a compiling install/attach/single-edit example to
[build-an-application.md](../../docs/build-an-application.md) and its capability
entry to [capability-index.md](../../docs/capability-index.md). Explain partial
candidate work, original attempt resolution and explicit disposal.

Close only when the public journey, negative twins, ownership/cleanup paths,
affected repository gates and scoped source/test review pass. The next phase may
trust exact recorded endpoints and one actual document head; it may not assume
historical navigation already exists.

## Phase 2: Read The Tree And Preview A Historical Step

**Depends on:** completed Phase 1.
**Outcome:** consumers can page history and query an old endpoint without
changing the active model or executing historical commands.

### Required behavior and public contract

Expose bounded HistoryPage rows and cursors through a fresh history READ.
Rows identify admitted user steps, parent/child relationships, labels and
qualified availability. Resolve them from completed step annotations and actual
owner correspondence. Metadata and model payload have separate admission;
a label, count, existence hint or disabled-button reason cannot disclose a
protected node.

A cursor binds the document incarnation, relevant history-view generation and
its continuation position. A foreign or invalidated cursor receives a typed
outcome; there is no silent restart that duplicates or omits rows. Traversal
honors the original request's cancellation, deadline, work and allocation policy.

Preview returns HistoryPreview with exact retained composite-source custody.
It is read-only and cannot fork, adopt, navigate or authorize mutation. Query
the chosen immutable endpoint, not today's branch. Corresponding Relational
and Signal bases must come from the same World source. Missing owner material
is unavailable; a current component cannot be substituted.

Historical data does not restore historical permission. Resolve current
disclosure independently of the historical model, including after a page or
preview handle has been retained. Derived output must be actual qualified
retained output, ordinarily reconstructed output with its proper status, or a
typed unavailable/pending result; never an invented authoritative snapshot.

The framework owns pages/previews and their live leases. Close releases only
that holder's custody and preserves lawful siblings. Destroying a tree/layout
cache must not destroy the journal or its real owner bindings.

### Implementation boundary

Add admitted history reads under ordinary application entry and Query-owned
observation progression. Reuse retained source/query owners; extend the owning
handoff if a public boundary is missing. UI state contains view data and IDs,
not raw observation factories or authority. The existing low-level history
declarations retain their audience and cannot become a competing unadmitted
application entry.

### Tests required before this phase closes

- Record root -> S1 -> S2 -> S3 in a real document. Preview S1 after S3.
  Independently verify complete old, current and untouched values, source
  identity and required derived-output posture. No command may run for preview.
- Revoke READ after listing and before detail. Check the intended refusal occurs
  before protected detail lookup; restoring the actual grant admits a fresh read.
- Exercise foreign/stale cursors, multiple page widths and cancellation at a real
  page/assembly boundary. Compare complete permitted rows with an independent
  reference, including hidden-row cases.
- Retain two genuine preview holders, close one, and read through the survivor;
  close the final owner through its actual lifecycle.
- Destroy/rebuild only the derived tree and repeat listing/selection from
  retained step annotations and real source owners.
- Compile-fail a preview used as navigation/mutation authority and compile the
  corresponding lawful read.

Keep the functional chain small in ordinary CI. Paging cost is bounded by the
returned width plus the explicitly admitted examined prefix; a tiny page must
not secretly materialize the full history.

### Laws, documentation and closure

Apply [Architecture](../../docs/coding-guidelines/arch_laws.md) 1-2, 5, 10,
15-20 and 22; [Performance](../../docs/coding-guidelines/perf_laws.md) on
declared traversal, lane separation and retained ownership; and
[Testing](../../docs/coding-guidelines/testing_laws.md) 2-4, 10-12 and 18-22.

Extend the application guide and capability index with compiling page/preview/
close calls, metadata versus payload permission, typed unavailable results and
cursor invalidation. Complete the affected gates and scoped source/test review.
The next phase receives exact read-only historical custody, not checkout
permission or a current-source proof.

## Phase 3: Checkout A Historical State And Preserve Both Futures

**Depends on:** completed Phases 1-2.
**Outcome:** one click can prepare and select an exact old model state, then an
ordinary edit creates a new future while the original future remains available.

### Required behavior and public contract

Checkout is a current, admitted navigation operation with an attempt key and an
expected actual document selection. It creates a writable owner-issued branch
from the chosen retained composite source, then adopts it through the document's
real head owner. Source availability, model membership and target currentness
are different proofs.

The selected historical source is expected to be old. Introduce a distinct
retained-source receiver through the existing World/Native owners. Preserve the
currentness comparison in branches/expected_basis.rs and existing expected-source
adoption. Historical readmission must never be relabeled current admission.

World establishes exact historical composite identity; Native/Signal admit its
corresponding retained bases and supported meaning. The current document slot
is separately guarded against concurrent replacement. Component unavailability,
foreign installation/model, retired incarnation, schema/program incompatibility
and stopped request must produce typed refusals before unauthorized effects.
Never reconstruct identity from numeric versions or payload equality.

Current security is outside historical document meaning. Checkout cannot roll
back grants, principal state, external delivery, durable idempotency or other
live control authorities. Declare the model's navigable domain scope. If the
component layout cannot preserve live authorities while selecting historical
model state, repair that owning composition before enabling this capability.
An old snapshot is never a fresh permission source.

Only declared snapshot-navigable scope is eligible. Escaped external effects
cannot be undone by checkout. Unsupported history returns a typed
RequiresSemanticCorrection or equivalent domain eligibility refusal; 9.18's
existing correction entry remains the route for inverse, compensation and
reconciliation. Historical inspection can still be separately lawful.

An edit after selecting S1 becomes a new child of S1; S2/S3 remain unchanged.
Navigation changes selection and real branch custody but does not fabricate an
authored edit node. Required derived output follows the selected source through
its normal owner and equivalence/rebuild contract.

Prepare -> owned advancement -> selected/pending/refused is compiler-visible.
Before adoption the old head remains selected; after adoption delivery failure
must preserve the actual selected result. Candidate creation/publication/close
failures retain their original continuation. Cancellation is not rollback.

Attempt resolution is document-scoped, not merely scoped to a newly forked
branch. Current permission precedes protected retry lookup. A successful retry
after later navigation returns its original outcome without moving the current
head again; same-key changed node/input refuses.

The required caller shape is a design target, not an existing API claim:

~~~rust,ignore
let expected = history.selection();
let navigation = history.navigate(&request)
    .checkout(node_id)
    .expected_selection(expected)
    .idempotency(click_key)
    .prepare()?;
match navigation.advance(&request) {
    HistoryNavigationAdvance::Selected(done) => show_selected(done),
    HistoryNavigationAdvance::Pending(owner) => retain_navigation(owner),
    HistoryNavigationAdvance::Refused { cause, owner } => retain_refusal(cause, owner),
}
~~~

### Implementation boundary

Extend Query's existing product-operation branch coordination, World historical
selection/creation and Native basis admission at their actual owners.
Query orchestrates; World owns composite ancestry/currentness; Native owns graph
basis identity; Signal owns definition/derived semantics. Keep document labels
and UI policy above those owners. New legality witnesses, if necessary, belong
to worth-proof and are constructible only by the granting authority. A
HistoryNodeId, JSON description or retained page cannot mint one.

### Tests required before this phase closes

Use root -> S1 -> S2 -> S3, checkout S1, and edit T2. Independently read complete
values, full occurrences, actual selected heads and derived values on both
futures; a second document must remain unchanged.

- Navigate back to original S3. Convict a linear redo stack, copying old fields
  into the current branch and replaying intervening commands.
- Supply equal-looking foreign bases, a recreated model, a missing historical
  component and an unsupported program/schema. Each reaches its intended typed
  refusal and has a lawful supported twin.
- List under valid READ, revoke only navigation, and prove no branch is adopted.
  Separately revoke current permission while old historical grants remain:
  checkout must not revive them.
- Prepare two attempts against one head. Pause one at actual owner guards,
  perform the other, release the pause and observe that the loser cannot replace
  the winner. A comment or standalone precheck is not the schedule.
- Cancel before fork, after genuine candidate work and while adoption waits.
  Observe head state and actual retained recovery/cleanup owners independently.
- Lose successful delivery, navigate elsewhere, retry the first key and change
  the requested node under that key. No duplicate fork, stale head reset or
  cached-authority bypass is permitted.

Compare nearby and distant retained-node jumps with the same selected payload.
Account for actual receiving-owner validation; neither path may replay all
intervening commands or materialize unrelated tree branches for adoption.

### Laws, documentation and closure

Apply [Architecture](../../docs/coding-guidelines/arch_laws.md) 4-5, 9-11,
14-17 and 19-23; [Domain Structure](../../docs/coding-guidelines/domain_structure_laws.md)
7-11; and [Testing](../../docs/coding-guidelines/testing_laws.md) 3-4, 6-12 and 17.

Compile the real application-guide checkout progression. Explain historical
source versus expected current selection, incompatible meaning, fresh security,
owned retry and selected-but-delivery-pending outcomes. Run affected owner and
public courts, normal gates and scoped QA before closing. The next phase may
trust complete checkout and fork preservation; no later phase supplies its
missing security or recovery proof.

## Phase 4: Parent Undo And Explicit Child Redo

**Depends on:** completed Phase 3.
**Outcome:** consumer undo/redo buttons use the existing checkout progression
and preserve every qualified alternative.

### Required behavior and public contract

Undo selects the recorded parent user step. Redo selects an explicitly identified
child. Multiple children return choice-required until a child is selected;
a remembered preference is descriptive and must be revalidated.

Root/no-parent, non-child, foreign, stale-selection, unsupported and unavailable
targets have typed outcomes. Both commands retain checkout's current permission,
expected head, idempotency and candidate/result/cleanup ownership. They must not
create another executor or a mutable redo stack. Direct jumps can cross several
steps without replaying their commands.

The consumer-facing names must distinguish state navigation from semantic
correction. Existing 9.18 reversal/reapplication still creates new ordinary
commits under its declared inverse, compensation or reconciliation contract.
Undo navigation never invokes those handlers, resends external effects or
claims to reverse an external payment/email.

### Implementation boundary

Add semantic parent/child target selection at the ordinary history entry.
Lower undo_step and redo_to(child) into Phase 3's same prepared navigation and
owned advancement. Derive UI affordances from admitted history; a disabled/enabled
button must not supply authority, reveal hidden branches or bypass preparation.

### Tests required before this phase closes

- Navigate S3 -> S2 -> S1; enumerate children S2 and T2; select each future and
  independently verify complete document state and unchanged alternate history.
- Exercise root, multiple-child choice, non-child, stale click, foreign document
  and permission-loss cases through the public commands, with lawful twins.
- Retry old navigation after a later successful move. Resolve original delivery
  without moving today's selected head again.
- Observe that no inverse/reapplication or external-effect handler runs for
  navigation. Separately exercise the existing semantic-correction route to
  verify it still appends the expected new commit.
- Compare a generated small sequence of branch edits, undo, redo and direct
  jumps with an independently implemented user-step tree oracle.

Exercise the same parent/child command with unrelated futures added. Additional
controller work may follow the chosen edge or admitted child page, not rebuild
the entire tree. Carry unchanged checkout cost evidence from Phase 3.

### Laws, documentation and closure

Apply [Architecture](../../docs/coding-guidelines/arch_laws.md) 5-6, 10, 12,
14, 16 and 20; [DX](../../docs/coding-guidelines/dx_laws.md) 8-11 and 19; and
[Testing](../../docs/coding-guidelines/testing_laws.md) 2-5 and 13.

Compile the normal button-command example in the application guide and correct
[recorded-source-correction.md](../../workspaces/worth-query/crates/worth-query/docs/execution/recorded-source-correction.md)
to explain the two meanings. Close with passing focused cases, affected gates
and scoped review of actual command routing and history preservation.

## Phase 5: Group A Gesture Into One History Step

**Depends on:** completed Phases 1, 3 and 4.
**Outcome:** several ordinary edits form one visible history edge without
exposing a partly applied group as the active document.

### Required behavior and public contract

Expose explicit begin/complete/abort ownership. The group executes its declared
ordinary operations on one private candidate lineage while other document readers
continue seeing the original head. On success the document owner atomically
adopts the endpoint, completes one user-step binding and records the attempt
outcome. Undo/redo navigate the group's endpoint and preceding user step.

Each operation preserves its current admission, invariants, outputs and actual
publication outcome. The group retains internal committed endpoints needed by
those owners. Metadata grouping is not a rollback contract. On a later refusal,
earlier candidate publications remain effects with their actual close/recovery
custody, even though the active document did not move.

Refuse nested groups and ineligible escaped-effect commands before group effects.
No implicit flattening, compensation, partial document application or silent
rebasing is allowed. Competing group completion/navigation shares the actual
document-head guard. Request stops and delivery loss follow the same before/
after adoption distinction as checkout.

Reserve bounded group and result ownership before effects. Retrying completion
cannot create another user step; changed membership/input under the same attempt
refuses. Every pending, failed and completed group has a registered close path.
Membership, candidate retention and in-flight result storage use explicit
installed resource limits; exceeding them cannot become an unbounded hidden
queue or silently commit only a prefix of the group to the active document.

### Implementation boundary

Compose existing ordinary owning operations, isolated candidates and guarded
document adoption under Query's history-step owner. Domain handlers remain
responsible only for declared domain effects. A UI gesture callback cannot
assign a branch, bypass a denied operation or own a hidden cleanup list.

### Tests required before this phase closes

- Execute two lawful edits with independently expected values. Observe the old
  document between operations, then the complete endpoint and exactly one user
  history edge after adoption; navigate undo/redo across that edge.
- Make the second operation reach a real installed invariant refusal after the
  first genuinely committed on the candidate. Independently read the unchanged
  active model and real staged first-step state and close/recover its owner.
- Cancel between operations and during guarded adoption; lose delivery after
  successful completion; retry each reachable owner. No false rollback,
  duplicate step or orphan branch is permitted.
- Race completion against a real competing navigation. Only the lawful guard
  winner may change selection; the loser retains its actual custody.
- Reject nesting and external-effect participation at the intended eligibility
  boundary, with a valid ordinary group counterpart.
- Exhaust the group's declared membership/candidate resource before its next
  operation and check the real selected/staged posture and continuation.

Vary group length separately from unrelated retained history. Work may grow with
the group's actual operations and their native changes; finishing must not scan
all historical groups or copy every model state for a one-edge UI result.

### Laws, documentation and closure

Apply [Architecture](../../docs/coding-guidelines/arch_laws.md) 4-7, 9, 14-16,
19 and 22; [Composition](../../docs/coding-guidelines/composition_laws.md)
2-7; [DX](../../docs/coding-guidelines/dx_laws.md) 4, 11 and 19; and
[Testing](../../docs/coding-guidelines/testing_laws.md) 3, 10-12 and 17.

Add a compiling grouped-gesture example and its refusal/cleanup handling to the
application guide. Close only with the public grouped journey, intended failure
cases, actual lifecycle evidence, affected gates and scoped review.

## Phase 6: Configure A Navigation Window And Bookmarks

**Depends on:** completed Phases 2-5.
**Outcome:** consumers choose how many unpinned user steps this feature keeps
navigable, across all futures, with honest ownership and no storage-cap claim.

### Required behavior and public contract

Implement a positive finite step-count window. RecentSteps(200) is the default;
the caller may choose another finite count. It applies across the entire
document tree, not independently per branch. Evict oldest eligible unpinned
steps by creation order, independent of display order or recent clicks.
Navigation itself does not create an authored edit step.

The current endpoint and explicit bounded bookmarks are protected. Report them
separately: their presence can make total navigable nodes exceed the unpinned
window. Installation resources bound bookmarks and in-flight owners. Exhaustion
has a typed outcome before unauthorized effects; no implicit unlimited mode or
silent bookmark loss is allowed.

Window and bookmark changes have independent current admission. Shrinking cannot
evict the active selection or invalidate a genuine in-flight preview/navigation.
Existing holders keep their admitted immutable state. New requests independently
satisfy current disclosure and window eligibility. An excluded node cannot be
newly opened or pinned through an old UI ID.

Pages show explicit truncated boundaries. They must not invent direct parentage
through excluded steps or leak excluded protected metadata. Exact step identities
and safe navigation remain distinct from the shape drawn by a renderer.

Release this feature's obsolete node/branch references through their existing
owners. Expose refused/pending cleanup and explicitly close session ownership.
A hidden all-history strong-reference map behind a finite visible window is
forbidden. Other readers, receipts and branches retain their own genuine rights.

Physical reclamation is outside this phase and milestone. Do not add catalog
deletion, log truncation, compaction, cold archival, secure deletion, byte/age
retention guarantees or a new GC policy. Releasing a navigation lease promises
neither immediate heap recovery nor disk recovery. Canonical metadata/catalogs
may still accumulate. A working byte-budget selector must not be advertised.

### Implementation boundary

Implement eligibility, bookmark ownership and bounded release under the same
framework-owned document history. The policy controls selectable user steps and
its own leases, not Native/World physical reclamation eligibility. Keep model
identity and current permission upstream of policy mutation. Do not put a
retention authority in a UI adapter or introduce another root ledger.

### Tests required before this phase closes

- Set a three-step window, create two futures, and compare retained choices
  against an independent oldest-first document-wide reference. This must reject
  an implementation retaining three additional steps on every branch.
- Bookmark an older endpoint, retain a genuine preview, append and shrink the
  window. Verify actual protected nodes, truncated edges and typed excluded-node
  refusals. A cached node ID cannot reopen or newly pin an excluded step.
- Release preview/bookmark holders independently; observe release of this
  controller's ownership while legitimate sibling readers remain usable.
  Do not assert immediate physical bytes reclaimed or catalog records deleted.
- Revoke only policy/bookmark authority while keeping READ lawful. Exhaust
  bookmark resources and cancel a bounded policy update at its actual boundary.
- Dispose the document/session with outstanding cleanup and verify no invisible
  controller-owned all-history references or pending branches are abandoned.

Vary the number actually excluded independently of unrelated retained history.
Policy maintenance must process the changed eligibility frontier and bounded
release work, rather than clone the complete tree on each appended step. A large
policy change may return owned progress instead of hiding unbounded work in Drop.

### Laws, documentation and closure

Apply [Architecture](../../docs/coding-guidelines/arch_laws.md) 5, 11 and 19-22;
[Performance](../../docs/coding-guidelines/perf_laws.md) on named retained-state,
bounded cleanup and lane costs; and
[Testing](../../docs/coding-guidelines/testing_laws.md) 3, 6, 10-12 and 17.

Compile window/bookmark/disposal examples in the application guide. Update
[storage-retention.md](../../workspaces/worth-query/crates/worth-query/docs/execution/storage-retention.md)
and the capability index to distinguish navigation availability from storage
reclamation and report pending/protected custody accurately. Close after these
tests, affected gates and scoped ownership review pass. Physical reclamation is
not an omitted test obligation; it is expressly deferred until Store integration.

## Phase 7: Deliver The Plug-In Consumer And Qualify The Composed Feature

**Depends on:** completed Phases 1-6.
**Outcome:** a normal consumer can install, render and operate the whole history
feature without implementing its authority, ancestry or lifecycle machinery.

### Required behavior and implementation boundary

Ship one executable Decl/Host document example with a UI-neutral paged view and
command adapter for node click, preview, undo, child redo, grouped edit, bookmark
and window configuration. The consumer supplies domain operations, labels and
renderer. It does not copy snapshots, manipulate raw branch slots or coordinate
owner cleanup. Compile a second small binding to reject coupling to the first
model's schema, labels or shape.

Preserve distinct authoritative objects: actual document selection, authored
step/group annotations and owner-issued source custody. The tree, layout and
button affordances are disposable derived views. Stable public descriptions
carry schema/session identity/version and no authority. Unsupported versions
refuse before effects; no legacy interpretation or automatic migration is
introduced for undeployed examples.

This is a process-local feature. It does not promise restart-stable selection,
bookmarks, attempt fates or retained history. After process loss descriptive IDs
cannot reconstruct authority. Do not add JSON snapshots or an alternate durable
backend to impersonate Store. Existing unrelated persistence guarantees remain
unchanged.

Use existing declaration/installation, Query ordinary-entry and document-history
owners, with existing Native/Signal/World receivers beneath them. Facades contain
exports only. Visibility/dependency enforcement must exclude raw owner access,
cert-only replay and counterfeit selection factories from normal consumers.
Inventory participating example/application wrappers and remove superseded
linear undo workflows during their cutover; do not remove distinct low-level
correction/history audiences or build a compatibility engine.

### Complete public acceptance journey

Through the actual consumer entry, record root -> S1 -> S2 -> S3; preview S1;
checkout S1 and author T2; undo T2; explicitly choose either future and directly
select S3. Independently verify complete values, actual selected heads, exact
occurrences and required derived outputs. Another genuine document is unchanged.

Then exercise the same composed entry with current authority revoked after
listing, a stale click, a real competing adoption, lost navigation delivery,
a denied second grouped operation, and a finite window with bookmark/preview
holders. Observe actual selected/candidate/cleanup postures. The public integration
must consume the real qualified receivers; unrelated passing owner tests plus a
fake tree cannot establish this journey. Destroy/rebuild the derived tree and
repeat lawful navigation.

Carry adequate phase evidence; rerun only contracts changed by integration.
No fixed test counts, full Cartesian matrix, proof ledger, implementation journal
or recursive certification machinery is required. Finish with appropriate
qa-loop, qa-tests and code-quality-qa review of the composed source, tests and
consumer guidance.

### Performance tests and execution lanes

Ordinary recording, page reads and navigation must not enumerate all historical
commits or copy complete model images. State growth in terms of changed step/group
size, visible/examined page prefix, affected persistent paths and the actual
receiving-owner validation granule. A retained-node jump must not replay all
intervening commands. Deferred maintenance and retained custody remain visible
costs, not free work hidden behind completion or Drop.

Vary depth and branching independently with small real histories in ordinary CI.
Use 1,000-step deep-history and broad-future qualification in the existing
scheduled cost lane, with setup, ordinary edits, selection, historical reads,
diagnostics and cleanup measured separately. Inspect structural work only where
needed to hold a demonstrated growth contract; do not fit incidental exact counts.
Report environment, configuration, repetitions and variance for timing claims.

Carry 9.18.1's qualified H10k evidence unless changed history/storage behavior
invalidates it. Expensive maxima stay ignored by default and run for affected
scale qualification, not every document edit or immaterial correction. Do not
add phase-specific binaries, profiles, features or a new generic runner.

### Documentation, laws and closure

The application guide must contain executable installation and every supported
normal command, plus typed denial, retry, interruption and disposal examples.
The capability index must lead to the actual compiling example. The correction
guide must distinguish state navigation from inverse/reapplication; the storage
guide must state the limits of navigation windows. Do not advertise future
physical reclamation or restart as an implemented option.

Apply [Architecture](../../docs/coding-guidelines/arch_laws.md) 1-23 at their
actual crossed boundaries; [Domain Structure](../../docs/coding-guidelines/domain_structure_laws.md)
1-3, 7-11 and 17; [DX](../../docs/coding-guidelines/dx_laws.md) 1-4, 8-13 and
18-21; [Testing](../../docs/coding-guidelines/testing_laws.md) 2-4, 11-12,
15 and 18-28; and the
[QA review guide](../../docs/coding-guidelines/qa_review_guide.md).

Close the phase and milestone only when all seven phases are individually
complete, the real composed consumer and affected repository gates pass, the
cost claims are qualified and final review finds no material in-scope defect.
Merge implementation and current consumer docs together. A tree widget, branch
facade or copied-snapshot prototype alone cannot close this phase. A private
CAD/UI adoption run and Store durability/reclamation are not public closure gates.

### Explicit Store successor handoff

After the [Database Foundation](../database-foundation/roadmap.md) durable
application/runtime recovery join is qualified, specify the durable history
integration under those existing owners. It must persist/readmit step/group
annotations, exact selected model head, bookmarks/window and submission fates
through the single durable lifecycle owner and real-process crash oracle.
This phase must leave those semantic distinctions intact without constructing
an empty successor directory or a competing persistence abstraction.

Physical pruning/reclamation follows that integration. Its future specification
must coordinate every reader, receipt, branch, correction, recovery and durable
coverage obligation; define checkpoint coverage, compaction, expiry/tombstones
and any byte/age guarantee; and prove actual safe physical recovery of space.
Store owns physical byte survival/reclamation; World/Native/Query keep semantic
eligibility. Those tests and implementation are outside 9.18.2.
