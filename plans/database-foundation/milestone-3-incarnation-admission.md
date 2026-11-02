# Home incarnation and changed-build admission

Authority: milestone-3 D7, D17 and Phase 3's competing-open, rebuild and
fencing courts. This slice follows accepted discard commit `b782d526b1`.
D16 waits for P-A and is not part of this slice.

## Boundary review

Store Application's descriptor admission already advances HomeIncarnation
under the real mutation lease. World receives that incarnation in its private
Home adapter, but RuntimeWorldOwnerIdentity currently compares only durable
owner scope: two live admissions of one Home compare equal. Its branch and
recovery authorities therefore need an incarnation fence. Query must consume
that same owner-issued fence in capability progression, product selection and
provider-session admission, before accessing a new incarnation. Recorded
receipts and occurrence descriptions remain data and are freshly readmitted;
they must not acquire this live-authority equality.

Proof has no Foundational dependency (Foundational depends on Proof). The
fenced shape and comparison belong in Proof; Foundation's existing
HomeIdentity/HomeIncarnation supply their descriptive values at the owner
boundary. A fence alone grants no owner access. The actual owners keep their
private constructors and existing process-instance checks.

The public Host fixture and child-process lifecycle harness already exist in
the Store Application court. They permit real competing Host opens without
new test hooks or a new binary. Reopen may encounter P-A's existing Native
Complete/RootLinkage denial; such evidence does not prove successful reopen.
The existing changed-build descriptor court proves authored binding/history
only. Derived lineage admission must be observed at its actual reuse owner.

## Implementation plan

1. Add real-process Host contention and killed-writer readmission courts under
   `process/application/admission/`. Require a typed lease refusal and actual
   serving success; preserve the exact reopen refusal if P-A blocks it.
2. Add the Proof incarnation fence; bind World live owner identities from the
   actual admitted Home adapter. Refuse stale authority at World and Query
   issuing/consuming boundaries. Keep durable descriptions readmittable.
   Add owner courts, including foreign-owner and current-authority controls.
3. Extend the changed-build journey from the existing descriptor control to
   Host open and real derived-output lineage observation. Never regenerate
   during open. Report any P-A readmission block with its exact typed cause.
4. Run named courts through owned Cargo/nextest selection and one direct run
   with RUST_MIN_STACK absent, affected normal targets, formatting, dirty line
   caps, boundary-check and generated-context check. Save complete lane logs.
   Commit coherent changes and request an independent adversarial review.

Production destinations are Proof's fence module, World's identity and
authority owners, and Query's existing capability/session/product owners.
No parallel lease, identity issuer, global incarnation table or durable format
is introduced. Host opening, Query installation, Native durability, Store
Section readmission and P-B close remain untouched. Any narrow P-D owner-side
admission seam is recorded explicitly in the final handoff.

## Requirement map and qualification limits

| Requirement | Implementation / court | Qualification |
| --- | --- | --- |
| D7 one writer through public Host | `admission::ownership::competing_host_processes_admit_exactly_one_writer_until_process_death` | Real writer process serves; competing parent receives `MutationOwnerContended`. After kill, the third process acquires the lease and reaches Native readmission. Serving after kill remains blocked. |
| D7 old World live selectors | `admission::world_incarnation::reopened_world_refuses_old_live_selectors_and_readmits_original_durable_occurrence` | Compiled, but actual reopen refuses `Relational(CompleteNativeRequired)` before stale-selector assertions. |
| D7 old recovery handles | `world_durable_release::killed_native_effect_reopens_original_world_custody_releases_and_reopens_again` | Compiled; first recovery refuses `Relational(Native(Durability(...)))`, class `CorruptCheckpoint`, detail `recovered source synchronization denied: RootLinkage`, before the added stale-handle assertion. |
| D7 Query capability progression and provider sessions | Original World owner carried at issuance; receiver checks it before progression/session work | Compiles and runs the affected ordinary owner targets. Actual N+1 capability/session refusal is not yet qualified through a filesystem Host. |
| D17 immutable comparison | Proof `live_fences_refuse_both_directions_of_incarnation_drift` | Passes, including same-fence positive control. A fence alone is not authority. |
| Changed build: binding and history | Existing `changed_implementation_build_preserves_authored_binding_and_exact_durable_history` | Passes in the 19-case Store Application owner target. |
| Changed build: public Host | `admission::changed_build::changed_host_build_keeps_initializer_closed_and_reaches_native_readmission` | Build B passes descriptor admission without initializer execution, then Native refuses `CorruptCheckpoint` / `RootLinkage`. This does not certify a serving changed-build Host. |
| Changed build: real derived lineage | `producer_predicate_checkpoint::changed_build::changed_build_preserves_recorded_output_and_reproduces_stale_output_only_on_demand` | Component court uses genuine producer output/checkpoint, compares the full accepted record and observes real producer phases. It does not replace the blocked directory Host journey. Both this changed-build case and the existing same-build reuse/source-change control pass (2/2). |

The direct new Host controls were run with `RUST_MIN_STACK` absent. Fresh
opening reaches serving and contention is typed, but both existing-Home paths
overflow the normal test thread stack. The same cases reach the exact Native
refusal under the committed Cargo stack setting. Observing opening custody in
place reduced test-side copies but did not remove this failure. Host opening is
reserved for P-A; this is an unresolved qualification block, not a passed lane.
The existing discard/reinitialize public Host control passes directly with the
stack override absent, preserving the fresh-Home regression control.

Query preview sessions remain in reserved installation ownership. Their
existing foreign-runtime check was not changed. This candidate does not claim
that every Query handle/session family is fully incarnation-qualified. D16,
Query installation splitting, Native durability and Store Section readmission
remain outside this candidate. No Close production module is changed.

## Corrections exposed by qualification

- Recovery cleanup initially reached catalog-private state. The product check
  now delegates to the catalog owner's incarnation admission.
- New typed variants exposed exhaustive match omissions in Query, Publication
  and Server. Each mapping preserves the original refusal or translates it to
  the corresponding typed no-effect outcome. No fallback arm was introduced.
- The first fence representation duplicated Home bytes already held by the
  sealed owner scope. Removing that duplicate fixed one metadata-capacity
  failure. The final fence carries only the nonzero admitted incarnation;
  receiving owners admit exact Home/slot before comparing it.
- The two-attempt preparation fixture still underfunded unrelated retained
  history metadata at 8192 bytes. It now uses the nearest publication-service
  fixture's 16384-byte history allocation. Active-attempt, recovery, observation
  and custody limits and every assertion remain unchanged. All 14 preparation
  and borrowed-call controls pass; product metadata enforcement is unchanged.
- The first Host probe expected `CorruptCanonicalArtifact` for RootLinkage.
  The actual Native adapter class is `CorruptCheckpoint`; the test expectation
  now preserves that exact typed class and detail.
- The old unavailable-World-checkpoint assertion in
  `ordinary_occurrences_reopen_original_history_after_later_native_advance`
  is retired by milestone-3 D12's implemented Complete Home checkpoint contract
  (Native Complete integration `9429165bd3`). It now requires actual Closed
  with a successful checkpoint. No history/occurrence assertion was removed.
  Reopen then reaches the separate `CompleteNativeRequired` block. Restoring
  its still-needed checkpoint-denial import fixed the successor compile.
- The Query workspace has no committed `ci` nextest profile. The attempted
  invocation ran no tests; its successor uses its available `default` profile.

## Facades and merge seams

Proof's facade exports `HomeIncarnationFence` and `HomeIncarnationDenial`.
Foundation adds only `HomeIncarnation::fence`; existing World and Query facade
exports carry their new refusal variants. No boundary configuration, Cargo
manifest, feature, durable format, Host opening or Close module changed.

The coordinator approved the single inventory-refusal group arm in Query
`application_installation/program/program_runtime.rs`. It is a P-A merge seam;
no other change was made there. Query `provider/idempotency.rs` propagates the
original typed session failure through existing `ExecutionDenied`; receipt
resolution and the `provider/idempotency/` directory are untouched (P-D seam).
World branch retirement's pre-effect incarnation check and Query's
`product_world/branch_close.rs` outcome mapping are narrow lifecycle seams.
Server's product-adapter diagnostic renders `stale_incarnation`; no
`compat_http` file changed.

Every exhaustive-match propagation site is listed below, relative to its
crate's `src/` directory. The owner checks and newly declared variants are
separate from these propagation-only changes.

Query Execution:

- `basis/product_branch/creation_recovery.rs`
- `domain_computation/execution_runtime/product_world/admission.rs`
- `domain_computation/execution_runtime/product_world/branch_close.rs`
- `domain_computation/primary_graph/application_attempt/compare_and_commit/commit_outcome/denial/execution.rs`
- `domain_computation/primary_graph/application_attempt/provider_compare_denial.rs`
- `domain_computation/primary_graph/application_attempt/provider_execution/phase/prepared/running/progression/commit_resolution/world_no_effect.rs`
- `domain_computation/primary_graph/application_attempt/workflow_transition_program/publication/operation/approval_denial.rs`
- `domain_computation/primary_graph/application_installation/program/program_runtime.rs`
- `domain_computation/primary_graph/conditional_operation/application_operation_reentry/denial/authorization.rs`
- `domain_computation/primary_graph/conditional_operation/application_operation_reentry/sealed_commit/denial.rs`
- `domain_computation/primary_graph/product_operation/generated_materialization/restoration/no_effect.rs`
- `domain_computation/primary_graph/provider/application_attempt_state/commit_preparation/relational_commit/commit_execution.rs`
- `domain_computation/primary_graph/provider/idempotency.rs`
- `domain_computation/primary_graph/publication_request_denial.rs`

Query Publication:

- `application_entry/mutation/performed/denial/recovery_posture/query.rs`
- `application_entry/mutation/recovery/interruption.rs`

Server: `product_adapter/operation_custody/diagnostic.rs`.
World: `recovery/cleanup.rs` maps its private cleanup denial.

Full command headers, source revisions, clean flags, timings, exit codes and
counts are indexed in `target/lane-evidence/pc-incarnation-lane-index.txt`.
No ignored test, exhaustive Home oracle or Store scenario lane was run.

## Baseline source qualification at f5d91091b8

- Query Execution and Publication normal owner targets: 2230 passed, 0 failed,
  0 ignored executed (3 skipped).
- Changed-build producer component and same-build reuse control: 2 passed.
- Final-source selected World/Server owner controls: 47 passed. Additional
  World branch/publication/recovery service controls: 81 passed.
- Store Application normal owner target: 19 passed. Store court ordinary
  selection has no tests; only its named process courts were executed.
- Host contention/native-boundary and changed-build/native-boundary controls:
  2 passed under the committed runner. Their direct normal-stack executions
  remain failed as documented above. The fresh public Host discard/reinitialize
  regression passes directly with `RUST_MIN_STACK` absent.
- Formatting, dirty and full-candidate line caps (66 Rust files), boundary-check
  and agent-context freshness pass. The first line-cap launch selected absent
  WSL; using the installed Git Bash ran the unchanged committed script green.

These results qualify the implemented owner checks and component behavior;
they do not close the outstanding filesystem fencing, serving-after-kill,
changed-build Host, preview-session or normal-stack reopen obligations.


## Independent review correction

The first independent reviewer rejected f5d91091b8 with these findings:

1. P2: directory World recovery release, continuation and settled adoption
   returned ReopenRequired before the catalog's incarnation comparison. This
   was a product design gap: fencing only at the deeper lookup missed the
   receiver's earlier durable-route return. All three receiver entries now
   admit their original handle before that return. Current and foreign owner
   handling remains at the actual catalog owner. Five recovery-service owner
   controls pass; filesystem N+1 assertions remain blocked as recorded above.
2. P2: capability read revalidation and admitted-operation read consumption
   reached generic runtime refusals before the Home fence. This was a product
   design gap: the capability-to-operation transition dropped explicit Home
   carriage and the guard covered only progression. Admitted operations now
   retain their issuing World owner. Capability read validation, read-attempt
   admission, idempotency read admission, managed recovery, migration and
   transaction receiver paths check stale incarnation before their existing
   affinity checks. Projection reuses the actual World owner already carried
   by its advancement phase. The separate stale-only guard grants no admission;
   foreign-owner checks remain mandatory at each receiver. Qualification follows
   this coherent correction batch.
3. P1: normal-stack Host reopen aborts. The reviewer located the public seam at
   admission/mod.rs calling Host opening.rs, but did not establish the precise
   overflowing production frame. This remains an unresolved product failure
   in reserved Host-opening ownership. A coordinator scope question is pending;
   no larger-stack result is substituted for normal-stack qualification.

The correction also adds a narrow pre-effect check in
`application_attempt/idempotency_resolution.rs` (P-D merge seam). No receipt
resolution algorithm changes. `read_set.rs` moves its ordinary/projected
admission methods into `read_set/admission.rs` to retain the line cap.
`WorthQueryProductTransactionCommitError::StaleIncarnation` uses the existing
product facade. No new construction door or Cargo feature is introduced.

Broader handle families, including preview installation and external recovery
handle admission, have not acquired full filesystem N+1 qualification in this
candidate. The new operation checks do not imply those families are complete.


The correction's read-set responsibility split first failed compilation because
three relative paths still named their previous parent. They now resolve the
same ValueDecodeState and retained decision facts owners from the child module;
no behavior or assertion changes. The first formatting attempt also exposed a
missing explicit module path; the declaration now follows this owner's existing
`#[path = "operation_admission/..."]` convention.

A new real-owner foreign-custody control,
`foreign_recovery_authority_refuses_release_continuation_and_adoption_without_effects`,
passes (1/1). It presents actual retained Native/World custody to another genuine
World owner through all three receiver methods, checks typed ForeignHandle and
proves the receiver head/catalog and original retained record are unchanged.
It is memory-owner affinity evidence, not a fabricated filesystem N+1 proof.


Boundary qualification rejected the new foreign-authority test file with
BC9002_TEST_TOPOLOGY: it introduced an additional source court rooted in
ExecutionRequest. The test is retained in the existing recovery-custody court
`recovery_service/tests.rs`, with all assertions including actual Native
Relational/Signal descriptors. Only the unchanged declarative resource-budget
function moves into `test_budgets.rs`, which constructs no runtime or execution
request. This preserves one court and the 400-line cap without a baseline or
boundary-configuration exemption. The test's module-qualified name changes
only by removing the intermediate foreign_authority_tests module.


## Publication receiver review correction

The second fresh review of a2ab9ea645 rejected one additional permitted owner
path: query_publication_home_binding compared fenced owner equality and returned
IdentityScope before the later durable preparation guard. The public selector
now returns Reservation(NoEffectCause::StaleIncarnation) before generic scope /
process checks. It preserves IdentityScope for foreign ownership.

A bounded audit of owner comparisons in World lifecycle service ports also
found prepared ordinary/conditional publication consumption and durable
publication consumption with early generic owner outcomes. They now compare the
same carried incarnation first, preserving each foreign-owner outcome. The
World owned-async source receiver similarly checks stale observations before
its Directory unsupported-operation refusal. Only current/foreign semantics
continue through their existing paths; this adds no Directory async support.
This last entry is a narrow P-D World call-admission merge seam. The two Query
legacy-bridge match arms propagate its new StaleIncarnation variant through
WorthQueryOwnedAsyncRuntimeDenial::StaleIncarnation. All affected public enum
variants remain exposed through their existing facades; no new construction
surface, manifest, boundary configuration or format changes.

The second review also retains the distinct reserved normal-stack and Native
readmission qualification blocks. No Host opening code is changed.


The publication-selector correction passes 19 World owner/public-port controls.
The Query caller trace also found legacy_bridge/services.rs require_selected
calling a boolean product-ownership predicate before World admission. It now
compares the selected lease's actual owner incarnation first, preserving
ProductSelectionMismatch for foreign ownership. Repository search for
integration_owns_product_branch found only its definition and this one caller;
no second caller remains to bypass that receiver. This is the same narrow
World call-admission seam, not an async-support implementation.


## Shared disclosure and recovery-capability correction

The third fresh review of b6792147bd rejected pending disclosure preparation:
its generic runtime comparison bypassed the capability's Home fence. The
capability's unchecked runtime-authority getter is removed. Its replacement
requires the concrete receiving World owner and returns Proof's typed Home
incarnation result, so existing getter callers must be audited at compile time.
Shared pending disclosure checks it before original-request validation, then
preserves original request/foreign-affinity/current-request/capture ordering.
Governed query, Resource and inspection callers preserve the typed authorization
cause through their own existing error owners. Resource and inspection public
stop kinds distinguish StaleIncarnation without exposing protected data.
Two pure error-projection tests contrast stale and foreign causes; they do not
fabricate capabilities or claim filesystem N+1 evidence.

The getter audit found recovery-inspection disclosure as a third consumer.
Its disclosure proof now carries the actual admitted Home owner through its
later receiver. Recovery handles already carry that owner inside their genuine
committed-publication binding; no duplicate handle field, counter or registry
is added. Recovery authority bases retain it at mint, and current runtime,
Resource recovery and redispatch receivers compare it before acting. Existing
pure mechanism tests take the additional carriage from their genuine existing
handles; no expected cause or assertion is weakened. Query's existing
RecoveryHandleDenialKind adds StaleIncarnation through its existing facade.
Redispatch and Resource recovery are additional narrow P-D owner-check seams;
no receipt or idempotency algorithm changes.

The baseline 66-file qualification count is explicitly tied to f5d91091b8.
Later full-candidate counts are recorded by the range-cap lane in the evidence
index, rather than presenting that earlier count as the current range.


The checked-accessor compile found workflow audit as an additional receiver
under a differently named subject/audit pair. Both accesses now use the checked
metadata accessor before independent audit work; StaleIncarnation stays a
technical Authorization error and can never become permission withholding.
Foreign ownership retains PolicyMismatch. The first compile also caught three
incorrect stop-projection patterns: branch StaleIncarnation is nested inside
AdmissionDenial::Refused(AdmissionRefusalKind), not a top-level denial variant.
Those patterns now preserve the actual existing typed shape.
