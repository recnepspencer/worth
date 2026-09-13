use worth_store_blob_chunks::{
    admit_canonical_qualification_lane,
    certification_test_authority::{execute_blob_harness, BlobHarnessExecutionInput},
    deny_ambient_chaos_corpus_as_canonical, deny_generated_expected_byte_artifact,
    deny_hidden_temporary_sidecar, deny_logical_size_only_heavy_qualification,
    deny_sparse_only_heavy_qualification, deny_whole_object_expected_buffer,
    DeterministicBytePatternProfile, HeavyBlobFixtureMaterializationMode,
};

use super::heavy_qualification::{
    canonical_heavy_fixture_pattern_plan_for_seed, non_canonical_chaos_stress_plan_for_seed,
};
use crate::{BlobHarnessProfile, BlobHarnessScenarioSeed, OracleFamilyKind};

#[test]
fn lowered_plan_requires_heavy_qualification_oracle_family() {
    let lowered = crate::lower_blob_simulation_seed_plan(heavy_multi_gb_scenario_seed()).unwrap();
    assert!(lowered
        .plan()
        .oracle_families()
        .contains(OracleFamilyKind::BlobHeavyQualification));
}

#[test]
fn canonical_pattern_profiles_execute_with_shared_evidence_schema() {
    let seed = local_scenario_seed();
    for pattern in DeterministicBytePatternProfile::canonical_heavy_blob_patterns() {
        let witness = execute_blob_harness(
            BlobHarnessExecutionInput::new(
                worth_store_blob_chunks::certification_test_authority::BlobHarnessStorageShape::new(
                seed.profile().envelope().profile(),
                seed.size_class(),
                seed.placement_class(),
                seed.security_scope(),
                ),
                worth_store_blob_chunks::certification_test_authority::BlobHarnessExerciseShape::new(
                seed.access_mode(),
                seed.failure_point(),
                seed.actor_mix(),
                seed.topology(),
                ),
            )
            .with_heavy_temp_file_materialization()
            .with_heavy_byte_pattern_profile(pattern),
        );
        let evidence = witness
            .heavy_fixture_evidence()
            .expect("pattern lane heavy evidence");
        assert_eq!(
            evidence.expected_digest_basis().byte_pattern_profile(),
            pattern
        );
        assert!(evidence.peak_allocation_count() > 0);
        assert!(evidence.cleanup_receipt().expect("cleanup").completed());
    }
}

#[test]
fn chaos_corpus_lane_is_explicitly_non_canonical() {
    let stress_plan = non_canonical_chaos_stress_plan_for_seed(&heavy_multi_gb_scenario_seed());
    let local_seed = local_scenario_seed();
    let witness = execute_blob_harness(
        BlobHarnessExecutionInput::new(
            worth_store_blob_chunks::certification_test_authority::BlobHarnessStorageShape::new(
                local_seed.profile().envelope().profile(),
                local_seed.size_class(),
                local_seed.placement_class(),
                local_seed.security_scope(),
            ),
            worth_store_blob_chunks::certification_test_authority::BlobHarnessExerciseShape::new(
                local_seed.access_mode(),
                local_seed.failure_point(),
                local_seed.actor_mix(),
                local_seed.topology(),
            ),
        )
        .with_heavy_temp_file_materialization()
        .with_non_canonical_chaos_stress(),
    );
    let evidence = witness
        .heavy_fixture_evidence()
        .expect("non-canonical chaos evidence");

    assert_eq!(
        stress_plan.byte_pattern_profile(),
        DeterministicBytePatternProfile::AmbientChaosCorpus
    );
    assert_eq!(
        stress_plan.backend_profile(),
        worth_store_physical_backend::HeavyFixtureBackendProfile::NonCanonicalChaosCorpus
    );
    assert_eq!(
        evidence.expected_digest_basis().byte_pattern_profile(),
        DeterministicBytePatternProfile::AmbientChaosCorpus
    );
    assert_eq!(
        evidence.backend_profile(),
        worth_store_physical_backend::HeavyFixtureBackendProfile::NonCanonicalChaosCorpus
    );
    assert_eq!(
        deny_ambient_chaos_corpus_as_canonical(),
        worth_store_blob_chunks::HeavyBlobQualificationDenial::AmbientCorpusNotCanonical
    );
}

#[test]
fn heavy_pattern_plan_identity_changes_with_pattern_profile() {
    let seed = heavy_multi_gb_scenario_seed();
    let first = canonical_heavy_fixture_pattern_plan_for_seed(
        &seed,
        DeterministicBytePatternProfile::IncompressibleSeeded,
        HeavyBlobFixtureMaterializationMode::StreamOnly,
    )
    .expect("heavy pattern plan");
    let second = canonical_heavy_fixture_pattern_plan_for_seed(
        &seed,
        DeterministicBytePatternProfile::RepeatedChunkDedupePressure,
        HeavyBlobFixtureMaterializationMode::StreamOnly,
    )
    .expect("heavy pattern plan");

    assert_ne!(
        first.expected_digest_basis().byte_pattern_profile(),
        second.expected_digest_basis().byte_pattern_profile()
    );
    assert_eq!(
        first.expected_digest_basis().logical_bytes(),
        second.expected_digest_basis().logical_bytes()
    );
}

#[test]
fn hostile_qualification_patterns_are_executed_lane_denials() {
    let seed = heavy_multi_gb_scenario_seed();
    let sparse = canonical_heavy_fixture_pattern_plan_for_seed(
        &seed,
        DeterministicBytePatternProfile::SparseDeclarationDenied,
        HeavyBlobFixtureMaterializationMode::StreamOnly,
    )
    .expect("hostile plan");
    let logical_only = canonical_heavy_fixture_pattern_plan_for_seed(
        &seed,
        DeterministicBytePatternProfile::LogicalSizeOnlyDenied,
        HeavyBlobFixtureMaterializationMode::StreamOnly,
    )
    .expect("hostile plan");
    let whole_object = canonical_heavy_fixture_pattern_plan_for_seed(
        &seed,
        DeterministicBytePatternProfile::WholeObjectExpectedBufferDenied,
        HeavyBlobFixtureMaterializationMode::StreamOnly,
    )
    .expect("hostile plan");
    let generated_expected = canonical_heavy_fixture_pattern_plan_for_seed(
        &seed,
        DeterministicBytePatternProfile::GeneratedExpectedByteArtifactDenied,
        HeavyBlobFixtureMaterializationMode::StreamOnly,
    )
    .expect("hostile plan");
    let hidden_sidecar = canonical_heavy_fixture_pattern_plan_for_seed(
        &seed,
        DeterministicBytePatternProfile::HiddenTemporarySidecarDenied,
        HeavyBlobFixtureMaterializationMode::StreamOnly,
    )
    .expect("hostile plan");

    assert_eq!(
        admit_canonical_qualification_lane(&sparse),
        Err(deny_sparse_only_heavy_qualification())
    );
    assert_eq!(
        admit_canonical_qualification_lane(&logical_only),
        Err(deny_logical_size_only_heavy_qualification())
    );
    assert_eq!(
        admit_canonical_qualification_lane(&whole_object),
        Err(deny_whole_object_expected_buffer())
    );
    assert_eq!(
        admit_canonical_qualification_lane(&generated_expected),
        Err(deny_generated_expected_byte_artifact())
    );
    assert_eq!(
        admit_canonical_qualification_lane(&hidden_sidecar),
        Err(deny_hidden_temporary_sidecar())
    );
}

fn local_scenario_seed() -> BlobHarnessScenarioSeed {
    BlobHarnessScenarioSeed::builder()
        .profile(BlobHarnessProfile::local())
        .placement_external()
        .security_scope_preserving()
        .read_only_access()
        .seed_actor_mix()
        .build()
        .unwrap()
}

fn heavy_multi_gb_scenario_seed() -> BlobHarnessScenarioSeed {
    BlobHarnessScenarioSeed::builder()
        .profile(BlobHarnessProfile::heavy_multi_gb())
        .placement_external()
        .security_scope_preserving()
        .read_only_access()
        .seed_actor_mix()
        .build()
        .unwrap()
}
