#[allow(unused_imports)] use crate::certification::ordinary_context::performance::claims::*;
use {worth_foundational::foundational_responsibilities, worth_foundational::performance_api::common_path as performance_common, worth_foundational::FoundationalAuthoritativePerformanceClaim, worth_foundational::FoundationalPerformanceBoundary, worth_foundational::FoundationalPerformanceBreadthLocalityPosture, worth_foundational::FoundationalPerformanceClaimConstructionDenial, worth_foundational::FoundationalPerformanceEvidenceStrength, worth_foundational::FoundationalPerformanceExecutionTemperature, worth_foundational::FoundationalPerformanceFallbackDebtPosture, worth_foundational::FoundationalPerformanceFreshnessRetentionPosture, worth_foundational::FoundationalPerformancePrimitiveLegalityDenial, worth_foundational::FoundationalPerformanceWorkClass, worth_foundational::FoundationalPolicyAdmissionPerformanceClaim, worth_foundational::FoundationalReplayMaterializationPerformanceClaim, worth_foundational::FoundationalSupportDerivedPerformanceClaim};

#[test]
fn common_path_claim_authoring_surfaces_live_under_the_performance_responsibility_home() {
    let names: Vec<_> = foundational_responsibilities()
        .iter()
        .map(|area| area.name())
        .collect();

    assert!(names.contains(&"performance"));
    let authoring = worth_foundational::performance_api::common_path::performance().claim();
    let _ = authoring.authoritative_execution();
    let _ = authoring.support_derived();
    let _ = authoring.replay_or_materialization();
    let _ = authoring.policy_admission();
}

#[test]
fn authoritative_claims_require_full_disclosure_and_canonicalize_work_order() {
    let claim = worth_foundational::performance_api::common_path::performance()
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .finish()
        .expect("authoritative claim");

    require_authoritative_claim(&claim);
    assert_eq!(
        claim.boundary(),
        worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution
    );
    assert_eq!(
        claim.included_work(),
        &[
            worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
            worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning,
        ]
    );
    assert_eq!(
        claim.excluded_work(),
        &[
            worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction,
            worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
            worth_foundational::FoundationalPerformanceWorkClass::ForensicParity,
        ]
    );
}

#[test]
fn support_replay_and_policy_claims_remain_family_distinct_lowered_types() {
    let support_claim = worth_foundational::performance_api::common_path::performance()
        .claim()
        .support_derived()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::SupportAssembly)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::BranchLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal,
        )
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::SupportOnly)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::StaleSupport)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Debt)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("support claim");
    require_support_claim(&support_claim);

    let replay_claim = worth_foundational::performance_api::common_path::performance()
        .claim()
        .replay_or_materialization()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::ReplayReconstruction)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::SnapshotBound)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::RebuildCapable,
        )
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::ColdPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ReplayDerived)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("replay claim");
    require_replay_claim(&replay_claim);

    let policy_claim = worth_foundational::performance_api::common_path::performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::DeltaBound)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("policy claim");
    require_policy_claim(&policy_claim);
}

#[test]
fn claim_builders_fail_closed_for_missing_or_incompatible_phase2_shapes() {
    let missing_disclosure = worth_foundational::performance_api::common_path::performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CompileTimeContract)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Deferred)
        .finish();
    assert_eq!(
        missing_disclosure,
        Err(worth_foundational::FoundationalPerformanceClaimConstructionDenial::MissingIncludedWorkDisclosure)
    );

    let incompatible_strength = worth_foundational::performance_api::common_path::performance()
        .claim()
        .support_derived()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::SupportAssembly)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::BranchLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal,
        )
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::SupportOnly)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::StaleSupport)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Debt)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish();
    assert_eq!(
        incompatible_strength,
        Err(
            worth_foundational::FoundationalPerformanceClaimConstructionDenial::EvidenceStrengthNotAllowedForClaimFamily
        )
    );

    let hot_path_support_collapse = worth_foundational::performance_api::common_path::performance()
        .claim()
        .support_derived()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::SupportAssembly)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Debt)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish();
    assert_eq!(
        hot_path_support_collapse,
        Err(worth_foundational::FoundationalPerformanceClaimConstructionDenial::PrimitiveLegality(
            worth_foundational::FoundationalPerformancePrimitiveLegalityDenial::SupportDerivedClaimsCannotUseHotPathTemperature
        ))
    );

    let support_only_executed_collapse = worth_foundational::performance_api::common_path::performance()
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::SupportOnly)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish();
    assert_eq!(
        support_only_executed_collapse,
        Err(worth_foundational::FoundationalPerformanceClaimConstructionDenial::PrimitiveLegality(
            worth_foundational::FoundationalPerformancePrimitiveLegalityDenial::CounterBackedExecutionClaimsCannotUseSupportOnlyTemperature
        ))
    );

    let overlapping_disclosure = worth_foundational::performance_api::common_path::performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .finish();
    assert_eq!(
        overlapping_disclosure,
        Err(
            worth_foundational::FoundationalPerformanceClaimConstructionDenial::OverlappingIncludedAndExcludedWorkDisclosure
        )
    );
}
