# BC9002 root consumer handoff: IN PROGRESS

Reviewed source: `5692704eb7bf41cc9c182c1506067b94a7836eb5`.
No production source, tests, adapters, baseline or boundary configuration changed.
The newly granted consumer paths passed both required checks before editing.
The remaining dependency is **not** a change on Lifecycle's test files.

## Boundary and implementation plan

The external testkit owns provider bytes and scripted physical fates; Native
owns authentic root construction, root admission, immutable region ownership
and their request reservations. Courts may supply the external provider and
observe Native's public basis, recorded components, envelopes, typed outcomes,
request ledgers and provider reads. They cannot receive Native's private root
by changing the provider.

The intended cutover remains: migrate actual provider-dependent scenarios,
split independent value logic locally, extend only the external provider,
delete all three local trait implementations and their hooks, shrink the
baseline, then qualify zero BC9002. The prior audit's public envelope custody
case is already migrated at `5692704eb7`; its source and evidence are unchanged.

The new user instruction says: "If a consumer migration needs a production
root change, stop and report it." This condition is met by the exact remaining
claims below. No root production change or new observer door is made here.
Grant 12 retires five named assertions in another handoff; none of these root
claims is among them.

## Exact private claims and nearest public observations

| Test (exact name) | Assertion not preserved by an unchanged public path | Closest existing public observation |
| --- | --- | --- |
| `publication_cost_distinguishes_new_and_reused_schema_authority` | First root's new/reused schema counts are 1/0; second's are 0/1; both schema authorities have the same allocation identity. | Actual publication cost recorder exposes aggregate root/authoritative byte costs; admitted basis exposes schema commitment. Equal commitments do not prove allocation reuse or these counters. |
| `relink_rejects_schema_authority_mutation` | A mutated sealed envelope supplied to the retained actual root's `relink_canonical_envelope` returns `SchemaRootMismatch`. | Real source-byte tampering and recovery refusal; it may stop earlier in artifact/original-input validation and does not execute this retained-root relink. |
| `relink_rejects_visibility_tuple_mutation` | Same retained-root relink returns `VisibilityCommitmentMismatch` for the opaque-aspect tuple mutation. | Source-byte tamper/recovery plus public visibility commitment; an earlier refusal is not this assertion. |
| `completeness_recomputes_visibility_commitment` | The genuine root is complete; cloning it and changing its private committed visibility to a different envelope's commitment makes `is_complete` false. | Genuine recovered branch basis/recorded component admission and mismatch refusal; public clients cannot alter that private root clone. |
| `visibility_commitment_binds_every_visible_tuple_axis` | Besides independent hash sensitivity, every mutant must fail relinking to the actual committed root. | Public recorded visibility changes and corrupt-source refusal. The hash-only half can become a root-free value unit, but dropping the relink half changes the test's claim. |
| `request_readmission_matches_real_root_and_keeps_only_actual_retained_owners` | Private `readmit_for_request` and `readmit_unchanged_for_request` produce the same axes/descriptor/region count as the actual root; unchanged and reconstructed roots share the exact region `Arc`; alias/drop sequence preserves then releases only their retained request backing. | Public checkpoint/recovery with exact basis and data, followed by request-ledger lifetime checks. It does not select the unchanged DATA constructor or expose its region identity. |
| `request_root_refusal_preserves_original_and_releases_partial_backing` | The unchanged DATA readmission is denied at `controller + size_of::<RelationalBranchRoot>() - 1`, refunds partial backing, then returns cancellation without changing the actual original axes. | Public recovery budget/cancellation refusal, refund and retry with unchanged basis. Its broader allocations may stop before this exact root reservation and cannot stand in for the existing boundary. |
| `streamed_visibility_preserves_original_buffered_preimage_and_request_stop` | Independent buffered preimage equals the actual axes and the request-funded private visibility operation; cancelled request returns its exact root denial. | Public basis supplies actual visibility and canonical envelopes. No public operation consumes those inputs as this request-funded visibility DATA subject. Its byte-only hash comparison may be a pure unit; its request-stop assertion is a court claim. |
| `strategy_artifacts_are_bound_into_the_committed_root` | Removing actual committed strategy artifacts must fail the retained-root relink with `VisibilityCommitmentMismatch`. | Public artifact readmission/recovery refusal after semantic byte mutation, which may reject before root relink. |
| `schema_transition_descriptors_are_each_bound_into_the_committed_root` | Removing each actual transition, continuation and reconciliation descriptor separately must fail that same relink with `VisibilityCommitmentMismatch`. | Ordinary public schema transition and exact recovered descriptors, with corrupt-source refusal. Those do not directly exercise the retained-root relink gate. |

`merge_execution_authority_is_bound_into_the_committed_root` in the granted
payload file already constructs a memory runtime; it is not an adapter
consumer merely because it shares the private assertion helper. It remains
unchanged. No durable setup is converted to a memory fixture.

## Production owner seams, not a request for a test port

The exact files outside this grant that own the disputed subjects are:

- `crates/worth-relational/src/branch/root.rs`: private actual root,
  committed axes, completeness and publication cost (getter at line 357).
- `crates/worth-relational/src/branch/root_capture_preparation.rs`: the
  actual new/reused schema decision and its pure cost projection.
- `crates/worth-relational/src/branch/root_readmission.rs`: actual retained
  root relink and schema/content/visibility validation (entry at line 179).
- `crates/worth-relational/src/branch/root_readmission/request.rs`: funded
  original DATA reconstruction and unchanged-region reuse (entries at 8/87).
- `crates/worth-relational/src/branch/root_visibility.rs`: request-funded
  visibility DATA operation, distinct from its uncharged pure hash.

Changing a test file's location does not change these owners' visibility.
Neither external provider behavior nor a normal public basis returns their
private root. A local trait bridge preserves the adapter being retired; a
synthetic root changes the authenticity oracle; a runtime/request in a local
unit retains the court-in-production problem. None is used.

An owner could extract genuine value-only validation and pair it with real
public flow courts, with a reviewed assertion-equivalence design, or explicitly
decide which exact private claims the public observations replace. That
decision/extraction belongs to the root owner. This note does not recommend
making private constructors, roots or counters public for tests.

## Search and qualification evidence

`bc-root-owner-preflight.log` and `.json` contain the exact commands, HEAD,
Lifecycle ref, all three committed quiet-diffs (exit 0), all three worktree
status probes (empty), private entry-point hits, public facade search (no
matches, rg exit 1 is expected), public descriptive accessors and unchanged
source/baseline status. The probe exits 0; tests 0 passed / 0 failed / 0 ignored.

This turn ran no Cargo, test or Clippy command. Formatting, line caps, boundary
and generated-context gates are **unrun this turn**, not claimed green. There
is no source phase to qualify. The historical precommit boundary result in
`queue4-boundary.log` records HEAD `1f9f39b138`, with a dirty worktree, and
three BC9002 findings at `cold_source.rs`, `cold_source/protected_reads.rs`
and `runtime_attachment.rs`. Those three adapters remain unchanged at the
current HEAD. This is not a fresh boundary run at `5692704eb7`; no new
zero-finding result exists. No ignored test, exhaustive oracle,
query-scale or scenario suite was run.

D13 and named digests remain blocked until the coordinator names the combined
integration head. No merge or push occurred. `BC9002 LANDED` is not claimed.
