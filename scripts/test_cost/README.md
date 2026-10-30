# Governed test cost

The standard-library census covers root, Query, Bank and Store. `worth-ui` is
excluded. Timings, CPU parallelism and setup frequency are telemetry; they never
decide whether a test passes.

`source_roots.py` resolves Cargo-declared and automatic target sources for both
source enforcement and LLVM ownership. It includes binary unit tests under
`src/bin/*.rs` and `src/bin/*/main.rs`, honors each automatic-target flag and
edition, and refuses missing or ambiguous declared sources. Small fixture
probes compare discovery with real `cargo metadata`; no compilation is needed.

`source_documents.py` keeps selected test text and enclosing lexical context
together. Imported command names and executable constants remain resolvable
without counting unrelated production calls. Named helpers referenced by test
code, including associated and trait-default methods, enter a conservative
same-target source closure. Literal `include!`
sources retain their real paths; selected unresolved includes refuse the census.

```text
python scripts/test_cost/census.py --all --check-sources
python scripts/test_cost/run_lane.py --workspace store
python scripts/test_cost/llvm_measure.py --workspace store --cargo-profile ci-test
python scripts/ci/root_gate.py
python scripts/ci/query_gate.py --list
python -m unittest discover -s scripts/test_cost -p test_*.py
```

`run_lane.py` executes every nonignored partition by default, including ordinary
cost cases and doctests. It never runs scheduled scale cases or ignored tests.
An explicit `--all --output PATH` source census writes one report under each
workspace subdirectory of PATH. Filtered runs prove only their selected subset. A complete nextest inventory is
not evidence that those tests executed: `executed-coverage.json` separately
compares terminal observations with the requested identities.

## Ownership and enforcement

* `source_scan.py` and `ratchet.source_violations` refuse Cargo, cargo_metadata
  and trybuild calls in tests, courts and testkits. This check belongs here
  pending the coordinator's replacement boundary checker; this task does not
  modify `tools/boundary-check`.
* `CargoRunner` captures compiler JSON, command, revision, dirty status, start,
  end and exit code. Immutable retained copies live in `target/lane-evidence`.
  Metadata reuse prevents nextest execution from rebuilding the graph.
* `budgets.json` declares build profiles and per-lane compiler-unit budgets.
  Libraries, their libtest executables and distinct binaries are distinct
  compiler units. Recompiling an identical unit in another target directory
  exceeds the budget. Package-level duplicate graphs remain visible with their
  features, profiles, target directories and check modes.
* The runner builds process helpers as independent executable targets. Tests
  read typed prebuilt manifests and refuse missing or malformed observations.
  They never silently build a fallback.
* `compile_batch.py` runs actual fail fixtures together with
  `cargo check --workspace --bins --keep-going`, once per required environment.
  Separate generated packages preserve each declared direct-dependency audience;
  an unrelated fixture cannot make a forbidden dependency available. It copies
  the owning workspace lockfile, checks dependency lock consistency and
  attributes diagnostics through real compiler spans. A dependency error or
  an unattempted fixture cannot satisfy a denial.
* Store's production fixture graph rejects certification and test features.
  `certification_authority_is_absent_in_the_production_environment` is an
  unconditional court test reading that independent production observation.
  Certification-enabled court dependencies cannot supply this evidence.
* Supported programs formerly passed to trybuild are ordinary executing court
  tests. Their bodies remain compiled by their owning court. Duplicate textual
  copies of root Proof supported fixtures have one source owner; crash sweep
  continuations are never deduplicated.
* `llvm_measure.py` measures the complete workspace feature graph with one
  `cargo test --workspace --all-targets --no-run` invocation. Measurement-only
  compiler flags emit IR and retain linking; no test body executes. Exact Cargo
  source, target, profile, features and hashed artifact filenames identify each
  IR file. `cargo llvm-lines --files` attributes generic instances without more
  compiler sessions. IR size is a compile-cost proxy, not elapsed build time.
* CPU accounting includes child processes. Generated slot reservations use
  measured CPU/wall ratios; missing CPU accounting stays unmeasured. Existing
  reservations remain until measurements exist. Committed sparse reservations
  retain provenance in `slots/<workspace>`. Hashed sample archives preserve
  measurements in fresh checkouts without historical `target` files. Bounded
  schema-2 catalog trees retain every immutable sample and source header without
  growing index files. Nextest reservations use the actual pool clamp; standalone
  executables retain CPU observations without a nextest reservation. Partial
  runs retain earlier samples and use the maximum successful observation.
  Explicit claim resolutions change pending notes independently of CPU timing.
* `coverage.py` proves structural selection; `complete_coverage.py` adds
  rustdoc-owned identities; `executed_coverage.py` rejects missing, repeated,
  unknown, failed or actually executed ignored cases. Harness-free programs
  execute their independently built main exactly once.
* Store's bounded sweep pool drains every case, joins every worker and reports
  failures in input order. Independent cases receive worker-local mutable
  state. Shared-state loops remain judgement work.
* Store setup recording lives in a closed-court testkit. Fingerprints include
  the complete recorded builder inputs. Disabled observation changes no builder
  behaviour; uninstrumented builders remain unmeasured.

## Requirement-to-test map

| Requirement | Executable evidence owner |
|---|---|
| Build artifacts and duplicate causes | `test_logs.py`, `test_ratchet.py`, `test_cargo_capture.py` |
| Nested compiler-call source boundary | `test_source_scan.py`, `test_rust_calls.py`, `test_workspace.py` |
| LLVM totals and helper attribution | `test_llvm_measure.py`, `test_llvm_artifacts.py`, `test_llvm_bulk.py`, `llvm_probe.py`, `test_logs.py` |
| CPU/slots and generated policy | `test_account.py`, `test_account_windows.py`, `test_telemetry.py`, `test_measured_slots.py`, `test_policy_catalog.py`, `test_slot_policy_catalog.py`, `test_reservations.py` |
| Repeated construction observations | `test_telemetry.py`; Store recovery-runtime court setup recorder tests |
| Sleep, timeout and polling census | `test_source_scan.py` |
| Exact nonignored selection and execution | `test_coverage.py`, `test_complete_coverage.py`, `test_executed_coverage.py`, `test_doc_contracts.py` |
| Full attempt history and selective repair provenance | `test_capture_report.py`, `test_verification_lineage.py`, `test_evidence_index.py` |
| Independent prebuilt executables | `test_store_prebuild.py`, `test_standalone.py`, `test_prebuild.py` |
| Real batched compiler fates and lockfile | `test_compile_batch.py`, `test_compiler_prebuild.py`, `compiler_probe.py` |
| Isolated declared dependency audiences in one batch | `test_compiler_audiences.py`, `test_fixture_graph.py` |
| Production certification absence | `test_compiler_prebuild.py`; `worth-testkit-compiler` production-denial tests; Store production-absence court |
| Clean supported-program migration | `test_pass_cutover.py`, `test_supported_programs.py` |
| Exhaustive independent sweep rewriting | `test_sweep_cutover.py`; Store sweep pool court tests |
| Store court selectors and categories | `test_store_plan.py`; `store-test-runner` plan tests |

## Evidence limits

Historical Landing 1 logs do not retain compiler-artifact JSON, per-process CPU
or construction fingerprints. Such cells are unknown, not zero. The historical
source census identifies its source revision separately from the earlier lane
timings. New build-only coverage is explicitly distinct from runtime coverage.

Serial workspace testing may be removed from a landing set only after its full
nonignored coverage, including harness-free programs and doctests, is proved
and the selected partitions have execution evidence. The TLC script lane keeps
its separate claim.

Known-cost validation preserves counterevidence. A named sequential oracle with
no matching whole-pool rule is reported with every searched rule; it cannot be
claimed to reserve the pool merely because an input description says so.
`verification_lineage.py` retains each failed attempt and every rerun separately.
A composed repair history is never presented as one green execution.

The declared-audience decision retires import-only dependency minimization from
commit `522c480f3d`. Its `unused_testkit_is_not_a_compiler_fixture_dependency`
assertion and the relocated-fixture expectation described the wrong graph:
negative imports and rustc's asserted suggestions depend on names available to
the original caller, including unreferenced declarations. Their replacements
assert the complete producer audience, exclude unrelated court dependencies,
and preserve target conditions. Every compiler fixture and diagnostic claim is
retained.

## Resource policy denial fixture composition

The former `resource_policy_proof_artifacts_cannot_be_deserialized` bundle
contained 21 `DeserializeOwned` denials and exceeded the 400-line snapshot cap
after the actual compiler's Serde suggestion update. Its bundle identity is
retired solely for this responsibility split; no type claim or diagnostic is
retired. `resource_policy_descriptor_artifacts_cannot_be_deserialized` retains
the original first 11 assertions for descriptors, validated declarations,
compatibility and lowered policy artifacts. The decision counterpart retains
the last 10 assertions for decision plans and revalidation proofs. The original
court test calls both fixtures, preserving all 21 assertions and all 21 complete
E0277 blocks. Golden changes rebase only fixture filenames, source coordinates
and source snippet gutters; error codes, labels, trait requirements and example
lists are unchanged from the current original snapshot. The separate nullable
optional codegen-stage fixture and its proof-gap ruling are unchanged.

The decision to preserve the original Root caller pattern grammar retires
`recursive_or_character_class_globs_require_explicit_expansion`, introduced in
commit `7910247d6d`. That reader test wrongly required refusal of valid caller
patterns such as `[a-f]*.rs`, `[g-r]*.rs` and `[s-z]*.rs`, which already existed
at base `1c4b93bb9a`. Standard `glob` pattern support and malformed-pattern
refusal tests replace that incorrect claim. No fixture membership, compiler
denial or named implementor assertion is removed or narrowed by this decision.


The batched Root production graph enables `serde_json/raw_value`. Thirteen
existing denial snapshots therefore name `RawValue` in the compiler's first
eight example implementors instead of the former eighth tuple example. These
snapshots now assert the actual complete examples; no named implementor is
masked by the renderer. Every target type, missing trait, error code, source
span, bound and denial claim remains checked. The old example order described
separate pre-batch dependency graphs and is retired by this required batching
cutover; it was not the target type's serialization authority claim.
