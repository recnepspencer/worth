use std::num::NonZeroU64;

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationInboundOccurrenceBinding, ApplicationInboundOccurrenceLimits,
    ApplicationInboundOccurrenceProtocol, ApplicationRetainedEffectBinding,
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentityBinding, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_declaration::{
    worth_query_application_schema, worth_query_aspect, worth_query_effect, worth_query_entity,
    worth_query_field, worth_query_operation, worth_query_operation_emits,
    worth_query_principal_binding, worth_query_relation,
};
use worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily;

pub(super) const SOURCE: &str = "inbound-test-source";
pub(super) const PROTOCOL: &str = "inbound.test.protocol";

worth_query_entity!(pub Mapping for InboundTestSchema);
worth_query_entity!(pub Principal for InboundTestSchema);
worth_query_entity!(pub Target for InboundTestSchema);
worth_query_aspect!(pub MappingFacts for InboundTestSchema, Mapping; identity = AspectIdentity(0x93910001), revision = AspectContractRevision(1),);
worth_query_field!(pub MappingIdentity for InboundTestSchema, Mapping, MappingFacts:
    worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity
    => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatus for InboundTestSchema, Mapping, MappingFacts:
    worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus
    => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_aspect!(pub PrincipalFacts for InboundTestSchema, Principal; identity = AspectIdentity(0x93910002), revision = AspectContractRevision(1),);
worth_query_field!(pub PrincipalIdentity for InboundTestSchema, Principal, PrincipalFacts:
    u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_aspect!(pub TargetFacts for InboundTestSchema, Target; identity = AspectIdentity(0x93910003), revision = AspectContractRevision(1),);
worth_query_field!(pub TargetKey for InboundTestSchema, Target, TargetFacts:
    String => StringApplicationValueBinding, read_only, equality);
worth_query_relation!(pub MappingTarget in InboundTestSchema, Mapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub IdentityBinding in InboundTestSchema, mapping Mapping {
    identity: MappingIdentity,
    status: MappingStatus,
    target: MappingTarget => Principal,
    principal_identity: PrincipalIdentity
});

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Notice(pub String);
worth_query_declaration::worth_query_portable_type!(Notice => "worth.query.inbound-test.notice.v1");
pub struct NoticeBinding;
impl worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding
    for NoticeBinding
{
    type Value = Notice;
    const IDENTITY_NAME: &'static str = "worth.query.inbound-test.notice.v1";
    fn validate(
        value: &Self::Value,
    ) -> Result<
        (),
        worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial,
    > {
        (!value.0.is_empty()).then_some(()).ok_or_else(|| {
            worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "notice must not be empty",
            )
        })
    }
}
impl ApplicationRetainedEffectBinding for NoticeBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(value.0.len()).unwrap_or(u64::MAX)
    }
}
impl ApplicationExternalEffectBinding for NoticeBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new(PROTOCOL),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 64;
    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.0.as_bytes().to_vec()
    }
}
worth_query_effect!(pub NoticeEffect for InboundTestSchema, payload NoticeBinding);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotifyInput;
worth_query_declaration::worth_query_portable_type!(NotifyInput => "worth.query.inbound-test.input.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub NotifyInputBinding for NotifyInput { identity: "worth.query.inbound-test.input.v1" });
worth_query_operation!(pub NotifyOperation for InboundTestSchema, input NotifyInputBinding);
worth_query_operation_emits!(NotifyOperation => [NoticeEffect]);
worth_query_operation!(pub OtherNotifyOperation for InboundTestSchema, input NotifyInputBinding);
worth_query_operation_emits!(OtherNotifyOperation => [NoticeEffect]);
worth_query_operation!(pub WideNotifyOperation for InboundTestSchema, input NotifyInputBinding);
worth_query_operation_emits!(WideNotifyOperation => [NoticeEffect]);

pub(super) fn limits() -> ApplicationInboundOccurrenceLimits {
    let n = |value| NonZeroU64::new(value).unwrap();
    ApplicationInboundOccurrenceLimits {
        maximum_envelope_bytes: n(256),
        maximum_verifier_work: n(256),
        maximum_payload_bytes: n(64),
        maximum_outstanding_dispatch_provenance: n(3),
        maximum_accepted_occurrences: n(1),
        maximum_accepted_bytes: n(512),
        maximum_concurrent_publications: n(1),
        maximum_discovery_work: n(16),
        replay_window_milliseconds: n(60_000),
        maximum_cleanup_work: n(16),
    }
}

fn wide_limits() -> ApplicationInboundOccurrenceLimits {
    let mut limits = limits();
    // The cost court keeps a thousand unrelated, genuinely inbound-bound
    // dispatches live while selecting one different effect.
    limits.maximum_outstanding_dispatch_provenance = NonZeroU64::new(1_002).unwrap();
    limits.maximum_accepted_occurrences = NonZeroU64::new(3).unwrap();
    limits.maximum_accepted_bytes = NonZeroU64::new(1_536).unwrap();
    limits
}

worth_query_application_schema! {
    pub schema InboundTestSchema {
        owner: inbound_admission_test,
        version: (1, 0),
        members: |schema| {
            let schema = schema
                .entity(Mapping::reference())
                .entity(Principal::reference())
                .entity(Target::reference())
                .aspect(Mapping::reference(), MappingFacts::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .aspect(Target::reference(), TargetFacts::reference())
                .field(Mapping::reference(), MappingIdentity::reference())
                .field(Mapping::reference(), MappingStatus::reference())
                .field(Principal::reference(), PrincipalIdentity::reference())
                .field(Target::reference(), TargetKey::reference())
                .relation(MappingTarget::reference(), Mapping::reference(), Principal::reference())
                .principal_binding(IdentityBinding::reference())
                .effect(NoticeEffect::reference())
                .operation(
                    NotifyOperation::reference()
                        .definition()
                        .external_effect_with_inbound(
                            NoticeEffect::reference(),
                            WorthQueryExternalEffectCorrelationFamily::new("inbound-test-family").unwrap(),
                            ApplicationInboundOccurrenceBinding::new(
                                ApplicationInboundOccurrenceProtocol::new(
                                    BoundaryProtocolIdentity::new(PROTOCOL),
                                    BoundaryProtocolVersion::new(1),
                                ),
                                SOURCE,
                                limits(),
                            ).unwrap(),
                        )
                        .no_aftermath()
                        .finish(),
                )
                .operation_decision_fact_budget(NotifyOperation::reference(), 1)
                .operation_projection_work_budget(NotifyOperation::reference(), 8)
                .operation_emit(NotifyOperation::reference(), NoticeEffect::reference())
                .operation(
                    OtherNotifyOperation::reference()
                        .definition()
                        .external_effect_with_inbound(
                            NoticeEffect::reference(),
                            WorthQueryExternalEffectCorrelationFamily::new("inbound-test-family").unwrap(),
                            ApplicationInboundOccurrenceBinding::new(
                                ApplicationInboundOccurrenceProtocol::new(
                                    BoundaryProtocolIdentity::new(PROTOCOL),
                                    BoundaryProtocolVersion::new(1),
                                ),
                                SOURCE,
                                limits(),
                            ).unwrap(),
                        )
                        .no_aftermath()
                        .finish(),
                )
                .operation_decision_fact_budget(OtherNotifyOperation::reference(), 1)
                .operation_projection_work_budget(OtherNotifyOperation::reference(), 8)
                .operation_emit(OtherNotifyOperation::reference(), NoticeEffect::reference());
            schema.operation(
                    WideNotifyOperation::reference()
                        .definition()
                        .external_effect_with_inbound(
                            NoticeEffect::reference(),
                            WorthQueryExternalEffectCorrelationFamily::new("inbound-test-family").unwrap(),
                            ApplicationInboundOccurrenceBinding::new(
                                ApplicationInboundOccurrenceProtocol::new(
                                    BoundaryProtocolIdentity::new(PROTOCOL),
                                    BoundaryProtocolVersion::new(1),
                                ),
                                SOURCE,
                                wide_limits(),
                            ).unwrap(),
                        )
                        .no_aftermath()
                        .finish(),
                )
                .operation_decision_fact_budget(WideNotifyOperation::reference(), 1)
                .operation_projection_work_budget(WideNotifyOperation::reference(), 8)
                .operation_emit(WideNotifyOperation::reference(), NoticeEffect::reference())
        }
    }
}
