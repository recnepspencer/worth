//! Resolve actual consumed reads into exact managed prerequisite custody.
use super::super::{DemandRegistryState, WorthQueryOutputDemandKey};
use super::{coverage_denial, prerequisite_denials};
use crate::domain_computation::primary_graph::{
    invariant_projection::ConsumedOutputEvidence,
    output_lineage::invalidation::InvalidationEditAdmission, SourceInvalidationOwner,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandRecoveryPosture,
};
use std::sync::Arc;

pub(super) fn select<'a>(
    state: &DemandRegistryState,
    downstream: &WorthQueryOutputDemandKey,
    inputs: impl Iterator<Item = &'a ConsumedOutputEvidence>,
    owner: &SourceInvalidationOwner,
    admission: &mut InvalidationEditAdmission,
    predecessors: &mut Vec<Arc<WorthQueryOutputDemandKey>>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    for consumed in inputs {
        let upstream = state
            .settlement_keys
            .get_exact_admitted(consumed.identity(), admission)?
            .ok_or_else(prerequisite_denials::stale_upstream_denial)?;
        if upstream.as_ref() == downstream {
            return Err(coverage_denial());
        }
        state.charge_record_lookup(&upstream, admission)?;
        let record = state
            .records
            .get(upstream.as_ref())
            .ok_or_else(coverage_denial)?;
        if !record.has_cached_ready() {
            let requested = consumed.requested_read(owner, admission).map_err(|stop| {
                use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
                match stop {
                    Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
                        prerequisite_denials::work_denial()
                    }
                    _ => prerequisite_denials::capacity_denial(),
                }
            })?;
            let mut denial = coverage_denial()
                .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable);
            denial.requested_output = Some(requested);
            return Err(denial);
        }
        predecessors.push(upstream);
    }
    Ok(())
}
