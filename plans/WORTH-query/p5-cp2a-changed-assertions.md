# CP2a assertion changes

## D1450: sanctioned legacy post-edit expectation

`owned_outputs/synchronous_edit/reconciliation/courts.rs::legacy` previously
asserted zero contacts on the first demand after a Retire-bearing edit. Replace
that contact assertion with one verification/adoption contact and zero commits,
then add the following demand's zero-contact/zero-commit assertion. Keep every
complete named-row, identity, adjacency and absence oracle assertion.

Reason: `orchestration/analysis-p5-cp2a-currentness.md` sections 2–4 identifies
that first contact as the existing required-wave NRU disclosure, before any
adoption. The assertion was already red at test-first `49ec84a655` (10 vs 9;
`build-cache/history53-d1347/p5-cp2-fixture-public-red4.log:2807`). It does not
state G1's post-adoption guarantee. D1450 explicitly sanctions this change.
Measured without the gate in `d1450-reconciliation-first.log`: both serial and
worker legacy courts pass, as do both G1 adoption courts.

## D1450: re-pin the public count fixture without the deleted gate

`reconciliation/commit_count.rs` measured three survivors as four contacts and
five commits; two survivors as three contacts and four commits
(`d1450-reconciliation-first.log:2248-2274`). Its old contact pins included
executions induced by the blanket gate. First journey pins are now s+1 contacts
and s+2 commits. The three-survivor tuple changes from (5,7) to (5,4).
The separate private cabinet has different exact output-role classifications;
its measured pin is recorded separately and is not inferred from this fixture.

The first follow-on required-wave demand is measured separately: for three
survivors it makes three verification/adoption contacts and zero commits
(`d1450-public-count-second.log`, original zero-contact assertion 15 vs 12).
The next demand must make zero contacts and zero commits. Every existing row,
identity, adjacency, orphan absence and unrelated-model assertion stays.
The growth/worker rerun must confirm all pins before qualification.

## Existing assertions retained

The inherited lone-root one-contact/re-demand-zero court and independent-writer
court are unchanged. `required_reapplication` retains all semantic assertions;
its first source-publication panic prints the actual Native outcome instead of
hiding it. CP2b's two explicitly pending sequential-net comparison courts remain
registered with their named CP2b reason, as sanctioned by D1430; no new ignores.

D1462 subsequently authorizes the original-root fixture correction detailed
below. The leaf1then0 guarantee itself stays unchanged.


## D1459: typed entry-specific numeric delivery contacts, with paired execution

The inherited required_output_courts::settle contact assertion changes from
unconditional1 to0 for an authored co-committed edit and1 for RecordedReverse /
RecordedReapply. The entry is the receipt's existing typed aftermath-causality
fact; no caller flag supplies it. Every receipt, row, lineage, source, replay,
ordering and arithmetic assertion is retained. Source publication calls now
enter the co_committed::publish_and_deliver owner, which measures exactly one
actual producer handler entry and one real Native commit across all branches,
checks the derived constant before start_required_outputs, checks real receipt
and delivery0/0 with immutable head/values, then reopens actual disk checkpoint
bytes and checks cold0/0 with exact source/output rows. No zero expectation is
supported only by a contact count.

D1459 explicitly approves this restatement. Evidence before approval:
d1456-reapplication-second.log:2241-2244 (old1 vs actual0) and
 d1456-ordinary-second.log:2240-2241 (hot/worker receipt and arithmetic23).
The existing metric owners define this handle's own lifetime. The paired
reapplication run is d1459-reapplication-fourth.log: hot and worker journeys
both pass the original reverse/reapply/replay/ordering assertions, including
1 contact for deferred correction delivery. The skip-source-producer mutation ran and failed at the pre-delivery derived
value assertion (d1459-mutant-skip-producer/runtime.log); SHA-256 restoration
is recorded alongside it. All eleven ordinary-part mutations fail at runtime
and restore exactly (d1459-mutations-runner.log).

A separate owner regression found by that journey was repaired without changing
an assertion: an ordinary computation locator must not be readmitted as a
Native publication for adoption. Both adoption callers now use the selected
head owner's Native-presence check. A genuine present publication retains its
exact readmission; a Native-proved absent route simply has no publication to
adopt and follows ordinary publication admission. No error is swallowed, no
Native origin is fabricated, and no re-execution trigger is changed.

## D1459: canonical revision helpers retain the one-execution oracle

The three canonical_transition edit_source helpers still require exactly one
real producer execution. They now measure that execution over the source edit
and its delivery, through the existing scoped execution observation and runtime
contact counter. The delivery handle alone starts after the co-committed
execution and therefore does not measure the interval these helpers intend.
No zero-contact expectation, guessed increment or test-selected entry flag was
introduced. Every returned producer receipt and downstream row/lineage/oracle
assertion is retained. This follows D1459's metric-owner distinction; focused
qualification is green in d1470-application-graph-final.log (all106 application-
graph courts, including the unchanged canonical retirement journey).

The same total-interval measurement now applies to the numeric causal-inspection
source journeys and the ordinary-source custody report court. The numeric
helper accepts the real source execution closure, so its observer begins before
that execution rather than adding a fabricated contact afterwards. Each caller
keeps its existing expected execution count, including actual no-op zero, and
all original receipt/source/query/materialization assertions. Evidence is
pending in d1459-causal-revisions.log and the ordinary-source report refresh;
that historical run did not constitute a green claim. D1470 now passes all106
application-graph courts; the full final-tree gate separately includes the
ordinary-source report.

## D1462 / D1463: immediate fixed refusal is atomic; deferred fixed ingress is #120

The old fixed_leaf_domain_refusal_closes_unavailable_without_reexecution journey
saved zero before the synchronous assessment refused. D1358/D1397 forbid that
partial effect. D1462 and D1463 sanction replacing its saved-zero / retained-source1 /
unavailable-evidence assertions with unchanged seed7, retained-source0, no Native
publication on any branch, and the real ProducerDomainDenied cause and canonical
RetentionAssessmentDenialValue bytes. Every source/producer decision/candidate
count, current source read, cleanup, keepalive, fresh-runtime/branch/cancellation,
recovery-page and no-reexecution oracle remains. Evidence is
`d1459-terminal-native-cause.log`; the new court also checks the typed #120 refusal.

No direct successful Native unavailable completion court existed: the direct
ordinary_source report covered unexecuted, cancelled and settled refusals only
(ordinary_source/mod.rs:77,89,121). The old fixed journey was its sole success
coverage. D1463 therefore adds direct tests on a genuine recorded inverse, through
the existing preparation and leaf handle, with a reciprocal's real division-by-zero
denial. Pending, shared custody, successful settlement and current-authority guards
remain tested. The genuine deferred hot/worker courts pass in d1463-deferred-fifth.log;
d1463-mutant-shared-custody and d1463-mutant-fixed-unsupported each fail at
runtime and restore by SHA-256. Their coverage is real, not inferred from the
postponed fixed ingress. No fixed ingress is
added and no inaccessible owner is faked. Public #120 owns that missing lifecycle.

## D1462: original root evidence on a genuine required wave

The former original_leaf pair was added under the removed blanket gate and
incorrectly demanded a cleanup execution on an empty leaf. It now uses3->1
members in original_root.rs, so a real surviving dependent enters the existing
required-wave certifier. Only scenario indices change (retired1/2, dependent1).
The immutable original Create-role oracle, final temporary-row absence and
strict original-versus-final observation distinction remain. Complete named
survivor/root/child/source rows and original-basis temporary-row reads strengthen
the same oracle. The unchanged lone-root1then0 court remains the leaf guarantee.
Runtime qualification passes in d1463-original-root-second.log; no retirement
gate or certifier change was made.

D1463 measured refinement:3->1 still verifies read-only, because survivor0 keeps
position42; its child is current and never enters a changed dependent wave
(d1463-original-root-first.log). Use4->2: retirement indices2/3 and survivor1
moves400->666, forcing the required wave. The two original Create roles, final
absence checks, and complete named rows remain. No currentness trigger changes.
The refined serial and worker runtime courts pass in
d1463-original-root-second.log; final gate coverage is recorded separately.

The refined required-wave fixture also exposed a metric-coordinate error in
this newly added court: program_output_settlement.rs:114-118 and
application_entry/demand/settlement.rs:91-95 explicitly exclude upstream runs
opened while another handle advances. A child's genuine cleanup of its parent
is therefore absent from the original root handle's lifetime counter. Preserve
the real-root-ran-again oracle using the existing per-root execution observer,
from immediately before the aggregate demand through completion. It names the
model's full-generation Native identity and excludes child producer roots.
No execution is added or inferred; every role, observation and row oracle stays.
This is the same observed-interval correction as D1459, not a relaxed contact pin.


### D1463 specialized scalar/aspect execution measurements

`aspect_input/courts.rs::perform`, `scalar_input/courts.rs::perform` and the
real sibling entry in `aspect_input/courts/source_affinity.rs` retain the
unchanged expected TOTAL producer count1 (and all original row, receipt,
revision, ABA, authority and source-affinity assertions). They install the
existing actual execution observer before the synchronous typed source edit
and measure through delivery. The original expression counted only the later
delivery handle, excluding the in-edit producer by its documented owner.
This is the same measurement correction as D1459, not a new zero-contact
expectation. `d1463-causal-ordinary-current.log` passes all aspect legs with
those original total-count oracles. At that checkpoint the two scalar provenance refusals remained red, and were
not reclassified as a metric difference. The D1470 continuation repair below
subsequently makes both green without an inspection fallback change.

## D1470: born-stale co-committed scalar continuation

D1468 measured master4405's ordinal0 as two delivery contacts and two real
producer publications, Native4 then5, with field revisions4. D1470 withdraws
D1465's zero-commit reobservation premise. A genuine P5 source+producer shares
Native3, then its own-effect successor publishes Native4 with one delivery
contact. The next delivery is measured independently. No no-effect ordinary
adoption from public#121 is introduced.

`scalar_input/courts.rs::perform` now pairs the old total-contact oracles with
one actual all-branch source Native commit, actual in-edit producer execution,
and Total already updated before delivery. Delivery expectations come from the
actual producer's `source_facts_superseded_by_own_effect` ordinals in its Native
commit evidence, never the ordinal, a fixture flag, or the delivery handle:
nonempty ordinals mean one genuine successor contact/commit; no own-effect
supersession means zero delivery contacts/commits. Existing no-op entries keep
zero executions. Every receipt, row, source, revision, ABA and authority oracle
remains; the new assertions are additional.

`scalar_input/output.rs`'s old `assert_ne(output_publication_commit,source_commit)`
is replaced only for the typed synchronous entry by equality: the ORIGINAL
producer and source share Native3. Deferred correction entries retain inequality.
The existing `producer_commit == root receipt` assertion remains: born-stale
settlement is Native4, while its original output publication is Native3. The
connection owner's documented distinction is preserved. Actual source-prefix
field projection excludes Total; the actual original producer's field-effect
projection includes it. The source-prefix view is the existing Native applied
source-effects owner, not a guess from the shared canonical envelope. A false
supplier using the genuine source receipt's own role roster is refused typed
under fresh real diagnostic admission. Inspection semantics, field revisions,
content normalization and postcondition rules are unchanged.

Evidence: `build-cache/history53-d1347/d1470-continuation-trace/` records
OwnPublicationContinuation, original producing Native3 retained, successor_of
matching its key, and inspection's existing fallback from supplier4 to
producer3. The original scalar-input court passes. The scalar-output court
passes with all old row/oracle assertions and the approved attribution pairing.
The retry courts cover serial and worker-only loss of pending delivery custody,
one real successor, Equivalent lookup of its actual idempotency binding, and
zero-contact/zero-commit follow-on delivery. Clean master4405 serial and worker follow-on measurements are complete: first
delivery2contacts/2commits, next0/0; P5 firstdelivery1/1, next0/0.
`d1470-master-followon/` retains commands and SHA restoration; its clean scratch
worktree was removed. Final-tree qualification is still pending; no completion
is claimed here.

## D1470 final audit of inherited observation intervals

`authored_ownership/sheet_boundary/court.rs::trim` and
`authored_ownership/support_currentness.rs::revise` preserve every expected
one/two execution pin and all currentness oracles. Their return value changes
from the later delivery handle's contacts to the actual handler observer over
the edit plus delivery, started before source publication. This is D1459's
entry/lifetime distinction, not an expectation change or guessed increment.
The same rule covers the ordinary-source custody report (unchanged total1),
the canonical edit helpers and numeric source journeys described above.
All135 authored-ownership courts pass in d1470-owned-applied-support-second.log,
including the unchanged foreign partial-shrink1/2, trimmed-sheet1 and
independent-writer assertions. The inherited lone-root test is unchanged.

The removal-hunk audit is retained in
build-cache/history53-d1347/d1470-assertion-diff.json (17 inherited measurement /
assertion hunks). Other additions strengthen receipts, effect attribution,
actual all-branch commit counts, retry and named rows. No new ignored test was
added beyond the two D1430 CP2b pending comparison courts. R1 was rerun against
the final D1450 legacy oracle: a genuine forced current republication fails
the actual all-branch0-commit assertion with1, then restores both owners by
SHA-256 (d1470-mutant-current-republication/).

## Final CP2a qualification

Final-tree complete root and Query gates are green against clean4405 with no
residual or identical-on-master debt: root8656/8631, Query8760/8622, including
doctests, standalone examples, compiler/proof prebuilds and full observed
coverage. Evidence: d1470-complete-qualification-comparison.json. The two
D1430 CP2b pending comparison courts are the only additional ignored cases.
The genuine deferred completion, original-root re-fixture, source-report,
canonical/causal, cold/worker and current-demand courts are all included.
Ordinary initial/Preserve byte and allocation captures remain exactly equal to
the same4405 baseline. The optimized private cabinet remains7commits/7contacts,
with live RetiredCell absence and its original27Create records asserted.
