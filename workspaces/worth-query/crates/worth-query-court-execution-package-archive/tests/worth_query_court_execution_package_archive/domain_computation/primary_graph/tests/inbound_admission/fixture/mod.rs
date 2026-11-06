pub use std::future::Future;
pub use std::pin::pin;
pub use std::sync::Arc;
pub use std::task::Context;
pub use std::task::Poll;
pub use std::task::Waker;
pub use std::time::Duration;
pub use std::time::SystemTime;
pub use worth_execution::ExecutionAllocationPolicy;
pub use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

pub use worth_query_admission::facade::authenticated_principal::*;
pub use worth_query_declaration::facade::application_schema::OperationEmits;
pub use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
pub use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
pub use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
pub use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
pub use worth_query_installation::facade::WorthQueryInstallationGeneration;
pub use worth_query_installation::facade::WorthQueryInstalledApplicationOperation;
pub use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
pub use worth_query_installation::facade::WorthQueryPortableDomainPackage;

pub use crate::domain_computation::primary_graph::tests::inbound_admission::schema::IdentityBinding;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::schema::InboundTestSchema;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::schema::Notice;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::schema::NoticeEffect;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::schema::NotifyOperation;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::schema::OtherNotifyOperation;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::schema::Target;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::schema::TargetKey;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::TestVerifier;
pub use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationInvariantProjectionAuthority;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationPrincipalKey;
pub use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
pub use worth_query_execution::facade::domain_computation::WorthQueryInboundVerifierHandle;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub struct InboundWorld {
    pub application: WorthQueryPrimaryGraphApplicationRuntime<InboundTestSchema>,
    pub invariant: WorthQueryApplicationInvariantProjectionAuthority<InboundTestSchema>,
    pub verifier: WorthQueryInboundVerifierHandle,
}

pub(super) use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installation::installed_world_with_verifier;
pub fn installed_world() -> InboundWorld {
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

    pub fn commit_nonselected_dispatch<Operation, Input>(
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
        let default_request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let request = &default_request;
        let program = self
            .application
            .with_application_advancement(request, |phase| {
                let phase_custody = &phase;
                let execution = &phase
                    .execution_request_for_for_test(self.application.product_runtime())
                    .expect("fixture phase belongs to its application");
                let branch = self.application.current_world();
                let identity = self
                    .application
                    .product_runtime()
                    .admit_product_occurrence_for_test(branch.occurrence_for_test())
                    .unwrap()
                    .branch_identity()
                    .clone();
                let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
                let external = authenticate(self.application.installed_schema(), &request);
                let selected = self
                    .application
                    .select_product_branch_for_test(execution, &identity)
                    .unwrap();
                let binding = self
                    .application
                    .installed_schema()
                    .principal_binding(IdentityBinding::reference())
                    .unwrap();
                let principal = selected
                    .resolve_authenticated_principal(
                        phase_custody,
                        &binding,
                        &external,
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let target = self
                    .application
                    .select_product_branch_for_test(execution, &identity)
                    .unwrap()
                    .resolve_entity(
                        phase_custody,
                        TargetKey::at("TargetKey"),
                        "target".to_owned(),
                        &request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let admission = self
                    .application
                    .select_product_branch_for_test(execution, &identity)
                    .unwrap()
                    .authorize_operation(
                        phase_custody,
                        &principal,
                        &target,
                        &operation,
                        TypedMutationPreconditions::new(),
                        &request,
                    )
                    .unwrap();
                let (_, projection, _) = self
                    .invariant
                    .project_admitted_operation(
                        phase_custody,
                        &admission,
                        |_, _| {},
                        AllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                let reads = self
                    .application
                    .begin_projected_application_read_attempt(
                        phase_custody,
                        admission,
                        projection,
                        AllocationPolicy::SystemAllocation,
                    )
                    .unwrap();
                let mut effects = reads
                    .complete_projected_dependencies(
                        phase_custody,
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .begin_effect_program();
                effects
                    .emit_external(NoticeEffect::reference(), Notice(text.to_owned()))
                    .unwrap();

                effects.finish().unwrap()
            })
            .expect("the fixture preparation owns its host call");
        let mut key = [0_u8; 32];
        key[..8].copy_from_slice(&seed.to_be_bytes());
        key[8] = 1;
        let mut fingerprint = key;
        fingerprint[8] = 2;
        match self.application.compare_and_commit_application(
            program,
            WorthQueryApplicationIdempotencyBinding::new_for_test(key, fingerprint),
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
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
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
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
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
    pub fn attempt_operation_on(
        &self,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        seed: u8,
        text: &str,
        emit_external: bool,
        request: Option<&WorthQueryRequestScope>,
    ) -> WorthQueryApplicationCommitOutcome {
        let default_request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let request = request.unwrap_or(&default_request);
        let program = self
            .application
            .with_application_advancement(request, |phase| {
                let phase_custody = &phase;
                let execution = &phase
                    .execution_request_for_for_test(self.application.product_runtime())
                    .expect("fixture phase belongs to its application");
                let identity = self
                    .application
                    .product_runtime()
                    .admit_product_occurrence_for_test(branch.occurrence_for_test())
                    .unwrap()
                    .branch_identity()
                    .clone();
                let external = authenticate(self.application.installed_schema(), request);
                let selected = self
                    .application
                    .select_product_branch_for_test(execution, &identity)
                    .unwrap();
                let binding = self
                    .application
                    .installed_schema()
                    .principal_binding(IdentityBinding::reference())
                    .unwrap();
                let principal = selected
                    .resolve_authenticated_principal(
                        phase_custody,
                        &binding,
                        &external,
                        request,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let target = self
                    .application
                    .select_product_branch_for_test(execution, &identity)
                    .unwrap()
                    .resolve_entity(
                        phase_custody,
                        TargetKey::at("TargetKey"),
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
                    .select_product_branch_for_test(execution, &identity)
                    .unwrap()
                    .authorize_operation(
                        phase_custody,
                        &principal,
                        &target,
                        &operation,
                        TypedMutationPreconditions::new(),
                        request,
                    )
                    .unwrap();
                let (_, projection, _) = self
                    .invariant
                    .project_admitted_operation(
                        phase_custody,
                        &admission,
                        |_, _| {},
                        AllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                let reads = self
                    .application
                    .begin_projected_application_read_attempt(
                        phase_custody,
                        admission,
                        projection,
                        AllocationPolicy::SystemAllocation,
                    )
                    .unwrap();
                let mut effects = reads
                    .complete_projected_dependencies(
                        phase_custody,
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .begin_effect_program();
                if emit_external {
                    effects
                        .emit_external(NoticeEffect::reference(), Notice(text.to_owned()))
                        .unwrap();
                }

                effects.finish().unwrap()
            })
            .expect("the fixture preparation owns its host call");
        self.application.compare_and_commit_application(
            program,
            WorthQueryApplicationIdempotencyBinding::new_for_test(
                [seed; 32],
                [seed.wrapping_add(1); 32],
            ),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    }
}

pub use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::authentication::authenticate;
pub use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::authentication::external_identity;

#[path = "authentication/mod.rs"]
pub mod authentication;
#[path = "installation/mod.rs"]
pub mod installation;
#[path = "scoped_dispatch/mod.rs"]
pub mod scoped_dispatch;
#[path = "wide/mod.rs"]
pub mod wide;
