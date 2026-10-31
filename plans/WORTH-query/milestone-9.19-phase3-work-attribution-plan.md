# Phase 3: attribute actual original SDK work before further optimization

## Boundary review

Commit ee67068e68 clears the concrete spatial/storage/structural mechanics but
not the unchanged 50k invoice guard. Fixed64 remote0/256/1024 whole-request work
is (2440583,2243133), (3497420,3207412), (3650762,3338221). More caches are not
justified until the remaining actual charges are localized.

The existing SDK RequestConsultObservationForTest records real cumulative
root-scope charged prefixes and actual consult ordinals on its issuing thread.
It does not issue a request, grant, Source, dependency or proof. Its install
callback is documented as cancellation-only/no work; install a no-op callback.
The existing count_for_test and charged_prefixes_for_test readers suffice. Do
not add a platform observer door, ledger, estimator, reconstructed bill or new
production instrumentation. Original ExecutionWorkCeiling report remains the
invoice oracle, original controls and the current request remain authoritative.

The geometry provider's existing Calls receiver owns passive fixture observers.
A cfg(test) stage recorder can mark actual update begin/end, decompose and pure
summary callbacks. Clone callbacks out of its mutex before invoking. A fixture
thread-local holder owns the actual SDK observation guard and stage ordinals;
a scope guard clears/restores both SDK observation and Calls receiver on success
or unwind. The stage callback samples existing count_for_test outside SDK
callbacks. No request or mutable platform handle is retained. A provider callback
on another issuing thread cannot inherit this observation; missing events must
fail the evidence rather than fabricate intervals.

Correlate stages with actual charged prefixes after the serial whole request
finishes. Validate consult/prefix order and the last charged prefix against the
actual SDK invoice. Display first/last callback anchors and conservative charge intervals between
anchors, explicitly allowing intervening framework work. account_completed_work
can share a consult ordinal across settlement and the marker. Bound each marker
by the last prefix with strictly smaller ordinal and the last prefix with ordinal
less than or equal to its ordinal; never assign ambiguous same-count charges to
one side. Compute gap bounds from those marker intervals, retaining ambiguity
as unattributed work. One linear merge of events/prefixes avoids repeatedly
cloning the complete prefix history at every callback. Required attempted
prefixes can exceed accepted work on SDK denial; these measured success/domain
refusal journeys must pass final observed-prefix == actual invoice before any
full attribution claim. No artificial poll/checkpoint is added. A gap from last
decompose to first summary also includes topology qualification, affected-frontier
construction and first Source acquisition. A gap after a final summary may
include scope handoff/preparation. Do not label such gaps as isolated owner costs.
No observer payload CPU/heap cost is claimed by this diagnostic.

Independent reviewer boundary approval agrees this fixture-only observer uses
existing SDK primitives and requests no platform/API change. The no-op callback
and thread-bound guard ownership follow that review's stricter recommendation.

## Implementation plan

1. Add cfg(test) provider/cost_stages.rs containing only the named stage enum,
   owned optional receiver callback and clone-before-call mechanics. Calls holds
   that concrete recorder. Add passive marker calls to update begin/end,
   decomposition and source/merged-summary callbacks. Keep all provider results,
   control checkpoints and Source reads unchanged.
2. Add growth/work_attribution.rs to own a scoped thread-local SDK guard, exact
   callback ordinals and RAII restoration. Register only that file as an exact
   SDK test-control caller. Use no-op SDK callback; stage callbacks sample only
   the installed guard's existing count reader.
3. Wrap only existing real mutation invoices in the observer; queries, population,
   reference oracles and source census remain as authored. Assert two complete
   provider update spans, actual events and original report/prefix agreement.
   Preserve all membership, Source +8/+8, snapshot and no-rebuild assertions.
4. Run existing fixed64 remote axis first, using the same leaf consumer binary
   and default compiler/cache configuration. Original invoice values should
   remain unchanged by passive tracing. Inspect intervals before planning a
   production optimization; do not run the full 50k court yet.
5. Run the affected small growth/public family, boundary/context/size/function/
   formatting gates and independent qa-loop/qa-tests/code-quality-qa review.
   Commit this focused diagnostic evidence slice. No Query private-owner rebuild
   or SDK rebuild is required by fixture-only observation changes.

Checked-scale qualification, predicate routing, exact refinement/disclosure and
Phases 4-6 remain open. Do not claim any performance repair from attribution alone.

## Boundary correction from the first real run

The first fixed64 run correctly refuses the proposed attribution: its last
observed prefix equals the actual invoice, but intermediate prefixes are not
monotone. The ordinary Query debug placement certifier uses real detached SDK
meters (test_placement/native_certification.rs and backend/meter/detached.rs).
The thread observer sees those different roots too. A matching final total does
not qualify intermediate correspondence. The unqualified thread stream cannot
become original-request evidence by guessing at drops or taking a maximum.
The first-run log is query919-work-attribution-axis.log; no cost conclusion is
accepted from it. Pause implementation and revise this boundary before edits.

The SDK meter owner can identify an actual live context by its private Rc
allocation and current ACTIVE_METER ancestry. Add only an opaque weak observation
scope there. It contains no grant, counter, stop or strong meter reference. Issue
it only from an actual currently active KernelContext. Its read-only admits check
requires that exact original meter to occur in the active stack, covering genuine
descendants while excluding detached diagnostic stacks and closed scopes.

Extend the existing RequestConsultObservationForTest with an optional such scope
and a scoped install constructor accepting the real KernelContext. Noncurrent
receiver refusal uses existing NoActiveExecutionScope; no new legal authority or
error category is introduced. Existing unqualified observation stays unchanged.
Filter consult/charge/stop observation before reading that scope's prefix. Keep
no-op callbacks and all original controls. No work-loan issuance, added poll,
checkpoint, synthetic meter id, copied root identity or billing estimate is used.

The fixture must install this observer inside the actual original invoice's
run_with_checkpoint callback, borrowing that real context. The weak observer remains through the issuing scope
close so the final report and saved records can be read; its scoped-active query
then returns false. RAII restores it after those reads or on unwind, without
strongly retaining the meter. Marker callbacks retain conservative same-consult
interval treatment. No Query production code changes are required.

Revised ordered steps: implement the weak private scope + scoped install; prove
root/actual-child charges with a deliberately detached different diagnostic and
noncurrent ancestor rejection in the existing SDK public court, then integrate
the real invoice context and rerun the small axis. Add exact caller records for
that SDK court file. Run focused SDK and public tests plus required gates and
independent review; do not rebuild Query cfg(test) for these observations.

Independent review approves the SDK owner correction. A Weak keeps allocation
metadata until the observer drops but does not strongly retain the live meter.
Stage readers must also query is_current_scope_observed_for_test before recording
a marker, so a detached provider callback cannot masquerade as an original
anchor with an unchanged consult count. This is passive metadata, not a poll,
work loan or authorization. Saved observation records remain readable at exit.

Prefix arithmetic continues to describe the actual root. Scoped installation is
therefore restricted to the actual current root (one ACTIVE_METER entry), also
refusing a current nested receiver. It does not relabel ancestor-inclusive Work
as a nested invoice. The public court verifies both nested-current and
noncurrent-ancestor rejection; ordinary nested unqualified guards still restore.

## Second real-run refusal

The scoped run still refuses intermediate attribution: final required prefix
matches the original invoice, but required Work can fall between checkpoints even
under one authentic root. SDK scope.rs documents inherited-control oracle/frontier
lanes whose completed report is not charged to the parent. Root-qualified
required-prefix evidence therefore must not be relabeled as monotonically
accepted invoice work. Keep the rejection until the actual decreasing records and
callback anchors are inspected. Add only diagnostic output before the failed
assertion to locate those records; no performance conclusion or acceptance guard
is changed. The root-filtered SDK observer remains correct for its documented
required-prefix scope; it promises no accepted intermediate settlement measure.

## Reviewed diagnostic scope after inspecting the decreasing record

At fixed64 refusal the single decrease is after all callback anchors: consult
32974 required2440566 to32976 required1420344, while last summary anchor is23340.
Final observed required prefix equals the separately accepted invoice2440583.
Do not infer a refund, unpaid work or accepted phase cost from that decrease.

Independent review approves a narrower raw diagnostic: require monotone consult
ordinals globally and required prefixes through the last callback anchor only,
including same-consult ambiguity. Print root-qualified active-stack required-work
intervals through those anchors. Expose every later negative jump as uninterpreted
settlement-tail evidence. Keep the accepted whole SDK invoice as separate truth;
never sum these raw windows into accepted phases or a dominant-owner proof. This
requires no further observer framework, inserted work, altered original invoice,
performance implementation or relaxed qualification guard.

Approve only work_attribution.rs with observed SerialRequest in the external
consumer leaf baseline/audit: it holds the same real public-integration original
invoice helper, not private runtime logic. Existing growth.rs still owns original
SerialRequest for the public query invoices, so retain its existing exact record.

## Final diagnostic evidence

The root-filtered small axis passes in17.53s. Accepted original refusal/commit
invoices are exactly unchanged from ee67068e68 at fixed64 remote0/256/1024:
(2440583,2243133), (3497420,3207412), (3650762,3338221). Callback-region required
prefixes and consult ordinals pass their correspondence checks. Every later
required-prefix decrease is printed as uninterpreted settlement-tail data. The
accepted whole invoice stays separate and authoritative. None of these raw
intervals is an accepted phase sum, refund, unpaid-work assertion or proof of an
exclusive/dominant owner. No performance optimization is claimed by this slice.

All45 public managed-index courts pass in18.93s with only the unchanged full
large court excluded. The final two SDK courts pass (3.35s focused rebuild,
0.00s runtime): actual root3 +child completed-work2/checkpoint3 +root7 gives15;
detached completed-work40/checkpoint61 gives101 outside the observed root;
noncurrent and current-nested installation refuse; marker scope becomes false
while detached/closed and true on actual restoration; a new scope on the same
SerialRequest cannot retarget the closed observer; nested and unwound guards
restore their actual previous observation.

The unchanged boundary/context executables validate current inputs successfully.
All10 changed/new Rust files meet the400-line cap, including meter.rs at399;
function review reports three reviewed advisories and zero scan errors. Formatting and
whitespace checks pass. Exact SDK caller and external consumer-helper placement
records are registered, with no parent subtree exemption. Query production is
compiled through the public consumer target; no Query cfg(test) rebuild or full
50k run was needed. The prior bank compile debt is unchanged and not rerun.

The measured raw callback windows select a concrete next investigation, rather
than prove its accepted cost: after decomposition and before first summary the
required-work gap can be634339..634341 at remote256. SourceTree.affected currently
builds a fresh unit map despite the actual writer already owning changed-node
keys with immutable SDK-funded backing. Review reuse of that exact map as a seed,
preserving final ancestor closure, retired-node behavior and independent Source
changes with no Node write. A seed membership hit alone must not falsely certify
its ancestors. That production change needs its own boundary review and plan.

Large checked-scale qualification and its original guards remain open; all other
unfinished Phase3 obligations and Phases4-6 remain unfinished.

Independent qa-loop, qa-tests and code-quality-qa review gives unconditional
clearance for this raw diagnostic/root-scoped observer slice with the limitations
above. No accepted phase attribution, scale improvement or Phase3 completion
is claimed.
