# P-D receipt and retry evidence

This is a scoped implementation handoff, not Phase 4 certification. Enabled courts still expose the excluded owner boundaries listed below. Independent review is recorded separately after the source and gates are qualified.

The reviewed source starts at `c068abc047`. Every log records its actual command, source HEAD, dirty state, start/end and exit code. A dirty run qualifies only the reviewed source delta that was present then; subsequent corrections require their targeted successor logs. `ignored` counts below use the nextest selected-binary inventory when present; otherwise they count unique unexecuted libtest ignored events. They include ignored tests independently of the expression and never mean executed tests. No ignored test was requested.

Build/check/format rows have 0/0/0 because they execute no tests. Metadata, unit-graph declarations and prebuild selection are auxiliary captures in `target/lane-evidence/receipt-lane-ledger.json`; that ledger also records content SHA-256 and compiler diagnostics. Superseded failures remain visible. A green successor is required before calling any failed behavior fixed.

## Commits

- `b10d48e157` test(home): add filesystem original-receipt retry courts
- `7e89d5f75d` fix(test-cost): resolve compiler units through Cargo dependency fingerprints
- `1d56175594` fix(test-cost): require agreement across dependency observations
- `df2a4d13b1` fix(world): observe original publications from durable owner sources
- `58d381004e` feat(foundation): describe durable receipt coordinates without live authority
- `5b5327ebdf` feat(native): expose original created-row data from readmitted artifacts
- `b84a8953d0` fix(world): retain root-free original publication inputs across history retirement
- `9b05f31613` fix(world): keep one original for an unchanged adopted Native leg
- `98b39a07bd` feat(query): resolve original receipts through sealed owner artifacts
- `0ef409c86e` fix(server): hand keyed mutations to Query before deleting retry cache
- `73b4b5934a` test(retry): compare complete original receipts through filesystem Host courts
- `0e2f4f4766` test(query): assert recorded response-loss outcomes and exact effect custody
- `be5fba01b0` test(query): keep recorded completion distinct in fixture consumers
- `51c0fe32c2` fix(query): transfer the original attempt result directly from publication
- `2eddc9ab78` test(retry): lend a fresh real parent to every consult injection
- `de13e9b9cb` test(query): retain the interleaved peer's original result receiver
- `97abf4065e` fix(query): preserve original delivery custody through failed cleanup
- `8cf0ae8708` fix(query): preserve description read stops during original receipt resolution
- `9807bf8855` test(retry): count committed occurrences independently of pure preparation
- `163bb2a762` fix(query): bound retained Native graphs at runtime ownership contracts
- `d742916691` fix(native): bound original memory observations at their owning contract
- `d344a37bfb` fix(server): lend the installed Query runtime to mutation selection
- `1ad7e327d8` fix(server): preserve the installed application mutation binding in configuration
- `8eca224e4a` fix(query): keep transport intent separate from the canonical handler input
- `01fde1fec7` test(host): bound retry cancellation observation to admitted business execution
- `13a09db9c1` fix(query): fund original delivery and cleanup custody before effects
- `6c3d1c9ef6` test(world): exercise original history and call admission through public courts
- `b97c95d3aa` test(query): borrow owned phases for original receipt qualification
- `381d9a0990` fix(bank): retain native completion and cleanup owners across recorded retries
- `41b6ca949b` test(world): retire owned fork custody before fixture close
- `086d695495` refactor(bank): inspect transferred custody through closed owners
- `8119edbd5c` fix(bank): observe recorded recovery claims under fresh admission
- `67317b6774` test(query): allow the full durable consult cancellation sweep
- `e8d3bd4086` test(query): retain the verified ordinary consult timeout
- `69cd03463a` fix(bank): revalidate live recovery disclosure without reconstructing receipts
- `c14ea576a8` fix(query): require original owners throughout workflow recovery
- `1e2ec5d037` test(bank): preserve closed carriers in dispatch and pending retry courts
- `1f30f95d30` test(bank): retain original workflow definition through retirement
- `9540bf50fb` test(bank): compare workflow retries with their retained originals
- `d68b34e087` fix(bank): select recorded history through fresh Query admission
- `c00d2306e5` test(bank): compare inbound outbox against its held Native owner
- `1e1fd3763c` test(query): retain first workflow evidence and compare retries as data
- `386a5881fe` test(query): retain original lineage evidence across durable retries
- `62de605548` test(host): preserve original correction and stopped-effect custody
- `dd8477f665` test(host): compare inline correction retries against original owners
- `87227991d6` test(host): compare causal meaning through recorded coordinates
- `5cf0287c8b` test(retry): compare original attempt owners and canonical basis axes
- `c3d8aaf1c0` chore(test-cost): retain receipt qualification measurements

## Lane captures

| Full log | HEAD | Exit | Pass / fail / ignored inventory |
|---|---|---:|---:|
| `target/lane-evidence/lane-evidence-format-query.log-c068abc04760-202611020855483749260000-a59be1fcc89c.log` | `c068abc047` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-retry-courts.log` | `c068abc047` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/format-query.log` | `c068abc047` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-retry-courts/build/binaries-build.log` | `c068abc047` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/lane-evidence-receipt-retry-behavior.log-c068abc04760-202611020911583255510000-f763063b11e1.log` | `c068abc047` | 96 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-retry-behavior.log` | `c068abc047` | 96 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-retry-behavior-configured.log` | `c068abc047` | 100 | 1 / 3 / 2 |
| `target/lane-evidence/receipt-courts-format-check.log` | `c068abc047` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-courts-line-caps.log` | `c068abc047` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compiler-units-tests.log` | `b10d48e157` | 0 | 22 / 0 / 0 |
| `target/lane-evidence/compiler-fingerprint-tests.log` | `b10d48e157` | 0 | 2 / 0 / 0 |
| `target/lane-evidence/receipt-world-format.log` | `b10d48e157` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-world-check.log` | `b10d48e157` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-world-check-corrected.log` | `b10d48e157` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compiler-fingerprint-real-artifacts.log` | `b10d48e157` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-owner-normal-build.log` | `b10d48e157` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compiler-fingerprint-package-identity.log` | `b10d48e157` | 0 | 3 / 0 / 0 |
| `target/lane-evidence/world-owner-normal.log` | `b10d48e157` | 100 | 306 / 1 / 0 |
| `target/lane-evidence/world-original-observation-build.log` | `b10d48e157` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/world-owner-format.log` | `7e89d5f75d` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-row-format.log` | `7e89d5f75d` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-original-court-format.log` | `7e89d5f75d` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-owner-gate-correction-build.log` | `7e89d5f75d` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-owner-gate-correction.log` | `7e89d5f75d` | 0 | 13 / 0 / 0 |
| `target/lane-evidence/world-original-observation-corrected-build.log` | `7e89d5f75d` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compiler-unit-final.log` | `1d56175594` | 0 | 22 / 0 / 0 |
| `target/lane-evidence/compiler-fingerprint-final.log` | `1d56175594` | 0 | 4 / 0 / 0 |
| `target/lane-evidence/compiler-real-artifact-final.log` | `1d56175594` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-original-observation-corrected.log` | `1d56175594` | 100 | 0 / 2 / 0 |
| `target/lane-evidence/world-original-source-courts-format.log` | `1d56175594` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-original-source-courts-build.log` | `1d56175594` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-original-source-courts.log` | `1d56175594` | 100 | 2 / 1 / 0 |
| `target/lane-evidence/original-row-owner-format.log` | `df2a4d13b1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-row-owner-check.log` | `df2a4d13b1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-receipt-format.log` | `df2a4d13b1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-receipt-check.log` | `df2a4d13b1` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-receipt-integrated-format.log` | `df2a4d13b1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-receipt-integrated-check.log` | `df2a4d13b1` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/query-public-receipt-format.log` | `df2a4d13b1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-public-receipt-check.log` | `df2a4d13b1` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/original-created-rows-format.log` | `58d381004e` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/resident-original-owner-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/resident-original-owner-check.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-owner-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-owner-check.log` | `5b5327ebdf` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/query-owner-transport-integrated-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-owner-transport-integrated-check.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-cutover-root-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-cutover-query-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-cutover-check.log` | `5b5327ebdf` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-terminal-integrated-root-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-terminal-integrated-query-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-terminal-integrated-check.log` | `5b5327ebdf` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-admission-integrated-root-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-admission-integrated-query-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-server-admission-integrated-check.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts.log` | `5b5327ebdf` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/binaries-build.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-owner-unit.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-ordinary.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-process.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-compile-fail.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-oracle.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-determinism.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-cost.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-exhaustive.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/selection-scale.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/build/request-selection.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/directory-original-source-courts/run-selection.log` | `5b5327ebdf` | 100 | 0 / 5 / 2 |
| `target/lane-evidence/directory-original-source-courts/telemetry.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-root-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-query-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-server-check.log` | `5b5327ebdf` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/sealed-original-root-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/sealed-original-query-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/sealed-original-server-check.log` | `5b5327ebdf` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/binaries-build.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/lane-evidence-recorded-receipt-court-format.log-5b5327ebdfa4-202611021534444881950000-ab6600e1bdb0.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-receipt-query-owner-build.log` | `5b5327ebdf` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-owner-unit.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-ordinary.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-process.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-compile-fail.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-oracle.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-determinism.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-cost.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-exhaustive.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/selection-scale.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/build/request-selection.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/run-selection.log` | `5b5327ebdf` | 0 | 4 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-courts/telemetry.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-receipt-query-owner-build/build/binaries-build.log` | `5b5327ebdf` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/compact-original-owner-commit-format.log` | `5b5327ebdf` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-receipt-court-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/owned-recovered-consumer-root-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/owned-recovered-consumer-query-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/owned-recovered-consumer-server-check.log` | `b84a8953d0` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/original-custody-integration-root-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-custody-integration-query-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-custody-integration-server-check.log` | `b84a8953d0` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/retained-original-consumer-root-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/retained-original-consumer-query-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/retained-original-consumer-server-check.log` | `b84a8953d0` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/original-owner-final-integration-root-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-owner-final-integration-query-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-owner-final-integration-server-check.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-receipt-query-normal.log` | `b84a8953d0` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/phase4-root-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/phase4-query-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/phase4-store-format.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/phase4-dirty-line-caps.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/phase4-boundary.log` | `b84a8953d0` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/original-receipt-query-normal/build/binaries-build.log` | `b84a8953d0` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-owner-normal/build/binaries-build.log` | `b84a8953d0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/phase4-agent-context.log` | `73b4b5934a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-root-owner-normal/build/binaries-build.log` | `73b4b5934a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-owner-normal/run.log` | `73b4b5934a` | 100 | 2219 / 17 / 1 |
| `target/lane-evidence/receipt-root-owner-normal/run.log` | `73b4b5934a` | 100 | 2427 / 123 / 31 |
| `target/lane-evidence/lane-evidence-query-data-courts-format.log-73b4b5934a04-202611021718580308140000-fef1b7948154.log` | `73b4b5934a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-owner-corrections/build/binaries-build.log` | `0e2f4f4766` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/lane-evidence-query-data-courts-format.log-0e2f4f476692-202611021725304076500000-fef1b7948154.log` | `0e2f4f4766` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-data-courts-format.log` | `0e2f4f4766` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-consumer-fixture-format.log` | `0e2f4f4766` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-owner-corrections-qualified/build/binaries-build.log` | `be5fba01b0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/public-host-original-receipt-matrix/build/binaries-build.log` | `be5fba01b0` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-owner-corrections-qualified/run.log` | `be5fba01b0` | 100 | 21 / 2 / 1 |
| `target/lane-evidence/direct-original-completion-format.log` | `be5fba01b0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/direct-original-completion-final-format.log` | `be5fba01b0` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/direct-original-owner-qualification/build/binaries-build.log` | `2eddc9ab78` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/server-original-receipt-matrix/build/binaries-build.log` | `2eddc9ab78` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/public-host-original-receipt-qualified/build/binaries-build.log` | `2eddc9ab78` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/direct-original-owner-qualified/build/binaries-build.log` | `de13e9b9cb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/public-host-original-receipt-qualified/run.log` | `de13e9b9cb` | 100 | 4 / 9 / 2 |
| `target/lane-evidence/direct-original-owner-qualified/run.log` | `de13e9b9cb` | 100 | 508 / 3 / 1 |
| `target/lane-evidence/owned-original-cleanup-query-format.log` | `de13e9b9cb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/owned-original-cleanup-root-format.log` | `de13e9b9cb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/owned-cleanup-handoff-final-format.log` | `de13e9b9cb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/owned-cleanup-receipt-qualification/build/binaries-build.log` | `9807bf8855` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/server-receipt-normal-and-process/build/binaries-build.log` | `9807bf8855` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/original-custody-contract-format.log` | `9807bf8855` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bounded-original-query-format.log` | `9807bf8855` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bounded-original-world-format.log` | `9807bf8855` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bounded-cleanup-receipt-qualification/build/binaries-build.log` | `163bb2a762` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bounded-server-receipt-normal-and-process/build/binaries-build.log` | `163bb2a762` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/bounded-cleanup-receipt-qualification/run.log` | `163bb2a762` | 0 | 81 / 0 / 1 |
| `target/lane-evidence/host-directory-receipt-owner-fixes/build/binaries-build.log` | `163bb2a762` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/native-memory-observation-format.log` | `163bb2a762` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/native-bounded-server-receipt-normal-and-process/build/binaries-build.log` | `d742916691` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/host-directory-receipt-owner-fixes/run.log` | `d742916691` | 100 | 1 / 1 / 2 |
| `target/lane-evidence/bank-closed-consumer-check.log` | `d742916691` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/server-runtime-selection-format.log` | `d742916691` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/installed-runtime-server-receipt-normal-and-process/build/binaries-build.log` | `d344a37bfb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/installed-runtime-server-receipt-normal-and-process/run.log` | `d344a37bfb` | 100 | 97 / 4 / 0 |
| `target/lane-evidence/server-application-config-format.log` | `d344a37bfb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/configured-server-receipt-matrix/build/binaries-build.log` | `1ad7e327d8` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/configured-server-receipt-matrix/run.log` | `1ad7e327d8` | 100 | 0 / 4 / 0 |
| `target/lane-evidence/transport-binding-business-consult-format.log` | `1ad7e327d8` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/separated-intent-server-receipt-matrix/build/binaries-build.log` | `01fde1fec7` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/host-business-receipt-cancellation/build/binaries-build.log` | `01fde1fec7` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/separated-intent-server-receipt-matrix/run.log` | `01fde1fec7` | 100 | 3 / 1 / 0 |
| `target/lane-evidence/funded-original-query-format.log` | `01fde1fec7` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/lane-evidence-funded-original-bank-format.log-01fde1fec7c1-202611022131406266880000-fa30b1898dae.log` | `01fde1fec7` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/lane-evidence-funded-original-bank-format.log-01fde1fec7c1-202611022132091621640000-fa30b1898dae.log` | `01fde1fec7` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/funded-closed-consumer-check.log` | `01fde1fec7` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/lane-evidence-funded-original-bank-format.log-01fde1fec7c1-202611022139250741240000-fa30b1898dae.log` | `01fde1fec7` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/closed-handoff-consumer-check.log` | `01fde1fec7` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/lane-evidence-funded-original-bank-format.log-01fde1fec7c1-202611022149089027730000-fa30b1898dae.log` | `01fde1fec7` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/funded-original-storage-format.log` | `01fde1fec7` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/closed-only-consumer-check.log` | `01fde1fec7` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/funded-original-bank-format.log` | `13a09db9c1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-original-custody-normal/build/binaries-build.log` | `13a09db9c1` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-phase-completion-root-format.log` | `13a09db9c1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-phase-completion-query-format.log` | `13a09db9c1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-phase-completion-bank-format.log` | `13a09db9c1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-phase-completion-caps.log` | `13a09db9c1` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-phase-completion-boundary.log` | `13a09db9c1` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-phase-completion-context.log` | `13a09db9c1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-phase-completion-caps-git-bash.log` | `13a09db9c1` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-topology-correction-query-format.log` | `6c3d1c9ef6` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-topology-correction-bank-format.log` | `6c3d1c9ef6` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-phase-boundary-corrected.log` | `381d9a0990` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-original-owner-normal/build/binaries-build.log` | `381d9a0990` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-original-owner-normal/run.log` | `381d9a0990` | 0 | 307 / 0 / 0 |
| `target/lane-evidence/world-public-original-courts/build/binaries-build.log` | `381d9a0990` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-public-original-courts/run.log` | `381d9a0990` | 100 | 63 / 2 / 3 |
| `target/lane-evidence/closed-only-transfer-format.log` | `381d9a0990` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-history-teardown-format.log` | `381d9a0990` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-history-fork-cleanup-correction/build/binaries-build.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/world-history-fork-cleanup-correction/run.log` | `086d695495` | 0 | 2 / 0 / 3 |
| `target/lane-evidence/bank-recorded-owners-normal/build/binaries-build.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-recorded-owners-normal/run.log` | `086d695495` | 100 | 219 / 12 / 0 |
| `target/lane-evidence/recorded-recovery-query-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-recovery-bank-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-recorded-recovery-correction/build/binaries-build.log` | `086d695495` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-owner-query-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-owner-bank-format.log` | `086d695495` | 1 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-owner-foundation-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-owner-correction-query-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-owner-correction-bank-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-owner-qualified-query-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-owner-qualified-bank-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/admitted-token-bank-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-exact-recorded-recovery/build/binaries-build.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-exact-recorded-recovery/rail-prebuild.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-exact-recorded-recovery/run.log` | `086d695495` | 100 | 13 / 2 / 0 |
| `target/lane-evidence/expired-claim-disclosure-bank-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-original-constructor-fences.log` | `086d695495` | 0 | 4 / 0 / 0 |
| `target/lane-evidence/approval-recorded-owner-bank-format.log` | `086d695495` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-funded-original-claim-owners/build/binaries-build.log` | `e8d3bd4086` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-outbox-example-format.log` | `e8d3bd4086` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-funded-original-claim-owners/run.log` | `e8d3bd4086` | 0 | 12 / 0 / 1 |
| `target/lane-evidence/current-recovery-disclosure-format.log` | `e8d3bd4086` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/current-live-recovery-disclosure-format.log` | `e8d3bd4086` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-current-disclosure-and-workflow/build/binaries-build.log` | `e8d3bd4086` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-current-disclosure-and-workflow/rail-prebuild.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-current-disclosure-and-workflow/run.log` | `69cd03463a` | 100 | 3 / 2 / 0 |
| `target/lane-evidence/workflow-original-owner-query-format.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/workflow-original-owner-bank-format.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-workflow-original-handoff/build/binaries-build.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-workflow-original-handoff/rail-prebuild.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-workflow-original-handoff/run.log` | `69cd03463a` | 100 | 0 / 2 / 0 |
| `target/lane-evidence/recorded-workflow-owner-close-query-format.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-workflow-owner-close-bank-format.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/workflow-governed-identity-query-format.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/workflow-recorded-close-bank-format.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-workflow-governed-input-correction/build/binaries-build.log` | `69cd03463a` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-workflow-governed-input-correction/rail-prebuild.log` | `c14ea576a8` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-workflow-governed-input-correction/run.log` | `c14ea576a8` | 0 | 2 / 0 / 0 |
| `target/lane-evidence/bank-workflow-original-owner-courts/build/binaries-build.log` | `c14ea576a8` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/remaining-closed-fixture-bank-format.log` | `c14ea576a8` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-closed-original-owner-courts/build/binaries-build.log` | `1e2ec5d037` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/retained-definition-bank-format.log` | `1e2ec5d037` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-retained-definition-owner-courts/build/binaries-build.log` | `1f30f95d30` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-workflow-caller-bank-format.log` | `1f30f95d30` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-recorded-workflow-qualified-courts/build/binaries-build.log` | `9540bf50fb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-recorded-workflow-qualified-courts/rail-prebuild.log` | `9540bf50fb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-recorded-workflow-qualified-courts/run.log` | `9540bf50fb` | 100 | 172 / 10 / 10 |
| `target/lane-evidence/query-workflow-binding-owner-qualification/build/binaries-build.log` | `9540bf50fb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/recorded-bank-history-format.log` | `9540bf50fb` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/query-workflow-binding-owner-qualification/run.log` | `d68b34e087` | 0 | 54 / 0 / 1 |
| `target/lane-evidence/bank-recorded-consumers-correction/build/binaries-build.log` | `d68b34e087` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-recorded-consumers-correction/rail-prebuild.log` | `d68b34e087` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-recorded-consumers-correction/run.log` | `d68b34e087` | 0 | 10 / 0 / 1 |
| `target/lane-evidence/query-host-workflow-normal-and-cancellation/build/binaries-build.log` | `d68b34e087` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/host-normal-and-original-cancellation/build/binaries-build.log` | `d68b34e087` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/host-normal-and-original-cancellation/run.log` | `d68b34e087` | 100 | 312 / 20 / 2 |
| `target/lane-evidence/bank-inbound-original-owner-example.log` | `d68b34e087` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/inbound-native-owner-format.log` | `d68b34e087` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/bank-inbound-native-owner-example.log` | `c00d2306e5` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/workflow-recorded-courts-format.log` | `c00d2306e5` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/workflow-original-court-batch-format.log` | `c00d2306e5` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/workflow-recorded-owner-ready-format.log` | `c00d2306e5` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/workflow-original-evidence-normal/build/binaries-build.log` | `1e1fd3763c` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/workflow-original-evidence-normal/run.log` | `1e1fd3763c` | 100 | 70 / 1 / 0 |
| `target/lane-evidence/query-workflow-original-owner-type-fences.log` | `1e1fd3763c` | 0 | 8 / 0 / 0 |
| `target/lane-evidence/host-recorded-correction-format.log` | `1e1fd3763c` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/host-workflow-recorded-retry-corrections/build/binaries-build.log` | `62de605548` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/host-workflow-recorded-retry-corrections/run.log` | `62de605548` | 100 | 12 / 3 / 2 |
| `target/lane-evidence/host-inline-original-format.log` | `62de605548` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/host-inline-original-retry-correction/build/binaries-build.log` | `dd8477f665` | 101 | 0 / 0 / 0 |
| `target/lane-evidence/host-inline-causal-coordinates-format.log` | `dd8477f665` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/host-inline-causal-original-retry-qualified/build/binaries-build.log` | `87227991d6` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/host-inline-causal-original-retry-qualified/run.log` | `87227991d6` | 100 | 2 / 1 / 2 |
| `target/lane-evidence/original-attempt-owner-axes-format.log` | `87227991d6` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-attempt-owner-axes-qualified/build/binaries-build.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/original-attempt-owner-axes-qualified/run.log` | `5cf0287c8b` | 0 | 4 / 0 / 2 |
| `target/lane-evidence/receipt-completion-format-root.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-completion-format-query.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-completion-format-bank.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-completion-format-store.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-completion-caps-dirty.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-completion-caps-range.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-completion-boundary.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |
| `target/lane-evidence/receipt-completion-context.log` | `5cf0287c8b` | 0 | 0 / 0 / 0 |

## Qualification and successors

These are composed results from the named normal lanes and exact failed/affected successor cases. They are not claims of whole-suite reruns at the last commit. Full per-case failure disposition is in [receipt-retry-failures.md](receipt-retry-failures.md).

| Affected target | Normal evidence and corrections | Final qualified disposition |
|---|---|---|
| Query Execution library | `query-original-owner-normal/run.log`, `direct-original-owner-qualified/run.log`, `query-original-owner-corrections-qualified/run.log`, `bounded-cleanup-receipt-qualification/run.log`; later input/recovery changes in `query-workflow-binding-owner-qualification/run.log` | All 17 original failed/timed-out cases have later passing events; normal baseline 2219 passing, 17 failed, one ignored inventory. No full rerun. |
| Query Publication | `query-workflow-binding-owner-qualification/run.log` | Normal Publication 27 pass; affected Execution input/recovery 27 pass; combined 54/0, one ignored inventory. |
| Workflow approval court | `workflow-original-evidence-normal/run.log`, `host-workflow-recorded-retry-corrections/run.log`, `original-attempt-owner-axes-qualified/run.log` | Normal 70/1, sole failure subsequently passes; affected fork passes. |
| Host ordinary court and public retry | `host-normal-and-original-cancellation/run.log`, `host-workflow-recorded-retry-corrections/run.log`, `host-inline-causal-original-retry-qualified/run.log`, `original-attempt-owner-axes-qualified/run.log`, `public-host-original-receipt-qualified/run.log`, `host-directory-receipt-owner-fixes/run.log` | Ordinary 312/20; ten exact failed cases pass later, ten remain at excluded owner boundaries. Public concurrency, cancellation, changed intent/later history and no-effect retry pass; six public reopen/sibling cases remain blocked. |
| Bank libraries | `bank-recorded-owners-normal/run.log`, `bank-exact-recorded-recovery/run.log`, `bank-current-disclosure-and-workflow/run.log`, `bank-workflow-governed-input-correction/run.log` | Normal 219/12; all twelve have later passing events. |
| Bank ordinary/affected courts | `bank-recorded-workflow-qualified-courts/run.log`, `bank-recorded-consumers-correction/run.log` | 172/10, then exact ten pass; ten ignored inventory tests never execute. New duplicate-close custody proof passes. |
| Bank inbound example | `bank-inbound-native-owner-example.log` | Actual harness=false example exits0 with callback/outbox assertions and zero remaining/dispatch/blocked assessment; no libtest count. |
| World | `world-original-owner-normal/run.log`, `world-public-original-courts/run.log`, `world-history-fork-cleanup-correction/run.log` | Owner normal307/0; public63/2 then both exact failures pass. Three public ignored inventory tests never execute. |
| Foundation / Native / root Server | `receipt-root-owner-normal/run.log` | 2427/123, 31 ignored inventory. Foundation115/0, root Server12/0; every failure is Native and individually indexed. |
| Server ordinary/public matrix | `installed-runtime-server-receipt-normal-and-process/run.log`, `separated-intent-server-receipt-matrix/run.log` | 97/4 then three exact positive matrix cases pass; actual killed-process reopen remains RootLinkage. |
| Store exact original occurrence source court | `world-original-source-courts.log` | 2/1; enabled Complete Home reopen case remains CompleteNativeRequired. No Store oracle/scenario suite ran. |
| Constructor/type fences | `query-original-constructor-fences.log`, `query-workflow-original-owner-type-fences.log` | Four passing original-data constructor tests and eight passing workflow-owner rustdocs (four positive and four compile-fail); positive and compile-fail cases. |
| Compiler unit attribution | `compiler-units-tests.log`, `compiler-fingerprint-real-artifacts.log` | 22 Python tests pass; real artifact capture disambiguates both sha2 units by dependency fingerprint identity. |

## Failure causes, corrections and responsibility

- Product defects: process/index receipt authority and Server cache were replaced by exact owner inputs and the actual stable Query handoff. Ordinary World try-lock contention no longer invents owner loss. The active original completion crosses a funded one-shot slot; failed cleanup retains its actual original/envelope; nested request stops remain exact. These are product corrections, not changes to expected successful behavior.
- Ownership/placement defects exposed by compilation: retained Native graphs required bounded private owner contracts; Server selection must borrow its installed runtime; closed HTTP delivery must transfer a Send-safe closed carrier and return the exact record/permit on abandoned reply. Missing imports, private paths, moved values, parse errors and stale enum matches were corrected at their exact caller. Every failed compile capture retains its compiler messages in the JSON ledger; later affected normal targets compile.
- Workflow product binding defect: encoded conventional input was incorrectly classified as governed authority. The receipt now records actual admission.governed_input_identity, while retaining original input separately. Recovery mismatch checking is unchanged. Duplicate Recorded workflow close no longer treats descriptive outbox fields as a live obligation; the real original retains its recovery door.
- Consumer/test defects after the deliberate D9 cutover: repeated keyed calls still expected Performed/AlreadyCommitted/live output or Signal getters. Tests retain the genuine first owner, require recorded data and compare original Native, settlement, World and causal meaning. Real effect/contact, security, actor, current-authority, historical-byte, active-reversal, cleanup and delivery assertions remain. The requirement map names deliberately retired claims and their governing commit/spec decision.
- Resource-court defect: one total unit short now reaches post-Native original capture, so expecting no effect was wrong. Exact WorkExhausted ProductUnpublished custody, one effect, live obligations and real owner inspection replace that false fate. Calibration and cancellation checks remain unchanged.
- Fixture phase-order defects: same-key lineage evidence must retain its original transition, not a replay boolean; fork cleanup must retire its actual Signal owner; Home clone refusal precedes World sealing and legacy conditional cleanup. Exact typed stages are asserted. The final successful Home close remains enabled and red.
- Tool defects: visible Cargo keys did not distinguish sha2 units with different dependency units. Dependency fingerprint identity and agreement across observations repair attribution without choosing a candidate. Initial nextest execution lacked the committed config path; owned selection now passes it explicitly. Initial bare bash selected unavailable WSL; the explicit installed Git bash runs the same official cap script. Boundary failures were repaired through actual court topology, lawful owned phases and exact documentation markers; no baseline exception was added.

## Unfixed and unrun

- P-A: before-checkpoint recovery refuses RootLinkage while synchronizing the recovered original source; after-checkpoint recovery refuses CompleteNativeRequired. The precise read-only call-chain/residency diagnosis is in `C:/programming/worth worktees/orchestration/m3/p4_receipts_status.md`. Enabled Host/Server lost-response, no-effect, cancellation and stale-selector reopen courts still require real Homes. They are not reported as passing.
- P-B: Query handler retirement refuses the actual Supplier set-balance handler as Unsupported before seal; its underlying Relational Source retirement is also explicitly Unsupported. Ten ordinary Host failures include this close family, distinct P-A reopen failures, two Native checkpoint Envelope(2) Damaged failures and a checkpoint-consolidation memory refusal. Those production paths are expressly excluded.
- Durable sibling creation refuses DurableBranchPublicationRequired before key resolution. The real court is enabled; no memory substitute or invented format proves this requirement.
- Interim memory checkpoint retry needs the coordinator D9/D13 ruling: transport lacks original World occurrences. No live receipt or authority is fabricated on restore.
- Native normal target has 123 unresolved failures: 84 report UnsupportedLegacySemantics, 13 RootLinkage, three ColdStorageUnavailable, and 23 distinct allocation-quote, replay/codec, canonical-envelope, field-text, backing-layout and checkpoint assertions. The full per-case index/log is preserved. Their exact assertion messages establish the failures, not a completed diagnosis of which implementation/test side is wrong. No excluded recovery/format/accounting repair is claimed.
- No ignored tests, exhaustive Home oracle, Store scenario lane, Query gates, scale/timing runs or tool self-test suite were executed. Final certification belongs to the coordinator.

## Facades and boundary configuration

Granted integration entries: World exact original publication observation/readmitted facade; Foundation recorded owner/head/attempt/settlement vocabulary; Native original created-row accessor and compact observation facade; Query recovered receipt/causal/outbox and typed original-owner execution branches; Host-forwarded ProductUnpublishedCause; Bank historical_original forwarding to existing Query admission. Removed historical receipt facades, retry-cache wiring and stale authority variants are clean cutovers.
The only boundary configuration change is `tools/boundary-check/config/control-callers.txt`: exact Host request-stop observer, Native-effect/World-compare fate court, Host/Server concurrent history observers and Bank closed HTTP retained-effect scheduling caller. No caller-family widening, feature, constitution baseline or Source-opening exception was added.
