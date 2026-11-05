//! The ordinary planar algorithm has a distinct installed external-effect contract here.
use super::*;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    worth_query_effect, worth_query_operation, worth_query_operation_creates,
    worth_query_operation_emits, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_unlinks, worth_query_operation_writes,
    worth_query_structured_value_binding,
};

pub struct EscapingNoticePayload;
impl ApplicationStructuredValueBinding for EscapingNoticePayload {
    type Value = String;
    const IDENTITY_NAME: &'static str = "worth.query.certification.escaping-notice-payload.v1";
    fn validate(value: &String) -> Result<(), ApplicationValueValidationDenial> {
        if value.len() <= 128 {
            Ok(())
        } else {
            Err(ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "notice is too large",
            ))
        }
    }
}
impl ApplicationExternalEffectBinding for EscapingNoticePayload {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("worth.query.certification.escaping-planar"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 128;
    fn external_effect_bytes(value: &String) -> Vec<u8> {
        value.as_bytes().to_vec()
    }
}
worth_query_effect!(pub EscapingPlanarNotice for Schema: TopologySchemaBinding, payload EscapingNoticePayload);
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct EscapingPlanarInput(pub PlanarMutation);
worth_query_structured_value_binding!(pub EscapingPlanarInputBinding for EscapingPlanarInput {
    identity: "worth.query.certification.escaping-planar-input.v1"
});
worth_query_operation!(pub EscapePlanar for Schema: TopologySchemaBinding, input EscapingPlanarInputBinding);
worth_query_operation_emits!(EscapePlanar => [EscapingPlanarNotice]);
worth_query_operation_reads!(EscapePlanar => [Body, BodyKey, PositionX, PositionY, Length, PlanarSuccessor]);
worth_query_operation_writes!(EscapePlanar => [BodyKey, PositionX, PositionY, Length]);
worth_query_operation_creates!(EscapePlanar => [Body]);
worth_query_operation_links!(EscapePlanar => [PlanarSuccessor]);
worth_query_operation_unlinks!(EscapePlanar => [PlanarSuccessor]);

pub struct EscapingPlanarBinding<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> ApplicationMutationBinding<S> for EscapingPlanarBinding<S> {
    type Input = EscapingPlanarInput;
    type InputBinding = EscapingPlanarInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = EscapePlanar;
    type Decision = ();
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<S>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;
    const IDENTITY: &'static str = "worth.query.certification.escaping-planar-mutation.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.escaping-planar-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.escaping-planar-command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        S,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        S,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}
impl<S: TopologySchemaBinding> ApplicationMutationIntent<S> for EscapingPlanarInput {
    type Binding = EscapingPlanarBinding<S>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<S> {
        PlanarMutationScope::new(BodyKey::reference(), self.0.scope_key.clone())
    }
}
