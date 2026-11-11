# Open items after round2

Reviewed source HEAD: b2ba8dc3b102d71b899052d7ccc95a0718b76bbf. Overall task remains IN PROGRESS.

## H10: integration

All three genuine dev main controls remain0xc00000fd with1MiB PE reserves. See layout-r3/ordinary/{invariant_reads,required_feature,shared_execution}.log and round2-remaining-main-frames.md. Consumer-root succeeds. No stack/thread/profile workaround. C2 does not waive H10. The main-path next work remains Query mutation/commit/producer phase-carrier layout; no new product batch was attempted after the recorded round limit.

## Bare Store configuration: integration

The last Store-only cargo check exits0 but is not warning-clean. Full JSON diagnostics, rather than stderr tail, show3unused-import plus7dead-code warnings. Source paths below are unchanged across d2994fc364..b2ba8dc3b1, but these remain explicit unresolved integration hygiene. No suppression, widened visibility or narrowed payload membership is used. The attempted import-only narrowing was reversed after independent review because it exposed cascading dead code.

- unused import: `RecoveredReleaseCustody`: crates\worth-store\src\physical_runtime\durability\publication\current_root_owner\recovered_custody.rs:7
- unused imports: `GenerationZeroNoReleaseCustody`, `PreparedRecoveredCustodyBacking`, and `RecoveredNoReleaseCustody`: crates\worth-store\src\physical_runtime\record_serving\mod.rs:74
- unused import: `recovery::admit_recovered_obligation`: crates\worth-store\src\physical_runtime\work\mod.rs:82
- method `release_custody` is never used: crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody.rs:243
- variants `Selected` and `GenerationZero` are never constructed: crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\no_release.rs:8, crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\no_release.rs:9
- fields `selected_root` and `selected_root_sha256` are never read: crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\no_release.rs:16, crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\no_release.rs:17
- variants `HeadV2`, `NoRelease`, `PendingWal`, and `OrderedHistorical` are never constructed: crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\release_custody.rs:5, crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\release_custody.rs:6, crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\release_custody.rs:7, crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\release_custody.rs:11
- field `grant` is never read: crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\storage.rs:12
- associated items `required_bytes`, `owned_heap_bytes`, `charged_bytes`, and `finish` are never used: crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\storage.rs:16, crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\storage.rs:45, crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\storage.rs:50, crates\worth-store\src\physical_runtime\record_serving\admission\recovered_custody\storage.rs:54
- function `admit_recovered_obligation` is never used: crates\worth-store\src\physical_runtime\work\recovery\recovered_admission.rs:12

## Held boundary findings: P-A/P-C

Final boundary lane exits1 with15findings; all17 integration-owned census/court findings are removed. No baseline, root, caller permission or exception changed. These exact held paths remain for their assigned owner; no branch was merged this resume.

- P-C: BC7013_CERTIFICATION_CONSTRUCTION_SEAL workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_catalog_readmission.rs: factory `world_recovery_authority` requires its owner or an exact caller
- P-C: BC7013_CERTIFICATION_CONSTRUCTION_SEAL workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_compilation/fresh_installation.rs: factory `prepare_fresh_home_schema_installation` requires its owner or an exact caller
- P-C: BC7013_CERTIFICATION_CONSTRUCTION_SEAL workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_compilation/fresh_installation.rs: factory `prepare_native_schema_for_home` requires its owner or an exact caller
- P-C: BC7013_CERTIFICATION_CONSTRUCTION_SEAL workspaces/worth-store/crates/worth-store-court-application/tests/worth_store_court_application/process/application/no_pending_owners/transition_native_effect/native_original.rs: factory `bind_query_native_proposal` requires its owner or an exact caller
- P-A: BC9002_TEST_TOPOLOGY crates/worth-relational/src/runtime/home_program_transition/body_digest/tests.rs::file
- P-A: BC9002_TEST_TOPOLOGY crates/worth-relational/src/runtime/home_program_transition/catalog_digest/tests.rs::file
- P-A: BC9002_TEST_TOPOLOGY crates/worth-relational/src/runtime/state/subsystems/record_identity/partition_frontier/tests.rs::file
- P-A: BC9002_TEST_TOPOLOGY crates/worth-relational/src/tests/durability/contracts/native_checkpoint/allocation/encoded_payload.rs::file
- P-A: BC9002_TEST_TOPOLOGY crates/worth-relational/src/tests/durability/contracts/semantic_result/execution_posture.rs::file
- P-A: BC9002_TEST_TOPOLOGY crates/worth-relational/src/tests/durability/medium/cold_source.rs::file
- P-A: BC9002_TEST_TOPOLOGY crates/worth-relational/src/tests/durability/medium/cold_source/protected_reads.rs::file
- P-A: BC9002_TEST_TOPOLOGY crates/worth-relational/src/tests/support/durable_medium/runtime_attachment.rs::file
- P-C: BC9002_TEST_TOPOLOGY crates/worth-runtime-world/src/durability/artifact/retention_tests.rs::file
- P-C: BC9002_TEST_TOPOLOGY crates/worth-runtime-world/src/durability/occurrence_custody/retention_tests.rs::file
- P-C: BC9002_TEST_TOPOLOGY crates/worth-runtime-world/src/lifecycle/owner/execution_service/tests/public_ports/released_source.rs::file
