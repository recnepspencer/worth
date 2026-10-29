use {
    worth_foundational::performance, worth_foundational::performance_api,
    worth_foundational::FoundationalPerformanceAccessPatternPosture,
    worth_foundational::FoundationalPerformanceBoundary,
    worth_foundational::FoundationalPerformanceBreadthLocalityPosture,
    worth_foundational::FoundationalPerformanceBudgetKind,
    worth_foundational::FoundationalPerformanceEvidenceStrength,
    worth_foundational::FoundationalPerformanceExecutionTemperature,
    worth_foundational::FoundationalPerformanceFallbackDebtPosture,
    worth_foundational::FoundationalPerformanceFreshnessRetentionPosture,
    worth_foundational::FoundationalPerformanceWorkClass,
    worth_foundational::FoundationalPolicyAdmissionReceiptConstructionDenial,
};

#[test]
fn common_path_and_lower_lane_expose_the_same_budget_surface() {
    assert_eq!(
        performance().budget_definitions(),
        worth_foundational::performance_api::lower_lane::policy::foundational_performance_budget_definitions()
    );
}

#[test]
fn policy_receipts_lower_pre_execution_admission_with_budget_and_stronger_evidence_gap() {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission,
        )
        .breadth_locality(
            worth_foundational::FoundationalPerformanceBreadthLocalityPosture::DeltaBound,
        )
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(
            worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
        )
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("policy claim should build");

    let receipt =
        worth_foundational::performance_api::lower_lane::policy::policy_admission_receipt(claim)
            .budget_decision(
                worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
                8,
                8,
            )
            .budget_decision(
                worth_foundational::FoundationalPerformanceBudgetKind::Locality,
                1,
                1,
            )
            .finish()
            .expect("verified receipt should build");

    assert_eq!(
        receipt.evidence_strength(),
        worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission
    );
    assert_eq!(
        receipt.stronger_evidence_still_required(),
        worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt
    );
    assert_eq!(
        receipt.included_work(),
        &[worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning]
    );
    assert_eq!(
        receipt.excluded_work(),
        &[worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly]
    );
}

#[test]
fn widened_and_rejected_policy_receipts_remain_family_distinct_and_fail_closed() {
    let widened_claim = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::DensityAdaptive)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::WidenedWithExplicitDisclosure)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("widened claim should build");

    let widened_receipt = performance()
        .policy_admission_receipt(widened_claim)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Density,
            4,
            6,
        )
        .widen_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity)
        .finish()
        .expect("widened receipt should build");
    assert_eq!(
        widened_receipt.widened_work(),
        &[worth_foundational::FoundationalPerformanceWorkClass::ForensicParity]
    );

    let rejected_claim = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::RestoreRecovery)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim,
        )
        .breadth_locality(
            worth_foundational::FoundationalPerformanceBreadthLocalityPosture::SnapshotBound,
        )
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::RebuildCapable,
        )
        .execution_temperature(
            worth_foundational::FoundationalPerformanceExecutionTemperature::RecoveryOnly,
        )
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ReplayDerived,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Rejected)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("rejected policy claim should build");

    let rejected_receipt = performance()
        .policy_admission_receipt(rejected_claim)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::FreshnessSensitive,
            3,
            0,
        )
        .deny_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .finish()
        .expect("rejected receipt should build");
    assert_eq!(
        rejected_receipt.denied_work(),
        &[worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction]
    );

    let dishonest_verified = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission,
        )
        .breadth_locality(
            worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal,
        )
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        )
        .execution_temperature(
            worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath,
        )
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("verified policy claim should build");

    let denial = performance()
        .policy_admission_receipt(dishonest_verified)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
            2,
            3,
        )
        .finish();
    assert_eq!(
        denial,
        Err(
            worth_foundational::FoundationalPolicyAdmissionReceiptConstructionDenial::VerifiedDeferredOrDebtReceiptsCannotWidenBudget
        )
    );

    let compile_time_claim = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::MaintenancePlanning)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CompileTimeContract)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::HistoricalRetained)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("compile-time policy claim should build");

    let compile_time_denial = performance()
        .policy_admission_receipt(compile_time_claim)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
            2,
            2,
        )
        .finish();
    assert_eq!(
        compile_time_denial,
        Err(
            worth_foundational::FoundationalPolicyAdmissionReceiptConstructionDenial::CompileTimeContractsCannotBecomePolicyReceipts
        )
    );

    let contradictory_rejected_claim = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::RestoreRecovery)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim,
        )
        .breadth_locality(
            worth_foundational::FoundationalPerformanceBreadthLocalityPosture::SnapshotBound,
        )
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::RebuildCapable,
        )
        .execution_temperature(
            worth_foundational::FoundationalPerformanceExecutionTemperature::RecoveryOnly,
        )
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ReplayDerived,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Rejected)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("rejected policy claim should build");

    let contradictory_denial = performance()
        .policy_admission_receipt(contradictory_rejected_claim)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::FreshnessSensitive,
            3,
            0,
        )
        .deny_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .widen_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .finish();
    assert_eq!(
        contradictory_denial,
        Err(
            worth_foundational::FoundationalPolicyAdmissionReceiptConstructionDenial::OverlappingDeniedAndWidenedWorkDisclosure
        )
    );
}
