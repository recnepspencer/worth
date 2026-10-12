//! Dormant executor transport for the actual typed successor's Drop proof.
//! No method can stand in for production execution: each panics if called.
use super::super::demand::{
    MatchedRequiredPredecessors, PerformedMembers, ResolvedRequiredPredecessors,
};
use super::selected::InstalledSelectedProducerExecutor;
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema as Schema;
use crate::domain_computation::primary_graph::{
    application_output_demand::SelectedReadyReadmission, output_lineage::AcceptedCurrentCandidate,
};
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn executor(
) -> Arc<dyn InstalledProducerExecutor<Schema>> {
    Arc::new(LifetimeOnly)
}
struct LifetimeOnly;
impl InstalledSelectedProducerExecutor<Schema> for LifetimeOnly {
    fn advance_ready_on_selected<'basis, 'owner>(
        &self,
        _phase: &'owner WorthQueryAdvancementPhase<'owner>,

        _runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        _principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        _request_scope: &WorthQueryRequestScope,
        _shared: &'owner crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            'owner,
            Schema,
        >,
        _positioned: &'basis PositionedRelationalSnapshot,
        _selected: &SelectedReadyReadmission,
        _candidate: Option<&'basis AcceptedCurrentCandidate>,
        _resolved: Option<&ResolvedRequiredPredecessors<'_, Schema>>,
        _installed: &super::super::registry::InstalledProducerProvider<Schema>,
        _delivery_branch: WorthQueryProductBranch,
        _producer_contacts_in_this_demand: usize,
        _refresh_permission: Result<(), WorthQueryOutputDemandDenial>,
        _performed: &'owner std::cell::RefCell<PerformedMembers>,
        _request_admission: &mut InvalidationEditAdmission,
    ) -> Result<
        super::required_cue::RequiredCueProgress<'basis, 'owner, Schema>,
        ProducerExecutionStop,
    >
    where
        'owner: 'basis,
        Schema: ApplicationSchema,
    {
        panic!("lifetime fixture never executes a producer")
    }

    fn inspect_frozen_ready<'basis>(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _inspection: super::required_cue::FrozenInspection<'basis, '_, Schema>,
        _admission: &mut InvalidationEditAdmission,
    ) -> Result<super::required_cue::FrozenCurrentInspection<'basis>, ProducerExecutionStop>
    where
        Schema: ApplicationSchema,
    {
        panic!("lifetime fixture never executes a producer")
    }

    fn execute_on_selected<'owner>(
        &self,
        _phase: &'owner WorthQueryAdvancementPhase<'owner>,

        _runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        _principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        _request_scope: &WorthQueryRequestScope,
        _shared: &'owner crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            '_,
            Schema,
        >,
        _matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,
        _required_output: RequiredOutputExecution,
        _input: ValidatedProducerInput<'_>,
        _successor_of: Option<[u8; 32]>,
        _commit_authority: WorthQueryProducerCommitAuthority,
        _edition: super::super::InstalledProducerEdition,
        _limits: WorthQueryOutputDemandLimits,
        _request_admission: &mut InvalidationEditAdmission,
        _producer_contacts: &mut usize,
    ) -> Result<super::outcome::PreparedProducerExecutionPlan<'owner>, ProducerExecutionStop> {
        panic!("lifetime fixture never executes a producer")
    }

    fn preserved_readiness_output_admitted(
        &self,
        _receipt: &WorthQueryApplicationCommitReceipt,
        _admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        panic!("lifetime fixture never executes a producer")
    }
}
impl InstalledProducerExecutor<Schema> for LifetimeOnly {
    fn execute(
        &self,
        _phase: &WorthQueryAdvancementPhase<'_>,
        _runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        _principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        _request_scope: &WorthQueryRequestScope,
        _branch: WorthQueryProductBranch,
        _required_output: RequiredOutputExecution,
        _input: ValidatedProducerInput<'_>,
        _successor_of: Option<[u8; 32]>,
        _commit_authority: WorthQueryProducerCommitAuthority,
        _edition: crate::domain_computation::primary_graph::application_contribution::producer::InstalledProducerEdition,
        _limits: WorthQueryOutputDemandLimits,
        _request_admission: &mut InvalidationEditAdmission,
        _producer_contacts: &mut usize,
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop> {
        panic!("lifetime fixture never executes a producer")
    }

    fn resources(
        &self,
        _source: &dyn Any,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryProducerDemandResources> {
        panic!("lifetime fixture never executes a producer")
    }

    fn readiness_record(
        &self,
        _receipt: &WorthQueryApplicationCommitReceipt,
        _admission: Option<&mut InvalidationEditAdmission>,
    ) -> Result<
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        WorthQueryOutputDemandDenial,
    > {
        panic!("lifetime fixture never executes a producer")
    }

    fn preserved_readiness_output(&self, _receipt: &WorthQueryApplicationCommitReceipt) -> bool {
        panic!("lifetime fixture never executes a producer")
    }
}
