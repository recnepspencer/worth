//! Request admission for the installed producer's receipt-role projection.

use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;

pub(super) fn admit_readiness_record(
    role: &str,
    receipt: &WorthQueryApplicationCommitReceipt,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    let resource_denial = |stop| {
        let kind = match stop {
            Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            }
            _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        };
        denial(kind, "")
    };
    admission.charge_external_work(4).map_err(resource_denial)?;
    let initialized = std::mem::size_of::<worth_relational::facade::identity::EntityId>()
        .checked_add(std::mem::size_of::<
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        >())
        .and_then(|work| work.checked_add(4))
        .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
    admission
        .charge_external_work(
            u64::try_from(initialized)
                .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?,
        )
        .map_err(resource_denial)?;
    receipt
        .output_correspondence()
        .admit_selected_role_lookup(role, admission)
        .map_err(resource_denial)
}
