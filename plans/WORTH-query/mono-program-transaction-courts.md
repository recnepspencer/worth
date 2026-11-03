# Program transaction custody courts

Two fresh-authentication courts use the existing Host `commit_for_program` door:
a genuine foreign-runtime change requires literal `Err(ApplicationMismatch)`;
a sibling-branch change requires literal `Err(BranchMismatch)`. Both verify
native retention remains 7. Their literal contract oracles are independent of
the implementation. Existing test names remain intact; two are added.

The court change adds one test module, its anchor and this record, with no new
production form, authority type, access door or public signature. The commit
also integrates the required master sync through `65e5548d14`.

## Mutations

Each mutation removed only the named local block in
`worth-query-execution/src/domain_computation/primary_graph/product_operation/transaction.rs`,
inside `commit_for_program`. Both compiled and failed the intended assertion:

| Removed guard | Selected test suffix | Actual mutant result | Pass/fail/ignored/filtered |
| --- | --- | --- | --- |
| Lines 121–128, application custody | `program_transaction_refuses_a_fresh_change_from_another_application` | `Err(BranchMismatch)` | 0/1/0/340 |
| Lines 129–131, branch custody | `program_transaction_refuses_a_fresh_change_on_another_branch` | `Ok(Denied(OperationBodyExecutionRequired))`, `ProposalBinding` | 0/1/0/340 |

Both returned Cargo exit 101. Original source bytes were restored after each.
The filtered pair passed before and after mutation: 2 passed, 0 failed, 0 ignored,
339 filtered out per run. No mutation remains in production.

## Verification

All Query Cargo commands use `--manifest-path workspaces/worth-query/Cargo.toml`.
Environment: `PYTHONUTF8=1`,
`WORTH_IMPLEMENTATION_BUILD_ID=11111111-1111-1111-1111-111111111111`,
`CARGO_TARGET_DIR=C:/worth-build-cache/target-cut-pub-mono`.

Each full test command appends these exact arguments to
`cargo test --manifest-path workspaces/worth-query/Cargo.toml`:

| Arguments | Passed / failed / ignored |
| --- | --- |
| `-p worth-query-execution --lib` | 2272 / 0 / 1 |
| `-p worth-query-court-decl-host --test worth_query_court_decl_host` | 339 / 0 / 2 |
| `-p worth-query-court-query-decl-host --test worth_query_court_query_decl_host` | 341 / 0 / 2 |

Each full binary ran once. Filtered runs use the decl-host command plus
`transaction_custody -- --test-threads=1`; mutants replace that filter with the
corresponding table suffix.

Gate commands:

- `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml --check -p worth-query-court-decl-host`
- `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --all-targets`
- `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .`
- `cargo run --manifest-path tools/agent-context/Cargo.toml -- check`
- `& "C:/Program Files/Git/bin/bash.exe" scripts/ci/check_workspace_rust_line_caps.sh dirty`
- `git diff --check`, `git diff --cached --check`, `git diff --check origin/master...HEAD`

Fmt, boundary, context, caps (112 Rust files) and all diff checks exited 0.
Boundary/context also passed preflight. Clippy exited 101:
`65 previous errors; 1 warning emitted` for the court target. Its 65 errors
are in 26 files unchanged from `62881e61e5` (58 thread-call errors, 7 other
lints); it emitted 54 warnings overall. Both own Rust files have 0 diagnostics.

The same reviewer's single qa-loop, qa-tests and code-quality-qa pass: PASS, no
findings. `python scripts/quality/scrutinize_rust_functions.py --dirty .` exited
0: 112 files, 80 advisory candidates, 0 scan errors; the sole own fixture
candidate was accepted intact.

## Readings I chose

The latest correction requires fresh cases. Expired authentication and clock
ownership stay queued; the parked branch is untouched. Untouched Clippy debt
is reported under AGENTS.md without exemptions or baseline changes. Stop before
slice 4.
