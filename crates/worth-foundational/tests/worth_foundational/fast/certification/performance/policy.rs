use worth_foundational::{
    performance, performance_api, FoundationalPerformanceAccessPatternPosture,
    FoundationalPerformanceBoundary, FoundationalPerformanceBreadthLocalityPosture,
    FoundationalPerformanceBudgetKind, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceWorkClass,
    FoundationalPolicyAdmissionReceiptConstructionDenial,
};

#[test]
fn common_path_and_lower_lane_expose_the_same_phase4_budget_surface() {
    assert_eq!(
        performance().budget_definitions(),
        performance_api::lower_lane::policy::foundational_performance_budget_definitions()
    );
}

#[test]
fn policy_receipts_lower_pre_execution_admission_with_budget_and_stronger_evidence_gap() {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::DeltaBound)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("policy claim should build");

    let receipt = performance_api::lower_lane::policy::policy_admission_receipt(claim)
        .budget_decision(FoundationalPerformanceBudgetKind::Breadth, 8, 8)
        .budget_decision(FoundationalPerformanceBudgetKind::Locality, 1, 1)
        .finish()
        .expect("verified receipt should build");

    assert_eq!(
        receipt.evidence_strength(),
        FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission
    );
    assert_eq!(
        receipt.stronger_evidence_still_required(),
        FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt
    );
    assert_eq!(
        receipt.included_work(),
        &[FoundationalPerformanceWorkClass::ValidationPlanning]
    );
    assert_eq!(
        receipt.excluded_work(),
        &[FoundationalPerformanceWorkClass::SupportReportAssembly]
    );
}

#[test]
fn widened_and_rejected_policy_receipts_remain_family_distinct_and_fail_closed() {
    let widened_claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::DensityAdaptive)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::WidenedWithExplicitDisclosure)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("widened claim should build");

    let widened_receipt = performance()
        .policy_admission_receipt(widened_claim)
        .budget_decision(FoundationalPerformanceBudgetKind::Density, 4, 6)
        .widen_work(FoundationalPerformanceWorkClass::ForensicParity)
        .finish()
        .expect("widened receipt should build");
    assert_eq!(
        widened_receipt.widened_work(),
        &[FoundationalPerformanceWorkClass::ForensicParity]
    );

    let rejected_claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::RestoreRecovery)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::SnapshotBound)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::RebuildCapable)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::RecoveryOnly)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ReplayDerived)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Rejected)
        .include_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("rejected policy claim should build");

    let rejected_receipt = performance()
        .policy_admission_receipt(rejected_claim)
        .budget_decision(FoundationalPerformanceBudgetKind::FreshnessSensitive, 3, 0)
        .deny_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .finish()
        .expect("rejected receipt should build");
    assert_eq!(
        rejected_receipt.denied_work(),
        &[FoundationalPerformanceWorkClass::ReplayReconstruction]
    );

    let dishonest_verified = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("verified policy claim should build");

    let denial = performance()
        .policy_admission_receipt(dishonest_verified)
        .budget_decision(FoundationalPerformanceBudgetKind::Breadth, 2, 3)
        .finish();
    assert_eq!(
        denial,
        Err(
            FoundationalPolicyAdmissionReceiptConstructionDenial::VerifiedDeferredOrDebtReceiptsCannotWidenBudget
        )
    );

    let compile_time_claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CompileTimeContract)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::ScanHeavy)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("compile-time policy claim should build");

    let compile_time_denial = performance()
        .policy_admission_receipt(compile_time_claim)
        .budget_decision(FoundationalPerformanceBudgetKind::Breadth, 2, 2)
        .finish();
    assert_eq!(
        compile_time_denial,
        Err(
            FoundationalPolicyAdmissionReceiptConstructionDenial::CompileTimeContractsCannotBecomePolicyReceipts
        )
    );

    let contradictory_rejected_claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::RestoreRecovery)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::SnapshotBound)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::RebuildCapable)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::RecoveryOnly)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ReplayDerived)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Rejected)
        .include_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("rejected policy claim should build");

    let contradictory_denial = performance()
        .policy_admission_receipt(contradictory_rejected_claim)
        .budget_decision(FoundationalPerformanceBudgetKind::FreshnessSensitive, 3, 0)
        .deny_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .widen_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .finish();
    assert_eq!(
        contradictory_denial,
        Err(
            FoundationalPolicyAdmissionReceiptConstructionDenial::OverlappingDeniedAndWidenedWorkDisclosure
        )
    );
}
