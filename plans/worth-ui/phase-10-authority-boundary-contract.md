# Phase 10 Committed Allocation Authority Boundary

> Terminology: every ledger in this contract is a product-domain runtime
> authority or deletion inventory, not a QA, proof, closure, or phase ledger.
> The historical-ledger retirement policy does not apply to these runtime
> semantics.

This contract freezes the authority topology while the Phase 10 cutover is in
progress. It applies to production code, substrate code, diagnostics,
certification support, and tests.

## Exclusive Roles

| Role | Sole owner | Permitted result |
| --- | --- | --- |
| Admit scroll evidence | host and Query evidence boundaries | sealed descriptive evidence or typed denial |
| Admit portal-anchor identity | runtime portal planning authority over admitted host evidence | typed target-plus-coordinate identity and bound source generation |
| Select allocation locality | allocation-neighborhood graph | admitted neighborhood catalog basis or typed denial |
| Commit and reuse receipts | allocation receipt ledger | ledger successor, committed receipts, and one activation capability |
| Mint activation attempt | allocation receipt ledger sealing | one move-only committed-allocation activation attempt |
| Prepare publication | committed-allocation activation attempt | one non-cloneable prepared activation or one typed denial |
| Publish live state | prepared committed-allocation activation | infallible single-use commit followed by receipt construction |
| Explain the outcome | receipt and denial projections | read-only evidence; never runtime authority |
| Retain committed geometry | allocation receipt commit lane | allocation-established geometry or explicit `not_known_at_allocation` posture |
| Compose committed evidence | committed allocation evidence set | viewport, scroll, portal, and later family rows without one-of replacement |
| Compose graph consequences | graph replan consequence aggregate | family consequences that remain part of one locality transaction and idempotency basis |

Host, Query, graph, receipt, and evidence substrate remain responsible for
their own proof-bearing products. The activation transaction composes those
products; it may not recreate their meaning from raw rows, labels, strings, or
digests.

## Forbidden Competing Authority

- No constructor outside receipt-ledger sealing may mint or remint an
  activation attempt or capability.
- No intermediate readiness, frame-gate, graph-transition, ledger-transition,
  scroll-evidence, or atomic-swap value may independently authorize
  publication.
- No lower publisher may be called by production, certification, diagnostics,
  or tests after its migration delta closes.
- No adapter or diagnostic projection may reconstruct authority from copied
  fields, raw rows, string keys, or digests.
- No host-observed anchor rectangle may be reported as derived portal
  placement. Anchor observation and committed allocation bounds are distinct
  evidence fields.
- No new allocation family may add a sibling evidence sidecar or replace a
  one-of inspection variant; it must compose through the committed evidence
  set sealed by the receipt/commit lane.
- No new invalidation family may add a transaction-level consequence sidecar;
  graph-owned family consequences compose through the canonical replan
  consequence aggregate and participate in transaction identity.
- No preparation path may mutate live graph, ledger, active-runtime,
  frame-replacement, durable-resize, or receipt state.
- No test fixture may construct authority that the ordinary admitted-catalog
  route cannot produce. Private checkpoints may select a failure point but may
  not mint authority.

## Dirty-Edge Freeze

`scripts/ci/check_worth_ui_phase_10_authority_boundaries.py` compares the
source tree with an exact path-and-count manifest. The manifest is a temporary
deletion ledger, not an approved final architecture.

During the cutover:

1. a rule may not gain a path or occurrence;
2. deleting an occurrence requires updating the manifest in the same change;
3. moving an occurrence requires naming the new canonical owner and removing
   the old edge;
4. the closeout target for every legacy publisher, movable intermediate, and
   synthetic fixture rule is zero;
5. Phase 10 cannot pass while the guard contains an unexplained exception.

## Completion Boundary

The final topology is:

`receipt ledger sealing -> move-only attempt -> prepare -> prepared activation -> commit_once -> receipt`

Every other participating module is either an input authority, a private
successor builder, or a read-only projection. There is no sibling publisher.
