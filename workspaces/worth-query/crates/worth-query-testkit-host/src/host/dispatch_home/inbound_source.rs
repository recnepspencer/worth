//! Declared source slots for Home security-fact readmission courts.
use super::notify::{
    BalanceNoticeBinding, BalanceNoticeEffect, BalanceValueBinding, CORRELATION_FAMILY,
};
use super::schema::DispatchHomeSchema;
use std::num::NonZeroU64;
use worth_foundational::facade::BoundaryProtocolIdentity;
use worth_query_decl::facade::{
    application_schema::{
        fragment::{Fragment, NoBindings, Registration},
        *,
    },
    worth_query_operation, worth_query_operation_emits,
};

pub const SOURCE: &str = "dispatch-home-inbound-source";
pub const OTHER_SOURCE: &str = "dispatch-home-other-inbound-source";
pub fn protocol_identity() -> BoundaryProtocolIdentity {
    BalanceNoticeBinding::PROTOCOL.identity().clone()
}

fn inbound(source: &str) -> ApplicationInboundOccurrenceBinding<BalanceNoticeEffect> {
    let n = |value| NonZeroU64::new(value).unwrap();
    ApplicationInboundOccurrenceBinding::new(
        ApplicationInboundOccurrenceProtocol::new(
            protocol_identity(),
            BalanceNoticeBinding::PROTOCOL.version(),
        ),
        source,
        ApplicationInboundOccurrenceLimits {
            maximum_envelope_bytes: n(1024),
            maximum_verifier_work: n(4096),
            maximum_payload_bytes: n(8),
            maximum_outstanding_dispatch_provenance: n(8),
            maximum_accepted_occurrences: n(8),
            maximum_accepted_bytes: n(8192),
            maximum_concurrent_publications: n(2),
            maximum_discovery_work: n(16),
            replay_window_milliseconds: n(60_000),
            maximum_cleanup_work: n(16),
        },
    )
    .unwrap()
}

worth_query_operation!(pub InboundNotice for DispatchHomeSchema, input BalanceValueBinding);
worth_query_operation_emits!(InboundNotice => [BalanceNoticeEffect]);
impl Registration<DispatchHomeSchema> for InboundNotice {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .external_effect_with_inbound(
                        BalanceNoticeEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new(CORRELATION_FAMILY).unwrap(),
                        inbound(SOURCE),
                    )
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 1)
            .operation_emit(operation, BalanceNoticeEffect::reference()))
    }
}
worth_query_operation!(pub InboundSibling for DispatchHomeSchema, input BalanceValueBinding);
worth_query_operation_emits!(InboundSibling => [BalanceNoticeEffect]);
impl Registration<DispatchHomeSchema> for InboundSibling {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .external_effect_with_inbound(
                        BalanceNoticeEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new(CORRELATION_FAMILY).unwrap(),
                        inbound(SOURCE),
                    )
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 1)
            .operation_emit(operation, BalanceNoticeEffect::reference()))
    }
}
worth_query_operation!(pub OtherSourceNotice for DispatchHomeSchema, input BalanceValueBinding);
worth_query_operation_emits!(OtherSourceNotice => [BalanceNoticeEffect]);
impl Registration<DispatchHomeSchema> for OtherSourceNotice {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .external_effect_with_inbound(
                        BalanceNoticeEffect::reference(),
                        WorthQueryExternalEffectCorrelationFamily::new(CORRELATION_FAMILY).unwrap(),
                        inbound(OTHER_SOURCE),
                    )
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 1)
            .operation_emit(operation, BalanceNoticeEffect::reference()))
    }
}

/// Source slots share their parent contribution's ownership closure.
pub struct InboundSourceSlots;
impl Fragment for InboundSourceSlots {
    const NAME: &'static str = "InboundSourceSlots";
    type Bindings = NoBindings;
    type Members = (
        InboundNotice,
        InboundSibling,
        OtherSourceNotice,
        super::inbound_action::InboundNoticeBinding,
    );
}
