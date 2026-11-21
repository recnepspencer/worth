//! Thin forwarding shims for `application_runtime` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_rebuild {
    use std::num::NonZeroUsize;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::rebuild::WorthQueryInboundIndexRepairDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn repair_completed_inbound_index(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, handle: &WorthQueryInboundVerifierHandle, maximum_world_slots: NonZeroUsize, maximum_changed_records_per_commit: NonZeroUsize, maximum_ancestry_commits_per_completion: NonZeroUsize) -> Result<bool, WorthQueryInboundIndexRepairDenial> {
            self.inner.repair_completed_inbound_index(phase, handle, maximum_world_slots, maximum_changed_records_per_commit, maximum_ancestry_commits_per_completion)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_receive {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundAdmissionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::receipt::WorthQueryInboundReceipt;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema, {
        pub fn receive_inbound_occurrence(&self, handle: &WorthQueryInboundVerifierHandle, envelope: &[u8], request: &WorthQueryRequestScope) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
            self.inner.receive_inbound_occurrence(handle, envelope, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_source_control {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::source_control::WorthQueryInboundSourceControlDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn retire_inbound_occurrence_source(&self, handle: &WorthQueryInboundVerifierHandle) -> Result<(), WorthQueryInboundSourceControlDenial> {
            self.inner.retire_inbound_occurrence_source(handle)
        }
        pub fn revoke_inbound_occurrence_source(&self, handle: &WorthQueryInboundVerifierHandle) -> Result<(), WorthQueryInboundSourceControlDenial> {
            self.inner.revoke_inbound_occurrence_source(handle)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_home_custody_dispatch_readmission {
    use crate::domain_computation::primary_graph::WorthQueryInboundAdmissionDenial as Denial;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn readmit_dispatch_completion_custody(&self, execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<(), Denial> {
            self.inner.readmit_dispatch_completion_custody(execution)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_home_custody_terminal_sources {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn readmit_inbound_terminal_sources(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, unpublished_native: &std::collections::BTreeSet<
                worth_relational::facade::history::CommitId,
            >) -> Result<
            (),
            crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial,
        > {
            self.inner.readmit_inbound_terminal_sources(execution, unpublished_native)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_home_custody_unpublished {
    use crate::domain_computation::primary_graph::provider::WorthQueryCanonicalCompletionRow;
    use worth_runtime_world::facade::WorldRecoveredUnpublishedEffects;
    use crate::domain_computation::primary_graph::WorthQueryInboundAdmissionDenial as Denial;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn admit_recovered_inbound_completion(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, effects: &WorldRecoveredUnpublishedEffects, entity: worth_relational::facade::identity::EntityId, row: WorthQueryCanonicalCompletionRow) -> Result<(), Denial> {
            self.inner.admit_recovered_inbound_completion(execution, effects, entity, row)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_receive_phases {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundAdmissionDenial as Denial;
    use crate::domain_computation::primary_graph::WorthQueryInboundVerifierHandle;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::phases::WorthQueryAuthenticatedInboundOccurrence;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn authenticate_inbound_occurrence<'a>(&'a self, handle: &WorthQueryInboundVerifierHandle, envelope: &'a [u8]) -> Result<WorthQueryAuthenticatedInboundOccurrence<'a, Schema>, Denial> {
            self.inner.authenticate_inbound_occurrence(handle, envelope)
        }
    }
}
