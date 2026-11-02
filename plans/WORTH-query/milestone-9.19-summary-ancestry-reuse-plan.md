# 9.19: exact invocation-local summary ancestry reuse

## Boundary review (833a83acf8)

SourceTree owns the admitted exact source-slot/node bijection, one actual root,
regions and parents. Cold/warm admission precedes summary qualification. The
summary qualifier borrows one immutable final tree and SourceRows; the query
adapter borrows one retained tree and actual SourceRows while recording coverage.
These are the only two SummarySources construction sites. Each callback currently
walks a requested Source node's parents again to test its current region.

SourceRows owns decoded-text reuse, not region membership. Its actual image and
per-use field/disclosure/nominal-binding checks remain unchanged. Summary metadata
must never enter SourceRows, Provider state, the retained root, or a registry.
Being a known/Ready node cannot itself admit a different receiving region.

Existing SDK maps fund copied cells; move-only arrays may be shared only through
a private paid Arc owner. The actual tree parent port already pays its page
lookup; private numeric topology tests can exercise derived ancestry mechanics
without constructing a SourceTree, SourceSlot, Native ID or qualified root.

## Plan

1. Create one private SummaryContext per qualifier/query invocation, borrowing
   the actual tree, SourceRows and original execution/control. Public
   SummarySources remains a closed per-region loan of that context. Every get
   checkpoints and resolves the actual slot through SourceTree before using
   any derived relation. SourceRows.get remains the final row handoff.
2. A concrete numeric ancestry memo lazily records only requested ancestor
   paths. Its SDK map holds Pending(actual parent, child backlink) or Ready
   metadata. Climb until a Ready ancestor or the actual root, bounded by the
   actual live node count. Repeated Pending means a cycle; unexpected parent=None
   must refuse rather than invent a root. Never overwrite a Pending backlink.
3. Unwind top-down. Publish Ready(node) only after all of its actual ancestors
   are Ready. Each private Ready owner carries checked depth and a paid SDK
   binary-jump array, plus the separately retained original Arc/header grant.
   No entries are evicted within this invocation. Thus, AFTER ensuring the
   requested Source node Ready, an absent receiving region is known not to be
   an ancestor. If both are Ready, lift by their exact depth difference and
   compare actual ordinals. Equal depths alone prove nothing. Source==region
   is a fast path after actual slot correspondence and liveness admission.
4. Keep original failure sticky; partial Pending/Ready scratch dies with the
   failed invocation. No cache hit supplies a region authorization or fabricated
   Source proof. No N-sized initializer, ordinal-frontier bitmap, per-pair cache,
   subtree scan or context shared across roots/versions/audiences.
5. Preserve summary recomputation/frontier and recorded coverage behavior.
   Providers that never resolve summary rows allocate no ancestry backing.

Files: source_structure/summary_context.rs binds the actual borrowed owners;
source_structure/ancestry.rs owns numeric relation mechanics and its pending
walk; ancestry/ready.rs owns paid jump derivation; focused tests live under
ancestry/. SummarySources is only the region receiver. The qualifier and adapter
create the context once outside their callback traversal.

## Evidence and compile discipline

- One compact numeric owner family: sparse-label chains around 64/128; repeated
  queries reuse measured actual parent callbacks; equal-depth forked siblings
  remain distinct; unvisited off-path region and Source==region; unexpected root
  termination/cycle; partial memory refusal/cancellation and same-graph fresh
  retry. Inspect only actual SDK ownership for release. Keep accepted SDK work
  separate from parent callback counts. No fabricated Ready tables or huge depths.
- Real B2 query: lawful root bounds reads populate ancestry before a genuine
  leaf-outside read refuses. Count the successful root read, then prove the same
  actual ready index is lawful after removing the hostile fixture mode.
- Small real repeated-extrema prefix query, actual Source=8 and authored oracle,
  repetitions 32/64, original SDK whole-read invoices. Report setup/union-of-path
  overhead honestly before large qualification; no fitted cost golden.
- Existing managed/Candidates small families, excluding the already accepted
  unchanged large geometry case unless a changed path invalidates its evidence.
  Box summaries do not resolve rows; lazy empty context changes no such path.
- Exact private-court baseline/audit only if required for the numeric owner tests,
  reviewed independently before acceptance. Reuse existing original SDK harness
  patterns, not a new test framework or public Source factory.
- Default Rust 1.99.1/sccache/shared target, no overrides and no killed builds.
  Format, required boundary/context, dirty line caps/function scrutiny, whitespace
  and affected compiled-document checks. Independent qa-loop/qa-tests/
  code-quality-qa clearance, then commit this slice.

Initialization follows the union of requested ancestor paths; a first request
may legitimately visit N nodes on a skewed tree. Ready tables cost O(V log H)
storage and map/jump work, where V is the visited union and H its actual depth.
Do not call the whole operation linear or universally sublinear. Stable ordinal
page lookup cost still depends on the existing directory; batching, churn,
spatial grouping and large prefix qualification remain separate obligations.

## Implemented evidence

SummaryContext now binds one actual immutable tree, SourceRows and original
execution/control. Both actual construction sites create it once outside their
traversal. Public per-region loans resolve actual slot correspondence first and
hand off through SourceRows.get only after derived ancestry accepts that region.
Pending state is never returned as membership, Ready publication is ancestor-
closed and arrays retain private original paid Arc/header custody. No original
summary/frontier, coverage or row-admission path was replaced by a provider claim.

- Private numeric owner 3/3, .02 seconds (one necessary owner build 4m35s). On actual
  sparse 64/128-node parent ports, first path construction makes exactly 64/128
  parent reads; 128 further membership checks add ZERO parent reads. Actual SDK
  accepted work for those complete numeric workloads is 1335392/2940983; these
  numbers include maps, paid jumps and repeated lookups, not SourceTree page work.
- Both equal-depth siblings are Ready and still reject cross-branch membership.
  An unvisited off-path region does not initialize another path. Genuine root
  termination, self-relation, unexpected root termination and cycle denial pass.
- Cancellation AFTER the 64-node Ready chain populated refuses at the next hit;
  original memory returns to zero. The predeclared 4 KiB 128-node request refuses
  partial Pending scratch after 9 actual parent reads, with native policy-memory
  cause requested 120/admitted 32/ancestor 0. It is not a root-entry refusal or
  claimed jump-allocation pinpoint. Both same graphs pass on fresh requests.
- Real B2 query first completes a lawful root bounds/scalar read, then the same
  context refuses a genuine root Source at its leaf receiver. Exactly one hostile
  mode read succeeds before ForeignSummarySource. Removing the mode makes the
  same existing ready index lawful; no rebuild/mutation supplies the retry.
- Real installed Source=8 with UTF-8/equal names: whole original SDK query invoices
  with 32/64 repeated summary operations INSIDE each callback are 1515245/2750445.
  Each result matches the independent starts_with roster. This is fixed-width
  repetition evidence, not source-width scaling or an exclusive ancestry invoice;
  each query gets a new context. The predeclared <3x repetition guard passes.
- Final affected public managed family 48/48 in 19.00s, excluding only the unchanged
  large geometry case accepted in 833a83acf8. Box summaries never resolve summary
  rows, so the new lazy context allocates no ancestry for that case. Its accepted
  49-case predecessor evidence remains applicable to that unchanged path.
  Candidates 9/9 in .63s. The text/cancellation/rename and other actual region courts
  remain in these families.
- Exact numeric test/file/root approvals and the actual public invoice helper's
  SerialRequest placement are recorded in the existing baseline/audit, each with
  independent qa-tests approval. No Source/Native/Ready/qualified-root factory.
- Required Cargo boundary/context, dirty Rust caps (10 files), function scrutiny
  (11 advisories/no scan errors), formatting and whitespace pass. Logs:
  C:/forge_workspace/query919-ancestry-{small,owner,managed,candidates,boundary,context}.log.

Prefix-width qualification remains next. Union-of-requested-path initialization
may visit N nodes, and table construction/map work remains explicit. Directory
batching, live-churn structure and spatial grouping are still separate repairs.

Independent reviewer gave unconditional qa-loop, qa-tests and code-quality-qa
clearance on the final code, tests, exact placement records and documentation.
The 11 function advisories preserve coherent owners or inherited boundaries;
no additional build or unrelated refactoring is required for this slice.

## Superseded after real prefix-width measurement

The jump-table implementation and its numeric evidence above describe commit
10eb08d5d7. The real Source38/70 prefix-width guard subsequently failed without
changed thresholds. The [bounded membership repair](milestone-9.19-summary-membership-cost-repair-plan.md)
removes those maps, Pending/Ready states and jump allocations. Current ownership
is four positive relation facts per immutable receiving region, with paid parent
walk fallback on a miss or eviction. Distinct regions may repeat parent reads;
no current all-parent-once or invocation-wide reuse claim follows from this plan.
The replacement owner and actual large prefix evidence gate its acceptance.
