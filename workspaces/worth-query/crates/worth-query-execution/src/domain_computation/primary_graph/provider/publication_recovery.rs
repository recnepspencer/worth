use super::{WorthQueryPrimaryGraphProvider, WorthQueryProviderIdempotencyResolution};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationSettlementRecoveryError,
};

impl WorthQueryPrimaryGraphProvider {
    pub(super) fn repair_equivalent_publication_settlement(
        &self,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        committed: &super::WorthQueryPrimaryGraphCommittedApplication,
    ) -> Result<(), super::WorthQueryProviderIdempotencyResolutionDenial> {
        runtime
            .repair_pending_publication_settlement(committed.commit_reference().commit_id)
            .map(|_| ())
            .map_err(|_| super::WorthQueryProviderIdempotencyResolutionDenial::Unavailable)
    }

    pub(in crate::domain_computation::primary_graph) fn recover_application_settlement(
        &self,
        settlement: &worth_relational::facade::publication::DeferredPublicationSettlement,
        branch: &worth_relational::facade::history::BranchId,
        product: &super::WorthQueryProductIdempotencyAffinity,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        WorthQueryApplicationSettlementRecoveryError,
    > {
        let _serialization = self.serialize_application_commit();
        let repaired = self.graph.with_runtime_mut(|runtime| {
            let repaired = runtime
                .repair_deferred_publication_settlement(settlement)
                .map_err(WorthQueryApplicationSettlementRecoveryError::Durability)?;
            if &repaired.branch_id != branch
                || &repaired != settlement.commit()
                || repaired != settlement.performed_result().commit
            {
                return Err(WorthQueryApplicationSettlementRecoveryError::Publication(
                    "deferred application settlement does not match its performed publication",
                ));
            }
            self.resume_pending_application_publication(runtime)
                .map_err(settlement_publication_denial)?;
            Ok(repaired)
        })?;
        match self.resolve_completed_application_idempotency(product, idempotency) {
            Some(WorthQueryProviderIdempotencyResolution::Equivalent(_)) => Ok(repaired),
            None | Some(WorthQueryProviderIdempotencyResolution::Absent) => {
                Err(WorthQueryApplicationSettlementRecoveryError::IdempotencyAbsent)
            }
            Some(WorthQueryProviderIdempotencyResolution::Drift) => {
                Err(WorthQueryApplicationSettlementRecoveryError::IdempotencyDrift)
            }
            Some(WorthQueryProviderIdempotencyResolution::Unpublished) => {
                Err(WorthQueryApplicationSettlementRecoveryError::IdempotencyAbsent)
            }
        }
    }
}

fn settlement_publication_denial(
    failure: crate::domain_computation::WorthQueryProviderSessionFailure,
) -> WorthQueryApplicationSettlementRecoveryError {
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
    match failure.kind() {
        Kind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryApplicationSettlementRecoveryError::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Kind::RetentionCapacityExhausted => {
            WorthQueryApplicationSettlementRecoveryError::RetentionCapacityExhausted
        }
        Kind::RetentionIdentityExhausted => {
            WorthQueryApplicationSettlementRecoveryError::RetentionIdentityExhausted
        }
        Kind::SnapshotIdentityExhausted => {
            WorthQueryApplicationSettlementRecoveryError::SnapshotIdentityExhausted
        }
        _ => WorthQueryApplicationSettlementRecoveryError::Publication(
            "application publication could not resume from its retained settlement",
        ),
    }
}
