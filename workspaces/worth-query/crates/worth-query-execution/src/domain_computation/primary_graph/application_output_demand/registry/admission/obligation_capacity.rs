use super::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandKey,
};

pub(super) fn prepare_performed_obligation_slots(
    state: &super::super::DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) -> Result<Vec<super::super::PerformedOutputObligation>, WorthQueryOutputDemandDenial> {
    let count = state
        .records
        .get(key)
        .map_or(0, |record| record.performed_obligations.len())
        .checked_add(1)
        .ok_or_else(performed_obligation_capacity_denial)?;
    let mut prepared = Vec::new();
    prepared
        .try_reserve_exact(count)
        .map_err(|_| performed_obligation_capacity_denial())?;
    Ok(prepared)
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn performed_obligation_capacity_denial(
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "performed output obligation allocation refused",
    )
    .with_recovery_posture(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable,
    )
}
