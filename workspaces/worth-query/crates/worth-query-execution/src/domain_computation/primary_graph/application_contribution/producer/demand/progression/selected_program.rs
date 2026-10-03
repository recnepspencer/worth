use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramIdentity, ApplicationProgramRevision,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    denial, FamilySourceQuery, FamilySourceValue, WorthQueryAdmittedOutputDemand,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryProducerCommitAuthority, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputDemandSource, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn advance_selected_program_output_demand<Family>(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandSource<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        identity: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        self.advance_selected_program_output_demand_with_source(
            access,
            demand,
            principal,
            request_scope,
            delivery_branch,
            |_, _| Ok(disclosure),
            identity,
            revision,
        )
    }

    /// Reuse the source meaning retained when the public demand was admitted.
    pub fn advance_selected_program_output_demand_from_retained<Family>(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        identity: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                    Schema,
                    FamilySourceQuery<Schema, Family>,
                > + 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let limits = demand.limits;
        self.advance_selected_program_output_demand_with_source(
            access,
            demand,
            principal,
            request_scope,
            delivery_branch,
            |retained, admission| {
                super::retained_read::read_retained_source::<Schema, Family>(
                    self,
                    retained,
                    principal,
                    request_scope,
                    delivery_branch,
                    limits,
                    admission,
                )
            },
            identity,
            revision,
        )
    }

    fn advance_selected_program_output_demand_with_source<Family>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: impl FnOnce(
            &crate::domain_computation::primary_graph::WorthQueryObservedSource<
                FamilySourceQuery<Schema, Family>,
            >,
            &mut super::InvalidationEditAdmission,
        ) -> Result<
            WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >,
            WorthQueryOutputDemandDenial,
        >,
        identity: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let selected = self
            .on_branch(delivery_branch)
            .select()
            .map_err(|selection| {
                WorthQueryOutputDemandDenial::product_selection(
                    selection,
                    "selected-program output occurrence could not be selected",
                )
            })?;
        let active = selected.inspect_selected_program().map_err(|inspection| {
            denial(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                format!(
                    "selected-program output activation could not be inspected: {inspection:?}"
                ),
            )
        })?;
        if active.identity() != &identity || active.revision() != &revision {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                format!("selected-program output {identity:?} revision {revision} is not active"),
            ));
        }
        let mut admission = self
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .read_admission(demand.limits.source_currentness_work());
        self.advance_output_demand_with_prepared_source(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            WorthQueryProducerCommitAuthority::SelectedProgram { identity, revision },
            &mut admission,
        )
    }
}
