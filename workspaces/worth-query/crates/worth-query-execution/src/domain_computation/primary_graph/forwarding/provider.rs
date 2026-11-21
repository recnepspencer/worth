//! Thin forwarding shims for `provider` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_provider_committed_dispatch_outbox {
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::correlation;
    use worth_query_execution_lineage::facade::primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxObservation;
    use worth_query_execution_query::facade::domain_computation::primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial as Denial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema, {
        pub fn observe_committed_dispatch_outbox_by_correlation(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
            self.inner.observe_committed_dispatch_outbox_by_correlation(phase, correlation)
        }
        pub fn observe_committed_dispatch_outbox(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, receipt: &WorthQueryApplicationCommitReceipt) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
            self.inner.observe_committed_dispatch_outbox(phase, receipt)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_provider_managed_application_recovery {
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::managed_application_recovery::*;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::WorthQueryProductUnpublishedRecovery;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn recover_admitted_unpublished_application<Operation, Input, Scope>(&self, recovery: &WorthQueryProductUnpublishedRecovery, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> Result<
            WorthQueryManagedApplicationRecoveryOutcome,
            WorthQueryManagedApplicationRecoveryDenial,
        >
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.recover_admitted_unpublished_application::<Operation, Input, Scope>(recovery, admission, idempotency)
        }
        pub fn recover_admitted_unpublished_application_in_advancement<Operation, Input, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, recovery: &WorthQueryProductUnpublishedRecovery, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> Result<
            WorthQueryManagedApplicationRecoveryOutcome,
            WorthQueryManagedApplicationRecoveryDenial,
        >
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.recover_admitted_unpublished_application_in_advancement::<Operation, Input, Scope>(phase, recovery, admission, idempotency)
        }
        pub fn recover_admitted_unpublished_program_source<Operation, Input, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess, recovery: &WorthQueryProductUnpublishedRecovery, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> Result<(WorthQueryManagedApplicationRecoveryOutcome, Option<crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource>), WorthQueryManagedApplicationRecoveryDenial>
        where Input: Clone + Send + Sync + 'static, {
            self.inner.recover_admitted_unpublished_program_source::<Operation, Input, Scope>(phase, __arg1, recovery, admission, idempotency)
        }
        pub fn recover_admitted_unpublished_source_operation<Operation, Input, Scope>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, __arg1: &crate::publication_boundary::WorthQueryProgramPublicationAccess, recovery: &WorthQueryProductUnpublishedRecovery, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>, idempotency: WorthQueryApplicationIdempotencyBinding) -> Result<
            WorthQueryManagedApplicationRecoveryOutcome,
            WorthQueryManagedApplicationRecoveryDenial,
        >
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.recover_admitted_unpublished_source_operation::<Operation, Input, Scope>(phase, __arg1, recovery, admission, idempotency)
        }
    }
}
