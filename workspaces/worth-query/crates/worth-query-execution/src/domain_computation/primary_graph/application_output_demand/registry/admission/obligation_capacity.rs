use super::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandKey,
};

pub(super) fn prepare_performed_obligation_slots(
    state: &super::super::DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
) -> Result<(Vec<super::super::PerformedOutputObligation>, usize), WorthQueryOutputDemandDenial> {
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
    let bytes = prepared
        .capacity()
        .checked_mul(std::mem::size_of::<super::super::PerformedOutputObligation>())
        .ok_or_else(performed_obligation_capacity_denial)?;
    if state
        .obligation_reserved_bytes
        .checked_add(bytes)
        .is_none_or(|peak| peak > state.obligation_budget_bytes)
    {
        return Err(performed_obligation_capacity_denial());
    }
    Ok((prepared, bytes))
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn performed_obligation_capacity_denial(
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "performed output obligation exceeds the registry custody limit",
    )
    .with_recovery_posture(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable,
    )
}
