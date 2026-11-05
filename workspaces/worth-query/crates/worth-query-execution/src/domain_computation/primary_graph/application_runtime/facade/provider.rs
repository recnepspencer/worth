//! The runtime's consumer methods over `domain_computation::primary_graph::provider`; each only calls its subsystem.
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
use crate::domain_computation::primary_graph::WorthQueryManagedApplicationRecoveryDenial;
use crate::domain_computation::primary_graph::WorthQueryManagedApplicationRecoveryOutcome;
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn observe_committed_dispatch_outbox(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, crate::domain_computation::vocabulary::primary_graph::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial>{
        <Self as crate::domain_computation::primary_graph::provider::CommittedDispatchOutboxOps<
            Schema,
        >>::observe_committed_dispatch_outbox(self, phase, receipt)
    }

    pub fn recover_admitted_unpublished_application<Operation, Input, Scope>(
        &self,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::provider::ManagedApplicationRecoveryOps<Schema>>::recover_admitted_unpublished_application(self, recovery, admission, idempotency)
    }

    pub fn recover_admitted_unpublished_application_in_advancement<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::provider::ManagedApplicationRecoveryOps<Schema>>::recover_admitted_unpublished_application_in_advancement(self, phase, recovery, admission, idempotency)
    }
}

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn recover_admitted_unpublished_program_source<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        unused_1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<(WorthQueryManagedApplicationRecoveryOutcome, Option<crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource>), WorthQueryManagedApplicationRecoveryDenial>
    where Input: Clone + Send + Sync + 'static,{
        <Self as crate::domain_computation::primary_graph::provider::ManagedApplicationRecoveryOps<Schema>>::recover_admitted_unpublished_program_source(self, phase, unused_1, recovery, admission, idempotency)
    }

    pub fn recover_admitted_unpublished_source_operation<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        unused_1: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        <Self as crate::domain_computation::primary_graph::provider::ManagedApplicationRecoveryOps<Schema>>::recover_admitted_unpublished_source_operation(self, phase, unused_1, recovery, admission, idempotency)
    }
}
