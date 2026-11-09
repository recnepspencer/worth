//! The committed open seals to one Signal stage without losing original custody.
use super::{
    WorldNativePreparationDenial, WorldNativePreparationRefusal, WorldOpenedPublicationAttempt,
};
use crate::publication::{
    CompositePublicationStage, PreparedCompositePublicationWithSignal, WithSignal,
};
use worth_relational::facade::mvcc::{
    PreparedRelationalCommitCandidate, ValidatedRelationalProposal,
};

#[derive(Debug)]
pub struct WorldPreparedPublicationWithSignal {
    pub(crate) prepared: PreparedCompositePublicationWithSignal,
}
impl WorldPreparedPublicationWithSignal {
    pub fn expected_head(&self) -> &crate::branch::ProductBranchObservation {
        self.prepared.expected_head()
    }
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.prepared.unpublished_recovery_handle()
    }
    pub fn cancel(self) -> crate::recovery::ProductUnpublishedOwnerEffects {
        self.prepared.into_attempt().retain_open()
    }
}
#[derive(Debug)]
pub struct WorldQuerySignalPreparation {
    open: WorldOpenedPublicationAttempt,
}
impl WorldQuerySignalPreparation {
    pub fn complete_query_native_preparation(
        self,
        candidate: PreparedRelationalCommitCandidate,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorldPreparedPublicationWithSignal, WorldNativePreparationRefusal> {
        self.open
            .complete_signal_preparation(Some(candidate), request)
    }
    pub fn retain(self) -> crate::recovery::ProductUnpublishedOwnerEffects {
        self.open.retain()
    }
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.open.unpublished_recovery_handle()
    }
}
impl WorldOpenedPublicationAttempt {
    pub fn bind_query_signal_proposal(
        self,
        proposal: ValidatedRelationalProposal,
    ) -> Result<
        (WorldQuerySignalPreparation, ValidatedRelationalProposal),
        WorldNativePreparationRefusal,
    > {
        if !self.original.resolved.intent().changes_signal()
            || !self.original.resolved.intent().changes_relational()
        {
            return Err(WorldNativePreparationRefusal {
                open: self,
                cause: WorldNativePreparationDenial::ComponentIntent,
            });
        }
        match self.bind_native_proposal(proposal) {
            Ok(proposal) => Ok((WorldQuerySignalPreparation { open: self }, proposal)),
            Err(cause) => Err(WorldNativePreparationRefusal {
                open: self,
                cause: WorldNativePreparationDenial::Native(cause),
            }),
        }
    }
    pub fn prepare_signal(
        self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        proposal: Option<ValidatedRelationalProposal>,
    ) -> Result<WorldPreparedPublicationWithSignal, WorldNativePreparationRefusal> {
        let candidate = match proposal {
            Some(proposal) => {
                let proposal = match self.bind_native_proposal(proposal) {
                    Ok(value) => value,
                    Err(cause) => {
                        return Err(WorldNativePreparationRefusal {
                            open: self,
                            cause: WorldNativePreparationDenial::Native(cause),
                        })
                    }
                };
                match runtime.prepare_validated_proposal(request, proposal) {
                    Ok(value) => Some(value),
                    Err(cause) => {
                        return Err(WorldNativePreparationRefusal {
                            open: self,
                            cause: WorldNativePreparationDenial::Native(cause),
                        })
                    }
                }
            }
            None => None,
        };
        self.complete_signal_preparation(candidate, request)
    }
    fn complete_signal_preparation(
        self,
        candidate: Option<PreparedRelationalCommitCandidate>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorldPreparedPublicationWithSignal, WorldNativePreparationRefusal> {
        if !self.original.resolved.intent().changes_signal() {
            return Err(WorldNativePreparationRefusal {
                open: self,
                cause: WorldNativePreparationDenial::ComponentIntent,
            });
        }
        Ok(WorldPreparedPublicationWithSignal {
            prepared: WithSignal::seal(self.complete_component_preparation(candidate, request)?),
        })
    }
}
