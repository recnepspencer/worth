//! Runtime, root and original-observation checks for detached fixed outputs.
use super::{RootConnection, WorthQueryProgramApplicationRuntime};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryPreparedRequiredOutputSource,
};
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Checks retained required-source custody before a detached continuation
    /// reads or mutates any program state. This validates existing authority;
    /// it neither duplicates the prepared token nor recovers roots from a receipt.
    pub fn validate_required_program_source<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        observation: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "required root is not installed",
            ));
        }
        if prepared.runtime_authority != self.runtime.runtime.authority_identity().as_u64()
            || &prepared.source_commit != receipt.committed_product_publication().composite_commit()
            || &prepared.source_commit != observation.selected_commit()
            || prepared.product_occurrence != observation.branch_incarnation()
            || branch.occurrence() != observation.branch_incarnation()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "required continuation differs from its original runtime or publication",
            ));
        }
        // Retained observations remain readable after retirement; progression
        // also requires the original branch to remain open now.
        self.runtime.on_branch(branch).select().map_err(|denial| {
            WorthQueryOutputDemandDenial::product_selection(denial, "required continuation branch")
        })?;
        self.runtime
            .select_application_read_observation(observation)
            .map_err(|denial| {
                WorthQueryOutputDemandDenial::product_selection(
                    denial,
                    "required continuation observation",
                )
            })?;
        self.runtime.output_demands.validate_owned_root_kind(
            prepared,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Required(std::any::TypeId::of::<Root>()),
        )?;
        use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
        let kind = match request.interruption() {
            Some(WorthQueryRequestInterruption::Cancelled) => Some(WorthQueryOutputDemandDenialKind::Cancelled),
            Some(WorthQueryRequestInterruption::DeadlineExceeded) => Some(WorthQueryOutputDemandDenialKind::TimedOut),
            None if self.runtime.authentication_is_expired(principal.valid_until()) => Some(
                WorthQueryOutputDemandDenialKind::SourcePrincipal(
                    crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind::ExpiredAuthentication)),
            None => None,
        };
        match kind {
            Some(kind) => Err(WorthQueryOutputDemandDenial::new(
                kind,
                "required continuation request",
            )),
            None => Ok(()),
        }
    }

    /// Validates an initial read-origin continuation without inventing a source receipt.
    pub fn validate_initial_program_source<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        observation: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>,
    {
        if !self.contains_output_root::<Root>()
            || branch.occurrence() != observation.branch_incarnation()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "initial output continuation differs from its original root or occurrence",
            ));
        }
        // Retained observations remain readable after retirement; progression
        // also requires the original branch to remain open now.
        self.runtime.on_branch(branch).select().map_err(|denial| {
            WorthQueryOutputDemandDenial::product_selection(denial, "required continuation branch")
        })?;
        self.runtime
            .select_application_read_observation(observation)
            .map(|_| ())
            .map_err(|denial| {
                WorthQueryOutputDemandDenial::product_selection(
                    denial,
                    "initial output continuation observation",
                )
            })
    }
}
