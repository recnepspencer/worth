use worth_foundational::{
    performance, FoundationalPerformanceAccessPatternPosture, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceBudgetKind,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
    FoundationalPerformanceWorkClass, FoundationalPolicyAdmissionReceipt,
    FoundationalPolicyAdmissionReceiptBuilder,
};
use worth_store_io_scheduler::{QueueDurabilityClass, QueueWorkDeclaration};

pub(in crate::physical_runtime) fn admit_record_queue_policy(
    work: &QueueWorkDeclaration,
) -> FoundationalPolicyAdmissionReceipt {
    let budget = work.requested_budget();
    let work_class = match work.durability_class() {
        QueueDurabilityClass::ReadOnly => FoundationalPerformanceWorkClass::AuthoritativeRead,
        QueueDurabilityClass::BufferedWrite | QueueDurabilityClass::WalCommit => {
            FoundationalPerformanceWorkClass::AuthoritativeMutation
        }
        QueueDurabilityClass::PlatformDurable => {
            FoundationalPerformanceWorkClass::PublicationDelivery
        }
    };
    admit_exact_queue_policy(
        budget,
        work_class,
        FoundationalPerformanceAccessPatternPosture::PointLookup,
        FoundationalPerformanceExecutionTemperature::HotPath,
    )
}

pub(in crate::physical_runtime) fn admit_checkpoint_background_policy(
    budget: worth_store_io_scheduler::BackgroundResourceBudget,
) -> FoundationalPolicyAdmissionReceipt {
    admit_exact_queue_policy(
        budget,
        FoundationalPerformanceWorkClass::AuthoritativeMutation,
        FoundationalPerformanceAccessPatternPosture::RebuildCapable,
        FoundationalPerformanceExecutionTemperature::ColdPath,
    )
}

pub(in crate::physical_runtime) fn admit_wal_reclamation_background_policy(
    budget: worth_store_io_scheduler::BackgroundResourceBudget,
) -> FoundationalPolicyAdmissionReceipt {
    admit_exact_queue_policy(
        budget,
        FoundationalPerformanceWorkClass::AuthoritativeMutation,
        FoundationalPerformanceAccessPatternPosture::RebuildCapable,
        FoundationalPerformanceExecutionTemperature::ColdPath,
    )
}

fn admit_exact_queue_policy(
    budget: worth_store_io_scheduler::BackgroundResourceBudget,
    work_class: FoundationalPerformanceWorkClass,
    access_pattern: FoundationalPerformanceAccessPatternPosture,
    temperature: FoundationalPerformanceExecutionTemperature,
) -> FoundationalPolicyAdmissionReceipt {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::DeltaBound)
        .access_pattern(access_pattern)
        .execution_temperature(temperature)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(work_class)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("the fixed record queue policy claim is valid");
    let receipt = performance().policy_admission_receipt(claim);
    [
        (
            FoundationalPerformanceBudgetKind::Breadth,
            sum(&[budget.queue_slots(), budget.worker_permits()]),
        ),
        (
            FoundationalPerformanceBudgetKind::Density,
            sum(&[
                budget.bandwidth_tokens(),
                budget.dirty_page_budget(),
                budget.cache_residency_hints(),
            ]),
        ),
        (
            FoundationalPerformanceBudgetKind::Locality,
            sum(&[
                budget.read_ahead_window(),
                budget.write_back_window(),
                budget.reclaim_permits(),
            ]),
        ),
        (
            FoundationalPerformanceBudgetKind::FreshnessSensitive,
            sum(&[budget.flush_permits(), budget.sync_debt()]),
        ),
    ]
    .into_iter()
    .fold(receipt, add_exact_budget)
    .finish()
    .expect("exact record queue budget decisions match their claim")
}

pub(in crate::physical_runtime) fn admit_scrub_background_policy(
    budget: worth_store_io_scheduler::BackgroundResourceBudget,
) -> FoundationalPolicyAdmissionReceipt {
    admit_exact_queue_policy(
        budget,
        FoundationalPerformanceWorkClass::ValidationPlanning,
        FoundationalPerformanceAccessPatternPosture::RebuildCapable,
        FoundationalPerformanceExecutionTemperature::ColdPath,
    )
}

fn add_exact_budget(
    receipt: FoundationalPolicyAdmissionReceiptBuilder,
    (kind, units): (FoundationalPerformanceBudgetKind, u32),
) -> FoundationalPolicyAdmissionReceiptBuilder {
    if units == 0 {
        receipt
    } else {
        receipt.budget_decision(kind, units, units)
    }
}

fn sum(units: &[u64]) -> u32 {
    let total = units
        .iter()
        .try_fold(0_u64, |total, unit| total.checked_add(*unit))
        .expect("admitted record queue resource units cannot overflow u64");
    u32::try_from(total).expect("record queue scope and byte limits fit policy units")
}

#[cfg(test)]
mod tests {
    #[test]
    fn zero_budget_categories_are_not_fabricated() {
        let budget = worth_store_io_scheduler::BackgroundResourceBudget::new()
            .with_queue_slots(worth_store_io_scheduler::QueueSlot::new(1).unwrap())
            .with_worker_permits(worth_store_io_scheduler::WorkerPermit::new(1).unwrap())
            .with_bandwidth(worth_store_io_scheduler::BandwidthToken::bytes(8).unwrap());
        assert_eq!(budget.flush_permits(), 0);
        assert_eq!(budget.sync_debt(), 0);
    }
}
