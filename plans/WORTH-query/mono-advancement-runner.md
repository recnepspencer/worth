# Engine-once slice 1: advancement runner

Only the advancement runner slice is implemented. The read-only design is
`C:/forge_workspace/.orchestration/aspect-cutover/briefs/pub-mono-design.md`.
Slice 2 and all its authority/core/brand types remain untouched.

## Boundary and cutover

Base: `d4a9f989fa591126abb2fa1645fa2543723b99b5`. Fetch and fast-forward checks
for master and the provider sync branch found both already incorporated. Relative
to the last measured source (`8165d842dd`), the only intervening change is a
four-line counted-read court classification fix; no fixture or production source
changed. The prior `mono-workflow-facts.json` is the requested comparison baseline.

The existing private generic `run_advancement<R>` method is now a thin FnOnce
adapter with stack Option slots. The private `runner` module's `run_advancement`
and `run_admitted_advancement` take a dynamic phase callback and own the entire
execution/report/unwind body. The old generic admitted body is deleted.
AssertUnwindSafe remains around the same body invocation; catch_unwind and
resume_unwind remain in that non-generic body. No Decl/Host facade signature changed.

A runner-local, non-generic `AdvancementHandoff` owns an erased discard callback,
not authority. It follows the former owned FnOnce into Execution and the former
result back out. This is needed because the native request polls after its body:
a late refusal destroys R inside Execution, where a panicking destructor is
contained. An outer slot alone would move that destructor outside containment.
The guard also disposes an unrun callback before refusal observation. On success
it stays armed through report recording and then releases the result to its
shell. RefCell borrows end before arbitrary invocation or destruction; there is
no allocation or new request. Accounted execution still returns a value whenever
the body ran, as before.

## Reproduction

Same Windows toolchain, LLVM tools, profile and public fixture as the last
baseline. The existing test profile has opt-level 0 and debug information 0;
no role/profile/feature settings changed. Run long Cargo commands in the
background, wait, and inspect logs.

```powershell
$env:WORTH_IMPLEMENTATION_BUILD_ID='11111111-1111-1111-1111-111111111111'
$env:CARGO_TARGET_DIR='C:/worth-build-cache/target-cut-pub-mono'
cargo test --manifest-path workspaces/worth-query/Cargo.toml `
  -p worth-query-execution --lib application_contribution::request_execution::advancement::
cargo test --manifest-path workspaces/worth-query/Cargo.toml `
  -p worth-query-court-decl-host --test worth_query_court_decl_host `
  --message-format=json *> target/pub-mono-logs/advancement-court-build.jsonl
cargo test --manifest-path workspaces/worth-query/Cargo.toml `
  -p worth-query-court-decl-host --test worth_query_court_decl_host `
  --no-run --message-format=json *> target/pub-mono-logs/advancement-measure-court-build.jsonl
python scripts/quality/query_monomorphization.py `
  --cargo-messages target/pub-mono-logs/advancement-measure-court-build.jsonl `
  --package worth-query-testkit-certification-canonical `
  --llvm-bin C:/Users/Esther/.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/bin `
  --output target/pub-mono-logs/mono-advancement-runner.json
```

The no-run measurement command certifies the exact successful Cargo artifact
without another codegen build after the full court run. The instrument is
unchanged. It independently checks nm/readobj external symbol totals and
size/readobj physical code totals. Its existing namespace, extent/padding and
alias limitations apply unchanged; this is a test-profile archive measurement,
not a downstream link-size or build-time prediction.

## Measured result

The method sees a saving. These are counts in the same canonical fixture archive,
against the last lane baseline, not the size of a final linked application:

| Metric | Last baseline | Slice 1 | Change |
| --- | ---: | ---: | ---: |
| Defined external symbols | 156,909 | 150,562 | -6,347 |
| Archive bytes | 654,806,940 | 621,368,176 | -33,438,764 |
| Physical code bytes | 46,360,124 | 44,893,095 | -1,467,029 |
| Slope, all owners, bytes/copy | 1,791,280.888167 | 1,789,349.764898 | -1,931.123269 |
| Slope, Query Execution, bytes/copy | 1,375,652.027173 | 1,373,643.581767 | -2,008.445405 |

Slope is exactly the requested sum of bytes/copies over normalized families with
at least seven copies. The all-owner roster has 3,679 qualifying families before
and 3,674 after; Query Execution has 2,236 before and 2,228 after. Fractions are
summed exactly before rounding to six decimal places. This fixture proxy is not
a measured derivative across different application sizes.

[The committed measurement](mono-advancement-runner.json) contains both archive
and raw-report hashes, independent LLVM totals, owner symbol counts, exact slope
fractions and the requested disappeared-family ranking. Only **14** normalized
families disappeared, so the top-40 selection contains all 14 without padding.
They total 349,730 old bytes and are sorted by old bytes descending, then name.
The largest was the old
admitted runner: 139,955 bytes across 100 copies. Its six closure families also
disappeared. Old disappeared-family bytes are not net savings: the typed shell
grew and the non-generic body now lives in its owning engine crate.

The physical-code reduction is 1,467,029 bytes, compared with the design's rough
350 KB estimate. The same reports also show reductions in retained Execution and
core families: for example, ExecutionWorkCeiling::run_within falls from 379,438
to 173,486 bytes. This is the measured fixture result, without extrapolating to
total linked size, release mode or downstream build time. Unattributed code is
58,481 bytes before and 67,886 after; shared alias bytes are zero in both runs.
Both independent symbol totals and both independent code-byte totals agree.
No growth test or timing test was added: this slice records the requested fixed
fixture measurement, with no claim of a scale or runtime-speed experiment.

## Courts and checks

Three new custody tests were pinned against the original runner before production
edits: **3 passed, 0 failed/ignored/measured, 2215 filtered**. They require:

- pending callback capture destruction before the result observer records refusal;
- a native late-cancellation result destructor panic contained by Execution,
  preserving the original exact Interrupted(Cancelled) precedence and cleanup;
- accounted execution retaining its returned value after body cancellation.

The required fourth test calls the real non-generic runner with an observable
result slot. A body panic leaves the slot empty, resumes the same Arc allocation
as its payload, records one Panicked observation, and leaves the original runner
usable. The literal expected denials/order and pre-cutover results are independent
of the new guard implementation; these tests do not copy its cleanup algorithm.
Existing advancement, nested budget, reentrancy, unwind-custody and native seam
cancellation tests are retained.

Final owner run: **14 passed, 0 failed/ignored/measured, 2205 filtered**.
Full decl-host court binary: **325 passed, 0 failed, 2 ignored, 0 measured,
0 filtered**. Runtime telemetry was 27.95 seconds. The existing ignored tests are
the two `cost::document_history::growth` thousand-sibling/thousand-step scheduled
query-scale qualifications; their retained-history fixtures and ignore reasons
were unchanged. The full court retains the existing advancement and panic
integration evidence as well as application/host behavior across the public seam.

Clippy command:

```text
cargo clippy --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --all-targets --message-format=json
```

The corrected run passes with **0 errors, 114 warning diagnostics**, including
repeated upstream warnings. Boundary-check and context check pass. Dirty line
caps pass for **4 Rust files**. Exact additional gate commands (all exit 0):

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

The facade source comparison is exact for this slice; no API snapshot generator
was run. Boundary output: `boundary-check: repository boundaries are valid`.
Context check emits no violations. These checks are gates rather than test counts.
The measurement no-run Cargo command and LLVM instrument both exit 0.

Failed setup/check runs executed zero tests: two E0599 errors attempted to
cancel a read-only CancellationToken (fixed by retaining its original source);
one E0433 used an incorrect test module path; then Clippy reported FnMut/FnOnce
not general enough for the test's stored closure (fixed by an explicit
higher-ranked phase argument type). No suppression or production semantic change
was used to resolve these test setup errors.

The three pre-edit test invocations used exactly:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --lib advancement::tests::panic_custody
```

The first exited 101 with two `error[E0599]: no method named cancel found for
struct worth_execution::CancellationToken`; the second exited 101 with
`error[E0433]: cannot find test_placement in super`; the corrected third passed
the three tests. The first Clippy run exited 101 with `implementation of FnMut
is not general enough` and `implementation of FnOnce is not general enough`.
The corrected Clippy run passes as counted above. Formatting without `--check`
ran twice successfully while applying those fixes. No failed test executions
were hidden behind a later pass; the failures were compilation diagnostics.

## Readings I chose

The required named non-generic runner functions live in a private owner module;
the existing generic method remains only as the signature-preserving shell.
Keeping exact poisoning/destruction behavior requires a private cleanup handoff
in addition to the phase callback. It is runner custody, not any Slice 2 type.
No authority, core-to-brand conversion, re-entry rule or forgery surface is added.
The original exact native cancellation precedence wins over destructor panic;
the pre-cutover court establishes this rather than guessing a nearby panic kind.

## Review and stopping point

The same independent lane reviewer applies qa-loop, qa-tests and code-quality-qa.
It identified the early capture-drop and late native result-drop edges before
implementation and verified the local guard correction. Independent composition
scan: 4 Rust files, 2 advisories, 0 errors. Final review passes with no unclosed
material finding. The reviewer independently recomputed every committed metric,
hash, owner count and exact slope fraction from both raw reports and verified
the final test/check logs. The 14 disappeared names and 349,730 old bytes agree.
Stop after Slice 1; no Slice 2 work is authorized here.
