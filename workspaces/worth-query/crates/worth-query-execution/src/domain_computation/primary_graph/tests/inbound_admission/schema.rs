#[path = "schema/records.rs"]
mod storage_model;
use std::num::NonZeroU64;
pub use storage_model::*;

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationInboundOccurrenceBinding, ApplicationInboundOccurrenceLimits,
    ApplicationInboundOccurrenceProtocol,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentityBinding, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_declaration::{
    worth_query_effect, worth_query_operation, worth_query_operation_emits,
};
use worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily;

pub(super) const SOURCE: &str = "inbound-test-source";
pub(super) const PROTOCOL: &str = "inbound.test.protocol";

pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        InboundTestSchema,
        Self,
        Mapping,
        Principal,
        <PrincipalIdentity as RecordSelection>::Value,
        <PrincipalIdentity as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "IdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    MappingIdentity::at("MappingIdentity"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatus::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::at("PrincipalIdentity"),
                ),
            },
        )
    }
}

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

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct InboundTestSchema;
impl Fragment for InboundTestSchema {
    const NAME: &'static str = "InboundTestSchema";
    type Bindings = NoBindings;
    type Members = (InboundModel, Contracts);
}
impl ApplicationSchema for InboundTestSchema {
    const OWNER: &'static str = "inbound_admission_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl InboundTestSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<InboundTestSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<InboundTestSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<InboundTestSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = schema
                .principal_binding(IdentityBinding::reference())
                .effect(NoticeEffect::reference())
                .operation(
                    NotifyOperation::reference()
                        .definition()
                        .external_effect_with_inbound(
                            NoticeEffect::reference(),
                            WorthQueryExternalEffectCorrelationFamily::new("inbound-test-family")
                                .unwrap(),
                            ApplicationInboundOccurrenceBinding::new(
                                ApplicationInboundOccurrenceProtocol::new(
                                    BoundaryProtocolIdentity::new(PROTOCOL),
                                    BoundaryProtocolVersion::new(1),
                                ),
                                SOURCE,
                                limits(),
                            )
                            .unwrap(),
                        )
                        .no_aftermath()
                        .finish(),
                )
                .operation_projection_work_budget(NotifyOperation::reference(), 8)
                .operation_emit(NotifyOperation::reference(), NoticeEffect::reference())
                .operation(
                    OtherNotifyOperation::reference()
                        .definition()
                        .external_effect_with_inbound(
                            NoticeEffect::reference(),
                            WorthQueryExternalEffectCorrelationFamily::new("inbound-test-family")
                                .unwrap(),
                            ApplicationInboundOccurrenceBinding::new(
                                ApplicationInboundOccurrenceProtocol::new(
                                    BoundaryProtocolIdentity::new(PROTOCOL),
                                    BoundaryProtocolVersion::new(1),
                                ),
                                SOURCE,
                                limits(),
                            )
                            .unwrap(),
                        )
                        .no_aftermath()
                        .finish(),
                )
                .operation_projection_work_budget(OtherNotifyOperation::reference(), 8)
                .operation_emit(OtherNotifyOperation::reference(), NoticeEffect::reference());
            schema
                .operation(
                    WideNotifyOperation::reference()
                        .definition()
                        .external_effect_with_inbound(
                            NoticeEffect::reference(),
                            WorthQueryExternalEffectCorrelationFamily::new("inbound-test-family")
                                .unwrap(),
                            ApplicationInboundOccurrenceBinding::new(
                                ApplicationInboundOccurrenceProtocol::new(
                                    BoundaryProtocolIdentity::new(PROTOCOL),
                                    BoundaryProtocolVersion::new(1),
                                ),
                                SOURCE,
                                wide_limits(),
                            )
                            .unwrap(),
                        )
                        .no_aftermath()
                        .finish(),
                )
                .operation_projection_work_budget(WideNotifyOperation::reference(), 8)
                .operation_emit(WideNotifyOperation::reference(), NoticeEffect::reference())
        })
    }
}
