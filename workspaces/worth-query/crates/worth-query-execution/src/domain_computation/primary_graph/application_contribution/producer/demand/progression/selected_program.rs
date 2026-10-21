use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationProgramIdentity;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::ApplicationSchema;

use super::denial;
use super::FamilySourceQuery;
use super::FamilySourceValue;
use super::WorthQueryAdmittedOutputDemand;
use super::WorthQueryOutputDemandAdvance;
use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;
use super::WorthQueryProducerCommitAuthority;
use super::WorthQueryProducerOutputFamily;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn advance_selected_program_output_demand<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

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
        let mut disclosure = Some(disclosure);
        self.advance_selected_program_output_demand_with_source(
            execution,
            phase,
            access,
            demand,
            principal,
            request_scope,
            delivery_branch,
            |_| Ok(disclosure.take()),
            identity,
            revision,
        )
    }

    /// Reuse the source meaning retained when the public demand was admitted.
    pub fn advance_selected_program_output_demand_from_retained<Family>(
        &self,

        phase: &WorthQueryAdvancementPhase<'_>,

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
        let phase_request = match phase.execution_request_for(&self.product_runtime) {
            Ok(request) => request,
            Err(cause) => return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "output-demand-phase")),
        };
        let execution = &phase_request;

        self.advance_selected_program_output_demand_with_source(
            execution,
            phase,
            access,
            demand,
            principal,
            request_scope,
            delivery_branch,
            |retained| {
                super::retained_read::read_retained_source::<Schema, Family>(
                    execution,
                    self,
                    retained,
                    principal,
                    request_scope,
                    delivery_branch,
                )
                .map(Some)
            },
            identity,
            revision,
        )
    }

    fn advance_selected_program_output_demand_with_source<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,

        disclosure: impl FnMut(
            &crate::domain_computation::primary_graph::WorthQueryObservedSource<
                FamilySourceQuery<Schema, Family>,
            >,
        ) -> Result<
            Option<
                WorthQueryApplicationOutputDemandSource<
                    FamilySourceQuery<Schema, Family>,
                    FamilySourceValue<Schema, Family>,
                >,
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
        phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| {
                denial(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "the phase belongs to another installed runtime",
                )
            })?;
        let selected = self
            .on_branch(delivery_branch)
            .select_with_request(execution)
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
        self.advance_as_caller(demand, |demand, performed| {
            self.advance_output_demand_with_prepared_source(
                execution,
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
                WorthQueryProducerCommitAuthority::SelectedProgram { identity, revision },
                performed,
            )
        })
    }
}
