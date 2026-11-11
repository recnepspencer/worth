# P-A merge lane evidence index

Reviewed source: `79de5c7ad597cb207fa17a2d38ec1b80da748bba`. All commands, HEADs, start/end times, dirty flags and raw paths are in pa-merge-lane-index.json and the logs. First Tier2 is at 589f6fce9c; later evidence applies only to its named inputs.

Counts below are actual distinct invocations, including failed attribution/tool attempts. Metadata/list/declaration operations do not run test bodies. A missing test count is N/A, never a green zero-test claim.

Operation counts: `{"build": 3, "cargo-metadata": 20, "cargo-rustc": 2, "cargo-test": 2, "cargo-tree": 1, "check": 22, "clippy": 16, "doc-test": 4, "native-execution": 14, "nextest-build": 17, "nextest-list": 73, "nextest-run": 30, "run": 7}`.

|Kind|HEAD|Exit|Pass/fail/ignored metadata|Log|
|---|---|---:|---|---|
|check|7b35408444|0|N/A|target/lane-evidence/pa-merge-bank-check.log|
|clippy|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-bank-clippy.log|
|run|d5fe9df913|1|N/A|target/lane-evidence/pa-merge-closure-boundary.log|
|run|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-closure-context.log|
|clippy|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-closure-native-lint.log|
|clippy|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-closure-topology-lint.log|
|run|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-consumer-root.log|
|run|d5fe9df913|1|N/A|target/lane-evidence/pa-merge-final-boundary-corrected.log|
|run|d5fe9df913|2|N/A|target/lane-evidence/pa-merge-final-boundary.log|
|run|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-final-context-corrected.log|
|run|d5fe9df913|1|N/A|target/lane-evidence/pa-merge-final-context.log|
|clippy|507880607f|0|N/A|target/lane-evidence/pa-merge-final-fixture-lint.log|
|clippy|507880607f|0|N/A|target/lane-evidence/pa-merge-final-native-lint.log|
|clippy|507880607f|0|N/A|target/lane-evidence/pa-merge-final-query-lint.log|
|clippy|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-final-signal-lint.log|
|clippy|507880607f|0|N/A|target/lane-evidence/pa-merge-final-store-lint.log|
|check|7b35408444|101|N/A|target/lane-evidence/pa-merge-fixture-check.log|
|clippy|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-fixture-clippy.log|
|check|7b35408444|0|N/A|target/lane-evidence/pa-merge-fixture-positive-check.log|
|cargo-test|589f6fce9c|0|1/0/0|target/lane-evidence/pa-merge-interim-door.log|
|clippy|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-owner-libs.log|
|clippy|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-owner-relational-court.log|
|clippy|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-owner-signal.log|
|check|7b35408444|0|N/A|target/lane-evidence/pa-merge-query-check.log|
|clippy|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-query-clippy.log|
|check|7b35408444|0|N/A|target/lane-evidence/pa-merge-root-check.log|
|clippy|589f6fce9c|101|N/A|target/lane-evidence/pa-merge-root-clippy.log|
|clippy|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-store-cert-clippy.log|
|check|7b35408444|0|N/A|target/lane-evidence/pa-merge-store-check.log|
|clippy|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-store-clippy.log|
|doc-test|589f6fce9c|0|30/0/0|target/lane-evidence/pa-merge-topology-doc.log|
|check|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-wasm32.log|
|build|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-bank-tier2/test-cost/bank/bank-binaries.log|
|nextest-run|589f6fce9c|100|4/3/2|target/lane-evidence/pa-merge-bank-tier2/test-cost/bank/run-bank-cost.log|
|native-execution|589f6fce9c|0|1/0/0|target/lane-evidence/pa-merge-bank-tier2/test-cost/bank/run-bank-inbound_completion-main.log|
|nextest-run|589f6fce9c|0|73/0/2|target/lane-evidence/pa-merge-bank-tier2/test-cost/bank/run-bank-process.log|
|nextest-run|589f6fce9c|100|484/1/38|target/lane-evidence/pa-merge-bank-tier2/test-cost/bank/run-bank.log|
|native-execution|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-bank-tier2/test-cost/bank/telemetry.log|
|nextest-build|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-bank-tier2/test-cost/bank/build/binaries-build.log|
|doc-test|589f6fce9c|0|74/0/0|target/lane-evidence/pa-merge-bank-tier2/test-cost/bank/documentation/doctests.log|
|check|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-boundary-closure/fixture-check.log|
|check|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-boundary-closure/root-check.log|
|nextest-run|d5fe9df913|100|0/5/0|target/lane-evidence/pa-merge-boundary-closure/fixture/run.log|
|nextest-run|d5fe9df913|0|4/0/279|target/lane-evidence/pa-merge-boundary-closure/root/run.log|
|nextest-build|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-boundary-closure/root/build/binaries-build.log|
|nextest-build|d5fe9df913|0|N/A|target/lane-evidence/pa-merge-boundary-closure/fixture/build/binaries-build.log|
|cargo-test|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-fixture-tier2/group-fixture-build.log|
|check|addc15f70f|0|N/A|target/lane-evidence/pa-merge-followup/fixture-check.log|
|check|addc15f70f|0|N/A|target/lane-evidence/pa-merge-followup/query-check.log|
|check|addc15f70f|0|N/A|target/lane-evidence/pa-merge-followup/root-check.log|
|check|addc15f70f|0|N/A|target/lane-evidence/pa-merge-followup/store-check.log|
|nextest-run|507880607f|0|3/0/0|target/lane-evidence/pa-merge-followup/fixture/run.log|
|nextest-run|507880607f|100|8/1/4|target/lane-evidence/pa-merge-followup/query/run.log|
|nextest-run|addc15f70f|0|1/0/2|target/lane-evidence/pa-merge-followup/root/run.log|
|nextest-run|507880607f|100|0/3/0|target/lane-evidence/pa-merge-followup/store/run.log|
|nextest-build|507880607f|0|N/A|target/lane-evidence/pa-merge-followup/store/build/binaries-build.log|
|nextest-build|addc15f70f|0|N/A|target/lane-evidence/pa-merge-followup/root/build/binaries-build.log|
|nextest-build|addc15f70f|0|N/A|target/lane-evidence/pa-merge-followup/query/build/binaries-build.log|
|nextest-build|507880607f|0|N/A|target/lane-evidence/pa-merge-followup/fixture/build/binaries-build.log|
|nextest-run|589f6fce9c|0|38/0/24|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-cost.log|
|nextest-run|589f6fce9c|0|28/0/6|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-determinism.log|
|nextest-run|589f6fce9c|0|573/0/0|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-execution.log|
|nextest-run|589f6fce9c|100|84/3/30|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-process.log|
|native-execution|589f6fce9c|0|1/0/0|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-rest-advanced_product_branching-main.log|
|native-execution|589f6fce9c|101|0/1/0|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-rest-application_correction-main.log|
|native-execution|589f6fce9c|0|1/0/0|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-rest-authored_workflow-main.log|
|native-execution|589f6fce9c|0|1/0/0|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-rest-observable_workflow-main.log|
|native-execution|589f6fce9c|0|1/0/0|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-rest-ordinary_product_workflow-main.log|
|nextest-run|589f6fce9c|100|6946/33/26|target/lane-evidence/pa-merge-query-tier2/test-cost/query/run-query-rest.log|
|native-execution|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-query-tier2/test-cost/query/telemetry.log|
|nextest-build|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-query-tier2/test-cost/query/build/binaries-build.log|
|doc-test|589f6fce9c|0|546/0/0|target/lane-evidence/pa-merge-query-tier2/test-cost/query/documentation/doctests.log|
|check|5a67931500|0|N/A|target/lane-evidence/pa-merge-repair/fixture-check.log|
|check|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-repair/query-check-initial.log|
|check|5a67931500|0|N/A|target/lane-evidence/pa-merge-repair/query-check.log|
|check|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-repair/root-check.log|
|check|5a67931500|0|N/A|target/lane-evidence/pa-merge-repair/store-check.log|
|native-execution|addc15f70f|0|2/0/0|target/lane-evidence/pa-merge-repair/bank/default-stack.log|
|nextest-run|addc15f70f|100|4/2/0|target/lane-evidence/pa-merge-repair/fixture/run-final.log|
|nextest-run|addc15f70f|95|N/A|target/lane-evidence/pa-merge-repair/fixture/run.log|
|native-execution|a59b8cca72|0|1/0/0|target/lane-evidence/pa-merge-repair/query/application_correction.log|
|nextest-run|a59b8cca72|100|8/11/50|target/lane-evidence/pa-merge-repair/query/run.log|
|nextest-run|a59b8cca72|100|40/1/64|target/lane-evidence/pa-merge-repair/root/run.log|
|nextest-run|addc15f70f|100|0/3/0|target/lane-evidence/pa-merge-repair/store/run.log|
|nextest-build|addc15f70f|0|N/A|target/lane-evidence/pa-merge-repair/store/build/binaries-build.log|
|nextest-build|a59b8cca72|0|N/A|target/lane-evidence/pa-merge-repair/root/build/binaries-build.log|
|nextest-build|a59b8cca72|0|N/A|target/lane-evidence/pa-merge-repair/query/build/binaries-build.log|
|nextest-build|addc15f70f|0|N/A|target/lane-evidence/pa-merge-repair/fixture/build/binaries-build.log|
|nextest-build|addc15f70f|0|N/A|target/lane-evidence/pa-merge-repair/bank/build/binaries-build.log|
|nextest-run|589f6fce9c|0|390/0/0|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-execution.log|
|native-execution|589f6fce9c|0|1/0/0|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-relational-branch_local_mvcc-main.log|
|nextest-run|589f6fce9c|100|2731/63/68|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-relational.log|
|nextest-run|589f6fce9c|0|52/0/13|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-root-compile-fail.log|
|nextest-run|589f6fce9c|100|34/5/12|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-root-cost.log|
|nextest-run|589f6fce9c|0|14/0/6|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-root-determinism.log|
|nextest-run|589f6fce9c|100|150/1/8|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-root-process.log|
|native-execution|589f6fce9c|0|1/0/0|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-root-rest-runtime_world_publication-main.log|
|nextest-run|589f6fce9c|0|2801/0/6|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-root-rest.log|
|nextest-run|589f6fce9c|0|2409/0/36|target/lane-evidence/pa-merge-root-tier2/test-cost/root/run-signal.log|
|native-execution|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-root-tier2/test-cost/root/telemetry.log|
|nextest-build|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-root-tier2/test-cost/root/build/binaries-build.log|
|doc-test|589f6fce9c|0|139/0/0|target/lane-evidence/pa-merge-root-tier2/test-cost/root/documentation/doctests.log|
|cargo-rustc|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-root-tier2/test-cost/root/prebuilt/proof/baseline-ir.log|
|cargo-rustc|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-root-tier2/test-cost/root/prebuilt/proof/guarded-ir.log|
|check|589f6fce9c|101|N/A|target/lane-evidence/pa-merge-root-tier2/test-cost/root/prebuilt/proof/package-check.log|
|check|589f6fce9c|101|N/A|target/lane-evidence/pa-merge-root-tier2/test-cost/root/prebuilt/compiler/production/check.log|
|build|589f6fce9c|101|N/A|target/lane-evidence/pa-merge-root-tier2/test-cost/root/prebuilt/compiler/production-codegen/codegen.log|
|check|507880607f|0|N/A|target/lane-evidence/pa-merge-round6/query-check.log|
|check|507880607f|0|N/A|target/lane-evidence/pa-merge-round6/store-check.log|
|nextest-run|507880607f|0|11/0/4|target/lane-evidence/pa-merge-round6/query/run.log|
|nextest-run|507880607f|100|0/3/0|target/lane-evidence/pa-merge-round6/store/run.log|
|nextest-build|507880607f|0|N/A|target/lane-evidence/pa-merge-round6/store/build/binaries-build.log|
|nextest-build|507880607f|0|N/A|target/lane-evidence/pa-merge-round6/query/build/binaries-build.log|
|nextest-run|589f6fce9c|100|991/17/0|target/lane-evidence/pa-merge-store-tier2/run-selected-native.log|
|native-execution|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-store-tier2/telemetry.log|
|nextest-build|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-store-tier2/build/binaries-build.log|
|build|589f6fce9c|0|N/A|target/lane-evidence/pa-merge-store-tier2/prebuilt/store/executables-build.log|
|legacy-fixture-lane|589f6fce9c|1|578/19/0|target/lane-evidence/pa-merge-fixture-tier2-driver.log|
|diagnostic-child|589f6fce9c|1|0/1/0|target/lane-evidence/pa-merge-preimage-child-exit.log|
|diagnostic-child|589f6fce9c|1|0/1/0|target/lane-evidence/pa-merge-memory-child-exit.log|
|diagnostic-child|589f6fce9c|1|N/A|target/lane-evidence/pa-merge-fixture-retention-child.log|
|diagnostic-child|589f6fce9c|1|0/1/0|target/lane-evidence/pa-merge-fixture-retention-owned.log|
|python-unit-test|589f6fce9c|0|20/0/0|target/lane-evidence/pa-merge-unit-plan-tests.log|
|python-unit-test|5a67931500|0|22/0/0|target/lane-evidence/pa-merge-unit-plan-example-tests.log|
|python-unit-test|5a67931500|0|21/0/0|target/lane-evidence/pa-merge-unit-plan-mixed-example-tests.log|
|python-unit-test|5a67931500|0|21/0/0|target/lane-evidence/pa-merge-unit-plan-paired-example-tests.log|

Original aggregate suite/body counts: pa-merge-initial-lane-counts.json. Zero-selected Query/Bank compile-fail and Bank determinism are UNRUN, not green. Aggregate gate exits propagate a workspace failure even for suites whose bodies passed.

Focused nextest selections execute no ignored body; nextest emits ignored metadata from other cases in the same binaries. Genuine consumer-root ran at merge source; no final-source main-stack claim is made. application_correction is a separate changed-contract standalone proof.

Formal model inputs unchanged: reused lifecycle-store-formal-toolchain.log; no formal rerun.

Primary validation total: 22 cargo checks, 16 clippy passes/attempts; 17 focused nextest builds plus one legacy fixture no-run build, 3 production/bin builds and 2 IR diagnostic rustc invocations; 30 nextest run invocations. Actual native executions, doctests and the one frozen-door test are separately enumerated.

Additional non-Cargo lane evidence is indexed above: one legacy fixture lane (308 child chunks, 578 passed/19 failed/2 incomplete), four diagnostic child probes, and four Python unit-plan runs. Final Python suite is21passed; the earlier22case draft modeled the compiler graph incorrectly and is superseded. Child fragment logs belong to their parent lane header; per-child timestamps were not recorded and are not invented.

Test-run accounting: 30 nextest invocations + one legacy fixture lane + 11 direct court/example commands (10 native and one cargo run) + four diagnostic children + four documentation commands + four Python unit suites + one frozen-door test =55 top-level test/probe commands. The legacy lane launches308 chunks; child processes spawned inside courts are not counted again. Four native phase_telemetry commands are bookkeeping, not tests.
