//! Whole-operation compiler input is consumed exhaustively by the contract owner.
use super::*;
impl WorthQuerySealedOperationContractCompilation {
    #[allow(clippy::type_complexity)]
    pub(in crate::application_operation) fn into_parts(
        self,
    ) -> (
        Option<ApplicationOperationBodyProgram>,
        WorthQueryInstalledApplicationOperationAuthorization,
        Vec<WorthQueryInstalledAbilityRequirement>,
        Option<usize>,
        usize,
        Vec<WorthQueryInstalledMutationPrecondition>,
        WorthQueryInstalledApplicationOperationExecutionPosture,
        InstalledExternalEffectContract,
        Option<WorthQueryInstalledAftermathContract>,
        WorthQueryOperationGraphReadContract,
        WorthQueryOperationTouchContract,
        WorthQueryOperationEmissionContract,
        usize,
        super::super::super::contracts::SigningLifecycleClosure,
        Vec<WorthQueryInstalledApplicationInvariantDescriptor>,
    ) {
        (
            self.body,
            self.authorization,
            self.ability_requirements,
            self.projection_work_budget,
            self.additional_authorization_fact_count,
            self.mutation_preconditions,
            self.execution_posture,
            self.external_effect,
            self.aftermath,
            self.graph_reads,
            self.touches,
            self.emissions,
            self.graph_mutation_count,
            self.signing_lifecycle,
            self.invariant_invocations,
        )
    }
}
