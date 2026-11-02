# K1c implementation evidence

Slices 1–5 establish the Native content owner, durable reconstruction, typed
Decl/Host door, Source/correction custody and bounded-cost qualification. The
sections below preserve each slice's contemporaneous status and every logged
attempt; an early “pending” statement describes that slice, not the final state.
Final counts, remaining untouched enforcement debt, independent reviewer verdict
and exact qualification scope are in the
[slice 5 closeout](milestone-9.19-k1c-content-slice5-evidence.md).

## Slice 1: Native transactional content authority

This is incremental evidence, not K1c completion. Native owns immutable admitted
frames and transaction/root custody. The public Decl/Host door, durable content
reconstruction, Source byte acquisition and final cost qualification are pending.

### Readings I chose

- Retire unused `AspectValue::Bytes(ContentRefId)` and its scalar/codec arms;
  `InlineBytes` remains the separate landed scalar. The legacy numeric content
  identifier and old wire tag have no compatibility decoder in this cutover.
- `ContentRefId` describes the complete framed SHA-256 identity. It is data, not
  permission. Actual admitted Native roots and staged frames carry authority.
- A field's existing codec digest names the entire field meaning. A distinct
  installed format digest selects the codec; explicit field-to-format aliases
  reuse one frozen installed descriptor. Schema authority binds aliases,
  descriptors and ceilings, including compact positional serialization.
- Slice 1 refuses durable writes and decoded content-root readmission explicitly.
  This includes registries with content formats but no reference fields. Replacing
  a live content root's immutable schema registry also refuses until slice 2 can
  reconstruct and check every reference. Ordinary edits compare actual registry
  identity in constant work and preserve retained bytes.
- Final publication checks the same content closure after consequence writes,
  before indexes or root publication. Original execution causes keep their type.
- New runtime courts belong in the external court; Native private unit tests use
  pure schema/readmission and exact chunk logic. No topology baseline was widened.

### Shape and independent evidence

No `AspectShape` form was added. Content uses the existing ContentRef field shape,
a full 32-byte content identity, and an ASCII format/version frame. Native limits
a frame to 16 MiB and transaction staging to 64 MiB. Above-limit content refuses
at admission. No inline numerical escape or alternate byte store was introduced.

The `worth-court-relational-certification` binary's `content` family covers real
Native writes, two field aliases sharing one installed format, missing/foreign
references, wrong format/version, literal frame integrity, corrupted payload,
noncanonical overflowing dimensions, altered canonical encoding, original work
and cancellation, savepoint custody, concurrent equal admission, independently
funded readmission wrappers, and pre-medium durable refusal. Its top-level
`cost::content` family measures persistent staging growth and cancellation.
The literal 75-byte frame and pinned digest were generated independently with
Python `struct`/`hashlib`; a position-based integer interpreter checks encoded
samples instead of reusing the writer. The owner-local collision falsifier injects
unequal bodies with equal claimed identities into the paid exact-body checker.
Native unit tests bind descriptor/ceiling/alias authority, compact empty/nonempty
snapshot roundtrips, and the conservative reconstruction fence. Their actual
execution counts appear in the ledger below; filters can include existing cases.

Reviewer findings closed so far: full field meaning versus format identity;
quadratic/unfunded staging; linear registry lookup; unfunded wrapper metadata;
mutable codec metadata; missing reconstruction closure; omitted schema authority;
compact tail order; no-reference-field retention; final consequence closure;
mechanical scalar retirement defects; and overflowing fixture dimensions.
The same reviewer approved slice 1 under qa-loop, qa-tests and code-quality-qa,
using AGENTS' untouched-debt exception for the recorded clippy failures.
No enforcement baseline or lint suppression was added. Later slices remain open.

### Run ledger

All cargo commands below ran in this worktree with logs under ignored
`target/k1c-logs`. Query courts use build id
`11111111-1111-1111-1111-111111111111`. A repeated failed run remains visible;
no failed gate is described as passing. `--no-deps` clippy runs are diagnostics
in addition to the required unmodified commands, never a substitute gate.


| Log | Exact command | Exit | Result |
|---|---|---:|---|
| `foundation-check` | `cargo check -p worth-foundational` | 0 | Completed successfully (no test count). |
| `native-check` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `native-check2` | `cargo check -p worth-relational` | 101 | error[E0603]: function `parse` is private; error[E0063]: missing field `content` in initializer of `root::RelationalBranchRoot`; error[E0027]: pattern does not mention field `content`; error[E0027]: pattern does not mention field `content`; error[E0063]: missing field `content` in initializer of `canonical_commit_envelope::CanonicalCommitEnvelope`; error: could not compile `worth-relational` (lib) due to 5 previous errors |
| `native-check3` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `content-court1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::` | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out; finished in 0.00s |
| `content-court2` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::` | 101 | error[E0599]: no method named `root_identity` found for reference `&RelationalBranchIdentity` in the current scope; error[E0599]: no method named `root_identity` found for reference `&RelationalBranchIdentity` in the current scope; error[E0599]: no method named `root_identity` found for reference `&RelationalBranchIdentity` in the current scope; error[E0599]: no method named `root_identity` found for reference `&RelationalBranchIdentity` in the current scope; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 4  |
| `fmt-root1` | `cargo fmt -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `fmt-query1` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query -p worth-query-declaration -p worth-query-execution -p worth-query-installation -p worth-query-package-archive -p worth-query-testkit-query -p worth-query` | 0 | Completed successfully (no test count). |
| `fmt-store1` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-aspect-native` | 0 | Completed successfully (no test count). |
| `content-court3` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::` | 0 | test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out; finished in 0.01s |
| `boundary1` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | Completed successfully (no test count). |
| `context1` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `fmt-root2` | `cargo fmt -p worth-foundational -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `content-court4` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out; finished in 0.14s |
| `foundation-artifacts1` | `cargo test -p worth-court-foundational-certification-boundary-artifacts --test worth_court_foundational_certification_boundary_artifacts` | 101 | error: unexpected end of macro invocation; error: could not compile `worth-court-foundational-certification-boundary-artifacts` (test  |
| `foundation-evidence1` | `cargo test -p worth-court-foundational-certification-boundary-evidence --test worth_court_foundational_certification_boundary_evidence` | 101 | error: unexpected end of macro invocation; error: could not compile `worth-court-foundational-certification-boundary-evidence` (test  |
| `foundation-lib1` | `cargo test -p worth-foundational --lib` | 101 | test result: FAILED. 83 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s |
| `native-value1` | `cargo test -p worth-relational --lib native_value` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2262 filtered out; finished in 0.05s |
| `native-codec1` | `cargo test -p worth-relational --lib aspect_value_canonical_codec_tests` | 0 | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 2256 filtered out; finished in 0.00s |
| `fmt-root3` | `cargo fmt -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 1 | See failure details below. |
| `content-court5` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | error[E0753]: expected outer doc comment; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 1  |
| `native-content1` | `cargo test -p worth-relational --lib content` | 101 | error[E0425]: cannot find type `RelationalRuntime` in this scope; error[E0422]: cannot find struct, variant or union type `DeclaredAspectContractBinding` in this scope; error[E0433]: cannot find type `RelationalRuntimeApi` in this scope; error[E0425]: cannot find type `RelationalRuntime` in this scope; error[E0433]: cannot find type `WorkerIntentBatch` in this scope; error[E0433]: cannot find type `MutationIntent` in this scope; error[E0433]: cannot find type `CreateIntent` in this scope; error[E0425]: cannot find function, tuple struct or tuple variant `BranchId` in this scope; error[E0425]: cannot find function, tuple struct or tuple variant `BranchId` in this scope; error[E0433]: cannot find type `TransactionCommitError` in this scope; error: could not compile `worth-relational` (lib test) due to 10 previous errors; 2 warnings emitted |
| `foundation-artifacts2` | `cargo test -p worth-court-foundational-certification-boundary-artifacts --test worth_court_foundational_certification_boundary_artifacts` | 101 | test result: FAILED. 160 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s |
| `foundation-evidence2` | `cargo test -p worth-court-foundational-certification-boundary-evidence --test worth_court_foundational_certification_boundary_evidence` | 101 | test result: FAILED. 108 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s |
| `foundation-lib2` | `cargo test -p worth-foundational --lib` | 0 | test result: ok. 84 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s |
| `clippy-root1` | `cargo clippy -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification --all-targets` | 101 | error: this `if` statement can be collapsed; error: could not compile `worth-proof` (lib) due to 1 previous error |
| `fmt-query2` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query -p worth-query-declaration -p worth-query-execution -p worth-query-installation -p worth-query-package-archive -p worth-query-testkit-query -p worth-query` | 0 | Completed successfully (no test count). |
| `clippy-query1` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query -p worth-query-declaration -p worth-query-execution -p worth-query-installation -p worth-query-package-archive -p worth-query-testkit-query -p worth-query --all-targets` | 101 | error: unnecessary use of `clone` to create a slice from a reference; error: unnecessary use of `clone` to create a slice from a reference; error: could not compile `worth-query-installation` (lib test) due to 2 previous errors; 2 warnings emitted |
| `fmt-store2` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-aspect-native` | 0 | Completed successfully (no test count). |
| `clippy-store1` | `cargo clippy --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-aspect-native --all-targets` | 0 | Completed successfully (no test count). |
| `fmt-root4` | `cargo fmt -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `content-court6` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out; finished in 0.12s |
| `native-content2` | `cargo test -p worth-relational --lib content` | 101 | error[E0432]: unresolved import `crate::runtime::RelationalRuntimeApi`; error: could not compile `worth-relational` (lib test) due to 1 previous error; 2 warnings emitted |
| `foundation-artifacts3` | `cargo test -p worth-court-foundational-certification-boundary-artifacts --test worth_court_foundational_certification_boundary_artifacts foundational::certification::` | 0 | test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s |
| `foundation-evidence3` | `cargo test -p worth-court-foundational-certification-boundary-evidence --test worth_court_foundational_certification_boundary_evidence foundational::certification::` | 0 | test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.01s |
| `clippy-root-scoped1` | `cargo clippy -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification --all-targets --no-deps` | 101 | error: using `chunks_exact` with a constant chunk size; error: could not compile `worth-foundational` (lib) due to 1 previous error; error: could not compile `worth-foundational` (lib test) due to 1 previous error |
| `query-decl1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration --lib native_predicate_contract_tests` | 0 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 478 filtered out; finished in 0.01s |
| `fmt-root5` | `cargo fmt -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 1 | See failure details below. |
| `query-identity1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query --lib native_value_identity_basis` | 101 | error[E0599]: no variant, associated function, or constant named `Bytes` found for enum `ApplicationValue` in the ; error[E0599]: no variant, associated function, or constant named `Bytes` found for enum ; error: could not compile `worth-query` (lib test) due to 2 previous errors |
| `native-content3` | `cargo test -p worth-relational --lib content` | 101 | error[E0583]: file not found for module `registry_rebinding`; error[E0616]: field `content` of struct `root::RelationalBranchRoot` is private; error: could not compile `worth-relational` (lib) due to 2 previous errors; 1 warning emitted |
| `query-hash-guard1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-installation --lib canonical_basis_residue` | 0 | test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 359 filtered out; finished in 0.89s |
| `content-court7` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | error[E0583]: file not found for module `registry_rebinding`; error[E0616]: field `content` of struct `root::RelationalBranchRoot` is private; error: could not compile `worth-relational` (lib) due to 2 previous errors; 1 warning emitted |
| `query-archive1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-package-archive --lib` | 0 | test result: ok. 179 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s |
| `clippy-root-scoped2` | `cargo clippy -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification --all-targets --no-deps` | 101 | error[E0583]: file not found for module `registry_rebinding`; error[E0616]: field `content` of struct `branch::root::RelationalBranchRoot` is private; error: could not compile `worth-relational` (lib) due to 2 previous errors; 1 warning emitted |
| `query-mutations1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query --test worth_query_court_query native_aspect_mutation_public_dx` | 101 | error[E0583]: file not found for module `registry_rebinding`; error[E0616]: field `content` of struct `root::RelationalBranchRoot` is private; error: could not compile `worth-relational` (lib) due to 2 previous errors; 1 warning emitted |
| `store-values1` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-aspect-native --lib` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| `boundary2` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 1 | error: process didn't exit successfully: `C:/worth-build-cache/target-cut-k1c-content\debug\boundary-check.exe --root  |
| `context2` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `fmt-root6` | `cargo fmt -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `fmt-query3` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query -p worth-query-declaration -p worth-query-execution -p worth-query-installation -p worth-query-package-archive -p worth-query-testkit-query -p worth-query-court-query` | 0 | Completed successfully (no test count). |
| `linecap3` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 112 Rust files |
| `diff-working1` | `git diff --check` | 0 | Completed successfully (no test count). |
| `diff-master1` | `git diff --check origin/master...HEAD` | 0 | Completed successfully (no test count). |
| `fmt-court1` | `cargo fmt -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `native-content4` | `cargo test -p worth-relational --lib content` | 0 | test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 2235 filtered out; finished in 1.71s |
| `fmt-root-check1` | `cargo fmt --check -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `content-court8` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content -- --nocapture` | 0 | test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out; finished in 0.10s |
| `fmt-query-check1` | `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query -p worth-query-declaration -p worth-query-execution -p worth-query-installation -p worth-query-package-archive -p worth-query-testkit-query -p worth-query-court-query` | 0 | Completed successfully (no test count). |
| `fmt-store-check1` | `cargo fmt --check --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-aspect-native` | 0 | Completed successfully (no test count). |
| `clippy-root-scoped3` | `cargo clippy -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification --all-targets --no-deps` | 101 | error: this `if` statement can be collapsed; error: this expression creates a reference which is immediately dereferenced by the compiler; error: mutable key type; error: this `if` statement can be collapsed; error: the following explicit lifetimes could be elided: 'a; error: the following explicit lifetimes could be elided: 'a; error: use of a disallowed method `std::ptr::from_ref`; error: use of a disallowed method `std::ptr::from_ref`; error: use of a disallowed method `std::sync::Arc::as_ptr`; error: could not compile `worth-relational` (lib) due to 9 previous errors; 7 warnings emitted |
| `fmt-root7` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `query-identity2` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query --lib native_value_identity_basis` | 101 | test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 2485 filtered out; finished in 0.03s |
| `content-court9` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content -- --nocapture` | 0 | test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out; finished in 0.07s |
| `boundary3` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | Completed successfully (no test count). |
| `fmt-root-check2` | `cargo fmt --check -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `query-mutations2` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-query --test worth_query_court_query native_aspect_mutation_public_dx` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 61 filtered out; finished in 0.04s |
| `clippy-root2` | `cargo clippy -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification --all-targets` | 101 | error: this `if` statement can be collapsed; error: could not compile `worth-proof` (lib) due to 1 previous error |
| `boundary4` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | Completed successfully (no test count). |
| `clippy-query2` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query -p worth-query-declaration -p worth-query-execution -p worth-query-installation -p worth-query-package-archive -p worth-query-testkit-query -p worth-query-court-query --all-targets` | 101 | error: unnecessary use of `clone` to create a slice from a reference; error: unnecessary use of `clone` to create a slice from a reference; error: could not compile `worth-query-installation` (lib test) due to 2 previous errors; 2 warnings emitted |
| `context3` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `query-identity3` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query --lib native_value_identity_basis` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2485 filtered out; finished in 0.02s |
| `clippy-root-courts1` | `cargo clippy -p worth-foundational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification --all-targets --no-deps` | 101 | error: unnecessary `>= y + 1` or `x - 1 >=`; error: unnecessary `>= y + 1` or `x - 1 >=`; error: the borrowed expression implements the required traits; error: this expression creates a reference which is immediately dereferenced by the compiler; error: unnecessary closure used with `bool::then`; error: called `.err().expect()` on a `Result` value; error: called `.err().expect()` on a `Result` value; error: this `if` statement can be collapsed; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 8  |
| `fmt-query-check2` | `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query -p worth-query-declaration -p worth-query-execution -p worth-query-installation -p worth-query-package-archive -p worth-query-testkit-query -p worth-query-court-query` | 0 | Completed successfully (no test count). |
| `clippy-court2` | `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | 101 | error: unnecessary `>= y + 1` or `x - 1 >=`; error: unnecessary `>= y + 1` or `x - 1 >=`; error: unnecessary closure used with `bool::then`; error: called `.err().expect()` on a `Result` value; error: called `.err().expect()` on a `Result` value; error: this `if` statement can be collapsed; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 6  |
| `content-court10` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content -- --nocapture` | 0 | test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 117 filtered out; finished in 0.07s |
| `fmt-root-check3` | `cargo fmt --check -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 1 | See failure details below. |
| `linecap4` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 113 Rust files |
| `fmt-root-check4` | `cargo fmt --check -p worth-foundational -p worth-relational -p worth-runtime-bridge -p worth-court-foundational-certification-boundary-artifacts -p worth-court-foundational-certification-boundary-evidence -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `diff-working2` | `git diff --check` | 0 | Completed successfully (no test count). |
| `diff-master2` | `git diff --check origin/master...HEAD` | 0 | Completed successfully (no test count). |
| `diff-cached1` | `git diff --cached --check` | 0 | Completed successfully (no test count). |
| Earlier manual line-cap check | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 95 Rust files. |
| `line-caps2` (manual) | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 1 | Two failures: preparation.rs and root_schema.rs each 403 lines / cap 400. Both responsibilities split; final scope passes. |

### Failure disposition

Required root clippy reports untouched `worth-proof/src/managed_access/coverage.rs:84`
(`collapsible_if`). Required Query clippy reports two untouched Installation
`application_query/inspection/tests.rs` errors at lines 51 and 103
(`cloned_ref_to_slice_refs`). The additional Native clippy diagnostic reports
nine errors in untouched aggregate-identity funding, publication preparation,
derived-relation, live-managed-source-history, companion public-cache and Source
program files. The court diagnostic also reports six untouched court errors:
`int_plus_one` twice, `unnecessary_lazy_evaluations` once, `err_expect` twice
and `collapsible_if` once. Two introduced needless-borrow errors were fixed;
the final court diagnostic has only those six pre-existing errors.
These remain repository debt under AGENTS; no lint was suppressed.

An early full Foundation artifacts run also reached an unrelated compile-boundary
harness refusal, `MissingPrebuild`. Subsequent focused certification runs do not
claim that compile-boundary harness passed. Introduced compiler, golden, scalar
family-count, header/import and boundary-layout failures were corrected and rerun.

`fmt-root3` reported an inner module doc comment after a module declaration;
`fmt-root5` could not resolve the newly split registry_rebinding module path.
Both were corrected; final root format check4 exits 0.

`boundary2` emitted BC9002_TEST_TOPOLOGY for nested content/cost.rs and
runtime-root source tests in root_content_readmission_tests.rs and
frame/collision_tests.rs. Courts were moved to the permitted topology; the
private collision and schema-fence tests now construct no runtime roots.
Final boundary runs report `boundary-check: repository boundaries are valid`.

`query-identity2` emitted `left: 24; right: 25` after Bytes retirement;
the three family counts now correctly expect 24 and the rerun passes 3/3.
`fmt-root-check3` showed one line-wrap diff after the needless-borrow fix;
the line was formatted and `fmt-root-check4` passed on the final tree.

Performance observed in the `cost::content` staging court: staging 256 frames
charged 7,337,824 Work; 512 charged 14,724,704 Work. Counts include paid directory
and format lookup bounds. Wall time is telemetry only. No new generic engine
was added to worth-query-execution in slice 1; that crate only retires the unused
arm and updates descriptive scalar samples. No rlib growth measurement is claimed.

## Sync before slice 2

Fetched both named sync branches. Incoming master is
`9d3e872a301acf6877fcd19f60bb8953a11fdef8` (60 commits since the prior
sync); the provider branch has no commits beyond that master. Three Native
conflicts required integration. The complete semantic plan and original Source
interpretation checks remain upstream-owned; both ordinary and retained-selection
application now reach the same content closure and temporary durability guard.
The publication helper keeps the incoming semantic authority kind and command
projection. A verbatim envelope-constructor extraction keeps its combined file
under 400 lines. No generic engine was added.

These checks cover the conflict-resolution owners, plus their focused Native
courts; incoming unchanged owners are not claimed as newly certified by this lane.

| Log | Exact command | Exit | Result |
|---|---|---:|---|
| `sync2-fmt` | `cargo fmt -p worth-relational` | 1 | error: expected one of `:` or `\|`, found `runtime`; error: expected identifier, found keyword `pub`; error: expected one of `:` or `\|`, found `input` |
| `sync2-linecap` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 1 | FAIL: crates/worth-relational/src/history/data/canonical_commit_envelope.rs is 403 lines (cap 400) |
| `sync2-check` | `cargo check -p worth-relational` | 101 | error: expected identifier, found keyword `pub`; error: expected one of `:` or `\|`, found `runtime`; error: expected identifier, found keyword `pub`; error: expected one of `:` or `\|`, found `input`; error[E0425]: cannot find value `runtime` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `runtime` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `runtime` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error: could not compile `worth-relational` (lib) due to 31 previous errors |
| `sync2-content` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content -- --nocapture` | 101 | error: expected identifier, found keyword `pub`; error: expected one of `:` or `\|`, found `runtime`; error: expected identifier, found keyword `pub`; error: expected one of `:` or `\|`, found `input`; error[E0425]: cannot find value `runtime` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `runtime` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `runtime` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error[E0425]: cannot find value `input` in this scope; error: could not compile `worth-relational` (lib) due to 31 previous errors |
| `sync2-selection` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification retained_entity_selection` | 0 | test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 158 filtered out; finished in 0.10s |
| `sync2-clippy` | `cargo clippy -p worth-relational --all-targets` | 101 | error: this `if` statement can be collapsed; error: could not compile `worth-proof` (lib) due to 1 previous error |
| `sync2-linecap2` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 892 Rust files |
| `sync2-fmt-check` | `cargo fmt --check -p worth-relational` | 1 | See correction below. |
| `sync2-boundary` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | Completed successfully (no test count). |
| `sync2-context` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `sync2-fmt2` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `sync2-check2` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `sync2-content2` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content -- --nocapture` | 0 | test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.06s |
| `sync2-fmt-check2` | `cargo fmt --check -p worth-relational` | 0 | Completed successfully (no test count). |
| `sync2-clippy2` | `cargo clippy -p worth-relational --all-targets` | 101 | error: this `if` statement can be collapsed; error: could not compile `worth-proof` (lib) due to 1 previous error |
| `sync2-boundary2` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | Completed successfully (no test count). |
| `sync2-context2` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `sync2-diff-master` | `git diff --check origin/master...HEAD` | 0 | Completed successfully (no test count). |
| `sync2-diff-cached` | `git diff --cached --check` | 2 | See correction below. |
| `sync2-diff-cached2` | `git diff --cached --check` | 0 | Completed successfully (no test count). |

The first extraction incorrectly applied visibility to two function parameters;
the formatter reported parse errors and compilation emitted 31 diagnostics from
that cause. Corrected parameters compile and both court filters pass. Initial
line-cap failure: `canonical_commit_envelope.rs is 403 lines (cap 400)`; moving its
constructor verbatim fixes it. The first format check and cached diff caught
unfinished formatting / a trailing blank line; final reruns pass. Required clippy
still fails untouched Proof coverage.rs:84 (`collapsible_if`). No lint or
enforcement suppression was added. The same reviewer approved this merge
integration under all three review skills and the AGENTS untouched-debt
exception; it is not slice 2 completion.

## Slice 2 durable content evidence ledger

The original build identity was `11111111-1111-1111-1111-111111111111`.
Commands ran as hidden background processes with output retained under
`target/k1c-logs`; each process wrote its exit before the log was inspected.
No workspace-wide build or test ran. Rows include unsuccessful development
runs, not only final gates. Filtered counts describe the actual executed binary.
The affected packages are Native, its certification court, shared Native testkit,
and the Store relational court. No Query execution generic engine changed.

| Log | Exact command | Exit | Result |
|---|---|---:|---|
| `slice2-fmt1` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-check1` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-fmt2` | `cargo fmt -p worth-relational` | 1 | See failure excerpts and correction notes below. |
| `slice2-check2` | `cargo check -p worth-relational` | 101 | error[E0583]: file not found for module `seed`; error[E0583]: file not found for module `positioned_seed`; error[E0432]: unresolved import `crate::history::data::PositionedEnvelopeSeed`; error[E0599]: no method named `position` found for struct `rmp_serde::Deserializer<ReadRefReader<'_, [u8]>>` in the ; error[E0004]: non-exhaustive patterns: `Err(recovery_stop::RelationalRecoveryStop::Content(_))` not covered; error[E0004]: non-exhaustive patterns: `recovery_stop::RelationalRecoveryStop::Content(_)` not covered; error: could not compile `worth-relational` (lib) due to 11 previous errors; 1 warning emitted |
| `slice2-fmt3` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-check3` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-fmt4` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-check4` | `cargo check -p worth-relational` | 101 | error[E0624]: associated function `capture` is private; error[E0624]: method `readmit` is private; error: could not compile `worth-relational` (lib) due to 2 previous errors |
| `slice2-fmt5` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-check5` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-fmt6` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-check6` | `cargo check -p worth-relational` | 101 | error[E0599]: no method named `version_id` found for struct `root::RelationalBranchRoot` in the current scope; error: could not compile `worth-relational` (lib) due to 1 previous error |
| `slice2-fmt7` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-check7` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-content1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content -- --nocapture` | 0 | test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.06s |
| `slice2-fmt8` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-check8` | `cargo check -p worth-relational` | 101 | error[E0425]: cannot find type `RelationalPersistentRegionSet` in module `super`; error[E0425]: cannot find type `RelationalBranchRootPublicationCost` in module `super`; error[E0425]: cannot find type `RelationalBranchRootSchemaAuthority` in module `super`; error[E0433]: cannot find `root_checkpoint` in `super`; error[E0433]: cannot find `RelationalRootRegion` in `super`; error[E0433]: cannot find `RelationalPersistentRegionSet` in `super`; error[E0433]: cannot find type `ExecutionByteBuffer` in this scope; error[E0599]: no method named `bytes` found for struct `worth_foundational::ContentRefId` in the current scope; error: could not compile `worth-relational` (lib) due to 10 previous errors; 1 warning emitted |
| `slice2-fmt9` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-check9` | `cargo check -p worth-relational` | 101 | error[E0425]: cannot find type `RelationalBranchRootPublicationCost` in module `super`; error[E0433]: cannot find `RelationalRootRegion` in `branch`; error: could not compile `worth-relational` (lib) due to 2 previous errors |
| `slice2-fmt10` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice2-recovery1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::recovery -- --nocapture` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 171 filtered out; finished in 0.03s |
| `slice2-fmt11` | `cargo fmt -p worth-relational` | 1 | See failure excerpts and correction notes below. |
| `slice2-check11` | `cargo check -p worth-relational` | 101 | error[E0583]: file not found for module `denial`; error[E0308]: `?` operator has incompatible types; error[E0599]: `!` is not an iterator; error: could not compile `worth-relational` (lib) due to 3 previous errors; 3 warnings emitted |
| `slice2-fmt12` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-check12` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-fmt13` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice2-recovery2` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::recovery -- --nocapture` | 101 | test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 171 filtered out; finished in 0.05s; error: test failed, to rerun pass `-p worth-court-relational-certification --test worth_court_relational_certification` |
| `slice2-native1` | `cargo test -p worth-relational --lib content -- --nocapture` | 101 | error[E0061]: this method takes 3 arguments but 2 arguments were supplied; error[E0061]: this method takes 2 arguments but 1 argument was supplied; error[E0277]: a value of type `BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>` cannot be built from an iterator ; error[E0277]: the trait bound `ExecutionImmutableBytes: From<Vec<u8>>` is not satisfied; error[E0308]: mismatched types; error[E0308]: `match` arms have incompatible types; error[E0004]: non-exhaustive patterns: `recovery_stop::RelationalRecoveryStop::Content(_)` not covered; error: could not compile `worth-relational` (lib test) due to 18 previous errors; 2 warnings emitted |
| `slice2-fmt14` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice2-recovery3` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::recovery -- --nocapture` | 101 | test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 171 filtered out; finished in 0.04s; error: test failed, to rerun pass `-p worth-court-relational-certification --test worth_court_relational_certification` |
| `slice2-native2` | `cargo test -p worth-relational --lib content -- --nocapture` | 101 | error[E0277]: a value of type `BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>` cannot be built from an iterator ; error: could not compile `worth-relational` (lib test) due to 1 previous error; 2 warnings emitted |
| `slice2-fmt15` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice2-recovery4` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.08s |
| `slice2-native3` | `cargo test -p worth-relational --lib content -- --nocapture` | 0 | test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2235 filtered out; finished in 1.44s |
| `slice2-fmt16` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice2-content2` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.09s |
| `slice2-artifact1` | `cargo test -p worth-relational --lib canonical_commit_artifact -- --nocapture` | 0 | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2262 filtered out; finished in 0.04s |
| `slice2-fmt17` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt1` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-store1` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 101 | error[E0277]: the trait bound `RelationalExecutionDenialCause: From<ExecutionAllocationDenial>` is not satisfied; error: could not compile `worth-relational` (lib) due to 1 previous error |
| `slice2-fmt18` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt2` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-store2` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 101 | error[E0659]: `EntityId` is ambiguous; error[E0659]: `PartitionId` is ambiguous; error[E0308]: mismatched types; error: could not compile `worth-store-court-relational` (test "worth_store_court_relational") due to 6 previous errors |
| `slice2-fmt19` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt3` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-store3` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 101 | test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 58 filtered out; finished in 2.66s; error: test failed, to rerun pass `-p worth-store-court-relational --test worth_store_court_relational` |
| `slice2-fmt20` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content3` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.53s |
| `slice2-native4` | `cargo test -p worth-relational --lib content -- --nocapture` | 101 | error[E0433]: cannot find `RelationalRuntimeApi` in `runtime`; error: could not compile `worth-relational` (lib test) due to 1 previous error; 3 warnings emitted |
| `slice2-fmt21` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt4` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-store4` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 101 | test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 58 filtered out; finished in 2.18s; error: test failed, to rerun pass `-p worth-store-court-relational --test worth_store_court_relational` |
| `slice2-fmt22` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content4` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.56s |
| `slice2-native5` | `cargo test -p worth-relational --lib content -- --nocapture` | 0 | test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 2235 filtered out; finished in 1.22s |
| `slice2-fmt23` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt5` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-store5` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 101 | test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 58 filtered out; finished in 2.10s; error: test failed, to rerun pass `-p worth-store-court-relational --test worth_store_court_relational` |
| `slice2-fmt-check1` | `cargo fmt --check -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt-check1` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml --check -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-clippy1` | `cargo clippy -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational --all-targets` | 101 | error: this `if` statement can be collapsed; error: could not compile `worth-proof` (lib) due to 1 previous error |
| `slice2-storeclippy1` | `cargo clippy --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --all-targets` | 101 | error: using `chunks_exact` with a constant chunk size; error: could not compile `worth-store-court-relational` (test "worth_store_court_relational") due to 1 previous error |
| `slice2-boundary1` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 1 | error: process didn't exit successfully: `C:/worth-build-cache/target-cut-k1c-content\debug\boundary-check.exe --root ; BC9002_TEST_TOPOLOGY crates/worth-relational/src/durability/checkpoints/content/tests.rs::file: {"source":"crates/worth; BC9002_TEST_TOPOLOGY crates/worth-relational/src/history/data/canonical_commit_envelope/tests.rs::file: {"source":"crat |
| `slice2-context1` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `slice2-caps1` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 122 Rust files |
| `slice2-diff-head1` | `git diff --check origin/master...HEAD` | 0 | Completed successfully (no test count). |
| `slice2-diff-dirty1` | `git diff --check` | 0 | Completed successfully (no test count). |
| `slice2-fmt24` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content5` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | error[E0277]: `?` couldn't convert the error to `RelationalContentDenial`; error: could not compile `worth-relational` (lib) due to 1 previous error |
| `slice2-native6` | `cargo test -p worth-relational --lib content -- --nocapture` | 101 | error[E0277]: `?` couldn't convert the error to `RelationalContentDenial`; error: could not compile `worth-relational` (lib) due to 1 previous error |
| `slice2-fmt25` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content6` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | error[E0624]: method `publish_prepared_candidate` is private; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 2  |
| `slice2-native7` | `cargo test -p worth-relational --lib content -- --nocapture` | 101 | error[E0308]: mismatched types; error: could not compile `worth-relational` (lib test) due to 1 previous error; 3 warnings emitted |
| `slice2-fmt26` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content7` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | test result: FAILED. 37 passed; 1 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.62s; error: test failed, to rerun pass `-p worth-court-relational-certification --test worth_court_relational_certification` |
| `slice2-native8` | `cargo test -p worth-relational --lib content -- --nocapture` | 0 | test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2235 filtered out; finished in 1.18s |
| `slice2-fmt27` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content8` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | test result: FAILED. 38 passed; 1 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.57s; error: test failed, to rerun pass `-p worth-court-relational-certification --test worth_court_relational_certification` |
| `slice2-native9` | `cargo test -p worth-relational --lib content -- --nocapture` | 0 | test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2235 filtered out; finished in 2.88s |
| `slice2-fmt28` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt6` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-store6` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 101 | error[E0277]: the trait bound `[u8; 8]: TryFrom<&[u8; 8]>` is not satisfied; error: could not compile `worth-store-court-relational` (test "worth_store_court_relational") due to 1 previous error |
| `slice2-fmt29` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-work1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content_durable_artifact_original_work_exhaustion_is_exact_and_pre_effect -- --nocapture` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 187 filtered out; finished in 0.07s |
| `slice2-boundary2` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | boundary-check: repository boundaries are valid |
| `slice2-context2` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `slice2-caps2` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 128 Rust files |
| `slice2-fmt30` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-capture-check1` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-fmt31` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content9` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | test result: FAILED. 41 passed; 1 failed; 0 ignored; 0 measured; 149 filtered out; finished in 1.73s; error: test failed, to rerun pass `-p worth-court-relational-certification --test worth_court_relational_certification` |
| `slice2-fmt32` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content10` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 1.00s |
| `slice2-fmt33` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-capture-check2` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice2-content11` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 1.03s |
| `slice2-native10` | `cargo test -p worth-relational --lib content -- --nocapture` | 0 | test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2235 filtered out; finished in 1.15s |
| `slice2-store7` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 9.14s |
| `slice2-fmt34` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt7` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-store8` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 5.99s |
| `slice2-store-cold1` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational residency_capture -- --nocapture` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 63 filtered out; finished in 23.13s |
| `slice2-fmt-check2` | `cargo fmt --check -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-storefmt-check2` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml --check -p worth-store-court-relational` | 1 | See failure excerpts and correction notes below. |
| `slice2-clippy2` | `cargo clippy -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational --all-targets` | 101 | error: this `if` statement can be collapsed; error: could not compile `worth-proof` (lib) due to 1 previous error |
| `slice2-storeclippy2` | `cargo clippy --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --all-targets` | 0 | Completed successfully (no test count). |
| `slice2-boundary3` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | boundary-check: repository boundaries are valid |
| `slice2-context3` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `slice2-caps3` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 135 Rust files |
| `slice2-diff-head2` | `git diff --check origin/master...HEAD` | 0 | Completed successfully (no test count). |
| `slice2-diff-dirty2` | `git diff --check` | 2 | 353 whitespace diagnostics: plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:1: trailing whitespace.; plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:2: trailing whitespace.; plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:3: trailing whitespace.. Quoted historical ledger outcomes are not new test runs. |
| `slice2-storefmt8` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-store9` | `cargo test --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --test worth_store_court_relational content:: -- --nocapture` | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 6.00s |
| `slice2-storefmt-check3` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml --check -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-storeclippy3` | `cargo clippy --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --all-targets` | 0 | Completed successfully (no test count). |
| `slice2-clippy-own1` | `cargo clippy -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational --all-targets --no-deps` | 101 | error: this `if` statement can be collapsed; error: this expression creates a reference which is immediately dereferenced by the compiler; error: unnecessary `if let` since only the `Some` variant of the iterator element is used; error: mutable key type; error: useless conversion to the same type: `durability::data::recovery_stop::RelationalRecoveryStop`; error: usage of `contains_key` followed by `insert` on a `BTreeMap`; error: the following explicit lifetimes could be elided: 'a; error: match can be simplified with `.unwrap_or_default()`; error: use of a disallowed method `std::ptr::from_ref`; error: returning the result of a `let` binding from a block; error: useless conversion to the same type: ; error: use of a disallowed method `std::sync::Arc::as_ptr`; error: could not compile `worth-relational` (lib) due to 18 previous errors; 8 warnings emitted |
| `slice2-boundary4` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | boundary-check: repository boundaries are valid |
| `slice2-context4` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `slice2-caps4` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 136 Rust files |
| `slice2-fmt35` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-content12` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.98s |
| `slice2-fmt-check3` | `cargo fmt --check -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-fmt36` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-clippy-own2` | `cargo clippy -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational --all-targets --no-deps` | 101 | error[E0308]: mismatched types; error: could not compile `worth-relational` (lib) due to 2 previous errors |
| `slice2-content13` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 101 | error[E0308]: mismatched types; error: could not compile `worth-relational` (lib) due to 2 previous errors |
| `slice2-fmt37` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-clippy-native-own3` | `cargo clippy -p worth-relational --all-targets --no-deps` | 101 | error: this `if` statement can be collapsed; error: this expression creates a reference which is immediately dereferenced by the compiler; error: mutable key type; error: the following explicit lifetimes could be elided: 'a; error: unused "map closure" when calling `Option::map_or_else` value; error: use of a disallowed method `std::ptr::from_ref`; error: returning the result of a `let` binding from a block; error: use of a disallowed method `std::sync::Arc::as_ptr`; error: could not compile `worth-relational` (lib) due to 11 previous errors; 8 warnings emitted |
| `slice2-clippy-court-own1` | `cargo clippy -p worth-court-relational-certification -p worth-testkit-relational --all-targets --no-deps` | 101 | error: unnecessary `>= y + 1` or `x - 1 >=`; error: unnecessary closure used with `bool::then`; error: called `.err().expect()` on a `Result` value; error: this `if` statement can be collapsed; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 6  |
| `slice2-content14` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 0.99s |
| `slice2-fmt-check4` | `cargo fmt --check -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-clippy3` | `cargo clippy -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational --all-targets` | 101 | error: this `if` statement can be collapsed; error: could not compile `worth-proof` (lib) due to 1 previous error |
| `slice2-storefmt-check4` | `cargo fmt --manifest-path workspaces/worth-store/Cargo.toml --check -p worth-store-court-relational` | 0 | Completed successfully (no test count). |
| `slice2-storeclippy4` | `cargo clippy --manifest-path workspaces/worth-store/Cargo.toml -p worth-store-court-relational --all-targets` | 0 | Completed successfully (no test count). |
| `slice2-boundary5` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | boundary-check: repository boundaries are valid |
| `slice2-context5` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `slice2-caps5` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 137 Rust files |
| `slice2-diff-head3` | `git diff --check origin/master...HEAD` | 0 | Completed successfully (no test count). |
| `slice2-diff-dirty3` | `git diff --check` | 2 | 353 whitespace diagnostics: plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:1: trailing whitespace.; plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:2: trailing whitespace.; plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:3: trailing whitespace.. Quoted historical ledger outcomes are not new test runs. |
| `slice2-fmt38` | `cargo fmt -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-clippy-native-own4` | `cargo clippy -p worth-relational --all-targets --no-deps` | 101 | error: this `if` statement can be collapsed; error: this expression creates a reference which is immediately dereferenced by the compiler; error: mutable key type; error: the following explicit lifetimes could be elided: 'a; error: use of a disallowed method `std::ptr::from_ref`; error: returning the result of a `let` binding from a block; error: use of a disallowed method `std::sync::Arc::as_ptr`; error: could not compile `worth-relational` (lib) due to 10 previous errors; 8 warnings emitted |
| `slice2-history-tail1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content_controlled_journal_seed -- --nocapture` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 194 filtered out; finished in 0.02s |
| `slice2-fmt-check5` | `cargo fmt --check -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational` | 0 | Completed successfully (no test count). |
| `slice2-clippy4` | `cargo clippy -p worth-relational -p worth-court-relational-certification -p worth-testkit-relational --all-targets` | 101 | error: this `if` statement can be collapsed; error: could not compile `worth-proof` (lib) due to 1 previous error |
| `slice2-diff-dirty4` | `git diff --check` | 0 | Completed successfully (no test count). |
| `slice2-boundary6` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | boundary-check: repository boundaries are valid |
| `slice2-context6` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `slice2-caps6` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 137 Rust files |
| `slice2-diff-dirty5` | `git diff --check` | 0 | Completed successfully (no test count). |
| `slice2-diff-dirty6` | `git diff --check` | 2 | 28 whitespace diagnostics: plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:506: trailing whitespace.; plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:513: trailing whitespace.; plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:514: trailing whitespace.. Quoted historical ledger outcomes are not new test runs. |
| `slice2-diff-dirty7` | `git diff --check` | 2 | 1 whitespace diagnostics: plans/WORTH-query/milestone-9.19-k1c-content-evidence.md:875: new blank line at EOF.. Quoted historical ledger outcomes are not new test runs. |
| `slice2-diff-dirty8` | `git diff --check` | 0 | Completed successfully (no test count). |

### Slice 2 result and independent courts

Native owns the complete actual reference closure through ordinary edits,
post-consequence publication, current schema replacement, checkpoint ingress,
journal recovery, retained selection and canonical relinking. Changed records use
paid ordered associations; reconstruction is an explicit whole-root traversal.
No Store dependency, independent byte store, untracked cache, new AspectShape form,
or generic Query execution engine was added. Original SDK backing now also funds
canonical artifact construction, captured envelopes, whole embedded checkpoints,
and controlled cold-head/carried-journal reads. The cold journal port exposes no
length quotation, so it reserves min(caller limit, 256 MiB) before the physical
read; the sealed actual copy then retains independent funding. This temporary is
released before canonical decoding. Physical members use their certified exact
locator length. The tiny cold-ingress fixture uses a 512 MiB receiving budget;
that is not a universal bound on simultaneous artifact, frame and decoded state.

- Native literal-frame and independently repaired-outer-digest courts prove actual
  content integrity, not merely outer checksum rejection. The grid oracle uses
  explicit dimensions, signed integer positions, literal frame bytes and a pinned
  independently computed digest; it does not call the production encoder/parser.
- Journal and Native checkpoint positives read the intended selected entity's
  left/right fields before checking the admitted literal bytes. Checkpoint
  omission and decoded-journal omission preserve actual references while removing
  their retained content; both refuse MissingContent before selected state changes.
- The genuine historical basis supplies retained selection; its restored reference
  carries exactly the historical bytes and removes unreachable current bytes.
  The restored checkpoint repeats the selected-field and byte checks. Raw retained
  selection journal replay retains its explicit unsupported profile and target.
- Rebinding a smaller frozen ceiling refuses, and a replaced rejecting validator
  is actually executed despite equal portable metadata. Current and historical
  schema contracts both participate in the root absence proof.
- Original-memory alias courts distinguish actual member and whole-checkpoint
  custody. Last aliases release their precise backing. Real publication/capture
  cancellation after funded allocation releases the original request's custody;
  memory/work refusal occurs before medium invocation and leaves selected state.
- Real Store homes prove content-bearing tail recovery, checkpoint replacement
  while an old frame reader retains custody, exact selected-field/numeric values,
  missing physical-member refusal and physical checksum corruption before service.
  A page-level corruption can reach Store's journal checksum first; the court
  asserts that precise earlier typed refusal rather than claiming Native parsed it.
- Store proven-no-effect matches its exact fate, preserves the selected descriptor,
  old field and bytes, and leaves new content absent. A calibrated real positioned
  write loses its answer after effect: the old selected field remains live, later
  writes fence, and compatible reopen reconstructs the actual new field/value.
  Wrong installed version refuses explicitly; a subsequent compatible reopen
  proves the refusal did not erase content. The independent literal integer oracle
  supplies the result, rather than directory reconstruction checking itself.
- Genuine cold carried export preserves unsupported-version and size refusals
  across retries; proven digest damage becomes sticky. The 256 MiB temporary must
  be funded before the physical port is invoked. Valid cold export reconstructs
  through the same existing checkpoint grammar. The focused pre-existing Store
  residency-capture court remains a separate regression on the same boundary.

### Review closure and readings chosen

The same independent reviewer read production and courts using qa-loop, qa-tests
and code-quality-qa. It found: loss of original funding in whole artifact and
checkpoint encoding; cold physical-read allocation before funding; temporary
physical bytes surviving decoding unnecessarily; codec-handle/ceiling changes
bypassing revalidation; historical contracts missing from the absence shortcut;
Store fate/unsupported-format evidence missing; selected-field result oracles
missing from directory-only recovery assertions; and non-content scalar aspects
and empty kinds traversed without charging. Each path was corrected in its owner.
The review did not authorize widening a source-court baseline or bypassing limits.

Canonical omitted authored_index_writes is exactly Unavailable in the current
producer grammar. It is descriptive absence, not complete-empty history or a new
Source issuance. The seed names that case through the existing canonical history
owner's unavailable constructor. The independently edited public artifact court
proves canonical admission of that exact omission. No unwrap_or guess decides it.

The existing raw retained-selection journal profile cannot reconstruct live
historical custody; it remains a precise unsupported refusal. The positive path
uses actual retained Native selection and Native checkpoint reconstruction. Cold
physical read limits are conservative certified bounds, never inferred from the
current field's small value. No new generic Query engine was introduced, so no
rlib growth measurement is claimed for this slice.

### Failed-run output and corrections

All unsuccessful runs remain in the ledger. Mechanical compiler failures included
controlled-seed module paths and visibility, new original-request parameters,
immutable SDK byte types, typed recovery exhaustiveness, and two necessary String
conversions accidentally removed during a scoped lint fix. The compiler refused
these; the final focused content court runs use the corrected code.

The two added private source-court profiles tripped BC9002_TEST_TOPOLOGY. They were
removed and their assertions moved to genuine public certification boundaries;
no baseline or caller permission was expanded. Store's chunks_exact_to_as_chunks
lint was corrected, including the subsequently exposed fixed-array conversion.
The scoped no-deps lint pass exposed eight own findings; chained conditions,
Option iteration, map entry, redundant conversions and the canonical omission
constructor closed them. Existing untouched lint failures are reported as debt
under AGENTS.md; this lane does not suppress or expand into them.

#### Recorded failure excerpts

`slice2-recovery2`:

```text
(66316) panicked at
crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\recovery.rs:282:5:
assertion failed: !artifact.as_bytes().windows(b"authored_index_writes".len()).any(|bytes|
            bytes == b"authored_index_writes")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test content::recovery::content_retained_selection_persists_restored_historical_bytes_and_unavailable_history_metadata ... FAILED

thread 'content::recovery::content_checkpoint_member_payload_keeps_original_serial_custody_until_last_clone' (31476)
panicked at

panicked at
crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\recovery.rs:172:5:
assertion failed: matches!(runtime.durability_authority().native_checkpoint(&denied,
    denied.allocation_policy()),
    Err(RelationalNativeCheckpointCaptureDenial::Request(worth_relational::facade::transactions::RelationalExecutionDen
ialCause::PolicyMemoryExhausted
    { requested: 75, admitted: 74, .. })))
test content::recovery::content_checkpoint_member_payload_keeps_original_serial_custody_until_last_clone ... FAILED
test content::recovery::content_native_checkpoint_reconstructs_bytes_under_installed_format_authority ... ok
```

`slice2-recovery3`:

```text
panicked at
crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\recovery.rs:176:5:
Request(PolicyMemoryExhausted { requested: 392, admitted: 74, ancestor: 0 })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test content::recovery::content_checkpoint_member_payload_keeps_original_serial_custody_until_last_clone ... FAILED
test content::recovery::content_native_checkpoint_reconstructs_bytes_under_installed_format_authority ... ok
test content::recovery::content_retained_selection_persists_restored_historical_bytes ... ok

failures:
```

`slice2-store3`:

```text
sible_value' (56856) panicked at crates\worth-store-court-relational\tests\worth_store_court_relational\process\relatio
nal\relational_home\content\corruption.rs:100:5:
Descriptor(Read(Open(RecordReadError { denial: ArtifactDamaged, observation: RecordReadObservation { touched_segments:
0, touched_pages: 0, touched_extents: 0, payload_bytes: 0, requested_bytes: 173, transfer_count: 1,
peak_transfer_width: 16384, explicit_copy_count: 0, copied_bytes: 0, generation_checks: 2, generation_rejections: 0,
peak_scratch_bytes: 0, manifest_blocks: 2, manifest_comparisons: 6, manifest_bytes: 1360, physical_work_count: 2,
first_physical_work: Some(PhysicalWorkIdentity { store: StableStoreIdentity([171, 236, 177, 85, 59, 199, 245, 48, 119,
215, 222, 238, 71, 78, 68, 148]), runtime: RuntimeIdentity(11658261541699245542), generation:
PhysicalWorkGeneration(LifecycleGeneration(3)), operation: PhysicalOperationIdentity(1) }), last_physical_work:

ve_exact_values' (50056) panicked at crates\worth-store-court-relational\tests\worth_store_court_relational\process\rel
ational\relational_home\content\mod.rs:30:44:
called `Result::unwrap()` on an `Err` value: Physical(RelationalStorePhysicalRefusal { stage: RecoveryAdmission,
detail: "BackendQualification(OwnershipContended)", kind: MutationOwnerContended, recovery_effects: None })
test process::relational::relational_home::content::content_store_reopen_tail_replacement_and_retained_reader_preserve_exact_values ... FAILED

failures:

failures:
```

`slice2-store4`:

```text
sible_value' (56372) panicked at crates\worth-store-court-relational\tests\worth_store_court_relational\process\relatio
nal\relational_home\content\corruption.rs:113:5:
assertion failed: flipped > 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test process::relational::relational_home::content::corruption::content_damaged_store_member_never_reopens_as_a_plausible_value ... FAILED
test process::relational::relational_home::content::corruption::content_missing_physical_member_refuses_reopen_before_service ... ok
test process::relational::relational_home::content::content_store_reopen_tail_replacement_and_retained_reader_preserve_exact_values ... ok

failures:
```

`slice2-store5`:

```text
sible_value' (30232) panicked at crates\worth-store-court-relational\tests\worth_store_court_relational\process\relatio
nal\relational_home\content\corruption.rs:118:5:
Journal(Unreadable { position: None, failure: Stream(RecordStreamFailure { kind: SelectedDataFrameChecksumDamaged,
completed: 0..0, pressure: None }) })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test process::relational::relational_home::content::corruption::content_damaged_store_member_never_reopens_as_a_plausible_value ... FAILED
test process::relational::relational_home::content::corruption::content_missing_physical_member_refuses_reopen_before_service ... ok
test process::relational::relational_home::content::content_store_reopen_tail_replacement_and_retained_reader_preserve_exact_values ... ok

```

`slice2-content7`:

```text
(41040) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\back
ing_refusal.rs:143:14:
test content::recovery::content_current_artifact_roundtrip_preserves_the_literal_frame ... ok
test content::journal_recovery::content_controlled_journal_seed_admits_the_canonical_omitted_history_tail ... ok
test content::journal_recovery::content_decoded_journal_omission_refuses_before_selecting_a_root ... ok
called `Result::unwrap_err()` on an `Ok` value: RecoveryPlan { config: RelationalRuntimeConfig { profile:
CertificationCore, execution: ExecutionConfig { runtime_name: "worth-relational", compiled_lane_policy: Disabled },
diagnostics: DiagnosticsConfig { profile: RelationalDiagnosticsProfile { capture_failures: true, capture_rollbacks:
true, capture_comparisons: true, detailed_traces_enabled: true, collect_all_invariant_failures: false,
```

`slice2-content8`:

```text
(17092) panicked at crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\dura
ble_publication.rs:286:5:
assertion failed: stopped.is_err()
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test content::durable_publication::content_durable_artifact_original_work_exhaustion_is_exact_and_pre_effect ... FAILED
test content::recovery::content_retained_selection_persists_restored_historical_bytes ... ok
test content::recovery::content_checkpoint_member_payload_keeps_original_serial_custody_until_last_clone ... ok
test content::checkpoint_controls::content_inline_checkpoint_insufficient_original_memory_never_selects_a_root ... ok
content staging charged Work: 256=7337824, 512=14724704
```

`slice2-content9`:

```text
panicked at
crates\worth-court-relational-certification\tests\worth_court_relational_certification\content\recovery.rs:183:5:
Request(PolicyMemoryExhausted { requested: 528834, admitted: 523938, ancestor: 0 })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test content::recovery::content_checkpoint_member_payload_keeps_original_serial_custody_until_last_clone ... FAILED
content staging charged Work: 256=7337824, 512=14724704
test cost::content::content_staging_growth_is_bounded_by_persistent_paths_and_cancellation_polls_first ... ok
test content::checkpoint_capture_controls::content_whole_checkpoint_peak_budget_refuses_before_allocating_the_outer_frame ... ok
test content::durable_publication::content_durable_artifact_cancellation_after_original_allocation_never_calls_medium ... ok
```

`slice2-clippy4`:

```text
error: this `if` statement can be collapsed
  --> crates\worth-proof\src\managed_access\coverage.rs:84:9
   |
84 | /         if let Some(root) = frontier.root {
85 | |             if (frontier.disposition)(root).is_none() {
86 | |                 return Err(D::MissingRoot);
87 | |             }
88 | |         }
   | |_________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#collapsible_if
   = note: `-D clippy::collapsible-if` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::collapsible_if)]`

error: could not compile `worth-proof` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...

```

`slice2-clippy-native-own4`:

```text
error: this `if` statement can be collapsed
  --> crates\worth-relational\src\aggregate_identity\funding.rs:78:9
   |
78 | /         if let Some(memory) = self.memory.as_mut() {
79 | |             if peak > memory.bytes() {
80 | |                 memory.resize(peak).map_err(|denial| {
81 | |                     MapKernelFailure::Domain(PlanDenial::Execution(
...  |
88 | |         }
   | |_________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#collapsible_if
   = note: `-D clippy::collapsible-if` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::collapsible_if)]`

error: this expression creates a reference which is immediately dereferenced by the compiler
   --> crates\worth-relational\src\authority\commit\pipeline\publication_execution\preparation.rs:204:45
    |
204 |         .freeze_with_managed_source_history(&source_interpretation, *execution)?;
    |                                             ^^^^^^^^^^^^^^^^^^^^^^ help: change this to: `source_interpretation`
    |
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#needless_borrow
    = note: `-D clippy::needless-borrow` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::needless_borrow)]`

error: mutable key type
  --> crates\worth-relational\src\derived_relation\affected\expression.rs:69:5
   |
69 |     let mut values = BTreeMap::<Position, BTreeSet<Member>>::new();
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: ... because it contains `Position`, which has interior mutability
   = note: ... because it contains `DerivedRelationHandle`, which has interior mutability
   = note: ... because it contains `Arc<RegisteredDerivedRelation>`, which has interior mutability
   = note: ... because it contains `RegisteredDerivedRelation`, which has interior mutability
   = note: ... because it contains `SourceBindingCache`, which has interior mutability
   = note: ... because it contains `Mutex<Weak<SourceProgram>>`, which has interior mutability
   = note: ... because it contains `Mutex`, which has interior mutability
   = note: ... because it contains `Atomic<u8>`, which has interior mutability
   = note: ... because it contains `UnsafeCell<<u8 as AtomicPrimitive>::Storage>`, which has interior mutability
   = help: for further informatio

error: this `if` statement can be collapsed
  --> crates\worth-relational\src\derived_relation\pinned.rs:81:9
   |
81 | /         if let Some(memory) = &mut self.memory {
82 | |             if peak > memory.bytes() {
83 | |                 memory.resize(peak).map_err(|cause| {
84 | |                     Denial::Request(crate::execution::denial::lease_denial(
...  |
89 | |         }
   | |_________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#collapsible_if

error: the following explicit lifetimes could be elided: 'a
  --> crates\worth-relational\src\history\data\canonical_commit_envelope\live_managed_source_history.rs:40:28
   |
40 |     pub(crate) fn verified<'a>(
   |                            ^^
41 |         envelope: &'a Arc<CanonicalCommitEnvelope>,
   |                    ^^
42 |     ) -> Option<&'a PreparedSourceImpactHistory> {
   |                  ^^
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#needless_lifetimes
   = note: `-D clippy::needless-lifetimes` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::needless_lifetimes)]`

error: the following explicit lifetimes could be elided: 'a
  --> crates\worth-relational\src\history\data\canonical_commit_envelope\live_managed_source_history.rs:45:37
   |
45 |     pub(crate) fn verified_material<'a>(
   |                                     ^^
46 |         envelope: &'a Arc<CanonicalCommitEnvelope>,
   |                    ^^
47 |     ) -> Option<&'a RetainedSourceHistoryMaterial> {
   |                  ^^
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#needless_lifetimes

error: use of a disallowed method `std::ptr::from_ref`
  --> crates\worth-relational\src\mvcc\publication\companion\cell\public_cache.rs:83:38
   |
83 |         if cache.0.image.as_ptr() != std::ptr::from_ref(image) {
   |                                      ^^^^^^^^^^^^^^^^^^
   |
   = note: Addresses are for allocation lookup and custody; canonical order uses declared identity
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods
   = note: `-D clippy::disallowed-methods` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::disallowed_methods)]`

error: use of a disallowed method `std::ptr::from_ref`
   --> crates\worth-relational\src\mvcc\publication\companion\cell\public_cache.rs:115:42
    |
115 |                 self.0.image.as_ptr() == std::ptr::from_ref(image)
    |                                          ^^^^^^^^^^^^^^^^^^
    |
    = note: Addresses are for allocation lookup and custody; canonical order uses declared identity
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: returning the result of a `let` binding from a block
  --> crates\worth-relational\src\mvcc\retained_entity_selection\payload.rs:34:5
   |
20 | /     let source = crate::storage::residency::probe_head(
21 | |         execution,
22 | |         &historical.with_symbol_admission(&runtime.services.symbols),
23 | |         |source| {
...  |
33 | |     .map_err(|failure| storage(&input.historical, failure))?;
   | |_____________________________________________________________- unnecessary `let` binding
34 |       source
   |       ^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#let_and_return
   = note: `-D clippy::let-and-return` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::let_and_return)]`

error: use of a disallowed method `std::sync::Arc::as_ptr`
  --> crates\worth-relational\src\validation\candidate_source\program.rs:44:24
   |
44 |         std::ptr::hash(Arc::as_ptr(&self.0), state);
   |                        ^^^^^^^^^^^
   |
   = note: Addresses are for allocation lookup and custody; canonical order uses declared identity
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods

error: could not compile `worth-relational` (lib) due to 10 previous errors; 8 warnings emitted

```

`slice2-clippy-court-own1`:

```text
error: unnecessary `>= y + 1` or `x - 1 >=`
  --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational\
managed_source_work_account\public_only.rs:38:17
   |
38 |         assert!(stopped.work <= EXTRA - 1);
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ help: change it to: `stopped.work < EXTRA`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#int_plus_one
   = note: `-D clippy::int-plus-one` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::int_plus_one)]`

error: unnecessary `>= y + 1` or `x - 1 >=`
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational
\managed_source_work_account.rs:292:21
    |
292 |             assert!(stopped.work <= EXTRA - 1);
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^ help: change it to: `stopped.work < EXTRA`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#int_plus_one

error: unnecessary closure used with `bool::then`
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\process\serial\relational
\managed_source_work_account.rs:104:27
    |
104 |             maximum_work: scenario.rule_limit.then(|| std::num::NonZeroU64::MIN),
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#unnecessary_lazy_evaluations
    = note: `-D clippy::unnecessary-lazy-evaluations` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::unnecessary_lazy_evaluations)]`

error: called `.err().expect()` on a `Result` value
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\managed_source
\nested.rs:110:40
    |
110 |                     return Err(stopped.err().expect("original nested request is cancelled"));
    |                                        ^^^^^^^^^^^^ help: try: `expect_err`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#err_expect
    = note: `-D clippy::err-expect` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::err_expect)]`

error: called `.err().expect()` on a `Result` value
  --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\managed_source\
row_read.rs:48:22
   |
48 |                       .err()
   |  ______________________^
49 | |                     .expect("original ticket cannot outlive cancellation")
   | |___________________________^ help: try: `expect_err`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#err_expect

error: this `if` statement can be collapsed
   --> crates\worth-court-relational-certification\tests\worth_court_relational_certification\relational\managed_source
\owner.rs:267:16
    |
267 |           } else if previous
    |  ________________^
268 | |             .as_ref()
269 | |             .is_some_and(|old| old.scope() == scope.scope())
...   |
284 | |         }
    | |_________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#collapsible_if
    = note: `-D clippy::collapsible-if` implied by `-D clippy::all`
    = help: to override `-D clippy::all` add `#[allow(clippy::collapsible_if)]`

error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 6
previous errors; 2 warnings emitted

```

## Slice 3 Decl and Host content evidence ledger

The original build identity was `11111111-1111-1111-1111-111111111111`.
Commands ran as hidden background processes with output retained under
`target/k1c-logs`; each process wrote its exit before the log was inspected.
No workspace-wide build or test ran. Rows include unsuccessful development
runs, not only final gates. Filtered counts describe the actual executed binary.
The affected packages are Native, its certification court, Query declaration,
installation, execution, package archive, package-archive testkit, and the
Decl/Host, Installation and package-archive courts. Execution engines are concrete;
typed permission/format shells remain generic. The produced rlib is measured below.

| Log | Exact command | Exit | Result |
|---|---|---:|---|
| `slice3-fmt1` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration` | 0 | Completed successfully (no test count). |
| `slice3-decl-check1` | `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration` | 101 | error[E0063]: missing field `content_format` in initializer of `schema_member::ApplicationSchemaMember`; error[E0616]: field `members` of struct `application_schema::declaration::ApplicationSchemaDeclarationBuilder` is ; error[E0004]: non-exhaustive patterns: `&schema_member::ApplicationSchemaMember::ContentFormat { .. }` not covered; error: could not compile `worth-query-declaration` (lib) due to 4 previous errors |
| `slice3-fmt2` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration` | 0 | Completed successfully (no test count). |
| `slice3-decl-check2` | `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration` | 101 | error[E0560]: struct `DeclaredCorrectionOutputFieldParts` has no field named `content_format`; error: could not compile `worth-query-declaration` (lib) due to 1 previous error |
| `slice3-fmt3` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration` | 0 | Completed successfully (no test count). |
| `slice3-decl-install-check1` | `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation` | 0 | Completed successfully (no test count). |
| `slice3-fmt4` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation` | 0 | Completed successfully (no test count). |
| `slice3-nativefmt1` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice3-decl-install-check2` | `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation` | 101 | error[E0004]: non-exhaustive patterns: ; error: could not compile `worth-query-installation` (lib) due to 1 previous error |
| `slice3-nativefmt2` | `cargo fmt -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice3-fmt5` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation` | 0 | Completed successfully (no test count). |
| `slice3-decl-install-check3` | `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation` | 101 | error[E0308]: mismatched types; error: could not compile `worth-query-installation` (lib) due to 3 previous errors |
| `slice3-native-check1` | `cargo check -p worth-relational` | 0 | Completed successfully (no test count). |
| `slice3-fmt6` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution` | 0 | Completed successfully (no test count). |
| `slice3-nativefmt3` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice3-execution-check1` | `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | 101 | error[E0004]: non-exhaustive patterns: `RelationalNativeCheckpointCaptureDenial::ArtifactFrame(_)` and ; error[E0004]: non-exhaustive patterns: `RelationalRecoveryStop::Content(_)` not covered; error: could not compile `worth-query-execution` (lib) due to 2 previous errors; 8 warnings emitted |
| `slice3-native-ceiling1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::schema_transition -- --nocapture` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 193 filtered out; finished in 0.03s |
| `slice3-fmt7` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution` | 0 | Completed successfully (no test count). |
| `slice3-execution-check2` | `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | 101 | error[E0432]: unresolved import `worth_relational::facade::content::RelationalContentAttachments`; error[E0425]: cannot find type `RelationalContentAttachments` in module `worth_relational::facade::content`; error[E0063]: missing field `content_attachments` in initializer of ; error[E0027]: pattern does not mention field `content_attachments`; error[E0004]: non-exhaustive patterns: ; error: could not compile `worth-query-execution` (lib) due to 37 previous errors; 10 warnings emitted |
| `slice3-fmt8` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host` | 0 | Completed successfully (no test count). |
| `slice3-public1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 101 | error[E0062]: field `content_attachments` specified more than once; error[E0063]: missing field `content_attachments` in initializer of ; error: could not compile `worth-query-execution` (lib) due to 19 previous errors; 8 warnings emitted |
| `slice3-archive-check1` | `cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-package-archive` | 101 | error[E0027]: pattern does not mention field `content_format`; error: could not compile `worth-query-package-archive` (lib) due to 1 previous error |
| `slice3-fmt9` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive` | 0 | Completed successfully (no test count). |
| `slice3-archive1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-package-archive --test worth_query_court_package_archive application_schema_record -- --nocapture` | 101 | error[E0027]: pattern does not mention field `content_format`; error: could not compile `worth-query-package-archive` (lib) due to 1 previous error |
| `slice3-caps1` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 121 Rust files |
| `slice3-public2` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 101 | error[E0432]: unresolved import `worth_query_installation::facade::ContentRefApplicationValueBinding`; error: could not compile `worth-query-execution` (lib) due to 1 previous error; 8 warnings emitted |
| `slice3-fmt10` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive` | 0 | Completed successfully (no test count). |
| `slice3-archive2` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-package-archive --test worth_query_court_package_archive application_schema_record -- --nocapture` | 0 | test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 76 filtered out; finished in 0.00s |
| `slice3-public3` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 101 | error[E0659]: `WorthQueryApplicationReadObservation` is ambiguous; error[E0716]: temporary value dropped while borrowed; error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 5 previous errors;  |
| `slice3-fmt11` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation` | 0 | Completed successfully (no test count). |
| `slice3-install1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-installation --test worth_query_court_installation content_ -- --nocapture` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.01s |
| `slice3-public4` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 101 | error[E0277]: `worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource` doesn't ; error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 1 previous error; 1  |
| `slice3-recipe-doc1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration --doc ApplicationContentBindingRecipe` | 0 | test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.00s; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.11s |
| `slice3-fmt12` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation` | 0 | Completed successfully (no test count). |
| `slice3-install2` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-installation --test worth_query_court_installation content_ -- --nocapture` | 0 | test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.01s |
| `slice3-public5` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 101 | test result: FAILED. 0 passed; 11 failed; 0 ignored; 0 measured; 327 filtered out; finished in 0.01s; error: test failed, to rerun pass `-p worth-query-court-decl-host --test worth_query_court_decl_host` |
| `slice3-native-content1` | `cargo test -p worth-relational --lib content:: -- --nocapture` | 0 | test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 2261 filtered out; finished in 1.26s |
| `slice3-query-clippy1` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation --all-targets` | 101 | error[E0063]: missing field `content_format` in initializer of ; error: this `if` statement can be collapsed; error: could not compile `worth-query-declaration` (lib) due to 1 previous error; error: could not compile `worth-query-declaration` (lib test) due to 4 previous errors |
| `slice3-native-fmt4` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice3-fmt13` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation` | 0 | Completed successfully (no test count). |
| `slice3-public6` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 101 | test result: FAILED. 1 passed; 10 failed; 0 ignored; 0 measured; 327 filtered out; finished in 0.15s; error: test failed, to rerun pass `-p worth-query-court-decl-host --test worth_query_court_decl_host` |
| `slice3-decl-doc2` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration --doc content -- --nocapture` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.01s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.10s; error[E0308]: mismatched types; error: aborting due to 1 previous error |
| `slice3-native-fmt5` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice3-fmt14` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation` | 0 | Completed successfully (no test count). |
| `slice3-native-contract2` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::schema_transition -- --nocapture` | 0 | test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 193 filtered out; finished in 0.02s |
| `slice3-caps2` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 153 Rust files |
| `slice3-diff-dirty1` | `git diff --check` | 0 | Completed successfully (no test count). |
| `slice3-public7` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 101 | test result: FAILED. 1 passed; 11 failed; 0 ignored; 0 measured; 327 filtered out; finished in 0.02s; error: test failed, to rerun pass `-p worth-query-court-decl-host --test worth_query_court_decl_host` |
| `slice3-query-clippy2` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation --all-targets` | 101 | error: this `if` statement can be collapsed; error: use of a disallowed method `std::any::TypeId::of`; error: could not compile `worth-query-installation` (lib) due to 4 previous errors; error: could not compile `worth-query-installation` (lib test) due to 4 previous errors |
| `slice3-native-clippy1` | `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | 101 | error: using `clone` on type `RelationalContentDenial` which implements the `Copy` trait; error: returning the result of a `let` binding from a block; error: could not compile `worth-relational` (lib) due to 2 previous errors; 8 warnings emitted |
| `slice3-boundary1` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | boundary-check: repository boundaries are valid |
| `slice3-context1` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `slice3-native-fmt6` | `cargo fmt -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice3-fmt15` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation` | 0 | Completed successfully (no test count). |
| `slice3-native-contract3` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content::schema_transition -- --nocapture` | 0 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 193 filtered out; finished in 0.02s |
| `slice3-public8` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 101 | error[E0308]: mismatched types; error: could not compile `worth-query-execution` (lib) due to 1 previous error; 8 warnings emitted |
| `slice3-execution-doc1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --doc content -- --nocapture` | 101 | error[E0308]: mismatched types; error: could not compile `worth-query-execution` (lib) due to 1 previous error; 8 warnings emitted |
| `slice3-query-clippy3` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation --all-targets` | 101 | error: this OR pattern can be rewritten using a range; error: called `.err().expect()` on a `Result` value; error: could not compile `worth-query-court-installation` (test "worth_query_court_installation") due to 2 previous  |
| `slice3-fmt16` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation` | 0 | Completed successfully (no test count). |
| `slice3-caps3` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 154 Rust files |
| `slice3-public9` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-decl-host --test worth_query_court_decl_host content_door -- --nocapture` | 0 | test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 327 filtered out; finished in 0.46s |
| `slice3-execution-doc2` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --doc content -- --nocapture` | 0 | test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.00s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 100 filtered out; finished in 0.18s; error[E0277]: the trait bound `F: OperationReads<<B as ApplicationMutationBinding<S>>::Operation>` is not satisfied; error: aborting due to 1 previous error; error[E0308]: mismatched types |
| `slice3-query-clippy4` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation --all-targets` | 101 | error[E0063]: missing field `content_format` in initializer of ; error: could not compile `worth-query-execution` (lib test) due to 1 previous error; 11 warnings emitted |
| `slice3-native-clippy2` | `cargo clippy -p worth-relational -p worth-court-relational-certification --all-targets` | 101 | error: returning the result of a `let` binding from a block; error: could not compile `worth-relational` (lib) due to 1 previous error; 8 warnings emitted |
| `slice3-native-fmtcheck1` | `cargo fmt --check -p worth-relational -p worth-court-relational-certification` | 0 | Completed successfully (no test count). |
| `slice3-query-fmtcheck1` | `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation` | 0 | Completed successfully (no test count). |
| `slice3-native-all1` | `cargo test -p worth-court-relational-certification --test worth_court_relational_certification content:: -- --nocapture` | 0 | test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out; finished in 1.05s |
| `slice3-install3` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-installation --test worth_query_court_installation content_ -- --nocapture` | 0 | test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.01s |
| `slice3-install-profiles1` | `cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-court-installation --test worth_query_court_installation managed_index_parent_profiles -- --nocapture` | 0 | test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.02s |
| `slice3-boundary2` | `cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .` | 0 | boundary-check: repository boundaries are valid |
| `slice3-context2` | `cargo run --manifest-path tools/agent-context/Cargo.toml -- check` | 0 | Completed successfully (no test count). |
| `slice3-execution-fmt1` | `cargo fmt --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution` | 0 | Completed successfully (no test count). |
| `slice3-diff-head1` | `git diff --check origin/master...HEAD` | 0 | Completed successfully (no test count). |
| `slice3-diff-dirty2` | `git diff --check` | 0 | Completed successfully (no test count). |
| `slice3-caps4` | `scripts/ci/check_workspace_rust_line_caps.sh dirty` | 0 | PASS: checked 155 Rust files |
| `slice3-query-clippy5` | `cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation --all-targets` | 101 | error: unnecessary use of `clone` to create a slice from a reference; error: use of a disallowed method `std::thread::scope`; error: use of a disallowed method `std::thread::Scope::spawn`; error: called `.err().expect()` on a `Result` value; error: using `clone` on type `CommitId` which implements the `Copy` trait; error: the loop variable `member_index` is only used to index `samples`; error: manual implementation of `.is_multiple_of()`; error: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its ; error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 64 previous errors;  |
| `slice3-query-fmtcheck2` | `cargo fmt --check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration -p worth-query-installation -p worth-query-execution -p worth-query-package-archive -p worth-query-court-decl-host -p worth-query-testkit-package-archive -p worth-query-court-package-archive -p worth-query-court-installation` | 0 | Completed successfully (no test count). |
| `slice3-native-court-clippy1` | `cargo clippy -p worth-court-relational-certification --all-targets --no-deps` | 101 | error: unnecessary `>= y + 1` or `x - 1 >=`; error: unnecessary closure used with `bool::then`; error: called `.err().expect()` on a `Result` value; error: this `if` statement can be collapsed; error: could not compile `worth-court-relational-certification` (test "worth_court_relational_certification") due to 6  |
| `slice3-diff-dirty3` | `git diff --check` | 0 | Completed successfully (no test count). |
| `slice3-diff-dirty4` | `git diff --check` | 0 | Completed successfully (no test count). |
| `slice3-diff-cached1` | `git diff --cached --check` | 0 | Completed successfully (no test count). |

### Slice 3 court map and failed-run outcomes

The final accepted production/court claims are documented in
[milestone-9.19-k1c-content-slice3.md](milestone-9.19-k1c-content-slice3.md).
Expected compiler diagnostics inside successful compile-fail rustdocs are not
failed runs; their process exit and test counts identify the distinction.

Development failures were surfaced and corrected, including missing new member
initializers, builder/module placement, exhaustive denial mappings, attachment
plumbing initializers, archive correction-output patterns, public fixture type
and borrow errors, missing declared operation/read permissions, lost Native
registry composition, exact contract equality at the typed handoff, and the
function-pointer lifetime mismatch during concrete-engine extraction. No failed
public run is treated as behavioral evidence. Native's final all-target lint
failure is the untouched retained-selection `let_and_return` debt, with its exact
location/output retained below; caused scoped lints were fixed.

The final execution artifact is **318,207,646 bytes**, SHA-256
`ecba15068e1d81ce6f6a7a346a8cf4ab5a9b5130112282653747d5267626b9c5`.
This is telemetry from the successful public9 unoptimized test build, not a
controlled before/after comparison. Engine work is concrete; typed permission,
format-predicate and wrapping shells remain generic.

The final Query all-target run completed with **64** lint errors, all in untouched
Decl/Host history/workflow court files; **zero** diagnostic paths intersect the
scoped dirty set. Native's second required all-target run had **one** untouched
`let_and_return` error. These use AGENTS.md's explicit untouched-debt exception.
The exact Query diagnostics are:

| Diagnostic output | Location |
|---|---|
| error: unnecessary use of `clone` to create a slice from a reference | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/cell_history/checkpoint_cases.rs:52` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/capability/grant_commands/mod.rs:40` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/capability/grant_commands/mod.rs:41` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/mod.rs:63` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/mod.rs:64` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/mod.rs:120` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/mod.rs:122` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/retained_capacity/mod.rs:24` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/managed_cleanup/retained_capacity/mod.rs:27` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/native_postures/mod.rs:59` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/native_postures/mod.rs:60` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/retained/native_postures/mod.rs:20` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/application_graph/source_only/retained/native_postures/mod.rs:21` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/native_workflow_operations/capability/grant_commands/mod.rs:40` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/native_workflow_operations/capability/grant_commands/mod.rs:41` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/context/handoff.rs:19` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/context/handoff.rs:20` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/context/handoff.rs:57` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/context/handoff.rs:58` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/cancellation.rs:158` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/cancellation.rs:160` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/handoff.rs:58` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/handoff.rs:60` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/handoff.rs:118` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/handoff.rs:120` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/recovery.rs:87` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/recovery.rs:90` |
| error: called `.err().expect()` on a `Result` value | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/resources.rs:48` |
| error: using `clone` on type `CommitId` which implements the `Copy` trait | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/group/mod.rs:154` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/cancellation.rs:98` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/cancellation.rs:99` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:45` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:46` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:98` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:99` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:165` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/race.rs:166` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/recovery.rs:82` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/recovery.rs:83` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:332` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:333` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:337` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:351` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:352` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/navigation/resources.rs:356` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs:26` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs:27` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs:60` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/paging/handoff.rs:61` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/capture_race.rs:40` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/capture_race.rs:41` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/handoff.rs:29` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/handoff.rs:31` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/recovery.rs:35` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/recording/recovery.rs:36` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/redo/handoff.rs:35` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/redo/handoff.rs:36` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/cancellation.rs:92` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/cancellation.rs:93` |
| error: use of a disallowed method `std::thread::scope` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/cancellation.rs:167` |
| error: use of a disallowed method `std::thread::Scope::spawn` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/certification/document_history/retention/cancellation.rs:168` |
| error: the loop variable `member_index` is only used to index `samples` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/cost/document_history/group.rs:180` |
| error: manual implementation of `.is_multiple_of()` | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/cost/document_history/growth/scenario.rs:72` |
| error: call to `std::mem::drop` with a value that does not implement `Drop`. Dropping such a type only extends its  | `workspaces/worth-query/crates/worth-query-court-decl-host/tests/worth_query_court_decl_host/cost/document_history/growth/scenario.rs:217` |

The direct Native court `--no-deps` lint pass completed with **6** errors in
untouched files, **zero** diagnostic paths in the dirty set. It was run because
the required Native dependency lint failed before reaching the court. Its exact
output locations are:

| Diagnostic output | Location |
|---|---|
| error: unnecessary `>= y + 1` or `x - 1 >=` | `crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account/public_only.rs:38` |
| error: unnecessary `>= y + 1` or `x - 1 >=` | `crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account.rs:292` |
| error: unnecessary closure used with `bool::then` | `crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account.rs:104` |
| error: called `.err().expect()` on a `Result` value | `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/nested.rs:110` |
| error: called `.err().expect()` on a `Result` value | `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/row_read.rs:48` |
| error: this `if` statement can be collapsed | `crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/owner.rs:267` |

### Development failure excerpts

These are excerpts from unsuccessful development runs; the ledger preserves each
exit and exact command/count. Final lint diagnostic inventories above are complete.

`slice3-decl-check1` (exit 101):

```text
error[E0063]: missing field `content_format` in initializer of `schema_member::ApplicationSchemaMember`
  --> crates\worth-query-declaration\src\application_aftermath\correction_binding\authored_identity\key.rs:28:21
   |
28 |             member: ApplicationSchemaMember::Field {
   |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `content_format`

error[E0063]: missing field `content_format` in initializer of `schema_member::ApplicationSchemaMember`
  --> crates\worth-query-declaration\src\application_aftermath\correction_binding\source_output.rs:51:9
   |
51 |         ApplicationSchemaMember::Field {
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `content_format`
error[E0616]: field `members` of struct `application_schema::declaration::ApplicationSchemaDeclarationBuilder` is
private
  --> crates\worth-query-declaration\src\application_schema\content\declaration.rs:15:14
   |
15 |         self.members.push(ApplicationSchemaMember::ContentFormat {
   |              ^^^^^^^ private field

error[E0004]: non-exhaustive patterns: `&schema_member::ApplicationSchemaMember::ContentFormat { .. }` not covered
   --> crates\worth-query-declaration\src\application_schema\identifier_validation.rs:43:15
    |
 43 |         match member {
error[E0004]: non-exhaustive patterns: `&schema_member::ApplicationSchemaMember::ContentFormat { .. }` not covered
   --> crates\worth-query-declaration\src\application_schema\identifier_validation.rs:43:15
    |
 43 |         match member {
    |               ^^^^^^ pattern `&schema_member::ApplicationSchemaMember::ContentFormat { .. }` not covered
    |
note: `schema_member::ApplicationSchemaMember` defined here
   --> crates\worth-query-declaration\src\application_schema\schema_member.rs:74:10
    |
 74 | pub enum ApplicationSchemaMember {
    |          ^^^^^^^^^^^^^^^^^^^^^^^
error: could not compile `worth-query-declaration` (lib) due to 4 previous errors
```

`slice3-decl-check2` (exit 101):

```text
error[E0560]: struct `DeclaredCorrectionOutputFieldParts` has no field named `content_format`
   --> crates\worth-query-declaration\src\application_aftermath\correction_binding\source_output.rs:188:21
    |
188 |                     content_format: None,
    |                     ^^^^^^^^^^^^^^ `DeclaredCorrectionOutputFieldParts` does not have this field
    |
    = note: all struct fields are already assigned

For more information about this error, try `rustc --explain E0560`.
error: could not compile `worth-query-declaration` (lib) due to 1 previous error
error: could not compile `worth-query-declaration` (lib) due to 1 previous error
```

`slice3-decl-install-check2` (exit 101):

```text
error[E0004]: non-exhaustive patterns:
`WorthQueryApplicationValueBindingInstallationDenialKind::DuplicateContentFormat`,
`WorthQueryApplicationValueBindingInstallationDenialKind::MissingContentBinding` and
`WorthQueryApplicationValueBindingInstallationDenialKind::ContentFormatMismatch` not covered
   --> crates\worth-query-installation\src\installed_index\application_schema\denial_mapping.rs:102:30
    |
102 |             let kind = match denial.kind() {
    |                              ^^^^^^^^^^^^^ patterns
`WorthQueryApplicationValueBindingInstallationDenialKind::DuplicateContentFormat`,
`WorthQueryApplicationValueBindingInstallationDenialKind::MissingContentBinding` and
`WorthQueryApplicationValueBindingInstallationDenialKind::ContentFormatMismatch` not covered
error: could not compile `worth-query-installation` (lib) due to 1 previous error
```

`slice3-decl-install-check3` (exit 101):

```text
error[E0308]: mismatched types
  --> crates\worth-query-installation\src\installed_index\application_schema\denial_mapping.rs:30:17
   |
29 |             let kind = match denial.kind() {
   |                              ------------- this expression has type
`WorthQueryApplicationCapabilityInstallationDenialKind`
30 |                 ValueBindingDenialKind::MissingContentBinding =>
WorthQueryInstalledApplicationSchemaDenialKind::ContentBindingMissing,
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected
`WorthQueryApplicationCapabilityInstallationDenialKind`, found
`WorthQueryApplicationValueBindingInstallationDenialKind`
error: could not compile `worth-query-installation` (lib) due to 3 previous errors
```

`slice3-execution-check1` (exit 101):

```text
error[E0004]: non-exhaustive patterns: `RelationalNativeCheckpointCaptureDenial::ArtifactFrame(_)` and
`RelationalNativeCheckpointCaptureDenial::Content(_)` not covered
  --> crates\worth-query-execution\src\domain_computation\primary_graph\application_checkpoint\denial.rs:21:15
   |
21 |         match error {
   |               ^^^^^ patterns `RelationalNativeCheckpointCaptureDenial::ArtifactFrame(_)` and
`RelationalNativeCheckpointCaptureDenial::Content(_)` not covered
   |
note: `RelationalNativeCheckpointCaptureDenial` defined here
  --> C:\forge_workspace\worth-cut-k1c-content\crates\worth-relational\src\durability\data\native_checkpoint_capture_de
nial.rs:7:1
error[E0004]: non-exhaustive patterns: `RelationalRecoveryStop::Content(_)` not covered
   --> crates\worth-query-execution\src\domain_computation\primary_graph\bootstrap\preparation.rs:237:11
    |
237 |     match stop {
    |           ^^^^ pattern `RelationalRecoveryStop::Content(_)` not covered
    |
note: `RelationalRecoveryStop` defined here
   --> C:\forge_workspace\worth-cut-k1c-content\crates\worth-relational\src\durability\data\recovery_stop.rs:18:1
    |
 18 | pub enum RelationalRecoveryStop {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
error: could not compile `worth-query-execution` (lib) due to 2 previous errors; 8 warnings emitted
```

`slice3-execution-check2` (exit 101):

```text
error[E0432]: unresolved import `worth_relational::facade::content::RelationalContentAttachments`
  -->
crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\effect_program\content.rs:14:5
   |
14 | use worth_relational::facade::content::RelationalContentAttachments;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^----------------------------
   |                                        |
   |                                        no `RelationalContentAttachments` in `facade::content`
   |
help: a similar name exists in the module
   |
error[E0425]: cannot find type `RelationalContentAttachments` in module `worth_relational::facade::content`
   -->
crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\effect_program\model.rs:116:44
    |
116 |         worth_relational::facade::content::RelationalContentAttachments,
    |                                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
   ::: C:\forge_workspace\worth-cut-k1c-content\crates\worth-relational\src\content\frame\mod.rs:21:1
    |
 21 | pub struct RelationalContentFrame(Arc<AdmittedFrame>);
    | --------------------------------- similarly named struct `RelationalContentFrame` defined here
error[E0063]: missing field `content_attachments` in initializer of
`application_attempt::effect_program::model::WorthQueryApplicationEffectProgram<_, _, _, _>`
  --> crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\capability_revocation_progr
am.rs:53:22
   |
53 |             program: WorthQueryApplicationEffectProgram {
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `content_attachments`

error[E0063]: missing field `content_attachments` in initializer of
`application_attempt::effect_program::model::WorthQueryApplicationEffectProgram<_, _, _, _>`
  --> crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\delegation_activation_progr
error: could not compile `worth-query-execution` (lib) due to 37 previous errors; 10 warnings emitted
```

`slice3-public1` (exit 101):

```text
error[E0062]: field `content_attachments` specified more than once
  --> crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\capability_revocation_progr
am.rs:64:13
   |
63 |             content_attachments: Default::default(),
   |             --------------------------------------- first use of `content_attachments`
64 |             content_attachments: Default::default(),
   |             ^^^^^^^^^^^^^^^^^^^ used more than once

error[E0062]: field `content_attachments` specified more than once
  --> crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\delegation_activation_progr
error[E0063]: missing field `content_attachments` in initializer of
`application_attempt::effect_program::model::WorthQueryApplicationEffectProgram<_, _, _, _>`
  --> crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\workflow_transition_program
\approval_decision\materialization.rs:38:18
   |
38 |         program: WorthQueryApplicationEffectProgram {
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `content_attachments`

error[E0063]: missing field `content_attachments` in initializer of
`application_attempt::effect_program::model::WorthQueryApplicationEffectProgram<_, _, _, _>`
   --> crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\workflow_transition_progra
error: could not compile `worth-query-execution` (lib) due to 19 previous errors; 8 warnings emitted
```

`slice3-archive-check1` (exit 101):

```text
error[E0027]: pattern does not mention field `content_format`
  --> crates\worth-query-package-archive\src\record\application_schema\member\operation\aftermath\correction_binding\so
urce_outputs.rs:51:13
   |
51 |           let ApplicationSchemaMember::Field {
   |  _____________^
52 | |             entity,
53 | |             aspect,
54 | |             field,
...  |
63 | |             unique: false,
error: could not compile `worth-query-package-archive` (lib) due to 1 previous error
```

`slice3-archive1` (exit 101):

```text
error[E0027]: pattern does not mention field `content_format`
  --> crates\worth-query-package-archive\src\record\application_schema\member\operation\aftermath\correction_binding\so
urce_outputs.rs:51:13
   |
51 |           let ApplicationSchemaMember::Field {
   |  _____________^
52 | |             entity,
53 | |             aspect,
54 | |             field,
...  |
63 | |             unique: false,
error: could not compile `worth-query-package-archive` (lib) due to 1 previous error
```

`slice3-public2` (exit 101):

```text
error[E0432]: unresolved import `worth_query_installation::facade::ContentRefApplicationValueBinding`
 -->
crates\worth-query-execution\src\domain_computation\primary_graph\invariant_projection\locked_reader\content.rs:8:59
  |
8 | use worth_query_installation::facade::{ApplicationSchema, ContentRefApplicationValueBinding};
  |                                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no
`ContentRefApplicationValueBinding` in `facade`

warning: unused import: `field_masks::admit_field_mask_categories`
  --> crates\worth-query-execution\src\domain_computation\authorization\application_disclosure\contract.rs:22:39
   |
error: could not compile `worth-query-execution` (lib) due to 1 previous error; 8 warnings emitted
```

`slice3-public3` (exit 101):

```text
error[E0659]: `WorthQueryApplicationReadObservation` is ambiguous
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:104:28
    |
104 | pub fn pin(host: &Host) -> WorthQueryApplicationReadObservation {
    |                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ambiguous name
    |
    = note: ambiguous because of multiple glob imports of a name in the same module
note: `WorthQueryApplicationReadObservation` could refer to the struct imported here
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\mod.rs:9:34
    |
  9 |     application_contribution::*, application_entry::*, application_installation::*,
error[E0716]: temporary value dropped while borrowed
   --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\courts.rs:89:22
    |
 89 |         let before = world::pin(&host).selected_commit();
    |                      ^^^^^^^^^^^^^^^^^                  - temporary value is freed at the end of this statement
    |                      |
    |                      creates a temporary value which is freed while still in use
...
104 |         assert_eq!(world::pin(&host).selected_commit(), before);
    |         ------------------------------------------------------- borrow later used here
    |
error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 5 previous errors;
```

`slice3-public4` (exit 101):

```text
error[E0277]: `worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource` doesn't
implement `Debug`
  --> crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\codec.rs:13:5
   |
 6 | #[derive(Clone, Debug)]
   |                 ----- in this derive macro expansion
...
13 |     pub cancel_during_encoding: Option<WorthQueryCancellationSource>,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Debug` is not implemented for
`worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource`
   |
error: could not compile `worth-query-court-decl-host` (test "worth_query_court_decl_host") due to 1 previous error; 1
```

`slice3-public5` (exit 101):

```text

thread 'content_door::courts::content_duplicate_format_declarations_refuse_before_installation' (23580) panicked at
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\courts.rs:159:5:
assertion `left == right` failed
  left: DuplicateMember
 right: DuplicateContentFormat
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test content_door::courts::content_duplicate_format_declarations_refuse_before_installation ... FAILED
thread 'content_door::courts::content_raw_missing_reference_is_refused_and_the_selected_value_stays_unchanged' (48960)
panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:39:31:
called `Result::unwrap()` on an `Err` value: MissingApplicationMutationDependency

thread 'content_door::courts::content_public_door_preserves_exact_order_and_deduplicates_equal_payloads' (56500)
panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:39:31:
called `Result::unwrap()` on an `Err` value: MissingApplicationMutationDependency

thread 'content_door::courts::content_public_door_preserves_exact_order_and_deduplicates_equal_payloads' (56500)
panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:39:31:
called `Result::unwrap()` on an `Err` value: MissingApplicationMutationDependency

thread 'content_door::courts::content_projection_budget_refuses_the_call_instead_of_returning_optional_absence' (6140)
panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:39:31:
called `Result::unwrap()` on an `Err` value: MissingApplicationMutationDependency

test result: FAILED. 0 passed; 11 failed; 0 ignored; 0 measured; 327 filtered out; finished in 0.01s
```

`slice3-query-clippy1` (exit 101):

```text
error[E0063]: missing field `content_format` in initializer of
`application_schema::schema_member::ApplicationSchemaMember`
  --> crates\worth-query-declaration\src\application_schema\operation_definition\body\signing_lifecycle\validation\test
s.rs:75:9
   |
75 |         Member::Field {
   |         ^^^^^^^^^^^^^ missing `content_format`

error[E0063]: missing field `content_format` in initializer of
`application_schema::schema_member::ApplicationSchemaMember`
  --> crates\worth-query-declaration\src\application_schema\operation_definition\body\signing_validation\tests.rs:41:63
error: this `if` statement can be collapsed
  --> crates\worth-query-declaration\src\application_schema\content\validation.rs:12:9
   |
12 | /         if let Member::ContentFormat { declaration } = member {
13 | |             if formats.insert(declaration.identity, declaration).is_some()
14 | |                 || !names.insert((&declaration.name, declaration.version))
...  |
18 | |         }
   | |_________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#collapsible_if
error: could not compile `worth-query-declaration` (lib) due to 1 previous error
error: could not compile `worth-query-declaration` (lib test) due to 4 previous errors
```

`slice3-public6` (exit 101):

```text
thread 'content_door::courts::content_corrupted_callback_result_is_caught_by_independent_numeric_correspondence'
(63316) panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\courts.rs:194:5:
assertion failed: matches!(cause.downcast::<ContentWriteDenial>().unwrap(),
    ContentWriteDenial::Content(RelationalContentDenial::EncodingInconsistency))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'content_door::courts::content_original_cancellation_during_encoding_refuses_before_publication_then_fresh_reque
st_succeeds' (23636) panicked at
thread 'content_door::courts::content_original_cancellation_during_encoding_refuses_before_publication_then_fresh_reque
st_succeeds' (23636) panicked at
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\courts.rs:269:13:
assertion failed: matches!(cause.downcast::<ContentWriteDenial>().unwrap(),
    ContentWriteDenial::Content(RelationalContentDenial::Execution(RelationalExecutionDenialCause::Cancelled)))

thread 'content_door::courts::content_sink_refusal_cannot_be_swallowed_by_a_successful_callback' (54408) panicked at
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\courts.rs:212:5:

thread 'content_door::courts::content_sink_refusal_cannot_be_swallowed_by_a_successful_callback' (54408) panicked at
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\courts.rs:212:5:
assertion failed: matches!(cause.downcast::<ContentWriteDenial>().unwrap(),
    ContentWriteDenial::Content(RelationalContentDenial::EncodingInconsistency))

thread 'content_door::courts::content_callback_refusal_cannot_be_swallowed_into_publication' (25172) panicked at
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\courts.rs:69:5:
test result: FAILED. 1 passed; 10 failed; 0 ignored; 0 measured; 327 filtered out; finished in 0.15s
```

`slice3-public7` (exit 101):

```text

thread 'content_door::courts::content_sink_refusal_cannot_be_swallowed_by_a_successful_callback' (22744) panicked at
crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:39:31:
called `Result::unwrap()` on an `Err` value: MissingOperationProgramDependency { operation: "Change", dependency:
ProgramTarget(RemoveAspect { entity: "Record", aspect: "Samples" }) }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'content_door::courts::content_unbound_reference_is_refused_by_the_existing_publication_seal' (21796) panicked
thread 'content_door::courts::content_raw_missing_reference_is_refused_and_the_selected_value_stays_unchanged' (23784)
panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:39:31:
called `Result::unwrap()` on an `Err` value: MissingOperationProgramDependency { operation: "Change", dependency:
ProgramTarget(RemoveAspect { entity: "Record", aspect: "Samples" }) }

thread 'content_door::courts::content_equal_descriptors_cannot_substitute_another_binding_type_at_either_door' (37648)
panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:39:31:
called `Result::unwrap()` on an `Err` value: MissingOperationProgramDependency { operation: "Change", dependency:
thread 'content_door::courts::content_equal_descriptors_cannot_substitute_another_binding_type_at_either_door' (37648)
panicked at crates\worth-query-court-decl-host\tests\worth_query_court_decl_host\content_door\world.rs:39:31:
called `Result::unwrap()` on an `Err` value: MissingOperationProgramDependency { operation: "Change", dependency:
ProgramTarget(RemoveAspect { entity: "Record", aspect: "Samples" }) }
test content_door::courts::content_duplicate_format_declarations_refuse_before_installation ... ok
test content_door::courts::content_sink_refusal_cannot_be_swallowed_by_a_successful_callback ... FAILED
test content_door::courts::content_raw_missing_reference_is_refused_and_the_selected_value_stays_unchanged ... FAILED
test content_door::courts::content_unbound_reference_is_refused_by_the_existing_publication_seal ... FAILED
test result: FAILED. 1 passed; 11 failed; 0 ignored; 0 measured; 327 filtered out; finished in 0.02s
```

`slice3-query-clippy2` (exit 101):

```text
error: this `if` statement can be collapsed
  --> crates\worth-query-installation\src\application_schema\value_binding\content.rs:27:9
   |
27 | /         if let Member::ContentFormat { declaration } = member {
28 | |             if declared.insert(declaration.identity, declaration).is_some() {
29 | |                 return Err(Denial::new(Kind::DuplicateContentFormat, &declaration.name));
30 | |             }
31 | |         }
   | |_________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#collapsible_if
error: use of a disallowed method `std::any::TypeId::of`
  --> crates\worth-query-installation\src\application_schema\value_binding\content.rs:83:40
   |
83 |         if primitive.binding_type() != std::any::TypeId::of::<ContentRefApplicationValueBinding>() {
   |                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: Type identity is for point lookup; canonical order uses declared identity
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#disallowed_methods
   = note: `-D clippy::disallowed-methods` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::disallowed_methods)]`
error: could not compile `worth-query-installation` (lib) due to 4 previous errors
error: could not compile `worth-query-installation` (lib test) due to 4 previous errors
```

`slice3-native-clippy1` (exit 101):

```text
error: using `clone` on type `RelationalContentDenial` which implements the `Copy` trait
  --> crates\worth-relational\src\durability\authority\runtime_rebuild\envelope_replay\commit_stop.rs:22:86
   |
22 |             crate::transactions::data::ConflictClass::ContentAdmission { denial } => denial.clone().into(),
   |                                                                                      ^^^^^^^^^^^^^^ help: try
dereferencing it: `(*denial)`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#clone_on_copy
   = note: `-D clippy::clone-on-copy` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::clone_on_copy)]`

error: returning the result of a `let` binding from a block
  --> crates\worth-relational\src\mvcc\retained_entity_selection\payload.rs:34:5
   |
20 | /     let source = crate::storage::residency::probe_head(
21 | |         execution,
22 | |         &historical.with_symbol_admission(&runtime.services.symbols),
23 | |         |source| {
...  |
33 | |     .map_err(|failure| storage(&input.historical, failure))?;
   | |_____________________________________________________________- unnecessary `let` binding
34 |       source
error: could not compile `worth-relational` (lib) due to 2 previous errors; 8 warnings emitted
```

`slice3-public8` (exit 101):

```text
error[E0308]: mismatched types
  -->
crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\effect_program\content.rs:72:32
   |
72 |         .prepare(locus, input, ValueDecodeSession::content_binding::<C>)
   |          -------               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ one type is more general than the other
   |          |
   |          arguments to this method are incorrect
   |
   = note: expected fn pointer `for<'a, 'b, 'c, 'd, 'e> fn(&'a ValueDecodeSession<'b>, &'c _, &'d _, &'e _) ->
Result<_, _>`
error: could not compile `worth-query-execution` (lib) due to 1 previous error; 8 warnings emitted
```

`slice3-execution-doc1` (exit 101):

```text
error[E0308]: mismatched types
  -->
crates\worth-query-execution\src\domain_computation\primary_graph\application_attempt\effect_program\content.rs:72:32
   |
72 |         .prepare(locus, input, ValueDecodeSession::content_binding::<C>)
   |          -------               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ one type is more general than the other
   |          |
   |          arguments to this method are incorrect
   |
   = note: expected fn pointer `for<'a, 'b, 'c, 'd, 'e> fn(&'a ValueDecodeSession<'b>, &'c _, &'d _, &'e _) ->
Result<_, _>`
error: could not compile `worth-query-execution` (lib) due to 1 previous error; 8 warnings emitted
```

`slice3-query-clippy3` (exit 101):

```text
error: this OR pattern can be rewritten using a range
  --> crates\worth-query-court-installation\tests\worth_query_court_installation\installation\managed_index_parent_prof
iles\queries.rs:32:84
   |
32 |     const STRUCTURE: ApplicationManagedIndexSourceStructure = if matches!(S::MODE, 5 | 6 | 7) {
   |                                                                                    ^^^^^^^^^ help: try: `5..=7`
   |
   = help: for further information visit
https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#manual_range_patterns
   = note: `-D clippy::manual-range-patterns` implied by `-D clippy::all`
   = help: to override `-D clippy::all` add `#[allow(clippy::manual_range_patterns)]`
error: called `.err().expect()` on a `Result` value
  --> crates\worth-query-court-installation\tests\worth_query_court_installation\installation\managed_index_parent_prof
iles\mod.rs:83:10
   |
83 |           .err()
   |  __________^
84 | |         .expect("an input-field operation cannot borrow a principal-scoped Parent");
   | |_______________^ help: try: `expect_err`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#err_expect
   = note: `-D clippy::err-expect` implied by `-D clippy::all`
errors
warning: build failed, waiting for other jobs to finish...
warning: unused import: `worth_query_declaration::facade::application_schema::ApplicationFieldPresence`
 --> crates\worth-query-package-archive\src\record\application_schema\tests\fixture\query.rs:3:5
  |
3 | use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: `worth-query-package-archive` (lib test) generated 1 warning (run `cargo clippy --fix --lib -p
error: could not compile `worth-query-court-installation` (test "worth_query_court_installation") due to 2 previous
```

`slice3-query-clippy4` (exit 101):

```text
error[E0063]: missing field `content_format` in initializer of
`worth_query_installation::facade::ApplicationSchemaMember`
  --> crates\worth-query-execution\src\domain_computation\primary_graph\product_operation\correction_source\authored_id
entity\candidate_tests.rs:28:39
   |
28 |                 logical_key: Box::new(ApplicationSchemaMember::Field {
   |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `content_format`

warning: type
`domain_computation::primary_graph::application_attempt::fact::source_read_authorization::SourceReadAuthorization` is
more private than the item `domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObserved
error: could not compile `worth-query-execution` (lib test) due to 1 previous error; 11 warnings emitted
```

`slice3-native-clippy2` (exit 101):

```text
error: returning the result of a `let` binding from a block
  --> crates\worth-relational\src\mvcc\retained_entity_selection\payload.rs:34:5
   |
20 | /     let source = crate::storage::residency::probe_head(
21 | |         execution,
22 | |         &historical.with_symbol_admission(&runtime.services.symbols),
23 | |         |source| {
...  |
33 | |     .map_err(|failure| storage(&input.historical, failure))?;
   | |_____________________________________________________________- unnecessary `let` binding
34 |       source
error: could not compile `worth-relational` (lib) due to 1 previous error; 8 warnings emitted
```

The same independent lane reviewer gave explicit **PASS** under qa-loop,
qa-tests and code-quality-qa after reading the final code, courts, plan and logs.
All findings were closed. It independently verified all final lint diagnostic
paths have zero overlap with the scoped dirty set and committed branch diff.
The reviewer ran no builds and made no mutations. Source/correction and slice 5
qualification remain outstanding.


## Slice 4: Source and correction

The same lane reviewer passed Source and correction integration under qa-loop,
qa-tests and code-quality-qa. See [the full slice 4 ledger](milestone-9.19-k1c-content-slice4-evidence.md)
for exact commands, all failed attempts, final counts, independent oracles, scoped
untouched lint debt and rlib telemetry. The pre-edit checkpoint RootUnavailable
and absent live-history limitations are explicit. Slice 5 qualification remains.
