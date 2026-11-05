//! The runtime's consumer methods over `history` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationHistoryCapture;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryCaptureLimits;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryCursor;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryPage;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryPageLimits;
use crate::domain_computation::primary_graph::application_query::WorthQueryCapturedApplicationHistorySubjects;
use crate::domain_computation::primary_graph::vocabulary::application_query::history_denial::WorthQueryApplicationHistoryDenial;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_query::ApplicationHistoryQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn admit_application_history_capture<'a, History>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        mut plan: crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan< 'a, Schema, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Query, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ResultBinding as ApplicationStructuredValueBinding>::Value, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Principal, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::PrincipalIdentity, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope, >,
        limits: WorthQueryApplicationHistoryCaptureLimits,
    ) -> Result<
        WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>,
        WorthQueryApplicationHistoryDenial,
    >
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::CapturePlanOps<
            Schema,
        >>::admit_application_history_capture(self, execution, history, plan, limits)
    }

    pub fn admit_governed_application_history_capture<'a, History, Capability, Operation, Input>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        mut plan: crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan< 'a, Schema, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Query, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ResultBinding as ApplicationStructuredValueBinding>::Value, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Principal, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::PrincipalIdentity, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope, >,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        limits: WorthQueryApplicationHistoryCaptureLimits,
    ) -> Result<WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::CapturePlanOps<
            Schema,
        >>::admit_governed_application_history_capture(
            self, execution, history, plan, capability, limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_application_history<'a, History>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<
        WorthQueryApplicationHistoryCursor<Schema, History>,
        WorthQueryApplicationHistoryDenial,
    >
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::OpeningOps<Schema>>::open_application_history(self, phase, history, external, subjects, security, truth, request, limits)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_governed_application_history<
        'a,
        History,
        ReadCapability,
        ReadOperation,
        ReadInput,
        MetadataCapability,
        MetadataOperation,
        MetadataInput,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: WorthQueryAdmittedApplicationCapabilityAccess<Schema, ReadCapability, ReadOperation, ReadInput>,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, MetadataCapability, MetadataOperation, MetadataInput>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryCursor<Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        ReadInput: ApplicationCapabilityRequest<
            Schema,
            ReadCapability,
            Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
        MetadataInput: ApplicationCapabilityRequest<
            Schema,
            MetadataCapability,
            Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
    >,{
        <Self as crate::domain_computation::primary_graph::application_query::OpeningOps<Schema>>::open_governed_application_history(self, phase, history, external, subjects, security, truth, read, metadata, request, limits)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_application_history_with_governed_subject<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryCursor<Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::OpeningOps<Schema>>::open_application_history_with_governed_subject(self, phase, history, external, subjects, security, truth, read, request, limits)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_application_history_with_governed_metadata<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryCursor<Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::OpeningOps<Schema>>::open_application_history_with_governed_metadata(self, phase, history, external, subjects, security, truth, metadata, request, limits)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_application_history<'a, History>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::FreshPagingOps<
            Schema,
        >>::read_application_history(
            self, phase, history, external, subjects, cursor, security, truth, request, limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_governed_application_history<
        'a,
        History,
        ReadCapability,
        ReadOperation,
        ReadInput,
        MetadataCapability,
        MetadataOperation,
        MetadataInput,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: WorthQueryAdmittedApplicationCapabilityAccess<Schema, ReadCapability, ReadOperation, ReadInput>,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, MetadataCapability, MetadataOperation, MetadataInput>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        ReadInput: ApplicationCapabilityRequest<
            Schema,
            ReadCapability,
            Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
        MetadataInput: ApplicationCapabilityRequest<
            Schema,
            MetadataCapability,
            Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
    >,{
        <Self as crate::domain_computation::primary_graph::application_query::FreshPagingOps<
            Schema,
        >>::read_governed_application_history(
            self, phase, history, external, subjects, cursor, security, truth, read, metadata,
            request, limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_application_history_with_governed_subject<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::FreshPagingOps<
            Schema,
        >>::read_application_history_with_governed_subject(
            self, phase, history, external, subjects, cursor, security, truth, read, request,
            limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_application_history_with_governed_metadata<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::FreshPagingOps<
            Schema,
        >>::read_application_history_with_governed_metadata(
            self, phase, history, external, subjects, cursor, security, truth, metadata, request,
            limits,
        )
    }
}
