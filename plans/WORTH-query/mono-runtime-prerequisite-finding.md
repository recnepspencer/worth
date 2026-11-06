# Runtime-core prerequisite: partial lint cleanup and stop

The branch starts at `d2dac17c1581efa533f3391d154fb9e80e417154`.
Fetch and fast-forward checks found master and the provider sync branch already
incorporated. The full engine-once design and `mono-admitted-operation.md` were
read before reviewing the warning owners. Slice 3 is **not implemented**.

## Sound partial change

Remove unused imports and re-exports in Query Execution and its existing owner
tests. Restrict the unmetered `admit_field_mask_categories` wrapper and its export
to `cfg(test)`: its only caller is the existing disclosure-contract owner test.
Production still uses the same metered `admit_field_masks` / `admit_categories`
path. No test body or expectation changes, no meaningful field is removed or
renamed, and no lint suppression is added.

This is an explicitly partial prerequisite commit, under the user's instruction
to commit what is sound and stop on a real conflict. It does not meet the
zero-warning gate, and it is not a completed runtime-core slice. No runtime core,
installation view, seal comparison or new authority type is added.

## Finding: meaningful fields are not dead data

The baseline Clippy run exits 0 but reports **54 library and 53 library-test
warnings** in Query Execution, plus seven upstream Relational warnings. The
never-read group includes these meaningful owner facts:

- `authorization/application_disclosure/decision/diagnostic_preparation.rs`:
  `DiagnosticPreparationDenial::Governance` stores the actual governance refusal.
  Its current inspection resource projection uses `capture(stop, |_| None)`;
  this does not read the original domain cause.
- `primary_graph/causal_inspection/application_inspection/admission.rs`:
  Authorization, CurrentAuthorization, Capability, Scope, Security and Governance
  variants retain original typed owner refusals. The public inspection denial
  currently offers coarse projections and selected resource/canonical details;
  several of these retained causes have no consuming projection.
- `primary_graph/application_output_demand/settlement/observation.rs`:
  `OriginalSettledProducerSource` retains committed settlement, publication and
  disclosure, the reused settlement, and restored checkpoint identity. These
  distinguish original provenance; removing them is not an import cleanup.
- `provider_session/graph_obligation/inspection_terminal.rs`:
  completion carries `managed_run`, `basis`, `basis_release` and `release`.
  `matches_report` checks session, plan and target, while those other completion
  facts remain unread. Their intended proof consumption must be settled by their
  owner rather than erased to satisfy the lint.
- `primary_graph/application_query/managed_index/selection.rs`:
  `MaintainedWatchSelection` retains `association`, `scope` and `native`.
  Watch state owns this selection while refresh validates a separate audience.
  Association and scope are semantic binding coordinates; the native Arc can
  additionally retain backing. These are not interchangeable with a destructor
  guard, and their intended consumer is unresolved here.

Other warnings concern genuine destruction custody, including retired predicate
records and retired application phases. They must not be confused with the
semantic facts above. None of these fields was deleted, renamed with an
underscore, wrapped to hide a warning, or supplied with a dummy read/getter.

The user's rules require preserving/reporting meaningful fields and also require
zero warnings before Slice 3. No behavior-preserving real consumer for all these
facts has been established. Deleting them loses exact causes/provenance; dummy
reads, unused accessors or hand-written formatting would conceal the finding.
Defining the real consuming inspection/provenance contracts changes behavior and
ownership beyond a lint cleanup and the authorized runtime/view cutover.
The independent reviewer reached the same finding. Stop at this boundary rather
than claim that the prerequisite is cleared.

## Finding: requested atomic rename exceeds the declared MSRV

The candidate initially renamed the two existing `fetch_update` calls to
`try_update`, preserving their arguments. Clippy rejects both with:

```text
error: current MSRV (Minimum Supported Rust Version) is `1.94.0` but this item is stable since `1.95.0`
= note: `-D clippy::incompatible-msrv` implied by `-D clippy::all`
```

The sites are `application_contribution/producer/output_evaluation.rs:151` and
`application_query/managed_index/nodes/lineage.rs:42`, under `primary_graph`.
The manifest's Rust 1.94 contract is unchanged. Both renames are reverted, and
their deprecation warnings remain. No MSRV attribute, manifest increase or lint
suppression is used. A checked owner-side compare/exchange loop is a possible
separate correction; it was not implemented or certified here.

## Verification

All commands use the existing Windows toolchain and, for Cargo:

```powershell
$env:WORTH_IMPLEMENTATION_BUILD_ID='11111111-1111-1111-1111-111111111111'
$env:CARGO_TARGET_DIR='C:/worth-build-cache/target-cut-pub-mono'
```

The Clippy command, run at the base, the rejected rename candidate and the final
partial candidate, is:

```text
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets --message-format=json
```

- Base: exit 0, **107 Query Execution warning diagnostics** (54 library, 53
  library-test) and **7 upstream Relational warnings**, 0 errors.
- Rejected `try_update` candidate: exit 101, **4 incompatible-MSRV error
  diagnostics** (two sites compiled as library and library-test), **89 Query
  Execution warnings** and **7 upstream warnings**. No tests run by Clippy.
- Final restored source: exit 0, **91 Query Execution warning diagnostics** and
  **7 upstream warnings**, **0 errors**. No unused-import warning remains.
  The requested **zero-warning gate is not met**.

The final Rust diff is only import/export deletion and the actual unit-test
wrapper's `cfg(test)` restriction. Both atomic call sites are byte-identical to
the landed base. No existing generic engine body was replaced in this partial
prerequisite.

The complete execution-library behavioral run was launched before the two
atomic renames were restored and before the final two unused re-export links
were pruned. Its results are identified separately below rather than presented
as a run of the final-source snapshot. The reviewer accepted reuse of that
behavioral evidence: the final corrections restore the original atomic spelling
and remove unused names, invalidating compilation evidence without introducing
a new behavioral seam. Final-source all-target Clippy, forgery tests and the
decl-host court rebuild/check the restored source. No modification time was
restored and no test was suppressed or weakened.

Final-source decl-host court command:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host
```

It passes with **325 passed, 0 failed, 2 existing ignored, 0 measured, 0
filtered**, exit 0. The two ignores remain the scheduled thousand-step and
thousand-sibling document-history growth qualifications. Runtime 38.65 seconds
is telemetry only. This retains the existing public application/host proof;
it does not prove an unimplemented runtime/view/seal cutover.

Final-source gates (all exit 0):

```text
cargo fmt --manifest-path workspaces/worth-query/Cargo.toml --check -p worth-query-execution
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
bash scripts/ci/check_workspace_rust_line_caps.sh dirty
git diff --check
git diff --cached --check
git diff --check origin/master...HEAD
git diff --exit-code -- workspaces/worth-query/crates/worth-query-decl/src workspaces/worth-query/crates/worth-query-host/src
```

Line caps pass for **11 Rust files**. Boundary-check prints
`boundary-check: repository boundaries are valid`; context check emits no
violations. The facade comparison is an exact source diff, not an API snapshot
generator. Formatting without `--check` ran twice, exit 0. Boundary, context,
format check and line caps also passed on the earlier candidate; its 11-file
line-cap set included the two rejected atomic renames instead of the two later
re-export deletions. Final source was checked again as listed above.

Complete execution-library command on the earlier candidate described above:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib
```

Result: **2234 passed, 0 failed, 1 existing ignored, 0 measured, 0 filtered**,
exit 0. Runtime 736.75 seconds is telemetry, not a gate. The existing
resource-evidence oracle was allowed to finish; no build or test was killed.
The existing disclosure-contract tests retain their independent literal mask
expectations, and the original owner/authority tests retain their existing
oracles. No new court is needed to mirror an import deletion. No owner test body
or expectation was changed.

Final-source forgery command:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib erased_core_forgery
```

Result: **13 passed, 0 failed/ignored/measured, 2222 filtered**, exit 0.
Its existing literal refusal and zero-work expectations exercise real admission
entries, with positive controls that reach their handler/request. These
independent expectations were not changed for this cleanup. This is the existing
Slice 2 forgery court, not the owed new installed-seal mismatch court.
No separate installation-owner package run or new installed-seal injection and
mutation run is claimed: those Slice 3 changes were not started.

The same lane reviewer used qa-loop, qa-tests and code-quality-qa. It independently
identified the meaningful-cause/provenance conflict, caught the incompatible
atomic rename, accepted its full restoration, and inspected the final callback
proof re-export deletions without reopening the accepted patch. It independently
checked the warning and test totals and final gate logs. Its approval is for a
behavior-preserving **partial prerequisite commit only**, with the zero-warning
gate unmet and Slice 3 unstarted. Final independent review passes for this limited
commit, with no unclosed finding in the eleven-file cleanup.

## Readings I chose and stopping point

The explicit pre-slice debt request permits touching its nongeneric imports;
the lane's usual instruction to leave unrelated nongeneric files alone does
not exclude this requested prerequisite. A real owner-test-only unmetered wrapper
belongs under `cfg(test)`, without changing its metered production counterpart.
Preserving the declared MSRV and exact original facts takes precedence over
calling an incomplete warning cleanup finished.

Slice 3, its owed seal forgery cases and their mutation runs, and its
monomorphization measurement are unstarted. No new shape is added. The previous
measurement remains `mono-admitted-operation.json`; this partial cleanup has no
new size or slope claim. Installation/views and the authority order remain at
the landed Slice 2 state and are not represented as certified Slice 3 results.
