//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_batch_planning {
    use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::batch::planning::WorthQueryApplicationQueryBatchReadPlan;
    use worth_query_execution_lineage::facade::primary_graph::application_query::batch::WorthQueryApplicationQueryBatchResourceDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::batch::WorthQueryApplicationQueryBatchAdmission;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_declaration::facade::application_schema::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn plan_application_query_batch_reads<B: ApplicationQueryBinding<Schema>>(&self, binding: &WorthQueryInstalledApplicationQueryBinding<Schema, B>, count: usize) -> Result<
            WorthQueryApplicationQueryBatchReadPlan,
            WorthQueryApplicationQueryBatchResourceDenial,
        > {
            self.inner.plan_application_query_batch_reads::<B>(binding, count)
        }
        pub fn plan_installed_query_batch_reads<Q, P, R, S>(&self, query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
                Schema,
                Q,
                P,
                R,
                S,
            >, count: usize, maximum_results: std::num::NonZeroUsize) -> Result<
            WorthQueryApplicationQueryBatchReadPlan,
            WorthQueryApplicationQueryBatchResourceDenial,
        > {
            self.inner.plan_installed_query_batch_reads::<Q, P, R, S>(query, count, maximum_results)
        }
        pub fn application_query_batch_from_installed_reads(&self, reads: &[WorthQueryApplicationQueryBatchReadPlan]) -> Result<
            WorthQueryApplicationQueryBatchAdmission,
            WorthQueryApplicationQueryBatchResourceDenial,
        > {
            self.inner.application_query_batch_from_installed_reads(reads)
        }
        pub fn extend_application_query_batch_once(&self, batch: &WorthQueryApplicationQueryBatchAdmission, reads: &[WorthQueryApplicationQueryBatchReadPlan]) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
            self.inner.extend_application_query_batch_once(batch, reads)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_live_lease_open {
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_query::ApplicationQueryLiveCauseBinding;
    use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::live::controls::WorthQueryApplicationLiveControls;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::live::outcome::WorthQueryApplicationLiveOpenDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::live::lease::WorthQueryApplicationLiveLease;
    use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
    use crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity;
    use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<'runtime, Schema>
    WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema, {
        #[allow(clippy::too_many_arguments)]
        pub fn open_application_query_live<
            'principal,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Target,
            Binding,
        >(self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>, principal: &'principal WorthQueryAuthenticatedPrincipal<
                Schema,
                Principal,
                PrincipalIdentity,
            >, scope: WorthQueryApplicationEntityIdentity<Schema, Scope>, parameters: ApplicationQueryParameterSet<Query>, controls: WorthQueryApplicationLiveControls) -> Result<
            WorthQueryApplicationLiveLease<
                'runtime,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
                Target,
                Binding,
            >,
            WorthQueryApplicationLiveOpenDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
            Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>, {
            self.inner.open_application_query_live::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Target, Binding>(phase, query, principal, scope, parameters, controls)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn open_governed_application_query_live<
            'principal,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
            Target,
            Binding,
            Capability,
            Operation,
            Input,
        >(self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, query: WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>, principal: &'principal WorthQueryAuthenticatedPrincipal<
                Schema,
                Principal,
                PrincipalIdentity,
            >, scope: WorthQueryApplicationEntityIdentity<Schema, Scope>, capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >, parameters: ApplicationQueryParameterSet<Query>, controls: WorthQueryApplicationLiveControls) -> Result<
            WorthQueryApplicationLiveLease<
                'runtime,
                Schema,
                Query,
                Parameters,
                QueryResult,
                Principal,
                PrincipalIdentity,
                Scope,
                Target,
                Binding,
            >,
            WorthQueryApplicationLiveOpenDenial,
        >
        where
            QueryResult: WorthQueryApplicationProjection<Schema, Query>,
            Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
            Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>, {
            self.inner.open_governed_application_query_live::<Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope, Target, Binding, Capability, Operation, Input>(phase, query, principal, scope, capability, parameters, controls)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_history_capture_plan {
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_declaration::facade::application_query::ApplicationHistoryQueryBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::SubjectPlan;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::SubjectScope;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryCaptureLimits;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryDenial as Denial;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::capture_plan::WorthQueryAdmittedApplicationHistoryCapture;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn admit_application_history_capture<'a, History>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, plan: SubjectPlan<'a, Schema, History>, limits: WorthQueryApplicationHistoryCaptureLimits) -> Result<WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>, {
            self.inner.admit_application_history_capture::<History>(execution, history, plan, limits)
        }
        pub fn admit_governed_application_history_capture<'a, History, Capability, Operation, Input>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, plan: SubjectPlan<'a, Schema, History>, capability: WorthQueryAdmittedApplicationCapabilityAccess<
                Schema,
                Capability,
                Operation,
                Input,
            >, limits: WorthQueryApplicationHistoryCaptureLimits) -> Result<WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            Input:
                ApplicationCapabilityRequest<Schema, Capability, Scope = SubjectScope<Schema, History>>, {
            self.inner.admit_governed_application_history_capture::<History, Capability, Operation, Input>(execution, history, plan, capability, limits)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_history_fresh_opening {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::SubjectScope;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryCursor as Cursor;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryPageLimits as Limits;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess as CapabilityAccess;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_declaration::facade::application_query::ApplicationHistoryQueryBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryCapturedApplicationHistorySubjects as Subjects;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub fn open_application_history<'a, History>(&'a self, phase: &WorthQueryAdvancementPhase<'_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, subjects: &Subjects<Schema, History>, security: WorthQuerySelectedProductOperation<'a, Schema>, truth: &WorthQuerySelectedProductOperation<'a, Schema>, request: &'a WorthQueryRequestScope, limits: Limits) -> Result<Cursor<Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>, {
            self.inner.open_application_history::<History>(phase, history, external, subjects, security.into_inner(), truth.inner(), request, limits)
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
        >(&'a self, phase: &WorthQueryAdvancementPhase<'_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, subjects: &Subjects<Schema, History>, security: WorthQuerySelectedProductOperation<'a, Schema>, truth: &WorthQuerySelectedProductOperation<'a, Schema>, read: CapabilityAccess<Schema, ReadCapability, ReadOperation, ReadInput>, metadata: CapabilityAccess<Schema, MetadataCapability, MetadataOperation, MetadataInput>, request: &'a WorthQueryRequestScope, limits: Limits) -> Result<Cursor<Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            ReadInput: ApplicationCapabilityRequest<
                Schema,
                ReadCapability,
                Scope = SubjectScope<Schema, History>,
            >,
            MetadataInput: ApplicationCapabilityRequest<
                Schema,
                MetadataCapability,
                Scope = SubjectScope<Schema, History>,
            >, {
            self.inner.open_governed_application_history::<History, ReadCapability, ReadOperation, ReadInput, MetadataCapability, MetadataOperation, MetadataInput>(phase, history, external, subjects, security.into_inner(), truth.inner(), read, metadata, request, limits)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn open_application_history_with_governed_subject<
            'a,
            History,
            Capability,
            Operation,
            Input,
        >(&'a self, phase: &WorthQueryAdvancementPhase<'_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, subjects: &Subjects<Schema, History>, security: WorthQuerySelectedProductOperation<'a, Schema>, truth: &WorthQuerySelectedProductOperation<'a, Schema>, read: CapabilityAccess<Schema, Capability, Operation, Input>, request: &'a WorthQueryRequestScope, limits: Limits) -> Result<Cursor<Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            Input:
                ApplicationCapabilityRequest<Schema, Capability, Scope = SubjectScope<Schema, History>>, {
            self.inner.open_application_history_with_governed_subject::<History, Capability, Operation, Input>(phase, history, external, subjects, security.into_inner(), truth.inner(), read, request, limits)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn open_application_history_with_governed_metadata<
            'a,
            History,
            Capability,
            Operation,
            Input,
        >(&'a self, phase: &WorthQueryAdvancementPhase<'_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, subjects: &Subjects<Schema, History>, security: WorthQuerySelectedProductOperation<'a, Schema>, truth: &WorthQuerySelectedProductOperation<'a, Schema>, metadata: CapabilityAccess<Schema, Capability, Operation, Input>, request: &'a WorthQueryRequestScope, limits: Limits) -> Result<Cursor<Schema, History>, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            Input:
                ApplicationCapabilityRequest<Schema, Capability, Scope = SubjectScope<Schema, History>>, {
            self.inner.open_application_history_with_governed_metadata::<History, Capability, Operation, Input>(phase, history, external, subjects, security.into_inner(), truth.inner(), metadata, request, limits)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_query_history_fresh_paging {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::SubjectScope;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryCursor as Cursor;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryPage as Page;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryPageLimits as Limits;
    use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess as CapabilityAccess;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_declaration::facade::application_query::ApplicationHistoryQueryBinding;
    use worth_query_declaration::facade::application_schema::ApplicationSchema;
    use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryApplicationHistoryDenial as Denial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_query::history::WorthQueryCapturedApplicationHistorySubjects as Subjects;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub fn read_application_history<'a, History>(&'a self, phase: &WorthQueryAdvancementPhase<'_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, subjects: &Subjects<Schema, History>, cursor: &mut Cursor<Schema, History>, security: WorthQuerySelectedProductOperation<'a, Schema>, truth: &WorthQuerySelectedProductOperation<'a, Schema>, request: &'a WorthQueryRequestScope, limits: Limits) -> Result<Page, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>, {
            self.inner.read_application_history::<History>(phase, history, external, subjects, cursor, security.into_inner(), truth.inner(), request, limits)
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
        >(&'a self, phase: &WorthQueryAdvancementPhase<'_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, subjects: &Subjects<Schema, History>, cursor: &mut Cursor<Schema, History>, security: WorthQuerySelectedProductOperation<'a, Schema>, truth: &WorthQuerySelectedProductOperation<'a, Schema>, read: CapabilityAccess<Schema, ReadCapability, ReadOperation, ReadInput>, metadata: CapabilityAccess<Schema, MetadataCapability, MetadataOperation, MetadataInput>, request: &'a WorthQueryRequestScope, limits: Limits) -> Result<Page, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            ReadInput: ApplicationCapabilityRequest<
                Schema,
                ReadCapability,
                Scope = SubjectScope<Schema, History>,
            >,
            MetadataInput: ApplicationCapabilityRequest<
                Schema,
                MetadataCapability,
                Scope = SubjectScope<Schema, History>,
            >, {
            self.inner.read_governed_application_history::<History, ReadCapability, ReadOperation, ReadInput, MetadataCapability, MetadataOperation, MetadataInput>(phase, history, external, subjects, cursor, security.into_inner(), truth.inner(), read, metadata, request, limits)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn read_application_history_with_governed_subject<
            'a,
            History,
            Capability,
            Operation,
            Input,
        >(&'a self, phase: &WorthQueryAdvancementPhase<'_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, subjects: &Subjects<Schema, History>, cursor: &mut Cursor<Schema, History>, security: WorthQuerySelectedProductOperation<'a, Schema>, truth: &WorthQuerySelectedProductOperation<'a, Schema>, read: CapabilityAccess<Schema, Capability, Operation, Input>, request: &'a WorthQueryRequestScope, limits: Limits) -> Result<Page, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            Input:
                ApplicationCapabilityRequest<Schema, Capability, Scope = SubjectScope<Schema, History>>, {
            self.inner.read_application_history_with_governed_subject::<History, Capability, Operation, Input>(phase, history, external, subjects, cursor, security.into_inner(), truth.inner(), read, request, limits)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn read_application_history_with_governed_metadata<
            'a,
            History,
            Capability,
            Operation,
            Input,
        >(&'a self, phase: &WorthQueryAdvancementPhase<'_>, history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>, external: &WorthQueryAuthenticatedExternalPrincipal<Schema>, subjects: &Subjects<Schema, History>, cursor: &mut Cursor<Schema, History>, security: WorthQuerySelectedProductOperation<'a, Schema>, truth: &WorthQuerySelectedProductOperation<'a, Schema>, metadata: CapabilityAccess<Schema, Capability, Operation, Input>, request: &'a WorthQueryRequestScope, limits: Limits) -> Result<Page, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            Input:
                ApplicationCapabilityRequest<Schema, Capability, Scope = SubjectScope<Schema, History>>, {
            self.inner.read_application_history_with_governed_metadata::<History, Capability, Operation, Input>(phase, history, external, subjects, cursor, security.into_inner(), truth.inner(), metadata, request, limits)
        }
    }
}
