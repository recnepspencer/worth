use worth_foundational::{
    FoundationalAttachedPerformanceBundle, FoundationalAuthoritativePerformanceClaim,
    FoundationalMaterializedPerformanceReport,
};

fn require_materialized_report(
    _report: &FoundationalMaterializedPerformanceReport<
        FoundationalAttachedPerformanceBundle<FoundationalAuthoritativePerformanceClaim>,
    >,
) {
}

fn main() {
    let claim = worth_foundational::performance()
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
        )
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .unwrap();
    let bundle = worth_foundational::performance_bundle(claim).finish().unwrap();
    let attached = worth_foundational::attach_performance_bundle(
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryArtifact,
        bundle,
    )
    .unwrap();

    require_materialized_report(&attached);
}
