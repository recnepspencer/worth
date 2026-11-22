//! Thin forwarding shims for `application_query` methods onto `worth-query-execution-query`.

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
        pub fn read_application_history<'a, History>(
            &'a self,
            phase: &WorthQueryAdvancementPhase<'_>,
            history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
            external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            subjects: &Subjects<Schema, History>,
            cursor: &mut Cursor<Schema, History>,
            security: WorthQuerySelectedProductOperation<'a, Schema>,
            truth: &WorthQuerySelectedProductOperation<'a, Schema>,
            request: &'a WorthQueryRequestScope,
            limits: Limits,
        ) -> Result<Page, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
        {
            self.inner.read_application_history::<History>(
                phase,
                history,
                external,
                subjects,
                cursor,
                security.into_inner(),
                truth.inner(),
                request,
                limits,
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
            subjects: &Subjects<Schema, History>,
            cursor: &mut Cursor<Schema, History>,
            security: WorthQuerySelectedProductOperation<'a, Schema>,
            truth: &WorthQuerySelectedProductOperation<'a, Schema>,
            read: CapabilityAccess<Schema, ReadCapability, ReadOperation, ReadInput>,
            metadata: CapabilityAccess<
                Schema,
                MetadataCapability,
                MetadataOperation,
                MetadataInput,
            >,
            request: &'a WorthQueryRequestScope,
            limits: Limits,
        ) -> Result<Page, Denial>
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
            >,
        {
            self.inner.read_governed_application_history::<History, ReadCapability, ReadOperation, ReadInput, MetadataCapability, MetadataOperation, MetadataInput>(phase, history, external, subjects, cursor, security.into_inner(), truth.inner(), read, metadata, request, limits)
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
            subjects: &Subjects<Schema, History>,
            cursor: &mut Cursor<Schema, History>,
            security: WorthQuerySelectedProductOperation<'a, Schema>,
            truth: &WorthQuerySelectedProductOperation<'a, Schema>,
            read: CapabilityAccess<Schema, Capability, Operation, Input>,
            request: &'a WorthQueryRequestScope,
            limits: Limits,
        ) -> Result<Page, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            Input: ApplicationCapabilityRequest<
                Schema,
                Capability,
                Scope = SubjectScope<Schema, History>,
            >,
        {
            self.inner.read_application_history_with_governed_subject::<History, Capability, Operation, Input>(phase, history, external, subjects, cursor, security.into_inner(), truth.inner(), read, request, limits)
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
            subjects: &Subjects<Schema, History>,
            cursor: &mut Cursor<Schema, History>,
            security: WorthQuerySelectedProductOperation<'a, Schema>,
            truth: &WorthQuerySelectedProductOperation<'a, Schema>,
            metadata: CapabilityAccess<Schema, Capability, Operation, Input>,
            request: &'a WorthQueryRequestScope,
            limits: Limits,
        ) -> Result<Page, Denial>
        where
            History: ApplicationHistoryQueryBinding<Schema>,
            Input: ApplicationCapabilityRequest<
                Schema,
                Capability,
                Scope = SubjectScope<Schema, History>,
            >,
        {
            self.inner.read_application_history_with_governed_metadata::<History, Capability, Operation, Input>(phase, history, external, subjects, cursor, security.into_inner(), truth.inner(), metadata, request, limits)
        }
    }
}
