# Engine-once slice 2: admitted operation core and brand

Slice 2 of `C:/forge_workspace/.orchestration/aspect-cutover/briefs/pub-mono-design.md`
(§1, §2, §4) is implemented. Slice 3 and later are untouched.

## Boundary and cutover

Base: `c1b656df11` (slice 1 landed). Three commits sit on it:

- `1639f90a61` pins today's refusal precedence for a foreign, expired
  admission at every re-entry (injection 7) and passed on the base unchanged;
- `bfa9f31844` (P1) checks admission ownership before authentication lapse and
  before source-fact work at every entry, as the pin requires;
- `5aeab9fcf6` is the slice itself.

The admission's state now lives in one crate-private, move-only
`WorthQueryAdmittedOperation` (`authorization/admitted_operation_core.rs`) with no
application type, and no `Clone`, `Default`, `Deserialize` or `From`. The typed
`WorthQueryAdmittedApplicationOperation` is `{ core, tag, capability_input,
original_input }` and has inline forwarders onto non-generic core bodies.

The authorization basis is erased to its variant and binding. The typed shared
capability input stays in the brand slot (coordinator ruling F2 (b)), so
`capability_input()` is a plain typed accessor, and the brand is `Send` exactly
when its shared `Input` is `Send + Sync`, as before erasure. The generic basis and the generic body forms are deleted, with
no dual path. Both static assertions are `Send`-only. The core is not `Sync`
because it holds the dyn capacity reservation, and the former brand was not
`Sync` either.

The core is minted only at the three existing mint sites: conventional,
selected conventional and capability progression. `BrandTag` is copied from the
installed operation at mint (ruling F1 (d)). It holds runtime authority, seal
bytes, definition identity, and the `EqualityTypeId` of the Operation and Input
types, which are used for equality only. Installation captures both
`TypeId`s once in `installed_operation`, behind two new `#[doc(hidden)]`
accessors, and exposes the mutation binding descriptor to the engine.

Every mismatch is a loud typed refusal, and each one reuses an existing exact kind:

| Mismatch | Refusal |
| --- | --- |
| tag vs core runtime | `Refused(ForeignRuntime)` |
| tag vs core seal or definition | `Refused(StaleInstalledOperation)` |
| mint basis vs input slot presence | `Refused(InconsistentDecision)` |
| rewrap successor from another admission | `Refused(InconsistentDecision)` |
| no installed descriptor for the binding | `HandlerNotInstalled` |
| descriptor operation type or name | `Projection(Refused(StaleInstalledOperation))` |
| descriptor input type | `InputNotAdmitted` |
| ownership at idempotency / read attempt | `ForeignAdmission` / `ForeignApplication` |

No variant was added. Scope has no `TypeId` check: the descriptor carries none,
and `ForeignScope` already covers a foreign scope. Comparing the seal with the
installed operation at re-entry is left to slice 3's installation views. The
mutation entry checks the descriptor after the handler is looked up and before
the handler is called.

`worth-query-decl` and `worth-query-host` sources are byte-identical to the base.

## Reproduction

The toolchain, LLVM tools, test profile and instrument are the same as in slice 1.
The `after` archives were built from `5aeab9fcf6` after the full court binaries
ran.

```powershell
$env:WORTH_IMPLEMENTATION_BUILD_ID='11111111-1111-1111-1111-111111111111'
$env:CARGO_TARGET_DIR='C:/worth-build-cache/target-cut-pub-mono'
cargo test --manifest-path workspaces/worth-query/Cargo.toml `
  -p worth-query-court-decl-host --test worth_query_court_decl_host `
  --no-run --message-format=json > target/pub-mono-logs/slice2-court-build.jsonl
foreach ($f in 'certification-canonical','document-retention','certification-operation') {
  python scripts/quality/query_monomorphization.py `
    --cargo-messages target/pub-mono-logs/slice2-court-build.jsonl `
    --package worth-query-testkit-$f `
    --llvm-bin C:/Users/Esther/.rustup/toolchains/1.99.1-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/bin `
    --output target/pub-mono-logs/mono-slice2-$f.json
}
```

The `before` reports are `target/pub-mono-logs/mono-slice2-base-<fixture>.json`, measured
the same way at `c1b656df11`. The deltas therefore cover P1 and slice 2 together.
P1 only reorders existing checks.

## Measured result

These counts come from the fixture archives. They are not final linked size.

| Fixture | Metric | Slice-2 base | Slice 2 | Change |
| --- | --- | ---: | ---: | ---: |
| canonical | Defined external symbols | 150,585 | 150,134 | -451 |
| canonical | Archive bytes | 621,452,892 | 619,715,596 | -1,737,296 |
| canonical | Physical code bytes | 44,915,034 | 44,852,822 | -62,212 |
| canonical | Slope, all owners | 1,790,090.764898 | 1,789,712.916065 | -377.848833 |
| canonical | Slope, Query Execution | 1,374,384.581767 | 1,373,834.581767 | -550.000000 |
| document-retention | Defined external symbols | 77,116 | 76,865 | -251 |
| document-retention | Archive bytes | 286,444,650 | 285,633,012 | -811,638 |
| document-retention | Physical code bytes | 19,279,417 | 19,238,849 | -40,568 |
| document-retention | Slope, all owners | 549,420.651539 | 548,378.678266 | -1,041.973272 |
| document-retention | Slope, Query Execution | 324,503.840405 | 323,271.415405 | -1,232.425000 |
| certification-operation | Defined external symbols | 54,247 | 54,109 | -138 |
| certification-operation | Archive bytes | 237,386,672 | 236,852,162 | -534,510 |
| certification-operation | Physical code bytes | 13,958,450 | 13,939,883 | -18,567 |
| certification-operation | Slope, all owners | 455,108.902753 | 454,813.837300 | -295.065453 |
| certification-operation | Slope, Query Execution | 309,301.913443 | 308,889.067289 | -412.846154 |

Against slice 1's own canonical report (150,562 / 621,368,176 / 44,893,095 /
1,789,349.764898), slice 2 is -428 symbols, -1,652,580 archive bytes,
-40,273 code bytes and +363.151167 all-owner slope. That slope difference is
inherited, not added. The landed slice-1 tree (`c1b656df11`) already measures
+741.0 above slice 1's report, and slice 2 then lowers the slope by 377.8.

Slope is Σ bytes/copies over normalized families with at least seven copies,
summed exactly before rounding. Qualifying families, all owners, base → slice 2:

- canonical: 3,675 → 3,666;
- document-retention: 1,527 → 1,516;
- certification-operation: 1,150 → 1,140.

For Query Execution:

- canonical: 2,229 → 2,218;
- document-retention: 647 → 634;
- certification-operation: 569 → 558.

Families that disappeared:

- canonical: 16 families, 18,274 old bytes;
- document-retention: 34 families, 13,691 old bytes;
- certification-operation: 19 families, 4,895 old bytes.

Each fixture's list is complete within the top 40, so none is padded. They are
mostly the generic basis and brand bodies, for example
`WorthQueryOperationAuthorizationBasis<>::shared_capability_input` (4,646 bytes
across 23 copies) and the capability-revocation and delegation-activation binding
readers. Old disappeared bytes are not net savings.

The new shells cost something. The brand `mint` is 18,456 bytes across 24 copies,
and the conventional mint sites grew by about 10.8 KB each, because each typed
copy still moves the whole core. Drop glue fell by 41,266 bytes in canonical.
Unattributed code is about 68 KB (canonical), 76 KB (document-retention) and
46 KB (certification-operation) in both runs. Shared alias bytes are zero, and
both independent LLVM totals agree in every run.

The slice's saving is small: about 0.14% of code, against the design's
0.47 MB admission/progression area. Its purpose is the forgery court and the
erased core that slices 3–8 hand to once-compiled bodies.
[The committed measurement](mono-admitted-operation.json) holds four records,
one per fixture against the slice-2 base plus canonical against slice 1. Each
record has hashes, exact slope fractions, owner symbol counts and the
disappeared ranking.

## Courts and checks

The forgery court is
`authorization/operation_admission/erased_core_forgery_tests.rs` and its
`allocation.rs` and `precedence.rs` children. It is mounted from
`primary_graph/tests/application_attempt.rs`, because the installed worlds it
drives are private to primary_graph. It reaches the brand only through the
test-only doors in `operation_admission/forgery_doors.rs` and drives the real
entries.

| Injection | Test | Requires |
| --- | --- | --- |
| 1 | `a_core_rewrapped_as_another_operation_is_stale_before_any_work` | Stale; no handler, source, fact or request work (control reaches its handler) |
| 2 | `a_core_from_another_runtime_is_foreign_under_either_brand` | ForeignRuntime in both directions, no work |
| 3 | `a_flipped_seal_byte_is_stale_at_mutation_and_foreign_at_idempotency` | Stale at mutation, ForeignAdmission at idempotency |
| 4 | `a_successor_core_from_another_admission_is_refused_at_rewrap` | InconsistentDecision from the real rewrap |
| 5 | `a_brand_the_installed_binding_does_not_name_never_reaches_its_handler` | InputNotAdmitted / Stale, handler never started |
| 7 | `precedence::a_forged_and_expired_core_is_refused_as_forged_at_every_entry` | forged wins over lapse at projection, idempotency, read attempt and mutation |
| 8 | the doctests on `WorthQueryAdmittedApplicationOperation` | no core import (E0432), no `.core` (E0616), brand not `Send` over an `Rc` or `Cell` input (E0277) |
| 9 | by hand | `worth-query-decl`/`worth-query-host` diff empty |
| 10 | `allocation::brand_checks_do_not_allocate` plus the zero counts above | no request consult or charge on any forged run, and zero allocations in brand checks |

For injection 10, the brand seams take no `ExecutionRequest`, so none of them
can mint or charge one.

In a sabotage run, the tag, descriptor and rewrap identity checks were disabled.
Seven court tests failed. The eighth, the allocation probe, failed only because
the sabotage switch itself allocates.

Final runs (all exit 0):

- `-p worth-query-execution --lib`: **2229 passed, 0 failed, 1 ignored**.
  After the boundary fix, the court reran at **8 passed**.
- `-p worth-query-installation`: **363 + 14 + 1 passed**, 0 failed/ignored.
- Full `worth_query_court_decl_host`: **325 passed, 0 failed, 2 ignored**. The
  ignored tests are the existing growth qualifications.
- Full `worth-query-court-query-decl-host`: **340 passed, 0 failed, 2 ignored**.
- Doctests `WorthQueryAdmittedApplicationOperation`: **6 passed**.

Clippy (`-p worth-query-execution -p worth-query-installation --all-targets`)
reports no warning in a touched file. The totals match P1 (lib 54, lib test 53).
Installation's single `TypeId::of` carries an `expect` with its reason.

These also pass: `cargo fmt --check`, boundary-check, agent-context check,
dirty line caps (33 Rust files) and `git diff --check`.

## Decision-read entry ownership

`execute_mutation_handler_report_with_query_reads` used to prepare the shared
decision-read basis, which means product branch admission and the native
companion preflight, before preparation checked ownership. A forged or foreign
core with managed-index decision reads therefore caused product work before
it was refused. The entry now runs checks in this order:

1. The phase check, as preparation does.
2. `validate_runtime_ownership`, before the branch on the admission's
   managed-index reads. Neither branch can do work for a foreign core.
3. On the decision-read branch only, `validate_installed_mutation_binding`,
   before the shared basis is prepared.

Preparation still repeats both checks. They are pure and do not allocate.

Court: `erased_core_forgery::decision_entry`. It runs in its own world,
`decision_read_world`, in the execution crate's test fixtures. That world has
one principal, one Source query, one Parent query read through one managed
index, one candidate provider for that index, and one decision-read operation
with its principal-scoped mutation binding. `IdentityExecutionSchema` is
unchanged. Only this court installs the new world, so no other test pays for
it.

The world is a source fixture because minting an admission is crate-private.
It names the application runtime, so `src-court-baseline.json` gains one entry
for `decision_read_world.rs`, beside the other fixture worlds.

- Control: an owned admission declares a managed-index read, takes the
  decision-read branch, reaches its handler and consults its request.
- Each of three foreign cases is refused as `Projection(Refused(ForeignRuntime))`,
  with `NotStarted` work and zero source reads, fact admissions, request
  consults and request charges. The three cases are:
  - a plain foreign admission;
  - the target's brand around a foreign core;
  - a foreign brand around the target's core.
- Forged tag type: an owned admission whose tag input type is forged is
  refused as `InputNotAdmitted`, also with no work.

Each case is its own test, so a mutation reports every case it breaks.

Mutation results, each run on the court and then reverted:

- Hoisted ownership check disabled: all three foreign cases fail. The plain
  foreign case and the foreign core are refused late, as
  `Handler(Product(Refused(ForeignOwner)))`. The foreign brand does work
  before it is refused. The control and the forged-type case still pass.
- `validate_installed_mutation_binding` moved after basis preparation: the
  forged-type case fails because work is done. The other four still pass.

Owed to slice 3: on re-entry, compare the core's seal with the installed one.
Today's code does not do this either.

Runs (all exit 0):

- Forgery court: **13 passed**.
- `-p worth-query-execution --lib`: **2234 passed, 0 failed, 1 ignored**.
- No court crate was touched.

Clippy totals are unchanged (lib 54, lib test 53). The one warning in
`decision_query.rs` predates this change: the `Native` stop's field is never
read.

These also pass: fmt, boundary-check, agent-context, dirty line caps and
`git diff --check`.
