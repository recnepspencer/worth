# Phase5 typed topology bounds: evidence

> Scope update2026-11-01: this is a historical receipt/plan under the superseded
> public CAD scope. The Scene/Face application and its authoring/contact expansion
> are removed. Distinct generic numerical/provider fixes remain. This record does
> not reopen a public CAD prerequisite; see the [cleanup evidence](milestone-9.19-phase5-public-cad-cleanup-evidence.md).

The saved boundary review and plan preceded Rust edits. Independent review approved
one IntegerBox2<Space> numerical owner and Box2 as its concrete SignedFixture domain
specialization. ModelLocalMetre uses full u64 coordinates with the existing Metre /
worth.frames.model-local.v1 class. Raw Box2Coordinates<C=i64> keeps its public
{lower,upper} serde shape, with no phantom field or new serialized tag. Validated
boxes require the checked constructor. No cast, approximation, epsilon, stored
geometry, runtime model ID, dependency, manifest or build-root change is introduced.

One box/tree/contact mechanism serves both sealed classes. BoxNode appends Space;
BoxEditStorage defaults to SignedFixture, preserving existing adapters and exact
write-elision. Centroid normalization subtracts twice the coordinate minimum from
the widened endpoint sum: the signed specialization is exactly the old+2^64 bias,
and unsigned uses the exact widened sum. All130 Morton comparison bits, raw-bound
ties and full Source-key ties remain. Checkpoints, storage/Source resolution,
live-cell reuse and every topology decision are unchanged.

The real receiving consumer is SceneFaceRow::bounds(): actual projected points
become three validated point boxes, combined through that same union owner. Bounds
remain candidate enclosures. Frame-class typing does not establish common origin
between model instances; actual Scene/context custody remains necessary.

## Current receipts

- Existing numerical owner:13 PASS in0.03s after0.89s build. Existing signed
  orientation/contact/Morton oracles remain unchanged. New unsigned tests compare
  cold/warm insert/replace/remove queries with an independent linear census,
  retain generation-distinct occurrences, check literal full-width exact contact,
  and compare Morton ordering with explicit130-bit interleaving at extrema and
  deterministic varied coordinates. Validated bounds retain the raw value size.
- Existing owner doctests: two PASS, including a same-domain positive control and
  a cross-Space overlap compile failure; all doctests completed in0.97s.
- Existing consumer root built once in34.62s. Four Scene courts PASS in0.35s,
  including the new real full-u64 Native triangle publication/attachment/query.
  Its points and derived exact[1,1]..[u64::MAX,u64::MAX] enclosure retain the actual
  declared Metre/model-local class.
- Broader existing consumer family:205 PASS in98.65s. The four focused courts are
  included, yielding220 distinct selected checks, not224. Established unrelated
  cleanup failures remain explicitly excluded:
  partial_attempt_disposal_retries_actual_retired_cleanup_until_its_reader_releases
  and previous_head_cleanup_recovers_without_discarding_adoption_evidence_or_adopting_again.
- The unchanged19-minute Catalog geometry case is not rerun or counted. Independent
  review confirmed identical signed ordering/algorithms/layout and unchanged cost
  receivers, storage and controls. Alongside the current broad signed regression,
  the committed Bank size receipt remains applicable historical evidence. No
  current large run or new spatial-cost claim is inferred.

Commands used scripts/application_build.py with existing consumer_values and
consumer_root manifests/build roles, C:/worth-build-cache/target and64MiB runtime
test stacks. Logs remain outside the repository at
C:/forge_workspace/query919-phase5-typed-boxes-*.log.

## Repository and review gates

Actual Cargo boundary-check and agent-context check PASS. Scoped formatting and
line caps PASS for13 Rust files; the function scan reports seven coherent
advisories and zero errors (existing runtime journeys, explicit query shape,
checked tree traversal and edit arity). Strict UTF-8/C0/new-mojibake and configured
Git diff whitespace checks PASS for all20 dirty text files. Independent qa-loop,
qa-tests and code-quality-qa final reviews PASS with unconditional scoped commit
clearance. The reviewer verified current receipts and the explicit remaining limits.

Actual maintained Scene Source/provider, scene-context contour creation, exact
face contact, invalid Body-only contour rewire, spatial locality, public closure
and Phase6 remain unfinished. Typed candidate bounds do not close those obligations.
