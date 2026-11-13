//! Supplier-backed declaration: emission ceiling, actual typed payload and independent capabilities.
use super::schema::{ExternalMapping, IdentityBinding, Principal, Schema};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::{
    application_aftermath::*,
    application_operation::*,
    application_schema::{fragment::Registration, *},
    worth_query_effect, worth_query_operation, worth_query_operation_emits,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryOperationDurableCapability,
};
use worth_query_testkit_host::facade::host::dispatch_home::notify::{
    BalanceDenialBinding, BalanceNoticeBinding, BalanceValueBinding,
};
pub const FAMILY: &str = "compensation-original-notice";
pub const SOURCE: &str = "compensation-original-source";
worth_query_effect!(pub NoticeEffect for Schema, payload BalanceNoticeBinding);
impl Registration<Schema> for NoticeEffect {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.effect(Self::reference()))
    }
}
pub type Scope = ApplicationMutationFieldScope<Number>;
type Number = worth_query_decl::facade::application_schema::fragment::FieldSelection<
    super::schema::HomeRecords,
    super::schema::Account,
    super::schema::AccountIdentity,
    String,
>;

worth_query_operation!(pub NoticeOperation for Schema, input BalanceValueBinding);
worth_query_operation_emits!(NoticeOperation => [NoticeEffect]);
impl Registration<Schema> for NoticeOperation {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .external_effect_with_inbound(
                        NoticeEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new(FAMILY).unwrap(),
                        inbound(),
                    )
                    .aftermath(
                        DeclaredApplicationAftermathContract::runtime_with_external_owner(
                            DeclaredCorrectionMechanism::Compensation(DeclaredCompensation::new(
                                "recoverable-notice-reconciliation",
                                DeclaredAftermathPostcondition::BusinessPostcondition {
                                    identity: "notice-reconciled".into(),
                                },
                            )),
                            DeclaredReconciliationProcedure::new(
                                "recoverable-notice-reconciliation",
                            ),
                        ),
                    )
                    .finish(),
            )
            .operation_projection_work_budget(operation, 1)
            .operation_emit(operation, NoticeEffect::reference()))
    }
}
pub struct NoticeBinding;
impl ApplicationMutationAdmissionBinding<Schema> for NoticeBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Schema> for NoticeBinding {
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    type Input = u64;
    type InputBinding = BalanceValueBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = NoticeOperation;
    type Decision = ();
    type Denial = ();
    type DenialBinding = BalanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = IdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.query.compensation-original.notice.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.compensation-original.notice-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.compensation-original.notice-command.v1";
    fn scope_field() -> Number {
        Number::at("AccountNumber")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        IdentityBinding::reference()
    }
}
impl Registration<Schema> for NoticeBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.application_mutation_binding::<Self>())
    }
}

#[derive(Clone, Debug)]
pub struct Notice(pub u64);
impl ApplicationMutationIntent<Schema> for Notice {
    type Binding = NoticeBinding;
    fn input(&self) -> &u64 {
        &self.0
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(
            worth_query_decl::facade::application_schema::fragment::FieldSelection::<
                super::schema::HomeRecords,
                super::schema::Account,
                super::schema::AccountIdentity,
                String,
            >::at("AccountNumber"),
            "account-001".to_owned(),
        )
    }
}
pub struct NoticeHandler(pub Arc<AtomicUsize>);
impl OperationHandler<Schema, NoticeBinding> for NoticeHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        // The installed aftermath, rather than this supplier list, requires recovery custody.
        Some(&[
            WorthQueryOperationDurableCapability::ExternalEmission,
            WorthQueryOperationDurableCapability::ExternalDispatch,
            WorthQueryOperationDurableCapability::InboundCompletion,
        ])
    }
    fn decide(
        &self,
        _: &u64,
        _: &mut DecisionReader<'_, '_, '_, Schema, NoticeBinding>,
    ) -> HandlerResult<(), ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        value: &u64,
        _: (),
        writer: &mut CandidateWriter<'_, Schema, NoticeBinding>,
    ) -> HandlerResult<u64, ()> {
        match writer.emit_external(NoticeEffect::reference(), *value) {
            Ok(()) => HandlerResult::Completed(*value),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

fn inbound() -> ApplicationInboundOccurrenceBinding<NoticeEffect> {
    use std::num::NonZeroU64;
    let n = |value| NonZeroU64::new(value).unwrap();
    ApplicationInboundOccurrenceBinding::new(
        ApplicationInboundOccurrenceProtocol::new(
            BalanceNoticeBinding::PROTOCOL.identity().clone(),
            BalanceNoticeBinding::PROTOCOL.version(),
        ),
        SOURCE,
        ApplicationInboundOccurrenceLimits {
            maximum_envelope_bytes: n(1024),
            maximum_verifier_work: n(4096),
            maximum_payload_bytes: n(8),
            maximum_outstanding_dispatch_provenance: n(8),
            maximum_accepted_occurrences: n(8),
            maximum_accepted_bytes: n(8192),
            maximum_concurrent_publications: n(2),
            maximum_discovery_work: n(32),
            replay_window_milliseconds: n(60_000),
            maximum_cleanup_work: n(32),
        },
    )
    .unwrap()
}

pub enum Supplier {
    Complete(Arc<AtomicUsize>),
    MissingDispatch(Arc<AtomicUsize>),
}
impl Supplier {
    pub fn install(
        self,
        setup: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        match self {
            Self::Complete(calls) => setup.handler::<NoticeBinding, _>(NoticeHandler(calls)),
            Self::MissingDispatch(calls) => {
                setup.handler::<NoticeBinding, _>(MissingDispatch(calls))
            }
        }
    }
}
struct MissingDispatch(Arc<AtomicUsize>);
impl OperationHandler<Schema, NoticeBinding> for MissingDispatch {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[
            WorthQueryOperationDurableCapability::ExternalEmission,
            WorthQueryOperationDurableCapability::InboundCompletion,
        ])
    }
    fn decide(
        &self,
        input: &u64,
        reader: &mut DecisionReader<'_, '_, '_, Schema, NoticeBinding>,
    ) -> HandlerResult<(), ()> {
        NoticeHandler(self.0.clone()).decide(input, reader)
    }
    fn build_candidate(
        &self,
        input: &u64,
        decision: (),
        writer: &mut CandidateWriter<'_, Schema, NoticeBinding>,
    ) -> HandlerResult<u64, ()> {
        NoticeHandler(self.0.clone()).build_candidate(input, decision, writer)
    }
}
