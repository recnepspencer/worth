//! The runtime's consumer methods over `domain_computation::primary_graph::application_contribution`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOutputDemandSource;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation;
use crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerOutputFamily;
use worth_query_installation::facade::ApplicationSchema;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use crate::domain_computation::primary_graph::application_contribution::producer::ProducerInvocationAuthority;
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandAdvance;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::progression::denial;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::application_output_demand::host::OutputDemandHost;
use std::sync::Arc;
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_relational::facade::storage::RecordLifecycleState;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits;
use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_declaration::facade::application_program::ApplicationProgramIdentity;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::*;
use crate::domain_computation::primary_graph::output_reuse::compare_retained_output_dependencies;
use crate::domain_computation::primary_graph::output_reuse::compare_retained_output_witness;
use crate::domain_computation::primary_graph::output_reuse::retained_output_settlement_is_verified;
use crate::domain_computation::primary_graph::output_reuse::OutputDependencySelection;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn advance_output_demand<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
        >,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value: 'static,
    <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query: 'static,{
        <Self as crate::domain_computation::primary_graph::application_contribution::ProgressionOps<Schema>>::advance_output_demand(self, phase, demand, principal, request_scope, delivery_branch, disclosure)
    }

    pub fn advance_output_demand_from_retained<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                    Schema,
                    <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
                > + 'static,
    <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query: 'static,{
        <Self as crate::domain_computation::primary_graph::application_contribution::ProgressionOps<Schema>>::advance_output_demand_from_retained(self, phase, demand, principal, request_scope, delivery_branch)
    }

    pub fn output_demand_resource_profile(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile {
        <Self as crate::domain_computation::primary_graph::application_contribution::AdmissionOps<
            Schema,
        >>::output_demand_resource_profile(self)
    }

    pub fn admit_output_demand<Family>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_contribution::AdmissionOps<
            Schema,
        >>::admit_output_demand(self, phase, source_result, limits)
    }

    pub fn admit_performed_output_demand<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_contribution::AdmissionOps<
            Schema,
        >>::admit_performed_output_demand(self, execution, source_result, limits, prepared)
    }

    pub fn admit_output_demand_at<Family>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        source_result: WorthQueryApplicationOutputDemandSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
        >,
        limits: WorthQueryOutputDemandLimits,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_contribution::RetainedObservationOps<Schema>>::admit_output_demand_at(self, phase, source_result, limits, observation)
    }

    pub fn discard_prepared_required_output_source(
        &self,
        prepared: crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    ) {
        <Self as crate::domain_computation::primary_graph::application_contribution::SourceCustodyOps<Schema>>::discard_prepared_required_output_source(self, prepared)
    }

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
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
        >,
        identity: ApplicationProgramIdentity,
        revision: ApplicationProgramRevision,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value: 'static,
    <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query: 'static,{
        <Self as crate::domain_computation::primary_graph::application_contribution::SelectedProgramOps<Schema>>::advance_selected_program_output_demand(self, execution, phase, access, demand, principal, request_scope, delivery_branch, disclosure, identity, revision)
    }

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
        <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                    Schema,
                    <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
                > + 'static,
    <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query: 'static,{
        <Self as crate::domain_computation::primary_graph::application_contribution::SelectedProgramOps<Schema>>::advance_selected_program_output_demand_from_retained(self, phase, access, demand, principal, request_scope, delivery_branch, identity, revision)
    }

    pub fn select_output_producer<Family>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_contribution::SelectionOps<
            Schema,
        >>::select_output_producer::<Family>(self, phase, source, profile_kind)
    }
}
