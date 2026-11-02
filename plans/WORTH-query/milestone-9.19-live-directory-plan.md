# 9.19: compress the physical directory around live pages

## Boundary review

The six original audit findings are now repaired through commit1b806e72e3. The
additional churn axis remains real: Nodes::NodeLineage issues genuine descriptive
ordinals monotonically via the existing AtomicUsize owner, while NodePages is a
binary page directory whose implicit height reaches the highest retained ordinal.
The batching repair eliminates repeated shared-path copies but leaves unary
directory spines. A tiny live root with distant issued coordinates therefore
retains/traverses empty bit levels. Recycling ordinals would break genuine sibling
IDs and retained snapshots; the issuer must remain unchanged.

NodePages is the existing shared physical storage owner for provider nodes and
Source structural metadata, not Source identity, graph authority or publication.
Its private Directory cells own original paid Arc/header grants, leaf arrays are
move-only SDK backing, and PageEdit owns one predecessor plus an SDK ordered
touched map. Generic upstream/downstream Source, region, summary and completion
qualification must continue consuming the same Nodes/Writer APIs and genuine IDs.
No public key, ID, session, version, currentness or read authority changes.

## Plan (saved before Rust edits)

1. Compress the existing binary directory (ordinary Patricia/radix compression):
   each actual Directory records its aligned page prefix and discriminating bit
   height. A leaf has its exact page key and height0. A Branch has two nonempty
   children; a one-child result shares that child directly. Store no implicit
   high-water root height. With P live pages there are P leaves and at most P-1
   branches. Stable ordinals and NodeLineage allocation remain untouched.
2. Lookup checks the current actual directory prefix and descends at its actual
   branching bit. Skip absent bit levels entirely. A missing page cannot alias
   another leaf. Bound shifts by page keys (ordinal>>5), so all prefix operations
   and machine-word recursion bounds are explicit and overflow-safe. Exact paid
   Arc/header quotes include the new metadata fields.
3. Adapt the single batch sealer to compressed subtrees. Populate one paid
   temporary SDK array of (key, borrowed WorkingPage) pairs via the touched map's
   existing ordered inline iterator. Use finite array windows, find first/last
   keys and their least common prefix with the prior actual subtree, partition
   at that bit, then retain each affected nonempty ancestor once. Empty windows
   share old subtrees. Single-page replacement seals the sole builder; retirement
   collapses unary results without wrappers. No new Vec/map, second ID mapping,
   cached semantic proof or full predecessor scan. Pay actual array capacity,
   population, key/record reads, partition comparisons and recursion under the
   original request. This supersedes the failed repeated-map-rank approach below.
4. Preserve cold construction in the same owner, compressing branches as they
   finish. Existing real from_scratch arrays still define exactly their populated
   cells. Remove the old virtual-growth/unchanged-wrapper code and implicit-height
   representation. Keep receiver-local test allocation counts at actual new
   Branch allocations only; returning an existing child is not an allocation.
5. Extend focused physical evidence through new tests under existing pages/tests
   as a churn child module, with exact reviewer placement approval before adding
   constructors. One numeric test compares fixed two/three live-page shapes at
   increasing address gaps, including highest representable page keys and nonzero
   prefixes; assert identical lookup/replace invoices where the compressed shapes
   are equal, exact live-cell counts, retirement collapse, wrong-page rejection
   and original/sibling Arc isolation. No synthetic Source or qualified root.
6. A genuine NodeWriter test keeps two live nodes while actual NodeLineage issues
   thousands of IDs through ordinary push/remove/seal. Retain real predecessor
   and sibling snapshots. Verify old IDs never become new values, sibling IDs
   remain distinct, every live read is exact and fixed-live final physical storage
   stays bounded. Separately report actual original SDK final lookup/edit work;
   don't infer a whole-world invoice from the numeric backing tests. Existing
   actual partial-seal refusal/grant release/fresh retry owner evidence remains.
7. Update current batch shape expectations for compression: dense65/128 remains,
   sparse adjacent-page growth becomes two branches instead of historical11
   unary-inclusive cells, empty growth0. Keep historical evidence clearly dated.
8. One focused nodes::pages private owner build, then small public managed family
   and Candidates, real prefix large1k/2k/+4096 and large geometry1k/2k/+50k after
   small guards pass. Shared structural representation changed, so these fresh
   whole-world cost guards are justified; do not modify thresholds if they fail.
   Mandatory boundary/context, fmt/whitespace, dirty caps/functions and independent
   qa-loop/qa-tests/code-quality-qa gate a separate commit. Default Rust1.99.1,
   configured sccache/shared target, no aborted/restarted builds or extra matrix.

Proposed exact new private tests in nodes/pages/tests/churn.rs:
- compressed_live_pages_skip_historical_coordinate_gaps_and_collapse_retirements
- genuine_node_churn_keeps_live_backing_bounded_without_reusing_issued_ids

The live representation removes historical-coordinate unary storage/traversal.
A compressed radix tree is not an AVL guarantee: path depth is bounded by actual
live branching nodes and machine key width, not a universal log(P). Temporary SDK-array population, direct access and window partition work are
paid and may add logarithmic factors in actual P touched pages. No global performance, Source proof or public footprint
narrowing is inferred. Remaining Phase3/4/5/6 obligations stay open.

## Failed focused cost evidence and plan correction

The first completed owner run passes6/8. Fixed three-live-page shapes across five
address gaps already have identical whole SDK lookup/edit6525 and retained bytes
2296. But dense64/128 seals charge65870/147087 and FAIL the unchanged2.1x guard.
Repeated SDK-map rank walks at every recursive boundary are real extra work, not
mispriced work. Keep the guard; replace those walks with one original-funded
ExecutionArray of ordered (key, borrowed WorkingPage) pairs, populated once via the
existing SDK inline iterator. Binary partitions/read access then use paid array
positions. This is temporary physical scratch, not a map, authority copy or retained
second truth. Its actual capacity/old-new overlap is admitted before allocation;
no heap estimate or hidden unmetered Vec. Drop scratch after one consuming seal.
Update the previous no-new-array plan restriction because the real cost evidence
invalidated it; retain one ordinary authority/storage path and O(U) branch custody.

The genuine4096-edit harness also exhausted its single original1M-work request
before finishing, yielding typed WorkCeiling as designed. Each ordinary edit is
a separate invocation and should use a fresh actual original request through the
existing helper; keep each edit's policy1M rather than raising a budget or hiding
a stop. Retained directory cells keep their own original grants across requests.
Measure the final fixed-size operation separately under its actual original SDK
report. The earlier compile-only Nodes.clone mistake was corrected through actual
edit/seal; Nodes and SDK arrays remain move-only. No production API/guard weakened.

The array implementation passes the unchanged dense seal guard:4724/9329 with
actual65/128 branches. Fixed-live gaps now all report5664 SDK work and2296 retained
bytes. Owner7/8; the remaining harness stop confirms the reviewer found that
per-turn requests were still nested inside the setup controlled scope. Move the
loop entirely outside that scope so the declared ordinary edits are independent
invocations, retaining the same1M policy. Preserve the failed nested log separately;
no ancestor detach/reset or budget increase. Production remains unchanged.

## Final focused evidence (2026-10-31)

Corrected owner8/8 passes0.23s on the default2m00s build. All five fixed-three-page
coordinate shapes report exactly5664 original SDK lookup/edit work and2296 retained
bytes; the high-bit label gap no longer creates unary cells or work. Dense64/128
actual branch allocations remain65/128 with4724/9329 original seal work, passing
the unchanged2.1x guard. Sparse adjacent-page growth now allocates2 real branches,
superseding historical11 unary-inclusive cells; empty far growth remains0.

The real shared issuer advances4096 times while current backing retains at most
3 cells for two live nodes. Existing retired and sibling IDs never alias later
values; old/sibling snapshots preserve their actual data. Final fixed-size lookup/
edit reports1956 whole original SDK work. Each independent edit keeps its actual
unchanged1M policy; no nested ancestor is detached or quota reset. Partial-seal
work refusal, released scratch and same-data fresh retry remain green. Historical
snapshots keep their lawful original shares; no all-history memory-collapse claim.

Public managed50/50 passes19.55s and Candidates9/9 passes. Fresh real prefix
1k/2k/+4096 passes50.44s with refusal3395261/3613684/4058157 and
commit4326109/4653846/5009086, unchanged25%/50% guards. Large geometry1k/2k
passes its local25% guard: refusal2351456/2472910, commit2116611/2248906;
unrelated50k remains running. Required boundary/context, fmt/whitespace, dirty
caps6 files and scrutiny5 advisories/0 errors pass. Final large outcome, docs and
independent all-three clearance still gate commit.

## Large geometry failure: adjacent spatial rekey boundary

The fresh actual large geometry run FAILS its unchanged difference guard after
754.22s. Refusal1k/2k/+50k=2351456/2472910/3550155; commit=2116611/2248906/3301621.
Local25% and overall50% pass, but remote refusal exceeds eight times the observed
local step. All exact Source/queries/roots/commit checks pass. The complete failed
log is preserved as query919-live-directory-geometry-large-rekey-failure.log. No
large qualification or slice completion is claimed.

The actual pure spatial rekey currently calls remove then insert for an unchanged
genuine Source occurrence whose bounds changed. It retires its live leaf and
parent and appends replacements; Morton ordering can move that occurrence across
ranks. The actual root diagnostic shows30/32 A decomposition/summary callbacks on
the refused proposal versus20/22 for the successful small-bound move. These raw
callback intervals are not an exclusive accepted fee attribution. Static code
confirms avoidable leaf/parent identity churn, so evaluate a principled rekey that
keeps those actual cells instead of changing the guard or SDK price.

### Reviewed expansion plan before adjacent Rust edits

Use one actual extraction traversal for both removal and relocation, returning
the remaining root, the actual detached leaf and its one parent branch. Higher
ancestors refit/balance in the existing owner. Ordinary removal retires those two
cells (only the leaf for a singleton). Relocation replaces bounds on that exact
leaf, reinserts it with the detached actual parent branch, and shares the same
existing insertion/balancing mechanics. A singleton changes its actual leaf in
place. No new IDs, leaf, branch, source map, approximate bounds or authority are
needed for rekey. The staged provider backing is isolated; original stop/refusal
continues to prevent handoff, including while cells are temporarily detached.

Keep the one key comparator, actual Source identity, field/binding admission and
NodeWriter liveness. The generic pure fixture's extraction/relocation record is
descriptive data, not legality proof; topology still qualifies at the actual
Query/Native receiver. No second graph or provider path. Put extraction in the
existing box_tree edit owner (a child module if needed for400-line cap), and
thread an optional actually detached leaf/parent through the one insertion path.

Strengthen existing pure keyed edit court: unchanged leaf coordinate, zero new
Vec slots/retired cells across multiple rank-changing bounds updates, singleton
case and independent closed-box/union/AVL/occurrence oracle after every move.
No new fixture storage or private Source factory. Run those quick tests and
small real row/public families before repeating the expensive large guard.
No change to25%/50%/8x guard, measured Source population, incidence or SDK prices.
This expansion is required for current combined spatial/live-directory cost
acceptance; final all-three review and both large outcomes still gate commit.

### Rekey implementation and small evidence

One nonClone Detached record carries the actual still-live extracted leaf and
optional immediate parent. Ordinary deletion retires them; relocation rewrites
the same leaf and consumes nonClone Incoming along one existing insertion path,
using that detached parent exactly once at the destination leaf. Higher ancestors
refit/balance through the same helpers. Singleton rewrites its actual leaf. No
retire/resurrect or additional Source map, allocator or ID issue occurs.

The existing pure Tree court passes2/2, including the new six-rank-change/signed-y/
equal-bound-tie and extreme singleton cases. The actual leaf coordinate, Vec slot
count and complete live-cell roster remain unchanged; independent scan, whole
union, AVL/rightmost metadata and retained old snapshot parity hold after every
move. Pure numeric order2/2 remains green. Public managed51/51 passes18.97s, with
real cold/warm alternating rows, cancellation, failed summary and Source mutation
journal behavior. Candidates9/9 passes. Updated required boundary/context and
whitespace checks pass; dirty caps8 files and scrutiny6 advisories/0 errors pass.

The second large run has passed its local25% guard with refusal2204462/2321871
and commit1971205/2101351. Its50k result is pending; the prior failed8x guard stays
open. Fresh prefix large remains applicable: no prefix or shared directory code
changed after its successful run. No further private owner rebuild is justified
by this pure-provider edit; owner8/8 covers the unchanged directory implementation.

## Final real large qualification

The unchanged large geometry case now PASSES in738.71s:

| Local / unrelated geometry | Refusal whole SDK | Commit whole SDK |
| --- | --- | --- |
|1000 /0|2204462|1971205|
|2000 /0|2321871|2101351|
|2000 /50000|3256889|3031788|

The25% local,50% unrelated and8x difference guards all pass without changes to
prices, populations, incidence, Source/Parent limits or expected results. Full
canonical Source/closed queries, fixed source-bound/summary work, both prepared
roots before rules, original failed head and fresh commit all pass. The earlier
failed run is retained as evidence of the actual adjacent rekey defect. This
measures the combined compressed-directory and live-cell rekey implementation;
raw callback windows still do not become exclusive accepted fee attribution.

Fresh prefix large passes as above; shared/prefix code did not change after it.
Owner8/8, pure order2/2, keyed edit2/2, public managed51/51 (excluding these two
separately qualified large cases), Candidates9/9, updated boundary/context,
fmt/whitespace, caps8 files and scrutiny6 advisories/0 errors are green. Current
structural custody is bounded by live pages, while retained snapshots retain their
lawful shares; compressed paths are bounded by actual branches/key width rather
than universal logarithmic P. Independent qa-loop, qa-tests and code-quality-qa review clears the combined
live-directory/rekey slice without a remaining scoped blocker. Compiled-document
drift1/1 also passes. All six original audit findings plus churn are qualified.
Remaining retained predicate routing, refinement/disclosure, verified narrowing
and Phase4/5/6 work are not closed by these performance repairs.
