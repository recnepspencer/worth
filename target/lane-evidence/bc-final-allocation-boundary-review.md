# Independent shipping-allocation boundary review
Reviewer:/root/bc_final_allocation_boundary_review; read-only,0Cargo/build/test.
Reviewed source at e7f2490cb1 before edits.

The shipping path is recover_with_authority -> rebuild_runtime_from_plan ->
original_inputs::bind_replayed_schema -> readmit_unchanged_for_request. Its first
step reserves the complete Arc<Root> layout,including refcounts,before issuing a
replacement identity or constructing replacement DATA.

The old private refusal test obtains its authentic original root from a separate
setup request and uses issuer::default (uncharged). Shipping recovery instead
carries earlier request-funded graph/reconstruction allocations and creates the
funded issuer::for_request before replacement. Controller+sizeof(root)-1 cannot
be assumed to select that late public boundary. Ordinary captured replay root
_memory isNone,so do not claim its header itself is funded; earlier graph/region
and issuer costs are enough to require an actual holder attribution.

Existing public failures expose requested/admitted bytes,but not allocation-owner
identity. The consult observer records checks/work and its documented callback is
for signalling cancellation,not modifying memory admission. It cannot be used as
a budget-pressure side channel. A broad earlier memory refusal does not prove
that replacement Arc allocation was reached.

Exact unresolved invariant: establish earlier recovery work succeeded and refusal
occurred at the replacement Arc reservation,while proving refund and retained
original axes. No existing public observation establishing it was found in the
reviewed recovery/facade/consult paths. A pure completeness predicate cannot prove
live request cancellation/refund. No fake memory fixture,new private observer,
local trait bridge or assertion weakening is proposed.

Subsequent review of bc-final-root-boundary-plan.md found no blocking design issue
for the coherent retirement/completeness slice: actual is_complete must call the
new predicate,retain missing-evidence/reconstruction early guards,recompute
original visibility from Body DATA inside the predicate,and preserve all nine
comparisons. Envelope deref already supplies Body and its existing writer.
Delete the explicitly ordered whole dead-constructor test,including its successful
unchanged-sharing portion,under the new ruling; retain live unchanged refusal.
Verdict: proceed with that slice and qualify; not full BC9002 approval.
