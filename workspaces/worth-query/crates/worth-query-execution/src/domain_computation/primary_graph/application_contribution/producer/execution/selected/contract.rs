//! Installed selected producer execution obligations.
use super::*;

/// The selected Execute entry is a separate object-safe installed executor
/// obligation. Its owner implementation can use the retained mutation binding
/// and cold graph template without changing ordinary producer execution.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) trait InstalledSelectedProducerExecutor<
    Schema,
>:
    Send + Sync
{
    #[allow(clippy::too_many_arguments)]
    fn advance_ready_on_selected<'basis>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        shared: &'basis crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            '_,
            Schema,
        >,
        positioned: &'basis PositionedRelationalSnapshot,
        selected: &SelectedReadyReadmission,
        candidate: Option<&'basis AcceptedCurrentCandidate>,
        resolved: Option<&ResolvedRequiredPredecessors<'_, Schema>>,
        installed: &super::super::super::registry::InstalledProducerProvider<Schema>,
        delivery_branch: WorthQueryProductBranch,
        producer_contacts_in_this_demand: usize,
        refresh_permission: Result<(), WorthQueryOutputDemandDenial>,
        performed: &mut PerformedMembers,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<
        super::super::required_cue::RequiredCueProgress<'basis, Schema>,
        ProducerExecutionStop,
    >
    where
        Schema: ApplicationSchema;

    #[allow(clippy::too_many_arguments)]
    fn execute_on_selected(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        shared: &crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            '_,
            Schema,
        >,
        matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,
        required_output: RequiredOutputExecution,
        input: ValidatedProducerInput<'_>,
        successor_of: Option<[u8; 32]>,
        commit_authority: WorthQueryProducerCommitAuthority,
        edition: super::super::super::InstalledProducerEdition,
        limits: WorthQueryOutputDemandLimits,
        request_admission: &mut InvalidationEditAdmission,
        producer_contacts: &mut usize,
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop>;

    fn preserved_readiness_output_admitted(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial>;
}
