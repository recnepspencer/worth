use worth_foundational::{
    foundational_responsibilities, performance_api::common_path as performance_common,
    FoundationalAuthoritativePerformanceClaim, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceClaimConstructionDenial,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
    FoundationalPerformancePrimitiveLegalityDenial, FoundationalPerformanceWorkClass,
    FoundationalPolicyAdmissionPerformanceClaim, FoundationalReplayMaterializationPerformanceClaim,
    FoundationalSupportDerivedPerformanceClaim,
};

fn require_authoritative_claim(_: &FoundationalAuthoritativePerformanceClaim) {}
fn require_support_claim(_: &FoundationalSupportDerivedPerformanceClaim) {}
fn require_replay_claim(_: &FoundationalReplayMaterializationPerformanceClaim) {}
fn require_policy_claim(_: &FoundationalPolicyAdmissionPerformanceClaim) {}

#[test]
fn common_path_claim_authoring_surfaces_live_under_the_performance_responsibility_home() {
    let names: Vec<_> = foundational_responsibilities()
        .iter()
        .map(|area| area.name())
        .collect();

    assert!(names.contains(&"performance"));
    let authoring = performance_common::performance().claim();
    let _ = authoring.authoritative_execution();
    let _ = authoring.support_derived();
    let _ = authoring.replay_or_materialization();
    let _ = authoring.policy_admission();
}

#[test]
fn authoritative_claims_require_full_disclosure_and_canonicalize_work_order() {
    let claim = performance_common::performance()
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(FoundationalPerformanceWorkClass::ForensicParity)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .finish()
        .expect("authoritative claim");

    require_authoritative_claim(&claim);
    assert_eq!(
        claim.boundary(),
        FoundationalPerformanceBoundary::AuthoritativeExecution
    );
    assert_eq!(
        claim.included_work(),
        &[
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            FoundationalPerformanceWorkClass::ValidationPlanning,
        ]
    );
    assert_eq!(
        claim.excluded_work(),
        &[
            FoundationalPerformanceWorkClass::ReplayReconstruction,
            FoundationalPerformanceWorkClass::SupportReportAssembly,
            FoundationalPerformanceWorkClass::ForensicParity,
        ]
    );
}

#[test]
fn support_replay_and_policy_claims_remain_family_distinct_lowered_types() {
    let support_claim = performance_common::performance()
        .claim()
        .support_derived()
        .boundary(FoundationalPerformanceBoundary::SupportAssembly)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::BranchLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal,
        )
        .execution_temperature(FoundationalPerformanceExecutionTemperature::SupportOnly)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::StaleSupport)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Debt)
        .include_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("support claim");
    require_support_claim(&support_claim);

    let replay_claim = performance_common::performance()
        .claim()
        .replay_or_materialization()
        .boundary(FoundationalPerformanceBoundary::ReplayReconstruction)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::SnapshotBound)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::RebuildCapable,
        )
        .execution_temperature(FoundationalPerformanceExecutionTemperature::ColdPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ReplayDerived)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("replay claim");
    require_replay_claim(&replay_claim);

    let policy_claim = performance_common::performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::DeltaBound)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("policy claim");
    require_policy_claim(&policy_claim);
}

#[test]
fn claim_builders_fail_closed_for_missing_or_incompatible_phase2_shapes() {
    let missing_disclosure = performance_common::performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CompileTimeContract)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Deferred)
        .finish();
    assert_eq!(
        missing_disclosure,
        Err(FoundationalPerformanceClaimConstructionDenial::MissingIncludedWorkDisclosure)
    );

    let incompatible_strength = performance_common::performance()
        .claim()
        .support_derived()
        .boundary(FoundationalPerformanceBoundary::SupportAssembly)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::BranchLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal,
        )
        .execution_temperature(FoundationalPerformanceExecutionTemperature::SupportOnly)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::StaleSupport)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Debt)
        .include_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish();
    assert_eq!(
        incompatible_strength,
        Err(
            FoundationalPerformanceClaimConstructionDenial::EvidenceStrengthNotAllowedForClaimFamily
        )
    );

    let hot_path_support_collapse = performance_common::performance()
        .claim()
        .support_derived()
        .boundary(FoundationalPerformanceBoundary::SupportAssembly)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Debt)
        .include_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish();
    assert_eq!(
        hot_path_support_collapse,
        Err(FoundationalPerformanceClaimConstructionDenial::PrimitiveLegality(
            FoundationalPerformancePrimitiveLegalityDenial::SupportDerivedClaimsCannotUseHotPathTemperature
        ))
    );

    let support_only_executed_collapse = performance_common::performance()
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(FoundationalPerformanceExecutionTemperature::SupportOnly)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish();
    assert_eq!(
        support_only_executed_collapse,
        Err(FoundationalPerformanceClaimConstructionDenial::PrimitiveLegality(
            FoundationalPerformancePrimitiveLegalityDenial::CounterBackedExecutionClaimsCannotUseSupportOnlyTemperature
        ))
    );

    let overlapping_disclosure = performance_common::performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .finish();
    assert_eq!(
        overlapping_disclosure,
        Err(
            FoundationalPerformanceClaimConstructionDenial::OverlappingIncludedAndExcludedWorkDisclosure
        )
    );
}
