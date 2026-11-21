pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::provider_execution::application_attempt_affinity::*;
use worth_relational::facade::snapshots::SnapshotHandle;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;

pub trait ApplicationAttemptBasisCaptureExt: Sized {
    fn capture<Schema, Operation, Input, Scope>(
        application: &super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        snapshot: &SnapshotHandle,
        product: &crate::basis::WorthQueryProductBranchLease,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, super::super::WorthQueryApplicationCommitOutcome>
    where
        Schema: worth_query_installation::facade::ApplicationSchema;
    fn take_original_completion(
        &mut self,
    ) -> super::super::super::provider::OriginalCompletionReceiver;
}

impl ApplicationAttemptBasisCaptureExt for WorthQueryApplicationAttemptBasis {
    fn capture<Schema, Operation, Input, Scope>(
        application: &super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        snapshot: &SnapshotHandle,
        product: &crate::basis::WorthQueryProductBranchLease,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, super::super::WorthQueryApplicationCommitOutcome> {
        if admission.runtime_authority() != application.runtime.authority_identity()
            || snapshot.branch_id() != admission.graph_work_branch()
            || snapshot.branch_id()
                != product
                    .observation()
                    .basis()
                    .relational_basis()
                    .identity()
                    .branch_id()
            || snapshot.version_id()
                != product
                    .observation()
                    .basis()
                    .relational_basis()
                    .observation()
                    .version_id()
        {
            return Err(super::provider_denial::denied(
                super::super::WorthQueryApplicationCommitDenialStage::ManagedRunAdmission,
            ));
        }
        let (completion, receiver) =
            super::super::super::provider::OriginalCompletionPublisher::pending_slot(policy)
                .map_err(
                    super::super::provider_compare_denial::original_custody_allocation_denied,
                )?;
        Ok(Self {
            original_completion: Some(receiver),
            core: WorthQueryApplicationAttemptCore {
                completion,
                runtime: admission.runtime_authority(),
                operation_attempt: admission.admission_identity(),
                installed_binding: admission.binding_identity().clone(),
                installed_operation: admission.retain_installed_operation_fingerprint(),
                resource_binding: admission.retain_resource_binding_identity(),
                operation: admission.operation().into(),
                snapshot: snapshot.clone(),
                graph_work_session: admission.graph_work_session_identity(),
                graph_work_managed_run: admission.graph_work_managed_run_identity(),
                branch: admission.graph_work_branch().clone(),
                request: admission.publication_request().clone(),
                admitted_request: admission.retain_admitted_request(),
                operation_scope: admission.operation_scope_binding().clone(),
            },
            product: product.retained_clone(),
        })
    }

    fn take_original_completion(
        &mut self,
    ) -> super::super::super::provider::OriginalCompletionReceiver {
        self.original_completion
            .take()
            .expect("only the active original progression receives its result")
    }
}
