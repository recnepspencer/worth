//! One prepared authored Bridge projection, consumed by the same Signal original.
use super::*;
use worth_signal::facade::durability::{
    SignalCompositeAdvanceBinding, SignalDurableBranchMutationPort,
};

pub struct BridgePreparedAuthoredDefinitionPublication {
    pub(super) prepared: PreparedDefinition,
    pub(super) expected: SignalDurableBranchHandle,
    /// The predecessor's recorded node, where the planned leg publishes.
    pub(super) node: worth_foundational::facade::DurableSignalNodeIdentity,
}
impl BridgePreparedAuthoredDefinitionPublication {
    /// Capture the exact Signal leg this original performs, for World to
    /// record before Native's effect of a durable mixed original. Only that
    /// record is bounded by World's custody row; recording it grants nothing.
    pub fn planned_leg(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::conditional_execution::BridgePlannedDefinitionLeg,
        BridgeDurableDefinitionPublicationDenial,
    > {
        self.prepared
            .pending
            .planned_leg(execution, self.node, &self.prepared.definition)
    }
    pub fn admits_signal_basis(
        &self,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> bool {
        self.expected.basis().admission_identity() == basis.admission_identity()
    }
    pub fn completion(&self) -> BridgeDurableDefinitionCompletion {
        self.prepared.pending.completion()
    }
    pub fn begin_composite<D, I, E, Ctx, T, F>(
        self,
        mutation: &SignalDurableBranchMutationPort<D, I, E, Ctx, T>,
        execution: ExecutionRequest<'_, '_>,
        binding: SignalCompositeAdvanceBinding,
        grant: worth_proof::AdmittedCompositeSignalPublication,
        runtime_ctx: &mut Ctx,
        cancellation: &worth_signal::facade::branch::SignalOwnerCancellationToken,
        apply: F,
    ) -> Result<SignalDurableAdvanceTask, BridgeDurableDefinitionPublicationDenial>
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
        F: FnOnce(
            &mut worth_signal::facade::SignalTransaction<'_, D, I, E, Ctx, T>,
        ) -> Result<(), worth_signal::facade::SignalError>,
    {
        let Self { prepared, .. } = self;
        let PreparedDefinition {
            pending,
            definition,
            refusal,
        } = prepared;
        let result = mutation.begin_composite_definition_publication(
            execution,
            binding,
            grant,
            runtime_ctx,
            cancellation,
            pending.claimant(),
            pending.predecessor_contract().clone(),
            definition,
            pending,
            apply,
        );
        super::definition_result(result, refusal)
    }
}
impl std::fmt::Debug for BridgePreparedAuthoredDefinitionPublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BridgePreparedAuthoredDefinitionPublication")
            .finish_non_exhaustive()
    }
}
