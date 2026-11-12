//! A real admitted emission uses Bank's actual payload encoder and the external rail's protocol.
use super::schema::*;
use bank_domain::estate::{EstateAction, EstateDeathNotificationRequest};
use bank_domain::schema::{EstateActionInputBinding, EstateDeathNotificationRequestBinding};
use std::num::NonZeroU64;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_decl::facade::{
    application_aftermath::*,
    application_operation::*,
    application_schema::{
        fragment::{FieldSelection, Registration},
        *,
    },
    worth_query_effect, worth_query_operation, worth_query_operation_emits,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryOperationDurableCapability,
};
use worth_query_testkit_host::facade::host::dispatch_home::notify::{
    BalanceDenialBinding, BalanceValueBinding,
};
type Number = FieldSelection<HomeRecords, Account, AccountIdentity, String>;
pub type Scope = ApplicationMutationFieldScope<Number>;
pub const FAMILY: &str = "estate-death-notice-rail";
worth_query_effect!(pub EstateDeathNotificationEffect for RailHomeSchema, payload EstateDeathNotificationRequestBinding);
worth_query_operation!(pub NotifyRail for RailHomeSchema, input EstateActionInputBinding);
worth_query_operation_emits!(NotifyRail => [EstateDeathNotificationEffect]);
impl Registration<RailHomeSchema> for EstateDeathNotificationEffect {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.effect(Self::reference()))
    }
}
impl Registration<RailHomeSchema> for NotifyRail {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let inbound = ApplicationInboundOccurrenceBinding::<EstateDeathNotificationEffect>::new(
            ApplicationInboundOccurrenceProtocol::new(
                BoundaryProtocolIdentity::new("bank.estate.death-notification"),
                BoundaryProtocolVersion::new(1),
            ),
            "rail-primary",
            ApplicationInboundOccurrenceLimits {
                maximum_envelope_bytes: NonZeroU64::new(4096).unwrap(),
                maximum_verifier_work: NonZeroU64::new(20480).unwrap(),
                maximum_payload_bytes: NonZeroU64::new(256).unwrap(),
                maximum_outstanding_dispatch_provenance: NonZeroU64::new(64).unwrap(),
                maximum_accepted_occurrences: NonZeroU64::new(64).unwrap(),
                maximum_accepted_bytes: NonZeroU64::new(64 * 4096).unwrap(),
                maximum_concurrent_publications: NonZeroU64::new(8).unwrap(),
                maximum_discovery_work: NonZeroU64::new(1024).unwrap(),
                replay_window_milliseconds: NonZeroU64::new(60000).unwrap(),
                maximum_cleanup_work: NonZeroU64::new(64).unwrap(),
            },
        )
        .unwrap();
        let op = Self::reference();
        Ok(builder
            .operation(
                op.definition()
                    .external_effect_with_inbound(
                        EstateDeathNotificationEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new(FAMILY).unwrap(),
                        inbound,
                    )
                    .aftermath(
                        DeclaredApplicationAftermathContract::runtime_with_external_owner(
                            DeclaredCorrectionMechanism::Compensation(DeclaredCompensation::new(
                                "rail-reconcile",
                                DeclaredAftermathPostcondition::BusinessPostcondition {
                                    identity: "rail-reconciled".into(),
                                },
                            )),
                            DeclaredReconciliationProcedure::new("rail-reconcile"),
                        ),
                    )
                    .finish(),
            )
            .operation_projection_work_budget(op, 1)
            .operation_emit(op, EstateDeathNotificationEffect::reference()))
    }
}
pub struct NotifyRailBinding;
impl ApplicationMutationAdmissionBinding<RailHomeSchema> for NotifyRailBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<RailHomeSchema> for NotifyRailBinding {
    type Input = EstateAction;
    type InputBinding = EstateActionInputBinding;
    type Result = u64;
    type ResultBinding = BalanceValueBinding;
    type IdempotencyKey = u64;
    type Operation = NotifyRail;
    type Decision = ();
    type Denial = ();
    type DenialBinding = BalanceDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = RailIdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "worth.bank.rail-home.notify.v1";
    const HANDLER_IDENTITY: &'static str = "worth.bank.rail-home.notify-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.bank.rail-home.notify-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> Number {
        Number::at("AccountNumber")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        RailHomeSchema,
        RailIdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        RailIdentityBinding::reference()
    }
}
impl Registration<RailHomeSchema> for NotifyRailBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
#[derive(Clone)]
pub struct Notify(pub EstateAction);
impl ApplicationMutationIntent<RailHomeSchema> for Notify {
    type Binding = NotifyRailBinding;
    fn input(&self) -> &EstateAction {
        &self.0
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(Number::at("AccountNumber"), "account-001".into())
    }
}
pub struct NotifyRailHandler(pub Arc<AtomicUsize>);
impl OperationHandler<RailHomeSchema, NotifyRailBinding> for NotifyRailHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[
            WorthQueryOperationDurableCapability::ExternalEmission,
            WorthQueryOperationDurableCapability::ExternalDispatch,
            WorthQueryOperationDurableCapability::InboundCompletion,
        ])
    }
    fn decide(
        &self,
        _: &EstateAction,
        _: &mut DecisionReader<'_, '_, '_, RailHomeSchema, NotifyRailBinding>,
    ) -> HandlerResult<(), ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        value: &EstateAction,
        _: (),
        writer: &mut CandidateWriter<'_, RailHomeSchema, NotifyRailBinding>,
    ) -> HandlerResult<u64, ()> {
        let EstateAction::NotifyDeath {
            estate,
            notice,
            subject,
        } = *value
        else {
            return HandlerResult::DomainDenied(());
        };
        let payload = EstateDeathNotificationRequest::new(estate, notice, subject);
        match writer.emit_external(EstateDeathNotificationEffect::reference(), payload) {
            Ok(()) => HandlerResult::Completed(notice.get()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
