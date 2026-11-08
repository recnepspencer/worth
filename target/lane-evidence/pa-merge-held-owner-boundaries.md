# Held HAPPY and landing boundaries after the P-A merge

The complete first-pass failure census is in pa-merge-classified-failures.json.
These are not CLEANUP reclassifications and no court is weakened.

## Complete capture and selection ACK: P-A contract ruling

Court: `genuine_complete_section_retains_capture_and_resumes_same_selection_ack`
in Store relational `section/checkpoint/journal_tests/complete.rs:67`.
The original claim landed in9429165bd3. The new guard in a5fd44a313 requires
Native binding `durability/medium/binding/checkpoint_selection.rs:187` to obtain
an actual capture-base-slot snapshot. The court does real Native Complete,
physical publication and selection receipt, but does not perform the real
World Home seal that installs that snapshot. It now answers `CaptureNotBound`.

`durability/authority/checkpoint_complete/admission_projection.rs` installs the
slot only while sealing a real Home cut. `prepare_home` on the preparation
creates another capture; it is not a conversion of this already-selected one.
The bare Store relational package has no World or Store application dependency.
A fake Home identity, synthetic seal, replacement capture, or altered ACK check
would not preserve the same-capture claim. The owner must supply a lawful route
or port the claim to its actual composed court, preserving SelectionNotKnown,
physical selected capture identity, retry and close-custody assertions.
No protected source is edited here.

## Worker-only capture: P-C World / P-A Query

The fixture shared_execution worker-only request is intentionally unaccounted.
World `lifecycle/interim_checkpoint.rs:169-175` charges unconditionally; Query
`application_checkpoint/interim_transport.rs:46` has the same subsequent
checkpoint_cost issue. Native image payload accounting already guards this
policy. These are protected owner files. Do not fund or alter the example to
hide a worker-only policy defect. The genuine standalone main had separate green
evidence and is not the fixture-policy failure.

## Existing held cases

P-C: durable sibling fork and ProductBasisStale after release/reopen.
User: example main-stack architecture decision; stack repairs remain stopped.
P-A landing2: three remaining BC9002 adapter migrations.

## Topology capture setup: integration / coordinator boundary ruling

Five exact names and failures are in pa-merge-final-topology-blocks.json. At the
final source they fail checkpoint capture setup with WorkExhausted, not their
semantic comparison assertions. All five tests and the original 8M serving
policy remain. The added ExecutionResourceLease factory has been removed.

The scoped baseline search was:
`rg -n 'checkpoint_recovery/(support|computation_partition|support/installation)' tools/boundary-check/config/src-court-baseline.json`.
Rows2011-2016 allow a program runtime, primary-graph runtime, or original
ExecutionRequest/SerialRequest bootstrap provider; none allows an
ExecutionResourceLease provider. No Topology source baseline permits that root.
Moving it into installation.rs would still add a forbidden provider. A public
bootstrap opening cannot be nested inside another host opening (installed_opening.rs:70),
and an allocation policy cannot cross host calls (authority/request.rs:200).
No alias, baseline, permission or synthetic authority is added.

The prior funded-path run also reached ordinary keyed principal resolution and
failed ConsumerPrincipalBinding WorkExhausted under the unchanged 8M serving
policy. Its evidence is pa-merge-repair/fixture/run-final.log. That deeper
Query/Native serving finding remains HAPPY and is distinct from the current
capture setup gate. Its exact last charge is not established: the candidate
owners are selected_product/admitted/native.rs projection/index admission and
resolution/admitted_lookup.rs bounded identity lookup. Do not raise the serving
policy or claim a more specific cause without evidence.

## Native retired reconstruction warnings

32 unique dead-code diagnostics remain. The exact graph and nine still-claimed
contracts are inventoried in pa-merge-native-retirement-boundary.md. Two obsolete
interfaces have no current shipping analogue. Retirement authorization or owner
handoff is pending; do not delete those tests, invent a test door, or hide the
legacy graph under cfg(test). The other seven claims are not silently retired.
