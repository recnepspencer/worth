# Phase 3: reuse touched node pages within the actual isolated writer

## Boundary review

The restored checked-scale slice is still not cost-qualified. The last premerge
whole-request remote invoices were 5,360,656 / 5,110,305 versus the 2k baselines
3,149,263 / 2,849,684; the original 50% guard failed. Structural no-op preservation
and new-edge-only cycle checks already removed real redundant work. Do not alter
those guards or price estimates to declare success.

`NodeWriter::write` currently invokes NodePages replacement for every logical
write. Replacement copies a 32-cell immutable page and its directory path, even
when an AVL edit repeatedly writes different nodes in that same page. The actual
base root and node lineage are already borrowed by one isolated writer. Cache
only within that owner: no semantic-id/global/root-stamp cache is needed.
Unchanged immutable pages remain shared with the actual predecessor and sibling
writers. A touched-page tombstone must override that predecessor's value.

ExecutionArray is deliberately immutable and move-only. Do not add rewriting to
it or adopt a Vec/ticket. Its original-request builder already owns the exact
initialized allocation and the sole ticket. Add only initialized-element move
replacement to that mutable builder; keep Vec mutation private to its existing
Owned/BuildingFixedBacking owner. Consult the original request before/after,
consume input on refusal, maintain initialized-prefix length and keep a valid
new slot before dropping the old value. A distinct physical uninitialized-index
denial should preserve the owner's existing classification boundary.

An edit-local collection can hold request-bearing builders behind private Rc /
RefCell handles in original-paid ordered cells. It must not escape the writer.
First page touch copies its actual base cells once; subsequent writes replace
initialized cells. Reads resolve touched pages first, including None. Seal moves
each builder into its same immutable ExecutionArray and installs/removes that
page once. No builder, Cell, RefCell, request or editable alias enters retained
NodePages. Every logical write still enters the existing changed-node map, even
when writing equal values. Cancellation/refusal cannot publish a partial root.

This does not yet establish once-per-directory-subgraph work: installing distinct
touched pages can still copy common directory ancestors. Measure that separately
before adding anything. BinarySourceTree still requires its declared unique
parents/Source occurrences; physical page sharing does not collapse logical
incoming occurrences, shared endpoints or audience-specific admissions.

## Implementation plan

1. Add `replace_initialized(index, value)` to ExecutionRequestArrayBuilder and its
   private backing owner, with original consults, exact bounds, move/drop safety,
   no allocation/growth and the same seal/ticket. Add focused public SDK courts
   for pointer/charge stability, old/input Drop, uninitialized prefix and genuine
   cancellation/seal refusal. Retain the immutable-array compile-fail fences.
2. Add one concrete `nodes/pages/edit.rs` owner for touched-page builders, bound
   to its actual base and original request. Reuse existing ordered-cell payment
   and header-memory policy. Keep initialized pages and occupancy private; no
   retained-memory prediction, whole-root scan or payer chain.
3. Replace the ordinary per-cell immutable rewrite path with edit-local reads /
   writes and a consuming immutable-page install. Keep cold construction and
   final immutable NodePages readers intact. Adapt the private physical-backing
   tests to the real editor; remove the displaced per-cell path.
4. Exercise repeated same-page writes, tombstones, growth, empty-page pruning,
   old snapshot and independent sibling writer isolation at the actual storage
   owner. Compare original SDK invoices as writes versus distinct pages vary;
   claim only the measured page-copy boundary. If private owner compilation is
   required for these observations, run that exact target once rather than
   inventing a public test door or synthetic Source authority.
5. Run the SDK court, production check, existing pure edit/public managed family
   and small real 64+0/256/1024 remote axis on the merged contracts. Keep exact
   Source +8/+8, no-Node-write bad-summary and warm retained-edge duplicate tests.
   Use small shared-storage/occurrence cases before any full 50k rerun.
6. Run scoped mandatory gates and independent qa-loop, qa-tests and code-quality-
   qa review. Only proceed to the unchanged large qualification when the scoped
   evidence supports doing so. Commit a coherent reviewer-cleared slice; the
   broader checked-scale work cannot be called qualified while its gate fails.

No Phase 3 completion is claimed. Predicate routing, refinement, disclosure /
continuations, source-profile gaps and Phases 4–6 remain unfinished.

## Implementation and review observations

The SDK builder owns initialized replacement through the existing fixed backing;
sealed ExecutionArray remains move-only and immutable. New physical
UninitializedElement is handled explicitly by the actual Query material ordering,
Query recovery posture and bank HTTP denial receivers. The production Query
check passes in the pinned default dev configuration (50.70 seconds). The three
public SDK replacement courts passed, including original cancellation triggered
by the replaced value's Drop; final SDK/fence reruns are queued.

PageEdit now owns original-paid touched-page handles, copies an actual base page
once, reads touched tombstones before the predecessor, and seals the same
allocation before one installation per touched page. The previous per-cell
immutable page replacement path is removed. Every actual logical Node write still
enters changed-node tracking. Cold construction and retained immutable readers
continue through their existing owners. No page builder, RefCell, editable alias
or borrowed request reaches the retained root.

The independent reviewer found no production/API blocker and explicitly approved
both exact tests in nodes/pages/edit/tests.rs for their private physical subjects:
allocation identity/custody and actual original SDK work. The corresponding
constructor audit entries cover only those exact test names and that path.
Neither test constructs Source, Product, graph or audience authority. A single
focused Query cfg(test) owner build is justified for these observations; no
public test door is introduced. Final clearance awaits the running owner/public
regression/small-axis evidence. No large-scale pass or Phase 3 completion is
claimed at this checkpoint.

The runtime-root ratchet also requires an explicit module/type baseline decision
(boundary-check README). Admit only the new edit/tests.rs file's observed
ExecutionRequest and SerialRequest tokens, using the same independently reviewed
private-owner placement above. No parent path or other runtime roots are admitted.
The initial BC9002 finding is resolved by that exact record and the two per-test
private-subject audit entries, not a scanner exclusion.

## Verified mechanics checkpoint

- Production Query check passes (50.70 seconds).
- The single focused private owner build took 4m47s; all four tests pass in
  0.01s. Actual original SDK work is 4,640 / 7,520 for 64 / 128 repeated writes
  within one page, versus 427,040 for 64 distinct pages. This proves the stated
  physical reuse boundary, not whole-request improvement against the old path.
- All 45 public managed-index courts pass in 19.45s, excluding only the unchanged
  full large qualification. Actual fixed64 remote0/256/1024 refusal/commit work
  is (2440583,2243133), (3497420,3207412), (3650762,3338221). Source +8/+8,
  bounded membership comparisons, authored survivor oracles, hostile structure,
  no-Node-write summary refusal, snapshot isolation and no-rebuild checks remain.
- Three public SDK replacement tests and both immutable ExecutionArray
  compile-fail fences pass. No sealed-array rewrite or clone capability exists.
- Final boundary validation passes with exact private placement records;
  agent-context check passes. All 32 dirty Rust files meet the 400-line cap;
  function review reports 28 advisories and zero scan errors. Owned formatting
  and whitespace checks pass. The unchanged compiled-example document reader
  checks current document fences successfully (one test); its own source and
  configuration are unchanged, so no broad reader rebuild was needed.

Independent qa-loop, qa-tests and code-quality-qa review finds no code/test
blocker and approves committing the coherent storage/spatial/structural mechanics
separately from failed scale qualification. The bank causal compile is blocked by the pre-existing issues recorded below. The two prior failed large runs and their unchanged guards remain
in the checked-qualification plan. The small whole-request axis does not establish
an end-to-end improvement; no new 17-minute large run is justified yet. Current
mechanics are useful verified building blocks, not completion of the broader
checked-scale slice, Phase 3 or this milestone. Directory path installation and
repeated final-tree ancestry proof remain concrete next profiling boundaries.

### Bank integration check limit

The narrow bank-http-adapter cargo check reaches bank-server and fails there:
WORTH_IMPLEMENTATION_BUILD_ID is absent at two existing env! sites, and the
untouched bank-server application_query/denial/execution.rs lacks existing
ProjectionInspectionUnavailable, ProjectionInspectionIncomplete and
SourceObservationAllocation cases. All three variants are verified present in
HEAD before this slice; bank-server has no dirty changes. These failures are not
caused by the new UninitializedElement variant. Do not invent a build identity,
expand into unrelated denial repair, or call the bank check green. Its new
exhaustive physical-denial arm is reviewed but full bank compilation remains
unverified behind that pre-existing debt. Query production/public compilation,
SDK courts and negative fences all pass.
