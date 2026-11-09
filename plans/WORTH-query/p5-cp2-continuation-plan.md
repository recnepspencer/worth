# P5 CP2b: candidate rounds with one effectful publication per route

Status: approved with D1474 conditions A–D, 2026-11-09. Implementation follows
the conditions below and the independent re-review. CP2a is ececd430a1; prior plan a79812e1a6.
D1472 supersedes its multi-round origin, demand-control ceiling and scheduler
proposals. S6 still needs a separate plan review.

## First journey, prerequisites and completion

The first working journey is a generic owned required-output route whose
admitted source/decision support moves during the synchronous candidate, and
whose real verification execution applies no further effect. The source and
one final route publication commit once; a fresh demand reuses the final
qualified support. The actual producer runs are observed, rather than inferred
from the later delivery handle. No settlement exists before World cutover.

Prerequisites, immediately before their consumers:
1. After approval, merge public master 39ad4063e0 with a merge commit. Re-admit
   the roster/issuer and ordinary-part seams before changing candidate rounds.
2. Native must issue per-round applied coordinates and route-local next-round
   custody before Query can schedule a repeat. First-entry custody stays intact.
3. The delta net-plan and all-round read retention precede repeated commands.
   Receipt finalization follows final qualification, never an intermediate run.
4. The published 9.19/K6b read owner is rechecked before the Query read join.
   CP2b does not implement CAD stamp/read narrowing or change source_merge.rs.

Completion adds late writer/refusal, replay, cold allocator/frontier and exact
predecessor tests, runtime mutants and release-profile measurements. Every slice
ends at a coherent commit and report, green or with each residual filed as
identical-on-master. There is no claim that this finishes the cabinet's single
commit journey: #119/#100 and the later proprietary entry/S6 migration remain.

## Scope and hard boundary

At most ONE genuinely Native-effectful round per route may publish in CP2b.
There is no MultiRoundProducer variant, CHK tag, decoder change, version step,
public predecessor-origin variant, or durable JSON artifact. Existing CHK20.1
and ordinary/Birth/Preserve origins keep their meanings. A single-effect route
retains the actual effectful round's input; no-effect verification does not
become a second producing input. A no-effect-only final publication retains its
one actual completed run, never a fabricated writer.

Known NRU retirement-metadata producers remain on their existing post-commit
path A, exactly as CP2a. The cabinet and its public mirror still complete; they
are not redirected to a refusal or advertised as one commit. The two seeded
comparison courts stay ignored under public #119 / proprietary #100, with all
named-row and one-commit assertions unchanged.

If a new in-candidate path has already applied one effectful round and a second
would require a false single-input origin, Native refuses before publication,
with the route/source/family and #119 as the postponement cause. Nothing escapes
C. This guard is installed BEFORE repeats/netting can reach publication. It is
not applied to the retained legacy post-commit path. Existing owned routes retain CP2a post-commit successor delivery if their real
second run would be effectful. The new atomic-only candidate entry refuses; it
never publishes a false single-input origin. A speculative run must not leave
receipt, settlement or authority effects when the retained successor path wins.

If implementation identifies an existing productive, non-artifact producer
needing two effectful rounds, stop with its file:line and actual journey for
root's option (b) ruling. An intentionally non-terminating adversary is a court,
not evidence that a new permanent history variant is needed.

No publication/ownership/** or SHA/content-commitment edits; no whole-object
inputs, role-name interpretation, fake settlement, historical custody reader,
new JSON or weakening of ordinary conflicting-intent/Ambiguous refusals.
#115, public #119/proprietary #100, #121 and #94 are separate owners.

## Verified source and integration boundaries

Paths below use Q = worth-query-execution/src/domain_computation/primary_graph,
under workspaces/worth-query/crates; N = worth-relational/src, under crates.

- Q/application_contribution/producer/synchronous_outputs/wave.rs: drive()
  prepares each ranked candidate wave and discovers the next from its applied
  parents. This is the candidate scheduler. required_wave is post-commit only.
- Q/application_contribution/producer/coupled_assembly/preparation.rs: prepare_one
  distinguishes Retiring, Prepared, Verified and Source, appends an actual writer
  and retains its after-own pin. Prepared is not permission for another run.
- N/mvcc/publication/application_publication/coupled_assembly/part.rs:22-43
  rejects repeated first-entry routes. Its normalization at 92-99 and cumulative
  effect clone at 198-204 cannot become the per-round algorithm.
- The same directory's aggregate_read.rs clones the accumulated command prefix
  on each pin; pin_source also passes the accumulated touched-record set. Both
  are in the performance cutover, not concealed behind the new scheduler.
- Q/provider/application_attempt_state/commit_preparation/relational_commit/
  evidence_seal/postcommit_currentness/resolution.rs:41-45,88-135 owns admitted
  retirement/predecessor rebase and KeepSuperseded. Retire alone is no trigger.
- Q/application_attempt/fact/source_merge.rs merges one dependency at one Native
  revision. It cannot collect different round coordinates into one fact body.
- N/runtime/state/subsystems/record_identity/pending_allocations.rs: commit()
  consumes every reservation. recovery.rs:21 releases whole abandoned custody;
  no partial release of a netted record in a committing candidate was found.

Fetched public master and origin/integration are 39ad4063e0921b238b053c201047a3066cc88ca6.
The new owned_members/roster.rs compares exact expected/supplied full-generation
sets and rejects missing, repeated, foreign and dangling members. The changed
application_publication/completion.rs also checks the authored-retirement issuer
for restoration and historical selection. Preserve both checks when merging
CP2a's ordinary participant and source-prefix changes. Master still has the old
Preserve-only writer_custody interface and no participant_admission.rs; reconcile
by meaning, then re-run both owners' courts on the merged tree. Do not replace
master's roster with a Query inventory or drop CP2a's ordinary fences.

Fetch/status checked also through the authenticated private repository: main and
codex/query-919-cad-integration both point at e4ca4a74853df776a90c181279ff22361eaeebe8.
K6a S1's thread-read owner is landed (8fae6ec5716f). The cutover plan's K6b
section still schedules stamp read narrowing after K5/K6a; the fetched public
9.19 plan says Phase 6 unfinished. No separate published K6b lane was found in
the paginated branch inventory. This is published status, not a claim about
unpublished work on the other machine. Before slice 3, fetch again and inspect
any announced K6/K6b landing. Keep its source selection/narrowing policy intact;
CP2b stores the facts that owner admits, separately per round, without duplicating
its policy. No wait on an unpublished K6b result is needed for that separation.

## Route-specific certification and scheduling (B1)

Path A is Q/output_lineage/required_settlement/current_accepted.rs through
Q/application_contribution/producer/execution/source_readmission/
ready_on_selected.rs. Path B is Q/application_output_demand/currentness.rs.
Their post-commit NRU and UnsupportedFact distinctions remain unchanged.

No candidate pin is passed to a committed Ready certifier as a fake position.
The existing rebase/currentness owner gains an applied-data entry which uses
Native's admitted postconditions at exact D and the same rebase policy. It emits
candidate data, not CurrentComputation or Ready authority:

- qualified after own rebase;
- own-effect supersession with the actual fact ordinals;
- support changed by a later candidate writer, naming that fact and its round;
- full verification required, preserving UnsupportedFact versus NRU;
- the original typed read/control/resource refusal.

synchronous_outputs/wave.rs consumes these facts through its new round child
module. The route's entry posture below is retained explicitly. Query does not
select posture from Retire roles, command counts or a producer flag.

| Route/condition | Candidate decision after each actual run | Post-commit owner |
|---|---|---|
| Explicit authored/correction coupled writer, including multi-writer Preserve | One execution. Apply admitted rebase; NRU does not request another candidate run. Later foreign support change keeps the existing atomic refusal. | Path B/on-branch read-only full verification, as master. multi.rs:320-325 remains exactly two contacts for two writers. |
| Lone root/ordinary direct demand | No new candidate loop. Existing single publication and rebase. | Path B; unchanged lone-root court stays 1 then 0. |
| Owned synchronous required-tree route, qualified support | Continue discovery, retaining its exact parent/read support; no extra execution. | Path A after World, with the existing marks/full-verification distinctions. |
| Owned required-tree route with actual own-effect supersession, or its data invalidated by a later writer | Scheduler may execute a real next round under Native next-round custody, then requalify. B3 guard prevents a second effectful round from publishing. | Path A. Only a genuine owned no-effect verification proof tied to that commit's final route publication can install fresh G1 Ready support. |
| Required-tree NRU, including cabinet/public retirement-metadata mirror | Do NOT reschedule from NRU in D. Retain ordinary required-delivery custody and the first publication. | Existing path A disclosure/execution stays after commit. #119/#100 owns removing this artifact. |
| UnsupportedFact without actual supersession/moved support | Funded full comparison at the currentness owner; failure is not guessed Current. No automatic producer run solely because marks lack a posting. | Existing path A UnsupportedFact-only full verification. NRU is never accepted through this door. |
| Ordinary part, including D1470 Initial-absence scalar | A candidate verification proof can support reads/discovery only. It never mints Ready or adopts an ordinary publication. | Existing certifiers/real successor chain in delivery. D1470 remains 1 contact/1 successor publication, then 0/0; #121 stays postponed. |

The applied rebase must preserve admitted retirements as postconditions, so the
multi-writer's retire/create cell succession (handler.rs:290-328) is not an
oscillator. Do not compare literal pre-run facts before their own postconditions.
A later writer's change is distinguished from that same writer's own effect.
Permission, principal, request and installed meaning stay at C for every run.
Final support remains qualified at the complete D, not merely the last local pin.

A new generic owned route will exercise no-effect/effect/no-effect runs, induced
by distinct required writers moving an external declared input while its output
cutoff changes only once. Use hard-coded input values 0, 1, 3 and parity output 0, 1, 1:
its producing input is the middle 1; final verification supports 3. This is a new
required-tree scenario, not a change to explicit coupled-writer refusal policy.
First compare the actual legacy repeated-demand journey, then the candidate's
complete rows and typed predecessor observation. Do not infer re-entry from
roster membership or bypass installed connection order.

## Limits and the oscillator consequence

Choose the mutation builder, not WorthQueryOutputDemandControls. Add a typed
ApplicationProducerRoundLimits beside entry controls in worth-query-publication,
re-exported by Host. This is Query-specific policy, not cross-runtime meaning.
Pass its finite numeric bounds into execution-owned attempt counters; execution
must not depend upwards on publication or add a second policy authority. Live route/attempt counters belong to
Query's synchronous attempt. Carry the policy through request, staged request,
capture, retry and fixed/performed paths. Existing calls use default per-route 64
and total 4096; callers may choose other positive finite bounds. Replay of an
already committed source key resolves before starting any producer.

Proposed DX, implemented only after approval:

```rust
request.advanced().mutate(intent)
    .producer_round_limits(ApplicationProducerRoundLimits::new(
        NonZeroUsize::new(64).unwrap(), NonZeroUsize::new(4096).unwrap()))
    .without_source().idempotency(&key)
    .execute_performed::<Program, Root>(&host, allocation)
```

Count actual handler starts, including no-effect executions, not certification
reads. Reserve both route and attempt quota before dispatch; resource stops
keep their original causes. Exceeding a finite policy is a typed producer-round
budget refusal naming the route/source/family, actual count and bound. It does
NOT prove mathematical non-convergence. Do not introduce a NonConvergentProducer
classification based only on reaching 64; it must never label master's accepted
one-run Preserve or a finite producer needing a larger caller budget.

C2 is a deliberate deterministic source-read oscillator: its query reads a
source field which its producer flips, separate from the authored field. Each
real publication supersedes that consumed source fact. The unmodified master
reference must actually continue for 64 rounds, stopped by real request
cancellation via an existing scoped interleaver/observer, not by changing the
handler. No blanket retirement gate or failed setup qualifies it.

Under B3, that oscillator's candidate leg refuses #119 on its second effectful
round before World, NOT NonConvergentProducer at 64. A 64-effect candidate cannot
be both admitted and forbidden by B3. Release evidence therefore measures the
real 64-round legacy reference AND the early atomic candidate refusal separately.
No new NonConvergentProducer kind is needed in this reduced scope. A genuine
productive non-artifact case found by that investigation triggers root's B3 (b)
stop. A separate many-route finite court tests the attempt ceiling and a limit 1
owned-verification court tests the route ceiling, both as budget refusals.

The capability index and consumer guide must document both limits, the typed
budget refusal, preserved post-commit NRU/ordinary delivery, and the #119 seam.
No new policy changes intent identity or persists a JSON execution envelope.

## Native net plan, custody and identity (G2/G3)

Native owns one C-to-final delta compiler. Each round is normalized once against
its exact prior applied state; the net plan incrementally updates only touched
records, bindings, keys, fields/aspects, relations, content and foreign effects.
Cross-writer conflicting patches still refuse. Sequential same-route commands
use this owner, not a weakened ordinary merger against C.

Keep two indexes: route state (existing exact route semantics, next-round linear
custody and effect count) and creator custody keyed by full RecordRef including
generation. Only the same route's own create/delete pair may cancel. Deleting
another route's or the authored prefix's fresh record refuses typed naming both
writers and the record. Existing current-publication destruction/retarget fences,
required-owner coverage and ordinary-part admission all still run.

N1 and N4 cover genuine C deletion and old/new same-key replacement. X1/X2
refuse destruction of another route's or the authored prefix's fresh record.
N2 (same-route create/delete), N3 (later fresh update), N5 (exact revert), and
M-E2's elision proof are postponed to #119/#100: they require two effectful
rounds and cannot publish under B3. Do not build prepared-only cancellation
machinery to manufacture coverage of an unreachable entry. The B3 guard reserves
the boundary; no ID elision or partial release is implemented here.

Fresh IDs remain consumed by Native's existing reservation owner. Cold reopen
pins supported one-effect/no-effect identity/generation frontiers against the
legacy journey. Retained post-commit cabinet churn still allocates and retires
its IDs as on master. A committing candidate with netted fresh IDs remains
postponed, including its cold allocation-replay cutover.

B3's effect counter is Native-issued from actual application outcomes, including
observable output meaning, not a count of submitted commands or receipts. It
excludes receipt-only bookkeeping. A real second effect refuses before receipt
finalization. O1 mutates a no-effect verification into effectful and must break
the accepted one-effect journey; O2 substitutes first/last for its middle input.

## All-round reads, final evidence and receipt cutover (B2)

Before-commit conflict support and after-commit currentness evidence are distinct
products of the same actual runs. Retain immutable (candidate, route, round,
Native applied qualification, FactPayload) spans. Never flatten them into the
one-revision AuthoringSourceFacts map, merge them in source_merge.rs, overwrite
one round with another, or replace the first round's reads with the last.
Native applied revision increases for every round; parts.len() is not a round
coordinate when a route repeats. Equal values at two rounds are still two
read coordinates. K6b may narrow each span's fields; it does not merge spans.

At final qualification, the existing rebase owner resolves each span against
its authenticated own postconditions and final D. All external C reads remain
in commit conflict support even if their writes net away. Recorded currentness
support retains final-qualified observations plus proofs of how intra-candidate
reads were discharged; it is not a claim that the final producer read C.

CP2b never elides a fresh ID, so it must not record a dangling reference or
manufacture a retirement observation for a record that never published. The
cross-round elision proof and its M-E2 court move to #119 with cancellation.
Retain every reachable external and produced-record observation through the
existing rebase owner. No new durable proof journal is introduced.

Only the final route publication receives an idempotency binding and real
producer receipt. Intermediate rounds are private completed executions with
read/effect evidence, never Committed/AlreadyCommitted deliveries. Delay Query's
real receipt writer/appendix until Native final net and provenance qualification.
Use the sole effectful round's real producing input/key; zero-effect final
verification contributes fresh G1 support, not a fictional producing input.
The no-effect/effect/no-effect fixture checks the middle input 1 rather than 0/3.
If no round changed the route, preserve the actual final single-run publication
or existing adoption posture; do not invent an ordinary adoption for #121.

The actual one final Native result supplies the producer receipt. Query prepares
lineage/settlement storage before cutover and installs final verified backing
when the real group completes. ProductUnpublished keeps its existing recovery
custody; it never exposes a guessed successful receipt. Lost delivery/retry after
co-commit rejoins that final receipt, with zero new commits, hot and worker-only.
Explicit lookups for distinct intermediate keys must not return Equivalent.
For a retained legacy route, keep its real per-commit receipts and succession.

Follow-on demand courts compare master and candidate explicitly: preserve
D1450's NRU first verification contact(s)/zero commits, then 0/0, on legacy
retirement metadata. Supported owned final verification may already establish
G1, but assertions pair zero delivery contacts with actual in-edit executions,
exact final rows, the final receipt and zero extra all-branch commits. D1470's
ordinary scalar remains1/1 delivery then 0/0, with no expectation change.

## Destination topology and cost

Q/application_contribution/producer/coupled_assembly.rs stays an orchestrator
(currently 341 lines). New round logic lives under synchronous_outputs/rounds/:
mod.rs (one concrete coordinator), disposition.rs (route/certifier mapping),
limits.rs (attempt counters), reads.rs (round-coordinate spans), finalization.rs
(the final writer/receipt handoff). wave.rs drives it; preparation.rs supplies
actual prepared executions. No second post-commit required_wave is created.

Native coupled_assembly/continuation.rs owns linear next-round custody;
net_plan/ has record_delta.rs, creator_custody.rs and finalization.rs under a
small entry. aggregate_read.rs delegates incremental candidate observations to
the existing aggregate_identity owner. Applied pin retention carries a funded,
shared immutable touched-record index per revision, not copied cumulative Vecs.
The postcommit_currentness owner gains an applied-rebase child, with no copied
rebase policy. New legal authority, if actually needed, belongs in worth-proof;
Native's runtime completion evidence itself grants no permission.

Remove full-prefix normalized_intents_for_merge and effect.clone on each round.
Aggregate projection and pin creation must also avoid rebuilding every prior
command. Extend the existing aggregate owner with delta-fed, revision-scoped
read preparation; do not create a competing aggregate reader. Prepare final
canonical commands/effects once, then invoke existing content/ownership/hash
owners unchanged. One final part per route reaches them.

Target per-round complexity is O(own work + touched records/incident structural
granule), with indexed updates O(delta log U), U distinct touched records. Final
sealing is O(U + retained reads). No O(rounds × accumulated prefix) normalization,
cloning, roster reconstruction or serialization. Fund retained spans and maps;
worker-only paths use execution::request_stage::checkpoint.

## Courts and named runtime mutants

Each mutant must run its intended semantic leg, fail a runtime assertion and
restore source by SHA-256. A compile error, setup refusal or deadline alone is
not a kill. No source fingerprint/JSON certification system is added.

| Court | Native/Query mutant and required failing assertion |
|---|---|
| C1 first-entry/continuation | M-C1 skips consuming next-round custody; stale/duplicate/foreign token is then wrongly accepted. Existing first-entry route refusal stays. |
| B1 route map | M-B1 uses NRU as candidate-repeat; unchanged multi-writer contact 2 total and lone-root 1 then 0 go red. |
| C2 oscillator / limits | M-C2 removes the B3 second-effect stop: the candidate no-effect/provenance assertion fails; M-L1/M-L2 omit route/attempt quota and the actual bounded refusal goes red. Legacy64-round reference must first prove the intended oscillator, not a producer master accepts once. |
| N1 genuine deletion | M-N1 drops the genuine C retirement; exact live rows/origin fail. |
| X1/X2 creator custody | M-X drops same-route creator check; M-ID nets by ID only. Foreign-route and authored-prefix destruction must be wrongly accepted at the real Native net-plan boundary. Final public B3 refusal remains independently tested. |
| N4 identities | M-N4 conflates old/new same-key generations. Native final plan and supported cold frontier/identity assertions fail. |
| K1 all-round reads | M-K1 drops round 1 external support; a concurrent write to that input is wrongly admitted even when later output is equal. Keep round coordinates distinct. |
| B2 recorded support | M-E1 records round1 only; the middle-input/final-support/follow-on oracle fails. M-E2 is postponed with unreachable cancellation under #119. |
| B2 receipts/retry | M-R1 gives an intermediate run a receipt; its key wrongly resolves Equivalent or extra receipt/commit appears. Hot/worker retries require the actual final receipt. |
| O1 effect provenance | M-O1 marks no-effect verification effectful; the supported one-effect edit wrongly refuses #119. |
| O2 actual input | M-O2a/M-O2b choose first/last input; the predecessor read must return 1, not0/3. |
| Ordinary proof boundary | M-Q1 mints Ready from an ordinary candidate read proof; unchanged D1470 delivery 1/1 and original producing attribution fail. |
| Exact origin kind | At application_query/output_source_observation/predecessor.rs:30, assert HistoricalSelection and AuthoredRestoration separately through fresh read authorization; M-P1 swaps their causes and fails. Public origin enum remains unchanged. #94 stays postponed. |
| Commit/orphan/atomicity | M-G1 publishes a side-branch/readiness commit; all-branch observer fails. M-A1 publishes a prefix on later failure; C/head/complete rows fail. |

No additional ignores. The two comparison.rs seeded courts keep their semantic
oracles and exact #119/#100 ignore reasons. Slice 1's full gate records both as ignored. After those owners land, their independent legacy
named-row oracle remains the acceptance target for one-commit cabinet cleanup.

## Implementation slices after approval (B4)

Estimates below are planning estimates, not measured performance or binary caps.
Each includes optimized-build iteration and courts; new core logic is concrete,
with generic shells only where schema/provider dispatch actually needs them.

| Slice | Deliverable, dependency and size estimate | Release evidence and named mutants | Stop/qualification |
|---|---|---|---|
| 1 | Merge 39ad; route map, linear custody, B3 guard, builder limits and docs. About 200–350 production + 180–280 court lines, split at 400 lines/file or less; 1–1.5 lane-days including release builds; estimated native text delta 30–80KiB. | C1/B1, multi-writer/lone-root/D1470 unchanged; M-C1, M-B1, M-C2, M-L2. Existing one-effect owned journey stays green; repeat dispatch remains disabled. | Commit/report. Complete affected Native/Query owners and root/Query comparison green, or each residual filed as identical-on-master. Stop for changed authority/provenance or a real non-artifact multi-effect journey. |
| 2 | Delta net-plan, same-route creator ledger, incremental aggregate/pin support, consumed-ID policy. About 160–280 production + 160–260 court lines; 0.75–1.25 lane-days; estimated native text 20–60KiB. | N1/N4/X1/X2, custody/first-entry after master merge; M-N1, M-X, M-ID, M-N4. Cold supported frontiers and unchanged legacy churn frontiers. Release cabinet-scale 27-retirement workload plus touched-record growth. | Commit/report, same green-or-filed-baseline rule. Stop if allocation replay would require a format change or forbidden owner edits; do not claim publishing netted IDs while B3 blocks it. |
| 3 | Fetch K6/K6b status again; distinct round spans, applied rebase disposition, one final receipt/lineage handoff and owned-only final verification. About 250–450 production + 220–350 court lines; 1–1.5 lane-days; estimated text 30–90KiB. | Middle-input fixture, K1, final support/no netted fact, hot/worker crash/retry/follow-on; M-K1, M-E1, M-R1, M-Q1, M-O1, M-O2a/b, M-L1. | Commit/report, same green-or-filed-baseline rule. Stop with BLOCKED: owned two-effect route if the retained CP2a successor path cannot survive without a contract change. Also stop rather than duplicate K6 policy, fabricate Current/Ready, or require #121/#94. |
| 4 | Exact-kind predecessor courts, residual lifecycle/atomicity, final docs/ledger and release qualification. About 120–220 court/doc lines; 0.5–1 lane-day; no stored/public enum growth expected. | M-P1/M-G1/M-A1; rerun affected CP2a appendix/currentness/receipt mutants, exact bytes/allocation, private 7/7 and complete root/Query baseline comparison. | Commit/report, green or each residual filed identical-on-master; no changed assertion without reason. Then stop for the separate S6 plan review. |

Slices 1–2 install and court the real Native preparation/qualification owners
without enabling Query repeat dispatch. Slice 3 connects the first new repeat
journey only after delta netting, all-round support and final-only receipt
preparation are present together. No checkpoint ships a repeat with partial
conflict support, intermediate receipts or false single-input provenance.

Every slice uses the repo's supported root-landing/query-landing drivers and
focused owner courts; no nested Cargo framework or new test manifest. Run the
boundary/context/format/dirty-cap checks on the scoped changes. Source-only
reason updates in this plan revision need no runtime-gate rerun; they do not
unignore or alter a court.

Release telemetry uses the committed optimized profile, records workload,
hardware, warm/cold posture and repetitions, and compares growth/bytes rather
than fitting exact incidental work counts. Measure the actual source-entry
edit (including all synchronous producers) on the cabinet-scale fixture; keep
interactive supported/refused edits well under roughly 1 s. Also record the
64-round legacy oscillator reference and the candidate's early #119 refusal
separately. Never present 64 successful effectful candidate rounds as CP2b proof.
A slope in accumulated prior work blocks the slice: fix net/pin/aggregate
structure, not capacity or deadline. Do not claim the postponed cabinet final
single commit from the preserved 7/7 scratch probe.

## D1474 condition A: existing delivery parity

A convergent owned producer with two effectful runs must first be measured on
master, then complete through the retained CP2a successor path. New in-candidate
verification may adopt only a genuine no-effect result for an owned route.
The changed-assertion ledger must name each existing owned court that actually
moves from delivery 1/1 to 0/0, using its typed in-edit own-effect evidence and
D1459's one Native commit/real producer receipt/value-before-delivery pairing.
Do not presume all own-superseded rows have a no-effect successor. The existing
NRU shrink, explicit multi-writer, lone-root and D1470 ordinary scalar courts
keep their current expectations. Before slice 3 enables repeats, inventory the
owned born-stale cases and record any changed cases individually. An empty
migration list is valid only if the new owned verification court supplies the
actual newly reachable path; it is not evidence from the ordinary scalar.

The default total limit also needs a release-profile edit with 4,096 actual
handler starts, not a loop over counters. The final allowed start completes;
the 4,097th is refused before contacting a producer. Measure the edit's bound,
retained support/memory and elapsed time independently of the 64-effect legacy
oscillator. If the full bounded edit cannot satisfy the interactive target,
report the measured slope and repair its owner rather than changing the bound.

## D1474 implementation checkpoint: Native read locality

The preparation foundation now retains each immutable applied pin and captures
its successor from only the incoming sparse writer image. Allocation bindings
and mutation journals insert incoming entries instead of rebuilding their union.
The original C-to-final validator and publication owners remain authoritative.

Applied field lookup now derives data entries from the actual selected C
index generation through intervening applied deltas. Each image retains only
its own entity IDs and physical root. The nearest cached entry root is shared;
key differences and posting updates use the same Native maintenance functions
as committed indexes. Persistent cache snapshots permit funding and request
checks before traversal, without holding a cache lock. Retained entries keep
their original generation's funding alive. This issues no generation, Current
observation, publication, or Ready settlement, and changes no stored format.

The new-source hot/worker courts kill omission of the actual field delta at
runtime; the source edit cannot resolve its new ModelKey. SHA-256 restoration
is recorded externally. The restored synchronous owner suite remains 137 green
with the two authorized ignored comparison courts; no oracle was changed.

This is a foundation checkpoint, not completion of slice 2. Aggregate read
preparation still copies the cumulative business plan and recomputes unrelated
changed cohorts. Its next cutover must stay at the existing cohort/transition
owners, compute only the requested complete C/D overlap granule, preserve the
per-delta derived-law checks, and leave final private consequences to the one
complete C-to-final admission. Repeat dispatch remains disabled until the
all-round support and final-only receipt handoff are complete together.

The index/pin foundation is larger than the initial line estimate. Including
aggregate locality, revise slice 2's estimate to 900–1,300 production and
220–380 court lines, split by the existing owners at 400 lines/file or less;
2–3 lane-days and an unmeasured 80–180KiB optimized-text estimate. This is not
a size exemption or a measured release result. Cancellation/elision remains
postponed under #119/#100; this increase does not reopen that scope.
