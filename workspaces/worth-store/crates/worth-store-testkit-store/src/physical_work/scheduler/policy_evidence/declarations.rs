pub use worth_store_io_scheduler::BackgroundResourceBudget;
pub use {
    worth_foundational::performance,
    worth_foundational::FoundationalPerformanceAccessPatternPosture,
    worth_foundational::FoundationalPerformanceBoundary,
    worth_foundational::FoundationalPerformanceBreadthLocalityPosture,
    worth_foundational::FoundationalPerformanceBudgetKind,
    worth_foundational::FoundationalPerformanceEvidenceStrength,
    worth_foundational::FoundationalPerformanceExecutionTemperature,
    worth_foundational::FoundationalPerformanceFallbackDebtPosture,
    worth_foundational::FoundationalPerformanceFreshnessRetentionPosture,
    worth_foundational::FoundationalPerformanceWorkClass,
    worth_foundational::FoundationalPolicyAdmissionReceipt,
};

pub fn policy_receipt(budget: BackgroundResourceBudget) -> FoundationalPolicyAdmissionReceipt {
    policy_receipt_with_breadth_delta(budget, 0)
}

pub fn mismatched_policy_receipt(
    budget: BackgroundResourceBudget,
) -> FoundationalPolicyAdmissionReceipt {
    policy_receipt_with_breadth_delta(budget, 1)
}

pub fn policy_receipt_with_breadth_delta(
    budget: BackgroundResourceBudget,
    breadth_delta: u32,
) -> FoundationalPolicyAdmissionReceipt {
    policy_receipt_for(
        budget,
        breadth_delta,
        worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
    )
}

pub fn policy_receipt_for(
    budget: BackgroundResourceBudget,
    breadth_delta: u32,
    work_class: FoundationalPerformanceWorkClass,
) -> FoundationalPolicyAdmissionReceipt {
    policy_receipt_with_breadth_posture(budget, breadth_delta, work_class, false)
}

pub fn exhausted_policy_receipt(
    budget: BackgroundResourceBudget,
    work_class: FoundationalPerformanceWorkClass,
) -> FoundationalPolicyAdmissionReceipt {
    policy_receipt_with_breadth_posture(budget, 0, work_class, true)
}

pub fn policy_receipt_with_breadth_posture(
    budget: BackgroundResourceBudget,
    breadth_delta: u32,
    work_class: FoundationalPerformanceWorkClass,
    exhaust_breadth: bool,
) -> FoundationalPolicyAdmissionReceipt {
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
        .include_work(work_class)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .unwrap();
    let amount = |kind| match kind {
        worth_foundational::FoundationalPerformanceBudgetKind::Breadth => {
            (budget.queue_slots() + budget.worker_permits()) as u32 + breadth_delta
        }
        worth_foundational::FoundationalPerformanceBudgetKind::Density => {
            (budget.bandwidth_tokens() + budget.cache_residency_hints()) as u32
        }
        worth_foundational::FoundationalPerformanceBudgetKind::Locality => {
            (budget.read_ahead_window() + budget.write_back_window() + budget.reclaim_permits())
                as u32
        }
        worth_foundational::FoundationalPerformanceBudgetKind::FreshnessSensitive => 0,
    };
    let receipt = performance().policy_admission_receipt(claim);
    [
        worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
        worth_foundational::FoundationalPerformanceBudgetKind::Density,
        worth_foundational::FoundationalPerformanceBudgetKind::Locality,
    ]
    .into_iter()
    .fold(receipt, |receipt, kind| {
        let units = amount(kind);
        if units == 0 {
            receipt
        } else {
            let admitted = if exhaust_breadth
                && kind == worth_foundational::FoundationalPerformanceBudgetKind::Breadth
            {
                0
            } else {
                units
            };
            receipt.budget_decision(kind, units, admitted)
        }
    })
    .finish()
    .unwrap()
}
