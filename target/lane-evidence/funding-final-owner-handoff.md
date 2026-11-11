# Final funding qualification: exact outside-grant handoff

Root Clippy ran the full workspace/all-targets command and exited101 with18 lint errors. This is the current lane's failure and blocks FUNDING LANDED. Two errors are inside the granted area and are corrected: items-after-tests in durability/data/history_member_directory.rs (production impls moved before the same unchanged tests); and storage/.../values/metadata/tests.rs redundant formatting reference (same Debug/equality assertion). Root Clippy has NOT been rerun after these two corrections because16 known owner fixes remain unapplied. Do not call it a16-error measured rerun.

## Source edits requiring a ruling

The remaining16 lints span these13 files, outside the Native storage/checkpoints/data area and beyond the six exact lint-file grants. All are unchanged in the working tree. `proposed-funding-final-outside-clippy.patch` carries the precise proposal; `funding-final-outside-clippy-patch-check.log` proves its dry-run applies, not that its Rust compiles. No allow attribute, test deletion/ignore, budget, work-charge order or expected outcome changes.

| Exact repository path | Current lines | Last source commit |
|---|---:|---|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account/public_only.rs|82|61b17083ed|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/process/serial/relational/managed_source_work_account.rs|334|61b17083ed|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog.rs|292|de71a645bb|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/index_catalog/recovery.rs|345|b150b599fe|
|crates/worth-relational/src/tests/durability/contracts/caller_association.rs|221|98eb64486b|
|crates/worth-relational/src/tests/durability/medium/cold_envelope_matrix.rs|115|b150b599fe|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/nested.rs|159|9027e5747b|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/row_read.rs|252|e3ee4659d4|
|crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/managed_source/owner.rs|287|e9ee57498e|
|crates/worth-court-relational-branch-reference-compile-time/tests/worth_court_relational_branch_reference_compile_time/sources/worth_relational/ui/change_source_pass/selected_commit_mints_receipt.rs|25|7910247d6d|
|crates/worth-relational/src/derived_relation/evaluation/memo/tests.rs|351|c068abc047|
|crates/worth-relational/src/transactions/data/outcomes/created_entity_bindings.rs|269|9429165bd3|
|crates/worth-relational/src/transactions/data/outcomes/created_relation_bindings.rs|194|9429165bd3|

The corrections are: two equivalent unsigned upper-bound assertions (`<= EXTRA-1` to `< EXTRA`), constant bool::then_some, six redundant artifact-reader borrows, two Result::expect_err spellings with unit-valued success branches, one nested-if collapse preserving the same locked observation/read, one unnecessary Some/? return wrapper, one adjacency filter/map spelling preserving original enumerate identities, and two production-helper moves before unchanged test modules. No runtime feature, facade or protocol is added.

## Exact boundary caller ruling required

The new selected-image court fixture directly references the existing sealed `RelationalDurableRecoverySource::from_home_original_bytes` constructor. It preserves the actual selected-read protection, real initial inputs and exact uncovered canonical/transition tails, using the existing external Native source trait. The current public testkit `home_image_input` recipe accepts TestDurableMedium; these controls wrap that medium with the receiver-owned fault/custody source. The helper uses the existing genuine Native constructor, not an invented observer or byte-only authority mint. Nevertheless, its exact reference is not yet registered. This is my new court boundary mistake; the six-source/two-line grant does not authorize a third line silently.

Required append-only entry:

```text
native-home-original-source crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication/source_initial_inputs/checkpoint_admission/cold_materialization/selected_image.rs
```

`proposed-funding-final-selected-reader-caller.patch` is unapplied. The existing two granted contract_work entries stay exactly as granted, without reorder/pattern expansion. `funding-final-selected-reader-patch-check.log` verifies this additional one-line proposal applies.

`funding-final-cost-boundary.log` exits1 with that BC7013 plus exactly the three separately permitted queued BC9002 sites. The initial boundary also found my incorrectly nested cost category; the measured byte/peer controls are now in the existing TOP-LEVEL cost court family, with all assertions intact. The last boundary no longer reports that category error. No source-court baseline was increased or enforcement suppressed. The known BC9002 exception is the explicit user ruling in orchestration/m3/prompts/p3_reopen_cold_files.md:31; it does not excuse the extra BC7013 or Root Clippy.

## Completed and remaining

Runtime qualification is green: final top-level cost controls3/0/0; large Foundation pure unit1/0/0; filesystem Host5/0/0, unchanged bounded-tail1/0/0, Store recovery1/0/0 and CDC/recovery3/0/0. The Native nine existing cases plus new shared-custody control remain passing evidence within the earlier10/3 packet; no claim that that historical packet itself was green. Full Store Clippy, dirty59-file line caps, generated context and exact-scope audit pass. Full Root Clippy and mandatory boundary remain red as above. The earlier calibrated frame-report attempts failed because they were not complete Native transient footprints; the final fixture instead captures only the byte record and creates/updates the unrelated record AFTER the controlled read. Caller64MiB, pressure128KiB, payload8/65536 and all retry/lifetime/assertions are unchanged.

Foundation's independently reviewed two-file correction is committed as9d04a90050. Other funding implementation remains intact and dirty; no work is parked/discarded. No integration merge or push. The coordinator must name a new integration HEAD before merging, then the decoder/D13/named-digest/BC9002 queue resumes in its specified order.
