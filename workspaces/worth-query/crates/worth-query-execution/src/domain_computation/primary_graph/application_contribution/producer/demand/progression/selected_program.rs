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
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
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
        if active.revision() != &revision {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                format!("selected-program output revision {revision} is not active"),
            ));
        }
        self.advance_output_demand_with_commit_authority(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            WorthQueryProducerCommitAuthority::SelectedProgram { identity, revision },
        )
    }
}
