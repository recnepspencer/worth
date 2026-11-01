# Phase5 Bank current accounts boundary review

## Authoritative entry and preserved truth

Bank's AccountDiscoveryQuery/Binding v1 in bank-domain/queries/account_discovery.rs
is parameter-free, Public-disclosed and authorized by DiscoverOwnAccounts. Its
actual Principal scope uses BankPrincipalBinding/ExternalPrincipalMapping,
principal_field PrincipalIdentityField, and three real paths: PersonalOwner;
AccountAuthorizedUser -> AuthorizationAccount; reverse BusinessOwner ->
BusinessAccount. Ordinary current+pinned support, AccountIdentity projection,
AccountId Ascending ordering and 1024 result bound remain its existing contract.
DiscoverOwnAccounts currently allows the actual Principal rather than a mutable
account-grant policy; do not invent another permission model for a test.

The ordinary receiver is bank-server/ordinary/read/query/application_execution.rs
BankReadyQuery<AccountDiscoveryRequest>::execute_current, via the actual public
application request. BankAccounts contribution owns installation and provider
configuration. Add a distinct current_accounts current-only Parent to preserve
v1 ordinary/pinned behavior, using the unchanged actual Source binding and full
principal contract. No separate Public Source or application roster is authorized.
PendingPayments uses guarded paths and remains unsupported by Native Source.

## Mechanism ownership and cutover

Only certification fixtures currently own generic prefix mechanics. Production
Bank cannot depend on that fixture. Existing worth-execution::storage owns
controlled ordered storage/reduction mechanisms; Foundational explicitly excludes
hot-path storage algorithms. The prefix_tree AVL insertion/removal/rotation/height
and coordinate extrema are string-free caller-storage mechanics with checkpoint
and invalid callbacks. Promote just those to Execution storage, preserving one
implementation. PrefixOrder, UTF8 comparison, watch and prefix query traversal
remain in consumer_values. Delete its old AVL implementation; reexport the one
production mechanism under the example's existing prefix names.

Bank server owns a thin finite NodeWriter adapter over genuine SourceSlots,
ordered by slot.cmp. Maintained slots are stable owner+ordinal0+full-generation
entity occurrences. Validate before/after row access independently of comparison.
All/unit membership requires every Source row; Query's checked exhaustive receiver
emits candidates and existing materialization applies AccountId order. O(N)
all-output reads are honest; there is no pruning or search claim. Warm changes
must perform O(k log N) insert/remove rather than scanning/replacing a roster,
renumbering slots, or rebuilding the root. Source snapshots, READ, Native
maintenance, candidate completeness and publication remain Query/Native-owned.

## Dirty edges and evidence

Reject unrelated scope/installation handles, equal-head retained current-only
reads, cancelled/original-resource refusals. Preserve the old Source pinned read
and ordinary account APIs after actual mutations. Check union deduplication,
multiple remaining membership paths, grant/revoke, actual account creation,
empty principal and source/result limits with actual Bank receivers. Reuse
ordinary_reads testkit and external-rail-server court; no new SDK family/root.
Existing independent SourceAuth courts cover Source-only policy custody.

Generic mechanic extraction needs pure owned-storage falsifiers plus existing
prefix/model/runtime tests; no runtime court in Execution src. The final Bank
qualification must expose build-vs-warm-update and full original work, and
preserve capability/purpose/activity continuation paths through existing grouped
ordinary reads. No managed continuation, ranking or guarded Source support is
inferred from this additive all-accounts receiver.


## Resolved observation boundary

The independent reviewer confirmed the existing Bank certification_controls.rs
passive observation seam. Carry provider-owned atomic attempt/change counters
through Accounts configuration, installation and refused workflow-binding/runtime
custody. Return only copied counts from one doc-hidden fenced _for_test method.
It grants no Node/Source/Query runtime handle and cannot change provider behavior.
Use counts plus actual committed mutation and ordinary rows to distinguish warm
incremental update from hidden rebuild. Do not create a new invoice harness whose
O(N) all-result work would confound this specific claim. The existing caller
fence will name the exact current-accounts court/testkit path.

First Bank build found existing exhaustive output-settlement/attempt projections missing upstream ManagedIndexSourceDenied. Add the corresponding closed Bank category and explicit mapping in its current owners, preserving their descriptive-only role; ordinary current_accounts managed read failures already retain the actual full ManagedIndex denial through the existing Bank ManagedIndex variant. No wildcard or generic failure substitution.

Runtime schema feedback: a plain Bool probe lacks the required bounded canonical-json admitted decoder and is correctly UnsupportedProbeProfile. Use an empty WorthValue struct with an explicit bounded canonical String profile as the true all/unit probe. No false branch or relaxed framework profile; request/search semantics remain unchanged.

160-account setup hits the unchanged original Bank16M work ceiling before the result-limit test. First temporarily print the existing provider attempt counter snapshot at the actual installation refusal, as independently requested by reviewer. This bounds attribution (before/inside/after provider) without changing budget, population or adding an API; remove the diagnostic after the cause is established. Review/replan any substantive performance correction before edits.

The failed160 receipt reports zero provider callbacks. Existing StructuralRelationView already caches successful exact handle+registry+audience admission and LifecycleSource delegates correctly; no second cache is justified. Temporarily bracket Native authored-history preparation and scope-closure discovery with stage-only diagnostics on the actual original request. Remove all diagnostics after attribution. No semantic edit or budget change before the actual cause is known.

Stage attribution: activation history completes; actual Bank seed exhausts inside authored_history::prepare before its end, before maintenance scope discovery. Add temporary count-only brackets after AuthoredSourceInputs collection/delta construction and around history scope discovery to identify that receiving path before optimizing it.

Detailed trace: history input collection completes647 records/803 incidences and803 deltas, then scope closure begins482 Source records and exhausts. Bracket affectedness vs final inverse row pullback using the supplied Native work meter snapshots (diagnostic cumulative observations only, not exclusive whole-request invoices). No collector or memo optimization yet.


## Measured amendment: evaluation-local root memo comparison

Native history affectedness completes at1,655,513 work; final inverse rows0/64/128
observe1,656,959 /6,856,818 /12,071,794 before the unchanged16M original ceiling.
This is cumulative Native work, not an exclusive whole-request invoice. No provider
callback is reached. The no-Named three-path root repeats full root definition
identity text comparisons at every memo access. Existing admission caching is
correct and is not changed.

Independently reviewed repair: evaluation/memo.rs retains actual Handle and adds
private Root versus Named key comparison class. evaluate_position first performs
existing exact registry/audience admission and then retains that actual root in
its one local Evaluation. Root keys compare class/node/anchor/reverse without
root text; Named keys compare complete schema/version/name and local coordinates.
Navigation::Derived is the only definition-changing constructor. It resolves the
actual target through the same registry, normalizing back to Root only on exact
entry Arc identity with the retained root. No pointer ordering, name/digest
classification, global ordinal/cache or cross-evaluation key reuse.

Comparison work follows the changed fields: class/node/member/reverse fixed
comparison layouts plus actual anchor names; Named adds full identity version
and text. Projected members retain their full coordinate fixed-layout/name quote.
Handle/frame/storage retention accounting stays sizeof-based at its current owner.
A zero-unit final original control check follows actual key classification. Every
initial admission, physical/member read, support capture and failure stays present.

Pure owned-source/registered-recipe tests cover root coordinate separation,
Named nonaliasing, exact-owner normalization and Eq/Ord transitivity, plus real
union/intersection evaluator behavior and a controlled stop. They construct no
Native runtime or foreign runtime in src. The unchanged160 Bank runtime and
existing managed correction/Native derived admission cases are the integration
pair. Remove all stage diagnostics after attribution/repair. Collector duplicate
elision is out of this fix unless further measured evidence requires it.

After memo repair, both history and maintenance discoveries finish (161 scopes each; row0/64/128=569279/1864820/3164148), but original16M still stops before provider build/update. Next temporary count-only brackets cover Query cold acquisition, actual occurrence count, projection progress and provider-wrapper entry. Zero build/update counts do not exclude capacity/writer setup; no assumption that all callbacks were absent.


## Measured amendment: current Native row-read custody

The first actual cold scope is ID484 with160 rows. Its forward acquire_all and
occurrence issuance complete; projection reaches rows0/64/128 then original16M
stops before the generic provider wrapper. Native Rows.with_field currently
reprepares a complete inverse membership for every field despite the caller
retaining the actual current forward read. This is real repeated evaluation.

Before edits, reviewer cleared PreparedManagedIndexSourceRowRead<'borrow,'source>
owned at Native preparation/row_read.rs: exact &Rows, &Occurrence, and actual
ForwardBorrowed prepared ticket or InverseOwned prepared ticket. A forward
constructor validates actual program+scope, current projection.read(path/scope/
shape) and paid binary search of full-generation membership. Inverse preparation
performs the existing real read once. The token's field callback validates its
own actual current ticket and retains unchanged field_at audience/kind/control.
It retains no request or cached allow. Lifetimes retain the actual loan/ticket;
values still cannot escape the callback. Existing with_field delegates through
this same inverse issuer, preserving stale/removed occurrence refusal.

Query's existing project function receives optional actual forward support only
from cold acquire; it prepares one row-read per entry. Warm retained coordinates
use actual inverse preparation. Root field projection consumes that token; child
route behavior is unchanged. Errors never fall back to a coordinate-only path.
No source membership roster is cloned, cached globally, widened or forged.

Existing external Native managed_source field court/SourceOwner will prove
current positive fields, foreign scope/committed-vs-proposed ticket refusal and
removed retained occurrence refusal; existing field optionality and original
stops stay exercised. Bank160 under16M is the causal runtime acceptance case.
Remove every temporary stage diagnostic before final commit.

Row-read repair completes all160 projection rows and enters build attempt1, but original16M still stops before the next scope. Temporarily bracket actual Bank insertion progress and build return to distinguish constructor cost from downstream validation before adopting the independently reviewed sorted bulk constructor candidate.

Actual Bank insertion prints end160 before failure. Add temporary post-provider/seal/qualify and capture/apply/admit stage brackets to locate the remaining real receiver; no semantic change until measured. Native row court must create an actual declared aspect containing only unprojected other so its optional value-field absence is readable through existing structural state. Missing whole record/aspect state remains its existing denial; do not relax it. An early history_owner=None skip is rejected: packets still establish postings and complete intervals.

## Measured amendment: sorted finite construction

Diagnostic7 reaches provider return, writer seal, capture160 and apply end; original16M stops in cold structural admission. The existing all-slots iterator is strictly ordered by stable SourceSlot (fixed owner plus ordinal or full-generation entity); the Bank cold adapter currently pays O(N log N) reads/replacements/rotations to rediscover that order. Replace only this cold algorithm with checked O(N) postorder construction. Keep Query cold structural admission entirely intact, including source validation, exact bijection, parents, acyclicity and final control checks. No budget/population adjustment or history omission.

Execution storage ordered_tree/sorted.rs owns build_sorted_ordered_tree(count, fallible iterator, caller storage). A count-halving recursion carries only child root/height/extrema on O(log N) stack, checks every adjacent actual comparator is strictly Less, checks exact count including extra/missing, propagates iterator/storage stops, appends each node once, performs no storage reads/replacements and final checkpoint. Arbitrary-order builder/insertion/removal remain unchanged. Bank validates each genuine slot with sources.get before yielding it; no cloned roster. Reexport through existing Execution storage and Query host runtime facade. Pure independent BTreeSet/model checks for varied counts, operation counts and original stops/order/count negatives, then unchanged160 runtime acceptance. If savings insufficient, measure the next real work before expanding scope.

## Measured amendment: one-pass cold parent walks

Sorted construction preserves all checks but160 still stops in the same cold admission. Independent reviewer identified an actual duplicate traversal there: the DFS rewrites every local Gray mark Black and rereads each actual parent solely to finish the prior walk. Replace Gray/Black with one invocation-local map ordinal->walk seed. Skip any already visited seed before starting; a new seed has no prior tag. Encountering this seed tag within its active walk is a cycle; any other tag belongs to a previously completed walk and is a valid stopping point. Every newly visited ordinal is inserted once and its actual parent read once. A failed walk returns immediately, so no unfinished tag becomes a successful prior-walk witness. Preserve node_count bound, paid map access and original live polling. No persisted authority, public API, history shortcut or missing-structure allowance.

Keep existing cycle/sparse-chain/original cancellation/memory-release courts, strengthen parent callback counts to exact once and add exhaustive independent functional graph oracle including duplicate seeds and paths merging into completed walks. Execute the narrow Query owned-mechanic court then unchanged Bank160. No further expansion without receiving-path evidence.

## Measured amendment: exact-context discovery reuse

160 still exhausts cold admission after sorted construction and one-pass walks. Both authored and physical Native closure discoveries are visibly complete161-scope evaluations over the same actual draft; each repeats about3.8M Native work. Counts alone prove no correspondence. The record filters intentionally differ for field/no-op history, so neither subset reuse nor missing-history-owner skipping is permitted.

The current authored and maintenance stage wrappers create separate ExecutionWorkLoan scopes. A retained authored meter would be closed in the later stage. Combine only these two existing preparation parts inside one original Source stage, with one invocation-local SourcePreparationContext borrowing actual draft, original execution, runtime, selected-before and proposal. Preserve the no-program/no-affected/no-authored-history fast path without adding a funded stage. Pass the actual existing budget; never mint another allowance. Existing stage final native stop delivery remains at the outer owner.

The history preparation still builds the entire normal packet, all impacts and material/posting evidence. It may carry its actual completed closure and original work/memory meter into an invocation-local discovery list only when its relevant Entity RecordRef sequence equals the actual maintenance changed_source_records sequence AND its complete authored adjacency sequence equals the actual physical delta sequence. Exact sequence equality is deliberately stronger than normalized set equality: harmless ordering differences decline reuse. Compare full values under original paid work and a final live check. No digest/count/proxy proof. Match the exact borrowed context, actual first group and actual program on consumption; these retain registry/audience/limit and before/after custody. No closure reconstructed from packet coordinates. Carry support and its original funded meter together. The provider's current logical reads still use fresh actual views.

If input, group, program, context or unsupported posture differs, run existing complete maintenance discovery with its real meter. Authored no-op/scalar/endpoint extras preserve full discovery and all existing field-omission behavior. Empty scope and group-union semantics remain unchanged. A narrow helper module owns the local completed-discovery custody; no public facade or control is added. Existing Native managed Source and process work-limit courts, new current-loan court, Bank160 original16M and broader consumer Native history/correction tests are mandatory evidence. Remove every temporary diagnostic after the receiving fix is established.

Exact record comparison streams the actual publication records through the one existing maintenance relevance predicate; it allocates no duplicate input roster. The fallback alone creates a fresh maintenance meter/views. The no-program and no-history/no-affected fast paths remain outside the outer stage; an installed roster plus ordinary authored input still creates every required empty packet.

## Measured amendment: unchanged authored record-use entries

Exact-context reuse is taken in the real Bank160 seed (one actual closure trace), and141 scope builds now complete before original16M stops during scope142 acquisition. Preserve this real improvement and all161 required scopes. The collector record method still persistently reinserts every repeated existing record even when prior.union(incoming)==prior: allocated lifecycle records are revisited by canonical physical deltas. The reviewer previously identified and cleared this exact local elision. After the existing paid lookup, compare the actual two provenance flags; unchanged union performs the original final zero-work/liveness check and returns, while absent entries and added flags retain their actual ordered insertion. No command, incidence, first entry, history impact, field omission or authored no-op is omitted. Add Eq/PartialEq for the actual flag value only; no proxy/digest and no generic map behavior change. Existing actual authored field/membership history and original-stop courts plus unchanged160 are the proportionate receivers; no trivial mirrored flag test or new runtime family.

Restore the original initial check_live before affected-group metadata routing. Exact-comparison work must include the installed physical-source count used by the shared relevance predicate, not only RecordRef bytes; count that metadata under the original budget and quote both-side kind comparisons. Admit the invocation-local discovery-list pointer probes through the same actual budget, with a final zero-work poll before moving its completed closure. These are accounting/liveness corrections, not a new allowance.

## Measured amendment: immutable row-loan proof consumption

Collector elision reaches154/161 actual Bank roots before original16M stops. Independent reviewer traced both row-read backings: StructuralRelationView fixes immutable state/access/source, and Selected borrows the actual rebuild's unchangeable expected entry/projection root/basis/generation. Only funding counters are interior mutable. Private PreparedManagedIndexSourceRowRead already validates actual current read, program/scope/view and full row membership at construction and owns/borrows that exact ticket for its lifetime. No caller can substitute or mutate its Rows, occurrence or witness.

Remove repeated projection.read from this token's with_field. Retain explicitly named _membership custody (ForwardBorrowed read or InverseOwned read), original checkpoint0 and unchanged with_field_at audience/kind/control on every callback. Old Rows.with_field still issues a fresh current inverse token per call. No token enters a retained Entry or another invocation; no cached allow or new READ/currentness grant. Constructors continue all foreign-scope/committed-vs-proposed/removed failures. Rerun actual Native adversarial/cancel court and22 regressions, then unchanged Bank160; these preserve proof validity while deleting real repeated identity/support checks. This supersedes the earlier per-field ticket revalidation plan only for the immutable token that already carries its actual proof.

## Measured amendment: physical root memo partition

Bank18 still reaches149/161 scopes under unchanged16M (this also includes the stricter, correct physical-source-dependent equality quote). Native forward acquisition remains the refusing receiver. The root evaluator still uses one composite-key BTreeMap across all visited recipe nodes. Each access searches full node/anchor/direction identity against unrelated node entries. The unchanged fixed comparison policy is not reduced to fit a threshold.

Independently reviewed actual representation change: memo/storage.rs owns root buckets BTreeMap<(node,reverse), BTreeMap<NativeMemberKey, endpoint-set>>, plus the existing complete-Key Named map. Only visited buckets allocate. One Evaluation retains its actual admitted root owner; exact registered target normalization remains unchanged. Node/direction are physically dispatched once, no longer stored and compared on every root anchor. Root storage no longer repeats Handle/node/class per anchor. There is no recipe-sized array, global cache, pointer ordering, new semantic identity or public surface. Named definitions keep full identity and support semantics.

Controlled fallible Memo get/contains/remove/insert methods receive the disjoint original SourceRead. Each charges actual outer integer/direction comparison using existing11*levels policy, obtains the bucket once, then charges its actual anchor comparison and performs the inner lookup. Quote calculation must not silently search the bucket first. Root absence pays only the actual outer search; Named uses its existing full-Key search. Retention admits the real new bucket header before entry allocation and the actual inner anchor/set entry; Named retains its existing Key/set layout. Every operation ends with original checkpoint0. Navigation and final root extraction consume these methods with disjoint values/read borrows; remove the old quote-then-uncontrolled-index path entirely.

Pure cache evidence compares mixed actual root/Named handles, nodes, directions and full physical/projected anchors against an independent flat full-Key BTreeMap through insertion/replacement/removal/absence. Existing registered-recipe union/intersection, support capture and original Source stop tests remain. Then Native22 regressions and unchanged Bank160; no resource, population or profile changes. This is a concrete lookup/storage cutover, not a change to work pricing constants or member comparison semantics.

Bank19 root-memo partition passes4 pure model/recipe/stop tests and22 Native regressions. All161 actual Bank provider builds/qualifications now finish before original16M refusal. Add temporary boolean/end/count brackets at combined Source-stage completion, returned roots/check_complete and real merge/proposal caller invariant stages to locate the final refusal. No further semantic optimization until the real remaining receiver is known; no skip of complete checks or inference of full membership from changed-row closure.

Diagnostic8 proves combined Source stage succeeds after161 roots (Native program meter8,841,376, cumulative subset only), check_complete ends, proposal commit-boundary and other invariants end. Add aggregate-return, proposal-round/branch validation and symbol-round success/cold-retry boolean brackets to distinguish aggregate, symbol replay and later proposal assembly. No new semantic edit until that handoff is identified.

## Measured amendment: inline singleton root memo buckets

Diagnostic9 proves all161 roots, complete-check, Source stage, invariants,
aggregate, symbol round (no cold retry), mirror demand and proposal round succeed.
Refusal is later mandatory commit preparation. No check deletion is justified.
The measured Source program submeter is8,841,376 work; the actual Bank fixture has
160 one-row scopes through its simple three-path recipe, with one anchor at the
visited root nodes; the large owner retains genuine multi-anchor work. The
independent reviewer accepts reducing this dominant real storage/search cost
instead of weakening later checks.

Replace root Bucket's unconditional BTreeMap with private Empty/One/Many storage.
Empty contains no inner allocation. One retains the complete actual Member and
endpoint set inline and performs one full semantic equality comparison, including
projected domain/case names and carrier generation. Many retains unchanged BTree
11*levels comparison work. Named keys and actual outer node/direction searches
remain unchanged. Use actual enum layout in outer retention. Equal One insertion
preserves the original stored anchor and replaces only endpoints. A second
distinct anchor promotes to Many: admit both heap entries and both actual BTree
insertions before moving the existing One. No thresholds, approximate identity,
Bank-specific branch, new allowance or global cache. Many need not shrink after
removal; Empty/One removal and reinsertion preserve the exact map contract.

Update the existing flat-Key reference test and add focused actual singleton
transition, original-anchor and promotion-stop evidence. Retain original final
polls and staged failure propagation; no values are published from a stopped
Evaluation. Run pure Native memo owner,22 Native regressions and all5 actualBank
courts under unchanged16M/128MiB/160. Only successful end-to-end acceptance closes
the budget issue; a lower Native submeter is not success.

Bank22 singleton receipts:5 pure memo tests and22 Native regressions pass; actual160 bootstrap now succeeds unchanged16M/128MiB. Its query denial is the exact original QueryRead(ResultLimitExceeded) owned inside the existing Bank ManagedIndex carrier, not Bank Execution. Correct only the test oracle to borrow that exact typed cause; do not flatten production custody or accept a generic ManagedIndex/WorkBudgetExceeded denial. Then exercise160 parity and cancellation.

Bank24 exact limit carrier is now correct; ordinary accounts160 succeeds and managed lawful160 refuses QueryRead::ExecutionRequest(WorkExhausted) on its own fresh request. Bootstrap is solved. Since159 returns the actual Parent ResultLimitExceeded and160 reaches materialization, bracket the actual admitted managed read around Parent admission, selection, bounded Parent row projection and settlement before choosing a repair. Keep both original budgets, lawful160 and exact limit/cancel oracles.

The normal Bank entry uses receiver::read_managed_parent, while admitted_read is the separate decision-reader path. Remove the diagnostic-only admitted_read delta and bracket the actual shared result-tree materialization and complete_one_shot_with_capture row interning/terminal instead. The159 typed Parent cardinality denial already proves root selection completes.

## Measured amendment: one retained Source-meaning registry entry

The real public read diagnostic proves both ordinary and managed160 complete root
selection, result-tree projection and footprint collection. Managed completion
then exhausts during row-meaning interning after row64 and before128. The prior
ordinary160 read retains160 other coordinates in this actual registry. Four
funded/searching passes over meanings for each new coordinate are visible in
interning.rs: bucket sizing, mutable comparison, prepare_container reservation,
then final entry insertion. The current conservative lookup quote remains valid;
reduce the real descents, not its price. Checkpoint hashing already uses the
cached root-selection identity, so do not remove real hash work or add an Arc
semantic shortcut.

Retain one BTreeMap entry after one paid lookup. Split meanings/next_ordinal
borrows; use the same Occupied entry for scratch sizing, weak pruning, controlled
comparison, capacity reservation and Weak insertion. A Vacant entry prepares one
funded bucket and inserts it only after every fallible admission/hash succeeds.
Keep next_ordinal unchanged until that point. Delete prepare_container's displaced
searching wrapper; reserve remains with CoordinateMeanings, visible only within
this owner. Keep existing full coordinate/equality/checkpoint meaning and actual
lookup quote. No Source READ, semantic identity or authority changes.

Declare upgraded-Arc retirement custody before acquiring the registry guard so
unwind also unlocks before last-strong meaning Drop. Keep the incoming footprint
in pre-guard custody until successful meaning construction. Entry borrows end
before empty-map retirement; upgraded unmatched meanings and retired maps drop
outside the guard on success and typed error. Preserve all old release/reintern,
result-set, product/parameter partition and identity tests. Add one root-free
owned comparison-error/ordinal-preservation/retry regression in the existing
owner. Existing Bank160 (preceded by ordinary160) plus a repeated managed160 read
is the decisive managed registry integration; exact159-limit and originalcancel
remain unchanged. Run focused interner owner then final Bank/consumer families.

## Broader receiving amendment: sparse numeric root pages

Final Native external court passes98 tests with1 existing ignore. The first full
Bank run passes130 but53 process tests lack the existing external-rail executable;
that product is now built through its existing manifest. Four real regressions
are unchanged bootstrap fixtures: canonical scale192, root union250 and empty
Approver guarded frontier265 (two courts share canonical scale). Keep all original
16M/128MiB limits and populations; the160-only acceptance is insufficient.

The independent reviewer clears lowering actual extent-local root node/direction
coordinates from one comparison-tree entry per node to sparse pages plus direct
slots. memo/storage/root_pages.rs owns a usize-keyed BTreeMap of pages. Split
node/page/remainder before direction multiplication, so huge sparse node values
cannot alias or allocate a whole recipe. One page spans a machine-word count of
node positions; only actually visited pages and required initialized pointer-slot
prefixes exist. Each nonempty slot owns one Box<Bucket>. No global cache, recipe-
sized array, semantic cutoff or Bank-specific branch.

Pay the actual usize page-tree search before lookup, then bounded direct-slot
work. get/remove allocate nothing. Insert obtains one paid Entry; admit actual
page metadata, pointer-buffer growth/copy/initialization and actual Bucket box
before each allocation/mutation. Original final polls and typed failures remain.
Root definition custody, full Member equality, singleton promotion and Named map
semantics do not change. Extend the existing full-key model and owned storage
courts with page-boundary/direction and distant sparse keys, absent no-allocation,
bounded prefixes and every growth checkpoint stop. Reuse five memo, Native owner
and four existing Bank reproductions, then final broad families; no new root/SDK
family or admission policy.

Sparse memo paging passes7 pure tests and all5 Bank account cases, but the4 unchanged192/250/265 bootstrap cases still refuse16M. Temporarily observe the already-owned provider attempt counts at installation failure to identify before/inside/after Source staging for these larger populations before another semantic edit.

The larger failure counters reach195/195,206/251 and80/266 builds. Before another algorithm edit, temporarily record exact record/adjacency equality in actual discovery capture: larger Native effect ordering may decline the conservative exact-sequence reuse even when actual sets match. Report only booleans/counts; retain original comparisons, liveness and fallback.

Larger-fixture diagnostics confirm exact discovery reuse for every case. Preserve the guard. Temporarily record actual Source-stage charged work after authored history and at provider scope milestones, plus error-only acquire/build brackets, to distinguish remaining Native acquisition from tree qualification and later commit work before selecting any production repair. Process receiving court now passes55 after its required product is built.

## Measured cold Source column construction

Build32 establishes canonical history840357 units and completed roots7719506 units; history is not dominant. The larger250 and265 fixtures fail specifically inside source_structure::qualify after real acquisition, provider build and writer seal succeed. Cold qualification currently sends every region and parent through PageEdit, including paid persistent touched-page maps and their ordered sealing arrays despite having no predecessor.

Add source_structure/cold_columns.rs for one cold-only construction responsibility. Capture already enumerates all physical ordinals0..frontier, including holes. Allocate two original-request-funded Cell<Option<T>> scratch columns of that exact captured extent, check full captured ordinal order/no predecessor, duplicate children, each actual child's live region, and genuine image slot validation/duplicate postings. Create immutable NodePages once from each column. Preserve genuine Source posting map, exact live counts, existing admit_cold shape and one-pass cycle checks, Source final check, original controls and typed errors. Warm apply_edits remains the sole predecessor editing path. Remove only the displaced cold use of PageEdit. No omitted proof, source closure shortcut, budget change or fixture population change.

Verify the four unchanged large Bank regressions and five current-account cases first. Existing consumer malformed-cold court must continue rejecting omitted/duplicate Sources, duplicate parent and disconnected cycle; run broad consumer and large geometry growth along with Native and Bank receiving families before closure. Review the complete implementation with all three independent QA skills. Further repair is conditional on measured results.

Build34: direct cold columns restore both canonical192 receiving cases and all five current-account cases. The unchanged250 and265 cases still stop specifically in qualification;265 reaches121 builds versus80 previously. cold::acyclic still builds a persistent AVL ordinal-to-walk-tag map, allocating/copying paths for a complete captured cold roster whose ordinal extent is already known. Replace that map with one original-request-funded direct Cell<Option<usize>> column of the exact captured frontier. The explicit frontier comes from the actual captured edits length, not live count or a proxy. Preserve sparse holes, ordinal bounds, live node-count cap, active/completed walk distinction, exactly-once parent callback and final original poll. This is the sole cold cycle algorithm; warm checks unchanged. Update the independent exhaustive functional-graph court to pass actual extent, retain sparse1024-spaced chains and measured work growth, cancellation and released scratch. Memory denial now occurs before visiting parents if the full direct column cannot be admitted; assert that honest preflight behavior and unchanged exact original-memory denial. Add explicit out-of-extent seed/parent refusal. No semantic or budget change.

## Cold posting bulk construction

Build35 completes all251/266 roots, with actual Source work6930525/7483460; the original16M request still refuses later commit preparation. Preserve all populations and original policy. Qualification still incrementally inserts cold Source postings into a persistent AVL map, paying and physically copying paths despite a complete finite cold capture.

Add a checked native sorted constructor to existing ExecutionOrderedMap storage. It takes count plus owned-pair iterator, validates exact length and strictly increasing keys, funds existing Entry/Node layouts from the original request and builds each once in balanced in-order traversal. Return Ok(None) for invalid input and Some(empty) for valid empty input; request/allocation errors retain their typed causes. Native payloads move with their existing allocation custody and no heap clone; no ChargedBytes claim is invented. The ordinary funded final handoff remains mandatory. The native caller continues to own key comparison/construction work admission, as with native insertion.

Cold source_structure/cold_postings.rs owns a funded fixed pair scratch column, uses the EXISTING read_execution::fallible_heap through its fallible compare/swap port, checks duplicate full slots and feeds the ordered pairs into that constructor. Every actual slot still passes image validation in cold_columns. Each sort comparison/swap and linear construction is charged through the original Source control before work; no separate request/cache/payer. Bulk construction replaces only cold source posting path copies. Warm persistent insertion/removal unchanged. Tests use an independent BTreeMap plus actual AVL count/height/custody, exact input failures and cancellation/memory refusal with complete scratch release. Then run five cold-cycle tests and unchanged Bank receiving cases, followed by final Native/consumer/cost courts and independent gate.

Build36 bulk postings passes four focused Execution owner tests and lowers250/265 complete Source work to4115735/4469880. Both still refuse later in commit preparation. Before another semantic change, temporarily capture original SDK Source-stage charged_work and the work-ceiling refusal backtrace/counter. Earlier temporary build.rs used the same qualification label at allocation and final qualification; those labels alone do not prove which of those two positions refused. Retain only the supported broader cold-construction attribution in final evidence. No API/policy or production behavior change is selected by this diagnostic step.

Build37 actual SDK Source stage costs11106469, versus group4469880. The original remaining ceiling stops later in canonical authored-index field definition navigation. Temporarily expose only the existing Stage context by borrowed callback parameter to distinguish history SDK work from maintenance SDK work; adapt both existing callers (applied preparation and historical material capture), then remove this diagnostic-only signature. No new request or carried capability is introduced.

Build38 after-history original SDK accounting is7713471 of Source11106469 for265; group closure1082187 excludes stage-level input collection/impact/capture work. Bracket those actual history steps through the same borrowed diagnostic context before selecting collection or correspondence repair. Preserve the complete packet and unchanged meters.

## Measured authored allocation seeding

Build39 isolates6011697 SDK units in AuthoredSourceInputs collection,1082187 in closure,180080 in impact capture and429877 in exact correspondence. Collection is the dominant remaining owner. PendingRecordAllocations::canonical() proves actual allocation provenance and ordinal order, not RecordRef order; replay, partition choice and reclaimed slots can be non-monotone. Never assume Bank's observed order for the general owner.

Move allocation seeding from collection.rs to collection/allocations.rs. Check both actual filtered entity and relation streams for strict full-generation ID order under original stage admission. For proven ordered input, seed the existing records and incidence ExecutionOrderedMaps with the checked sorted constructor, funding one entry/node per allocation. Prepare all incidence values from actual immutable before/after metadata with the existing kind checks into original-request-funded scratch before the infallible bulk iterator. Keep one incidence observation owner used by both seeding and later relation collection. A non-strict stream uses the existing incremental allocation collector, preserving duplicate-union behavior and arbitrary partition/replay order. This is a local checked construction algorithm choice inside the same collector, not an alternate history authority or omitted packet. All subsequent authored commands, owner-induced physical fragments, adjacency, flags, no-ops and exact discovery comparison remain on the existing path.

Pay preflight ID comparisons, real metadata observations and linear construction through the same Source stage; original allocation/control errors always propagate, never trigger the ordering fallback. Prove stream classification with pure full-key order cases (interleaving, duplicate, partition/generation disordered inputs); receiving Native and Bank courts establish actual packet/runtime custody. First run the two unchanged250/265 regressions, then remove every temporary diagnostic and run final owner/receiving families and all gates before committing.

## Test topology correction from the actual gate

The final Cargo boundary gate refuses the new Execution sorted-map source tests
(BC9002: live ExecutionRequest/SerialRequest roots). Move those four request,
allocation and cancellation cases into the new execution/ordered_map.rs leaf in the existing
worth-court-execution target; remove their source test module/file. No
production behavior or API changes, new court root or SDK family are needed.
Use only public map/request receivers: independent BTreeMap plus rank/get/iteration,
continued edits/shared old views, observed single-entry retention versus N-entry
retention and original memory, exact malformed-input/stop/release/retry evidence.
Remove the inaccessible private Node-height assertion and the corresponding
runtime-height claim rather than adding a control or changing production for test
access. Midpoint balance is directly visible in the reviewed constructor; real
later edits and ranks protect the public storage behavior. Rerun that existing
Execution court family, actual boundary/context, dirty checks and independent QA.
Existing Bank/Native/consumer production runtime receipts remain applicable to
this test-only move.
