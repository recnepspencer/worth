# K1c implementation evidence

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
