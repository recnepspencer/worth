use crate::facade::runtime::ExecutionAllocationPolicy;
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime};
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

use worth_query_admission::facade::authenticated_principal::*;
use worth_query_declaration::facade::application_schema::{
    OperationEmits, TypedMutationPreconditions,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstalledApplicationOperation, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

use super::schema::{
    IdentityBinding, InboundTestSchema, Notice, NoticeEffect, NotifyOperation,
    OtherNotifyOperation, Target, TargetKey,
};
use super::verifier::TestVerifier;
use crate::domain_computation::execution_runtime::{
    product_world::test_product_world_resources, WorthQueryExecutionRuntimeInstaller,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationPrincipalKey, WorthQueryInboundVerifierHandle,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::facade::primary_graph::WorthQueryPrincipalResolutionMode;

pub(super) struct InboundWorld {
    pub application: WorthQueryPrimaryGraphApplicationRuntime<InboundTestSchema>,
    invariant: WorthQueryApplicationInvariantProjectionAuthority<InboundTestSchema>,
    pub verifier: WorthQueryInboundVerifierHandle,
}
#[cfg(feature = "test-world-operation-control")]
mod cost;
mod installation;
mod wide;
pub(super) use installation::installed_world_with_verifier;
pub(super) fn installed_world() -> InboundWorld {
    installed_world_with_verifier(Arc::new(TestVerifier))
}

impl InboundWorld {
    pub fn commit_other_dispatch(
        &self,
        seed: u64,
        text: &str,
    ) -> WorthQueryApplicationCommitReceipt {
        let operation = self
            .application
            .installed_schema()
            .installed_operation(OtherNotifyOperation::reference())
            .unwrap();
        self.commit_nonselected_dispatch(operation, seed, text)
    }

    fn commit_nonselected_dispatch<Operation, Input>(
        &self,
        operation: WorthQueryInstalledApplicationOperation<InboundTestSchema, Operation, Input>,
        seed: u64,
        text: &str,
    ) -> WorthQueryApplicationCommitReceipt
    where
        Operation: 'static,
        NoticeEffect: OperationEmits<Operation>,
        Input: Clone + Send + Sync + 'static,
    {
        let branch = self.application.current_world();
        let identity = self
            .application
            .product_runtime()
            .admit_product_occurrence(branch.occurrence())
            .unwrap()
            .branch_identity()
            .clone();
        let request = super::super::fixture::live_scope();
        let external = authenticate(self.application.installed_schema(), &request);
        let selected = self.application.select_product_branch(&identity).unwrap();
        let binding = self
            .application
            .installed_schema()
            .principal_binding(IdentityBinding::reference())
            .unwrap();
        let principal = selected
            .resolve_authenticated_principal(
                &binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let target = self
            .application
            .select_product_branch(&identity)
            .unwrap()
            .resolve_entity(
                TargetKey::reference(),
                "target".to_owned(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let admission = self
            .application
            .select_product_branch(&identity)
            .unwrap()
            .authorize_operation(
                &principal,
                &target,
                &operation,
                TypedMutationPreconditions::new(),
                &request,
            )
            .unwrap();
        let (_, projection, _) = self
            .invariant
            .project_admitted_operation(&admission, |_, _| {}, AllocationPolicy::SystemAllocation)
            .unwrap()
            .into_parts();
        let reads = self
            .application
            .begin_projected_application_read_attempt(
                admission,
                projection,
                AllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let mut effects = reads
            .complete_projected_dependencies(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
            .begin_effect_program();
        effects
            .emit_external(NoticeEffect::reference(), Notice(text.to_owned()))
            .unwrap();
        let program = effects.finish().unwrap();
        let mut key = [0_u8; 32];
        key[..8].copy_from_slice(&seed.to_be_bytes());
        key[8] = 1;
        let mut fingerprint = key;
        fingerprint[8] = 2;
        match self.application.compare_and_commit_application(
            program,
            WorthQueryApplicationIdempotencyBinding::new(key, fingerprint),
            ExecutionAllocationPolicy::SystemAllocation,
        ) {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
            other => panic!("other operation {seed} must issue a genuine dispatch: {other:?}"),
        }
    }

    pub fn commit_dispatch(&self, seed: u8, text: &str) -> WorthQueryApplicationCommitReceipt {
        self.commit_dispatch_on(self.application.current_world(), seed, text)
    }

    pub fn commit_dispatch_on(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
        seed: u8,
        text: &str,
    ) -> WorthQueryApplicationCommitReceipt {
        match self.attempt_dispatch_on(branch, seed, text) {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
            other => panic!("fixture must issue a genuine World dispatch: {other:?}"),
        }
    }

    pub fn attempt_dispatch_on(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
        seed: u8,
        text: &str,
    ) -> WorthQueryApplicationCommitOutcome {
        self.attempt_operation_on(branch, seed, text, true, None)
    }
    pub fn attempt_empty_program(&self, seed: u8) -> WorthQueryApplicationCommitOutcome {
        self.attempt_operation_on(
            self.application.current_world(),
            seed,
            "unused",
            false,
            None,
        )
    }
    fn attempt_operation_on(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
        seed: u8,
        text: &str,
        emit_external: bool,
        request: Option<&WorthQueryRequestScope>,
    ) -> WorthQueryApplicationCommitOutcome {
        let identity = self
            .application
            .product_runtime()
            .admit_product_occurrence(branch.occurrence())
            .unwrap()
            .branch_identity()
            .clone();
        let default_request = super::super::fixture::live_scope();
        let request = request.unwrap_or(&default_request);
        let external = authenticate(self.application.installed_schema(), request);
        let selected = self.application.select_product_branch(&identity).unwrap();
        let binding = self
            .application
            .installed_schema()
            .principal_binding(IdentityBinding::reference())
            .unwrap();
        let principal = selected
            .resolve_authenticated_principal(
                &binding,
                &external,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let target = self
            .application
            .select_product_branch(&identity)
            .unwrap()
            .resolve_entity(
                TargetKey::reference(),
                "target".to_owned(),
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let operation = self
            .application
            .installed_schema()
            .installed_operation(NotifyOperation::reference())
            .unwrap();
        let admission = self
            .application
            .select_product_branch(&identity)
            .unwrap()
            .authorize_operation(
                &principal,
                &target,
                &operation,
                TypedMutationPreconditions::new(),
                request,
            )
            .unwrap();
        let (_, projection, _) = self
            .invariant
            .project_admitted_operation(&admission, |_, _| {}, AllocationPolicy::SystemAllocation)
            .unwrap()
            .into_parts();
        let reads = self
            .application
            .begin_projected_application_read_attempt(
                admission,
                projection,
                AllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let mut effects = reads
            .complete_projected_dependencies(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
            .begin_effect_program();
        if emit_external {
            effects
                .emit_external(NoticeEffect::reference(), Notice(text.to_owned()))
                .unwrap();
        }
        let program = effects.finish().unwrap();
        self.application.compare_and_commit_application(
            program,
            WorthQueryApplicationIdempotencyBinding::new([seed; 32], [seed.wrapping_add(1); 32]),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    }
}

#[path = "fixture/authentication.rs"]
mod authentication;
use authentication::{authenticate, external_identity};
