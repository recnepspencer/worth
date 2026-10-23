# Authored create and delete correction

**Status:** Completed and covered by certified Phase 2 acceptance.
The [9.18 specification](../../plans/WORTH-query/milestone-9.18.md)
owns current milestone progress.

This plan records the public authored-identity slice's historical boundary review,
implementation order and verification notes. It implements the declared
new-identity correspondence for authored lifecycle inverses. Bank compensation
remains a separate business action; private adoption is outside this slice.

## Boundary review

The adversarial case is a deleted authored record followed by a different live
record with the same logical key. An inverse must neither resurrect the old
physical identity nor retire or claim that replacement merely because its key
looks right. The platform must compare the actual source and inverse output
roles, retain full generational identity, and compare the family-declared key.

The existing owners already supply most of the truth:

- A committed operation retains actual output correspondence, immutable committed
  patches, the installed correction association, and admitted pre-images.
- Relational's validated mutation footprint represents deletion as a whole-record
  change. Existing capture therefore retains demanded, admitted field values of
  the deleted record; it does not require a later read of that record.
- Candidate output completion verifies that Create roles name real created
  effects and Retire roles name real deleted entities. The registered provider
  effects retain those native references alongside the actual lowered intents.
- The prepared correction already carries the exact original occurrence through
  fresh admission and provider registration. That is the handoff at which an
  incorrect inverse candidate must be refused before native commit.
- Relational allocates physical identity and generations. Query must use a real
  Create reference for restoration and must not inject the retired entity ID.

The missing truth is the declared relationship between the original role, inverse
role and logical key. The created-output field manifest supplies historical
values, but does not declare a family or require the inverse to consume the
correct identity. Ordinary deletion by a decoded key alone is insufficient.

One typed family declares its logical key once. Each create/delete correction
association references that family and its actual source/inverse roles. The
portable declaration remains descriptive; installation checks it against real
schema fields and operation/output contracts. Native candidate validation, not
the derivation callback, owns the identity/key comparison.

Lifecycle inversion is distinct from compensation. A journal compensation
preserves the original journal and appends another business operation; it must not
be forced through authored entity retirement. Conversely, a recorded lifecycle
inverse without a family declaration must be rejected at installation.

Creation has no original field pre-image. Provide an explicit recorded-create
inverse constructor whose empty capture is valid only with its installed
Create-to-Retire correspondence. Supply original created-output fields to that
derivation. Do not invent a pre-image, require an unrelated parent edit, or label
the lifecycle inverse as domain compensation. Delete-to-Create continues to use
the existing exact admitted pre-image demand, including the family key.

## Implementation shape

Declaration places family/key authoring and portable correspondence under
`application_aftermath/correction_binding/authored_identity/`. Existing correction
association, schema authoring, canonical identity and archive owners carry that
meaning. Installation adds its subordinate validator beside source-output
validation. No separate registry or history table is introduced.

Execution places historical identity preparation and candidate comparison under
`primary_graph/product_operation/correction_source/authored_identity/`. The
existing prepared correction retains its immutable expected relationship. The
provider registration handoff calls the comparison on its actual output roles
and lowered candidate batch. Ordinary mutations without a recorded correction
take no historical identity path.

The family declares the complete key value meaning, including codec, presence,
units, frame and canonical value contract. Native schema authoring registers its
existing field recipe. Installation compares full meaning, requires exactly-one
source/inverse roles on that entity with opposite Create/Retire postures, and
requires a delete's retained demand to include the key. Repeated references to one
family identity must agree on its key meaning.

Before effects, Create-to-Retire must identify the exact original entity in the
actual inverse Retire role. Delete-to-Create must identify an actual new candidate
Create reference and its initialized logical key must equal the original deleted
record's retained key. The original source role must match that same pre-image
record. Existing candidate completion and Relational allocation establish the
subsequent physical identity. The canonical source causal edge and the two real
output correspondences preserve the logical relationship without another record
of correction authority.

## Ordered implementation and proof

1. Add typed family/key and role correspondence authoring to the existing
   association; serialize and canonicalize it through existing owners. Add
   installation negatives for missing family, wrong roles/entities/postures,
   changed key meaning, conflicting family definitions, and missing retained key.
2. Add recorded creation's explicit capture/material posture. Keep empty demand
   invalid for ordinary field/delete inverses. Preserve all existing compensation
   semantics and historical field witness checks.
3. Prepare the exact source identity/key only after fresh disclosure. Validate
   actual inverse candidate roles and key before provider registration can commit.
   Wrong original identity, wrong key, absent output, and an existing-identity
   replacement are refusals, even if the domain callback otherwise succeeds.
4. Extend the real public numeric application in distinct authored-lifecycle
   modules. Create and correct that exact creation; separately delete a genuine
   authored record and restore it from retained field values. Observe ordinary
   readback, original and successor full IDs, preserved logical key and payload,
   stale physical reference rejection, unrelated edit preservation, and exact
   correction retry. Use a stable parent disclosure scope so retirement does not
   make historical disclosure depend on the retired entity's liveness.
5. Convict the platform comparison with a wrong-key inverse candidate and a
   same-key different-identity candidate. Neither may publish. Owner tests cover
   malformed portable declarations; the public court covers real candidate,
   native allocation and observation boundaries.
6. Update current consumer guidance and capability discovery after the ordinary
   compiled example passes. Reuse accepted source-field, disclosure, compensation,
   program compatibility and recovery checks where these seams are unchanged.

## Verification and coordination

Read the current shared worktree before editing; preserve the coordinator's Bank,
fallible inverse-key and exact source-locator changes. Production edits wait for
the active native build to finish so its source remains stable. Coordinate small
shared-file hunks and run no independent Cargo or snapshot update. The coordinator
batches affected owner metadata/tests, the public example and enforcement with
unoptimized incremental builds, unrestricted jobs and a 64 MiB Rust stack.

Independent source and evidence review must assess the actual identity comparison,
capture posture and negative controls. Passing a deletion example alone does not
close this requirement. This slice does not certify whole Phase 2 or substitute
for Bank process, component, workflow or required-output acceptance.

## Same-head correction race addendum

The existing numeric and conditional-definition courts invalidate a prepared
target before publication. They establish exact-target refusal but do not place
two corrective publishers in flight at the same native World head. World already
provides an owner-issued pause at `BeforeProductCompareAndPublish`, after owner
execution and the last unlocked head check. Reuse that control with one paused
attempt; demanding two arrivals could deadlock a legitimate earlier Relational
contention refusal.

Add a separate numeric correction court. Each worker prepares its own ordinary
recorded inverse locally, because the erased prepared publication and domain
result are intentionally thread-local. Synchronize both completed preparations
before allowing the first publication. Wait for the first to reach the actual
World comparison, then start the second against the same retained target.
Observe its real no-effect/pending/unpublished owner outcome. Release the first
and preserve or explicitly release every returned recovery owner through its
ordinary API. Assert exactly one performed correction, one coherent final
Relational/Signal basis, the source causal link, the inverse field value, and
the preserved independent field. Do not convert an unknown terminal into a
successful loser or introduce a synthetic race outcome. Keep the existing
two-component World owner race evidence for its unchanged causal boundary.

Coordinate the one module registration with the composite agent. Metadata and
native execution remain in its shared batch; no additional build queue.

### Native branch serialization refinement

The first real run reached World's actual pre-CAS pause, but waiting for the
second correction to finish before releasing the first was causally impossible.
`provider_execution/phase/prepared/running/progression.rs` enters the existing
application branch commit lane before `registered_session.progress`. Its mutex
guard remains live through the native commit and World CAS. Distinct correction
keys avoid the command-key lane but still share the exact branch-incarnation
lane. The World pause itself runs outside the product cell's movement lock.

Use the already-existing
`observe_application_branch_commit_contention_for_test` recorder. It reports only
an actual `WouldBlock` from the ordinary lane's `try_lock`; it cannot pause,
release or authorize publication. Both candidates must still finish preparation
at the same original target. Pause the first at World CAS, start the second,
observe actual branch-lane contention and the unchanged product head, then
release the first. Only afterward inspect the second's precise native stale or
no-effect outcome and preserve any cleanup/recovery custody it returns. Keep the
one-performed correction, exact coherent World basis, causal source, inverse
value and unrelated-value assertions. Reuse the independent World mixed-owner
CAS-loss evidence for the lower boundary that permits both owner legs to run.

This changes the court's scheduling oracle to the real owning boundary; it does
not widen a timeout, change production serialization, or classify contention as
success. Reviewer approval of this proof boundary precedes the fixture edit.

### Authored physical reference proof

The first live-reference attempt exposed a fixture-domain mismatch:
`prior_output` resolves source-partition output lineage, while these authored
actions deliberately have no producer source partition. Its `Unavailable`
result precedes the entity liveness check and cannot prove stale references.

Use an ordinary declared `AuthoredReference` relation with the existing
`same_context_unbounded_retain_dangling` integrity policy. The creating handler
links the stable parent to its actual created entity in the same candidate.
Restoration allocates its successor without rebinding that relation. The
consumer follows the relation through the admitted decision reader and reads the
target's primary field while live. After retirement and same-key new-identity
restoration, the same native traversal must return `EndpointUnavailable`, which
the fixture counts before refusing publication. The actual native endpoint
projection checks the original full EntityId and `Live` lifecycle before minting
its typed identity. Keep independent receipt ID inequality, logical-key lookup,
payload, unrelated field and unchanged-head assertions. No producer, fabricated
partition, key-based retargeting or new public reference API is involved.

### Audit-retained relation projection

The native run established a lower owning boundary than the proposed
`EndpointUnavailable` oracle. Relational's
`record_changes/relation_lifecycle.rs` preserves the original endpoints while
changing the relation lifecycle to `RetainedDanglingForAudit`. Its ordinary live
adjacency projection excludes that lifecycle before Query traverses endpoints.
The honest consumer result is therefore zero live relations and its precise
`DomainDenied(MissingValue)`, with no publication. Keep the live preflight and
compare its actual endpoint's full EntityId with the original created receipt.
The successor remains independently readable under the same logical key and has
a different full identity. This proves the old relationship was not retargeted;
it does not claim an endpoint-validation denial that the native read never reaches.
