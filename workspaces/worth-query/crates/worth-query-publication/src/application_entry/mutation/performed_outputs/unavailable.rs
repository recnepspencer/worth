//! Explicit negative completion of one original fixed leaf root.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputLeaf, ApplicationRequiredOutputRoot,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial,
};

/// The source is saved and its root has terminally refused its inputs before any
/// output publication. This is descriptive completion, not a Ready settlement.
pub struct WorthQueryApplicationProgramOutputUnavailable {
    receipt: WorthQueryApplicationCommitReceipt,
    denial: WorthQueryOutputDemandDenial,
}
impl WorthQueryApplicationProgramOutputUnavailable {
    pub fn source_receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
    pub fn denial(&self) -> &WorthQueryOutputDemandDenial {
        &self.denial
    }
}
impl<Schema, Program, Root> WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>
where Schema: ApplicationSchema + 'static, Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema, Dependents = ApplicationOutputLeaf>
        + ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    RootDemand<Schema, Root>: Clone,
    Value<Schema, RootDemand<Schema, Root>>: WorthQueryApplicationProjection<Schema, Query<Schema, RootDemand<Schema, Root>>> + Clone,
    <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = Source<Schema, RootDemand<Schema, Root>>>,
    <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>> {
    /// Finish only a native, cleanup-free domain refusal of the original leaf.
    /// On every refusal this handle retains all original continuation custody.
    /// Interrupted, unpublished, successful and dependent-tree phases cannot
    /// become unavailable through this entrance.
    pub fn finish_unavailable(
        &mut self, application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryApplicationProgramOutputUnavailable, crate::application_entry::WorthQueryRequiredOutputPreparationDenial> {
        application.runtime().with_application_advancement(request.scope, |phase| {
        use crate::application_entry::WorthQueryRequiredOutputPreparationDenial as Denial;
        if self.complete || self.root_settlement.is_some() || self.pending_root_authority.is_some() || self.continuation.is_some() { return Err(Denial::Closed); }
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(Denial::Demand(crate::application_entry::WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch));
        }
        let (Some(receipt), Some(prepared), Some(root)) = (&self.source_receipt, &self.source_preparation, &mut self.root) else { return Err(Denial::MissingPerformedDelivery); };
        let access = worth_query_execution::publication_boundary::program_publication_access();
        application.validate_required_program_source::<Root>(&access, prepared, receipt,
            &self.source_observation.retained, request.principal, request.scope, request.branch)
            .map_err(|d| Denial::Demand(crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(d)))?;
        // Prepare the descriptive receipt before the native terminal transition.
        let receipt = receipt.clone();
        let denial = root.finish_unavailable::<Root>(&phase, application, request, prepared).map_err(Denial::Demand)?;
        self.complete = true;
        self.root = None;
        self.source_preparation = None;
        Ok(WorthQueryApplicationProgramOutputUnavailable { receipt, denial })

        }).map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::advancement)?
    }
}
