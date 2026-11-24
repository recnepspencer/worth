//! Only complete actual root custody enters the concrete recovered phase.
use super::*;
use crate::branch::RecordedRelationalComponentReference;
use crate::durability::authority::associated_effects::discovery::current_settlement;
use crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority;
use crate::runtime::RelationalRuntime;

impl RecoveredRelationalRuntimeAuthority {
    /// Descriptively verify that an already-readmitted Native effect advanced
    /// `predecessor` to `successor` without re-pinning a disposed predecessor.
    pub fn validate_recorded_publication_predecessor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        effect: &ReadmittedRelationalAssociatedEffect,
        predecessor: &RecordedRelationalComponentReference,
        successor: &RecordedRelationalComponentReference,
    ) -> Result<(), RelationalRecoveredPublicationAdmissionCause> {
        use RelationalRecoveredPublicationAdmissionCause as Cause;
        crate::execution::check_request(execution).map_err(Cause::Request)?;
        let Some(_operation) = runtime.owner_binding().admit() else {
            return Err(Cause::OwnerUnavailable);
        };
        require_settled_original(self, runtime, effect)?;
        super::subject::validate(execution, effect, predecessor, successor)
    }

    /// Descriptively verify an already-readmitted Native effect whose disposed
    /// predecessor image was pruned by a checkpoint, binding exact predecessor
    /// branch and commit identity plus all successor publication axes.
    pub fn validate_recorded_publication_pruned_predecessor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        effect: &ReadmittedRelationalAssociatedEffect,
        expected_branch: &crate::history::data::BranchId,
        expected_predecessor_commit: crate::history::data::CommitId,
        successor: &RecordedRelationalComponentReference,
    ) -> Result<(), RelationalRecoveredPublicationAdmissionCause> {
        use RelationalRecoveredPublicationAdmissionCause as Cause;
        crate::execution::check_request(execution).map_err(Cause::Request)?;
        let Some(_operation) = runtime.owner_binding().admit() else {
            return Err(Cause::OwnerUnavailable);
        };
        require_settled_original(self, runtime, effect)?;
        super::subject::validate_pruned_predecessor(
            execution,
            effect,
            expected_branch,
            expected_predecessor_commit,
            successor,
        )
    }

    pub fn admit_recorded_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        effect: ReadmittedRelationalAssociatedEffect,
        predecessor: &RecordedRelationalComponentReference,
        successor: &RecordedRelationalComponentReference,
    ) -> Result<
        worth_proof::RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
        RefusedRelationalPublicationReadmission,
    > {
        use RelationalRecoveredPublicationAdmissionCause as Cause;
        let mut partial = PartialRecoveredPublication {
            effect,
            predecessor: None,
            predecessor_lease: None,
            successor: None,
            successor_lease: None,
        };
        if let Err(cause) = crate::execution::check_request(execution) {
            return Err(partial.refuse(Cause::Request(cause)));
        }
        let Some(_operation) = runtime.owner_binding().admit() else {
            return Err(partial.refuse(Cause::OwnerUnavailable));
        };
        if let Err(cause) =
            require_settled_original(self, runtime, &partial.effect).and_then(|()| {
                super::subject::validate(execution, &partial.effect, predecessor, successor)
            })
        {
            return Err(partial.refuse(cause));
        }
        partial.predecessor = match self.admit_recorded_component(execution, runtime, predecessor) {
            Ok(basis) => Some(basis.into_basis()),
            Err(cause) => return Err(partial.refuse(Cause::Component(Box::new(cause)))),
        };
        partial.successor = match self.admit_recorded_component(execution, runtime, successor) {
            Ok(basis) => Some(basis.into_basis()),
            Err(cause) => return Err(partial.refuse(Cause::Component(Box::new(cause)))),
        };
        // Retain both actual owner-admitted observations for the concrete next
        // owner before exposing any permission to transfer this custody.
        for successor in [false, true] {
            if let Err(cause) = crate::execution::check_request(execution) {
                return Err(partial.refuse(Cause::Request(cause)));
            }
            let basis = if successor {
                partial.successor.as_ref()
            } else {
                partial.predecessor.as_ref()
            };
            let Some(basis) = basis else {
                return Err(partial.refuse(Cause::IncompleteCustody));
            };
            let lease = match runtime.retain_component_basis(basis) {
                Ok(lease) => lease,
                Err(cause) => return Err(partial.refuse(Cause::Retention(cause))),
            };
            if successor {
                partial.successor_lease = Some(lease);
            } else {
                partial.predecessor_lease = Some(lease);
            }
        }
        if let Err(cause) = crate::execution::check_request(execution) {
            return Err(partial.refuse(Cause::Request(cause)));
        }
        if let Err(cause) =
            require_settled_original(self, runtime, &partial.effect).and_then(|()| {
                super::subject::validate(execution, &partial.effect, predecessor, successor)
            })
        {
            return Err(partial.refuse(cause));
        }
        partial
            .finish()
            .map(worth_proof::RelationalPublicationReadmitted::from_owner)
    }
}

fn require_settled_original(
    recovered: &RecoveredRelationalRuntimeAuthority,
    runtime: &RelationalRuntime,
    effect: &ReadmittedRelationalAssociatedEffect,
) -> Result<(), RelationalRecoveredPublicationAdmissionCause> {
    use crate::durability::authority::RelationalAssociatedEffectSettlement as Settlement;
    use RelationalRecoveredPublicationAdmissionCause as Cause;
    if recovered.runtime_instance_id != runtime.runtime_instance_id()
        || !effect
            .subject()
            .belongs_to_runtime(runtime.runtime_instance_id())
    {
        return Err(Cause::ForeignRecoveryRuntime);
    }
    match current_settlement(effect.subject(), runtime) {
        Some(Settlement::PerformedPending) => return Err(Cause::PendingSettlement),
        Some(Settlement::PerformedSettled | Settlement::CheckpointReadmitted) => {}
        None => return Err(Cause::MissingSettlementEvidence),
    }
    Ok(())
}
