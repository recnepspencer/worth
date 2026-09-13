use std::collections::BTreeSet;

use worth_store_budgets::{PreExecutionBudgetEnvelope, PreExecutionBudgetScope};
use worth_store_contracts::WalRecordFamily;
use worth_store_layout_indexes::{
    layout_lsm_maintenance, LsmCompactionAdmissionRequest, LsmMaintenanceOwnerCaseObservation,
    LsmRunPublicationAdmissionRequest,
};
use worth_store_wal::StoreWalRecordIdentity;

pub fn observe_lsm_maintenance_owner_cases() -> Vec<LsmMaintenanceOwnerCaseObservation> {
    let current =
        worth_store_security::admitted_store_wal_checkpoint_security_scope_for_layout_partition_test();
    let foreign =
        worth_store_security::admitted_tenant_wal_checkpoint_security_scope_for_layout_partition_test();
    let page =
        worth_store_security::admitted_tenant_page_security_scope_for_layout_partition_test();
    let budgets = [
        PreExecutionBudgetEnvelope::maintenance_default(),
        zero_budget(),
    ];

    let mut observations = Vec::new();
    let mut publication = BTreeSet::new();
    let mut compaction = BTreeSet::new();
    for security in [&current, &foreign, &page] {
        for budget in budgets {
            retain_once(
                &mut observations,
                &mut publication,
                layout_lsm_maintenance()
                    .admit_run_publication(LsmRunPublicationAdmissionRequest::new(
                        security.witnesses(),
                        WalRecordFamily::DurableMutationIntent,
                        StoreWalRecordIdentity::new(43),
                        budget,
                    ))
                    .owner_case_observation(),
            );
            retain_once(
                &mut observations,
                &mut compaction,
                layout_lsm_maintenance()
                    .admit_compaction(LsmCompactionAdmissionRequest::new(
                        security.witnesses(),
                        WalRecordFamily::DurableMutationIntent,
                        StoreWalRecordIdentity::new(43),
                        budget,
                    ))
                    .owner_case_observation(),
            );
        }
    }
    observations
}

fn retain_once(
    observations: &mut Vec<LsmMaintenanceOwnerCaseObservation>,
    seen: &mut BTreeSet<worth_store_layout_indexes::LsmMaintenanceOwnerCaseId>,
    observed: LsmMaintenanceOwnerCaseObservation,
) {
    if seen.insert(observed.id()) {
        observations.push(observed);
    }
}

fn zero_budget() -> PreExecutionBudgetEnvelope {
    PreExecutionBudgetEnvelope::new(PreExecutionBudgetScope::Maintenance, 0, 0, 0, 0, 0)
}
