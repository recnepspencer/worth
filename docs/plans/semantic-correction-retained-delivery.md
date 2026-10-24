# Retained correction delivery and Bank transport

**Status:** Completed and covered by certified Phase 2 acceptance.
The [9.18 specification](../../plans/WORTH-query/milestone-9.18.md)
owns current milestone progress.

The boundary review, implementation order and verification notes below record
this completed public retained-delivery/Bank transport slice's historical planning.

## Boundary review

The Bank inverse action declares retained-commit delivery. The recorded inverse
currently publishes through the correct native action and causality gates, but
the root-free correction entry returns only a commit receipt. It does not request
or transfer the declared retained observation. A required-output root is not part
of this action and must not be invented to acquire delivery custody.

The existing SourceOnly operation owns publication recovery, exact keyed
resolution, retained history delivery and close. Its admission retains encoded
input and the original key. A correction must enter that owner with its actual
derived input, key and correction intent. An ordinary ReverseJournal invocation
would lose the correction's causal binding. Automatic ordinary retry after a
stale correction would likewise discard its exact prepared target.

The existing serialized Bank worker stores native custody before replying.
Completed source DTOs can expose the genuine receipt's descriptive coordinates;
normal close can remove the original transport record. Later source selection
must use native completed evidence and fresh authorization, without a receipt
registry. Pending publication, unpublished effects and failed cleanup keep their
existing native owners independently of delivery or response loss.

Review found that a correction has no single account captured by the transport
record. Retained activity reads must therefore carry an explicit account query
selection. The native retained observation selects the publication; fresh query
authorization decides account disclosure. Remove the transport record's implicit
account selection and use the same explicit read command for all money actions.
The process court must move current state after correction, then read its retained
publication and compare both values and publication identity. Nested correction
refusals must retain their native cancellation, deadline and admission meanings
through the existing HTTP classifiers.

Estate freeze restoration exposes the remaining delivery boundary: its installed
inverse is source completion, while journal correction declares a retained
commit. Generalize the new owning correction entrance to `prepare_operation`,
using the action's existing declared delivery mode. Both modes construct the
original SourceOnly owner before publication and retain exact inverse input,
key and correction intent. Request a client observation only for the declared
retained mode. A source-only result closes through the same native cleanup and
recovery paths without acquiring a snapshot obligation. Replace the unmerged
retained-specific entrance and its Bank callers; keep established nonowning
correction preparation for callers that already directly own native outcomes.

## Implementation

1. Add retained correction commit to the existing program owner, delegating the
   same correction guard and native commit with client observation requested.
2. Prepare a typed retained correction using the existing inverse derivation and
   ordinary admission. Before publication construct the existing SourceOnly
   owner from the exact encoded inverse and key. Transfer the publication outcome
   into that owner, including cleanup and recovery dispositions. Exact replay
   enters its existing retained-history delivery path without rerunning the
   inverse. Bind original correction intent during fresh recovery admission.
   Refused/stale correction attempts cannot fall into ordinary source execution.
3. Extend the actual Bank worker and money DTO with closed-source correction and
   native custody handoff. Reuse the original operation advance, authorization
   and close transitions. No transport-owned correction phase engine.
4. Prove closed source, intervening movement, fresh correction, retained read,
   normal close and exact response-loss retry through the actual process adapter.
   Include copied/mixed locator refusal, revoked authority and an authentic
   noncompleted native disposition. Preserve original accounting and effects.

## Verification and scope

The focused native Bank court and broader 95-test ordinary suite passed before
this delivery work. Reuse them as evidence of unchanged historical source and
accounting boundaries. Add focused retained-owner evidence, metadata-check
affected publication/Bank/HTTP consumers, then batch the required native and
process checks. Use cached unoptimized first-party builds, unrestricted jobs and
64 MiB Rust thread stacks. Existing independent reviewers inspect actual custody
and process evidence. Batch final boundary/context/format/cap checks with the
parallel Phase 2 slices. Private and later phases remain untouched.

## Historical retained-source and replay checkpoint

The native retained-delivery and unpublished-recovery courts exposed a missing
fixture lifetime: their coordinates survived close, but no selected source held
the optional history through replay. World correctly retires unprotected
predecessors after a settled publication; its ancestry selector then reports
`SourceUnavailable`. Completed idempotency evidence does not grant retained
ancestry or disclosure. This differs from mandatory unpublished/cleanup custody.

Both courts now explicitly keep the existing public source selection through
normal close and exact retry, as required by the milestone's retained-occurrence
journey. The retained-delivery court also releases that selection, publishes a
later movement, and requires the exact unavailable-source denial. Production
retirement, admission, and custody owners are unchanged. This tests optional
history lifetime separately from the corrective action's delivery lifetime.

The native recorded-correction family passes all four tests, including this
release control, the genuine durable-append failure, fresh-authority recovery,
cancelled close returning its original owner, stale-preparation refusal, and
exact retained replay. The seven native estate freeze tests also pass, including
capability revocation and the exact `SourceDiverged` refusal after restoration.
The existing independent QA reviewer accepted these bounded source/runtime
results. Payment workflow settlement, composite definition recovery, and the
actual process journeys had separate Phase 2 acceptance boundaries.
