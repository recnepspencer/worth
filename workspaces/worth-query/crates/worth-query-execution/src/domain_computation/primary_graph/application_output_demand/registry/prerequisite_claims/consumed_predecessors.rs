//! Resolve actual consumed reads into exact managed prerequisite custody.
use super::super::DemandRegistryState;
use super::super::WorthQueryOutputDemandKey;
use super::coverage_denial;
use super::prerequisite_denials;
use super::CheckpointPrerequisiteClaims;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture;
use std::sync::Arc;

pub(super) fn select<'a>(
    state: &DemandRegistryState,
    downstream: &WorthQueryOutputDemandKey,
    inputs: impl ExactSizeIterator<Item = &'a ConsumedOutputEvidence>,
    owner: &SourceInvalidationOwner,

    predecessors: &mut Vec<Arc<WorthQueryOutputDemandKey>>,
) -> Result<Option<CheckpointPrerequisiteClaims>, WorthQueryOutputDemandDenial> {
    let maximum = inputs.len();
    let mut checkpoint_predecessors: Option<CheckpointPrerequisiteClaims> = None;
    for consumed in inputs {
        let upstream = state
            .settlement_keys
            .get_exact_admitted(consumed.identity())?;
        let Some(upstream) = upstream else {
            // A checkpoint root has no executable source row until an ordinary
            // typed root demand reads its original input. A native reader may
            // nevertheless have verified this exact root and sealed its witness.
            // Retain only that static settlement; never synthesize a source key.
            let verified = consumed.claim_verified_checkpoint_root(owner)
                .map_err(|stop| match stop {
                    crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop::Admission(stop) => match stop {
                        worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. } | worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
                        | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => prerequisite_denials::work_denial(),
                        _ => prerequisite_denials::capacity_denial(),
                    },
                    _ => prerequisite_denials::stale_upstream_denial(),
                })?;
            if !verified {
                return Err(prerequisite_denials::stale_upstream_denial());
            }
            let claims = if let Some(claims) = &mut checkpoint_predecessors {
                claims
            } else {
                checkpoint_predecessors.insert(CheckpointPrerequisiteClaims::prepare(maximum)?)
            };
            claims.retain_verified(consumed.identity());
            continue;
        };
        if upstream.as_ref() == downstream {
            return Err(coverage_denial());
        }

        let record = state
            .records
            .get(upstream.as_ref())
            .ok_or_else(coverage_denial)?;
        if !record.has_cached_ready() {
            let requested = consumed.requested_read().map_err(|stop| {
                use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
                match stop {
                    Stop::WorkExhausted { .. }
                    | Stop::WorkExhaustedWithoutRequirement { .. }
                    | Stop::WorkCounterOverflow => prerequisite_denials::work_denial(),
                    _ => prerequisite_denials::capacity_denial(),
                }
            })?;
            let mut denial = coverage_denial()
                .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable);
            denial = denial.with_requested_output(requested);
            return Err(denial);
        }
        predecessors.push(upstream);
    }
    Ok(checkpoint_predecessors)
}
