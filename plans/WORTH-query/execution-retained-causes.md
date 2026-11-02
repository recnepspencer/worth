# Execution retained causes: partial slice and scope stop

This lane starts at `52e7dba3dc` on `codex/cut-pub-causes-20261101`.
Both required sync branches were fetched and merged before the slice;
both merges reported already up to date. The governing external plan was read
without changes. This report covers only exact inspection owner-cause
projection. The complete lane is **not finished**.

## Boundary and implementation

Inspection admission owns its original typed refusals. Public inspection
denials already retain those originals and their original request backing;
resource and canonical projections consume their respective causes. The new
`WorthQueryApplicationInspectionOwnerRefusal<'a>` projection borrows the exact
query authorization, current authorization, capability, scope, security and
governance causes. `owner_refusal()` also consumes diagnostic preparation's
governance cause. It does not reconstruct owner denials or change resource
classification. Outer query/scope/selection refusals use the same projection.

The existing governance denial enum is now publicly nameable through the
inspection facade. Host's existing inspection module alias carries both types.
The capability index documents that consumer path. There is no compatibility
path, decoder or migration. No AspectShape form is added. The only new public
shape is the borrowed six-variant owner-refusal enum; the governance enum's
existing three variants are unchanged.

## Field disposition

`a` means wired in this slice. `Pending` identifies work this lane has not
implemented or certified; it is neither deletion nor a completed disposition.

| Retained field | Outcome | Consumer or remaining obligation | Court |
|---|---|---|---|
| DiagnosticPreparationDenial::Governance.0 | a | `owner_refusal()` returns borrowed Governance | `inspection_preserves_diagnostic_preparation_governance_cause` |
| InspectionAdmissionDenial::Authorization.0 | a | Borrowed Authorization preserves full query denial | `inspection_preserves_query_authorization_cause` |
| InspectionAdmissionDenial::CurrentAuthorization.0 | a | Borrowed CurrentAuthorization preserves exact kind | `inspection_preserves_current_authorization_cause` |
| InspectionAdmissionDenial::Capability.0 | a | Borrowed Capability preserves full authorization denial | `inspection_preserves_capability_cause` |
| InspectionAdmissionDenial::Scope.0 | a | Borrowed Scope preserves full entity denial | `inspection_preserves_scope_cause` |
| InspectionAdmissionDenial::Security.0 | a | Borrowed Security preserves selected-basis refusal | `inspection_preserves_security_cause` |
| InspectionAdmissionDenial::Governance.0 | a | Borrowed Governance preserves all existing variants | `inspection_preserves_governance_cause` |
| InspectionCause::Query.0 | a | Outer query admission shares the same borrowed Authorization projection | `inspection_preserves_outer_query_cause` |
| InspectionCause::Scope.0 | a | Outer scope resolution shares the same borrowed Scope projection | `inspection_preserves_outer_scope_cause` |
| InspectionCause::Selection.0 | a | Outer branch selection shares the same borrowed Security projection | `inspection_preserves_outer_selection_cause` |
| OriginalSettledProducerSource::Committed.settlement | Pending | Original provenance comparison not implemented | None added |
| OriginalSettledProducerSource::Committed.publication | Pending | Original provenance comparison not implemented | None added |
| OriginalSettledProducerSource::Committed.disclosure | Pending | Original provenance comparison not implemented | None added |
| OriginalSettledProducerSource::StableReused.settlement | Pending | Reused provenance comparison not implemented | None added |
| OriginalSettledProducerSource::Restored.checkpoint | Pending | Restored provenance comparison not implemented | None added |
| WorthQueryInspectionReadCompletion.managed_run | Pending | Report completion check not extended | None added |
| WorthQueryInspectionReadCompletion.basis | Pending | Report completion check not extended | None added |
| WorthQueryInspectionReadCompletion.basis_release | Pending | Report completion check not extended | None added |
| WorthQueryInspectionReadCompletion.release | Pending | Report completion check not extended | None added |
| MaintainedWatchSelection.association | Pending | Watch refresh binding not extended | None added |
| MaintainedWatchSelection.scope | Pending | Watch refresh binding not extended | None added |
| MaintainedWatchSelection.native | Pending | Custody role/name not changed | None added |

Both requested `fetch_update` replacements are also unimplemented. Their
original checked arithmetic and MSRV remain unchanged. No retained field is
deleted or renamed in this slice.

## Scope stop

The stable-source Clippy run reports warnings in admission re-entry, which the
brief reserves to pub-mono and explicitly forbids this lane from editing:

- `application_contribution/producer/execution/source_readmission/ready.rs:5`:
  `function ready_work_denial is never used`.
- `application_query/admission/permission/current_ready.rs:83`:
  `method seal_current_ready is never used`.

Narrow owner inspection and independent review confirm those functions belong
to source readmission and ready-permission admission. The latter delegates to
the actively used `seal_current_ready_with_control`. Suppressing their lints or
inventing callers would hide the ownership conflict. Neither file was edited.
Zero Execution warnings therefore requires coordination with the pub-mono
owner. This report does not claim that these are the only remaining warnings.

## Courts and independent oracles

The new owner-local projection courts construct literal owner denials without
creating an application runtime. Full-denial equality and pointer identity
check that query/capability payloads survive as the original objects. Other
causes are checked against independently chosen literal refusal values.
Governance courts exercise all three existing variants separately at direct
admission and diagnostic preparation, requiring Admission classification and
no resource refusal. `unrelated_stops_do_not_project_owner_causes` excludes
foreign-access and kernel-panic stops. These are projection-contract courts,
not claims of end-to-end authorization issuance.

The shared public consumer was temporarily replaced with `None`. The first
run, before the three outer-route courts were added, returned **1 passed,
7 failed, 0 ignored/measured, 2235 filtered**, exit 101. The seven failures name
each requested positive cause court; the unrelated-stop negative control
passes. The expanded run returned **1 passed, 10 failed, 0 ignored/measured,
2235 filtered**, exit 101. Its additional failures name each outer route. These
are intended assertion failures after successful compilation, not build
failures. The mutation command for both runs was:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib causal_inspection::application_inspection::stop::owner::tests
```

Source bytes were restored in `finally` without restoring their modification
time. The following final-source library build/run reports all eleven new
courts passing, including the negative control. No mutation or instrumentation
remains in source.

## Verification

All Cargo runs use a lane-isolated target directory:

```text
CARGO_TARGET_DIR=C:/worth-build-cache/target-cut-pub-causes
WORTH_IMPLEMENTATION_BUILD_ID=11111111-1111-1111-1111-111111111111
```

Boundary/context tools use `C:/worth-build-cache/target-cut-pub-causes-tools`.
Long Cargo commands run in hidden background PowerShell processes with output
and exit status in `%TEMP%/worth-pub-causes`; no build is killed by a tool timeout.
One waiting helper was stopped before it had started any final build, to add
the outer-route courts before final verification.

An attempted initial baseline Clippy run overlapped source construction and
failed with **one E0583 error**, **39 Execution warning diagnostics** and
**7 Relational warnings**, exit 101. Its failure output was:

```text
error[E0583]: file not found for module `tests`
  --> .../application_inspection/stop/owner.rs:49:1
49 | mod tests;
```

The test file had not yet been created when that compiler read the module.
This is not baseline or final-source evidence. A subsequent stable-source
run of the same command exited 0 with **71 Execution warning diagnostics**,
**7 Relational warnings**, **0 errors**:

```text
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets --message-format=json
```

It compiled the original two grouped new tests before they were split/extended;
final-source Clippy is repeated below. The zero-warning requirement is unmet.

Final-source gates already completed, all exit 0:

```text
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
bash scripts/ci/check_workspace_rust_line_caps.sh dirty
git diff --check origin/master...HEAD
git diff --check
```

Boundary-check reports `boundary-check: repository boundaries are valid`.
Context emits no violations. Line caps check **8 Rust files**, all pass.
The same gates also passed on the earlier two-test candidate.

Final-source complete library command, exit 0:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib
```

Result: **2245 passed, 0 failed, 1 existing ignored, 0 measured, 0 filtered**.
This includes all eleven added owner-projection courts. The existing ignore is
the subprocess maintenance-cleanup probe whose parent supplies its payload.
Runtime 646.40 seconds is telemetry only.

Final-source full Decl/Host binary, exit 0:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host
```

Result: **325 passed, 0 failed, 2 existing ignored, 0 measured, 0 filtered**.
The ignores remain the scheduled thousand-step and thousand-sibling history
growth qualifications. Runtime 38.04 seconds is telemetry only. This is broad
existing public facade/installed behavior regression evidence; its existing
expected results were not changed.

Final-source Query/Decl/Host inspection filter, exit 0:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query-decl-host --test worth_query_court_query_decl_host inspection
```

Result: **47 passed, 0 failed/ignored/measured, 295 filtered**. Runtime 10.65
seconds is telemetry only. Existing public inspection, genuine binding/runtime
affinity and original causal-evidence journeys retain their existing literal
row/refusal oracles and genuine positive counterparts. No expected result was
changed. This does not certify the pending provenance or completion consumers.
Final-source public Query inspection filter, exit 0:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query --test worth_query_court_query inspection
```

Result: **3 passed, 0 failed/ignored/measured, 61 filtered**. Runtime 0.02 seconds
is telemetry only. Two existing tests compile-reference real public inspection
signatures and outcome-navigation traits; the third executes the existing
inspection grammar journey and independently requires a settled outcome after
its genuine read completion. Their original expectations are unchanged, and
they add no claim about pending consumers.
Final-source all-target Clippy repeated the command above, exit 0: **71
Execution warning diagnostics, 7 upstream Relational warnings, 0 errors**.
No zero-warning claim is made. The two confirmed reserved-file warnings remain.
Final-source formatting, exit 0 with no output:

```text
cargo fmt --manifest-path workspaces/worth-query/Cargo.toml --check -p worth-query-execution
```

The formatting application before the expanded mutation also exited 0.

The seven upstream Relational warning diagnostics are out of scope:

- `mvcc/publication/companion/cell/managed_source.rs:15`: private
  `CompanionBranchCellCore` in `ManagedIndexSourceObservation::core`.
- The same file, line 27: private `CompanionBranchCellCore` in
  `ManagedSourceRootImage::core`.
- The same file, line 29: private `CompanionRootImage` in
  `ManagedSourceRootImage::image`.
- The same file, line 207: private `CompanionBranchCellCore` in
  `ManagedSourceReadConditions::matches_reserved`.
- The same file, line 207: private `CompanionRootImage` in that method.
- `validation/candidate_source/preparation.rs:225`: unread `closure`.
- `validation/candidate_source/authored_history.rs:54–55`: unread `records`
  and `incidence` (one diagnostic).

Public-court builds additionally emit two distinct untouched unused-import
warnings: `application_schema` in the document-retention testkit's
`certification/application_graph/cell_history/checkpoint.rs:6`, and
`GovernedRetentionHistory as History` in the Decl/Host court's
`certification/document_history/retention/governed.rs:14`. The testkit warning
is repeated in both the full Decl/Host and inspection-host build logs. Neither
source was edited.

## Independent review

The same independent reviewer uses qa-loop, qa-tests and code-quality-qa for
this slice. It accepted the borrowed cause design and literal/pointer oracles,
flagged the missing capability-index entry, and closed that finding after the
entry was added. It accepted the incremental split into seven positive courts
and one negative control. It independently confirmed the admission re-entry
scope blocker. A later focused evidence check identified missing coverage for
the three additional outer routes; three local literal-oracle courts were added.
Final independent review passes for this explicitly limited partial slice.
The reviewer independently checked all final counts/gates, both mutation
failure sets and source restoration. Both source/evidence findings are closed;
the full lane's zero-warning scope blocker remains. No approval for items 3–6
or for the zero-warning requirement is implied.

## Readings I chose

The instruction to preserve exact governance causes is fulfilled through the
owner-cause projection, rather than classifying governance as a resource stop.
This keeps governance refusals classified as Admission. The instruction to
stop for outside-lane scope governs the zero-warning conflict; completing this
sound first slice does not certify the remaining lane. The brief's explicit
package-scoped commands govern verification instead of wider skill defaults.
Its specifically required downstream court binaries are affected proof targets
for the Execution change even though their sources are unchanged; inspection
runs filter within their registered binaries rather than building a workspace.
