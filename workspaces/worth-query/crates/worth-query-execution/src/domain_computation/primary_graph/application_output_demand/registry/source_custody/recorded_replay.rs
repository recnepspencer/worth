//! Join original recorded meaning to an actual retained source; never reconstruct custody.
use super::{PreparedOutputRootKind, WorthQueryOutputDemandRegistry};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryOutputSourceEvidence,
    WorthQueryOutputSourceReplay,
};
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn resolve_replay_source(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        selector: WorthQueryOutputSourceReplay<'_>,
        kind: PreparedOutputRootKind,
    ) -> Result<WorthQueryOutputSourceEvidence, WorthQueryOutputDemandDenial> {
        checkpoint(context)?;
        if let WorthQueryOutputSourceReplay::Retained(receipt) = selector {
            let receipt: WorthQueryOutputSourceEvidence = receipt.clone().into();
            self.validate_recovery_root_kind(&receipt, kind)?;
            return Ok(receipt);
        }
        let WorthQueryOutputSourceReplay::Recorded(recorded) = selector else {
            unreachable!("retained selector returned above")
        };
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (commit, custody) in &state.source_custody {
            checkpoint(context)?;
            let Some(source) = custody.source.as_ref() else {
                continue;
            };
            if !source.receipt.matches_recorded_original(recorded) {
                continue;
            }
            if custody.completed
                || !super::super::post_commit::custody_reader::visible(&state, commit)
            {
                break;
            }
            if let Some(retired) = &custody.retired {
                return Err(retired.clone());
            }
            if custody.root_kind != kind {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "recorded source belongs to another output root",
                ));
            }
            return Ok(source.receipt.clone());
        }
        Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            "recorded original has no live retained source custody",
        ))
    }
}

fn checkpoint(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), WorthQueryOutputDemandDenial> {
    (if context.has_accounting() { context.checkpoint(1) } else { context.poll() })
        .map_err(|cause| WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ExecutionRequest(crate::domain_computation::primary_graph::application_contribution::advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause))), "registered-source-replay"))
}
