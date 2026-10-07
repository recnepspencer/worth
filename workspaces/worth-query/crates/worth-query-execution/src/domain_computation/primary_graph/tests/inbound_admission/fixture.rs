use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime};

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
mod faults;
mod wide;
pub(super) fn installed_world() -> InboundWorld {
    let declaration = InboundTestSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "inbound_admission_test",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap();
    let (runtime, authority) = installation.into_parts();
    let schema = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    let binding = schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    let mut bootstrap = authority
        .prepare_primary_graph(&runtime, &schema, test_product_world_resources())
        .unwrap();
    bootstrap
        .bind_principal(
            &binding,
            WorthQueryApplicationPrincipalKey::new("principal").unwrap(),
            1_u64,
            external_identity(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    bootstrap
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Target::reference(),
                WorthQueryApplicationEntityKey::new("target").unwrap(),
            )
            .field(TargetKey::reference(), "target".to_owned()),
        )
        .unwrap();
    let invariant = bootstrap.retain_invariant_projection_authority();
    let application = bootstrap
        .publish_application_runtime(
            runtime,
            authority,
            schema,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap();
    let operation = application
        .installed_schema()
        .installed_operation(NotifyOperation::reference())
        .unwrap();
    let verifier = application
        .install_inbound_occurrence_verifier(&operation, Arc::new(TestVerifier))
        .unwrap();
    InboundWorld {
        application,
        invariant,
        verifier,
    }
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
            .project_admitted_operation(&admission, |_, _| {})
            .unwrap()
            .into_parts();
        let reads = self
            .application
            .begin_projected_application_read_attempt(admission, projection)
            .unwrap();
        let mut effects = reads
            .complete_projected_dependencies()
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
            .project_admitted_operation(&admission, |_, _| {})
            .unwrap()
            .into_parts();
        let reads = self
            .application
            .begin_projected_application_read_attempt(admission, projection)
            .unwrap();
        let mut effects = reads
            .complete_projected_dependencies()
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
        )
    }
}

fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://inbound-test.example", "alice").unwrap()
}

fn authenticate(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<
        InboundTestSchema,
    >,
    request: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<InboundTestSchema> {
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("inbound-test").unwrap(),
            WorthQueryAuthenticationMethod::new("test-identity").unwrap(),
        ),
        TestIdentityAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate((), request)).unwrap()
}

struct TestIdentityAdapter;
impl WorthQueryAuthenticationAdapter for TestIdentityAdapter {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        "inbound-test-identity-adapter"
    }
    fn validate<'a>(
        &'a self,
        _: (),
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                external_identity(),
                WorthQueryAuthenticationAudience::new("inbound-test").unwrap(),
                WorthQueryAuthenticationMethod::new("test-identity").unwrap(),
                now,
                now + Duration::from_secs(60),
                vec![],
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
