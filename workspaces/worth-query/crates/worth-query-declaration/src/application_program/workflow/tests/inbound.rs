use std::{num::NonZeroU64, time::Duration};

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

use super::*;
use crate::application_schema::{
    ApplicationEffectMarkerIdentity, ApplicationInboundOccurrenceBinding,
    ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol,
    ApplicationRetainedEffectBinding,
};

struct RemoteEffect;
struct RemotePayload;

impl ApplicationStructuredValueBinding for RemotePayload {
    type Value = ();
    const IDENTITY_NAME: &'static str = "worth.query.tests.workflow.remote-payload.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
impl ApplicationRetainedEffectBinding for RemotePayload {
    fn retained_bytes(_: &Self::Value) -> u64 {
        0
    }
}
impl ApplicationEffectMarkerIdentity<TestSchema> for RemoteEffect {
    type PayloadBinding = RemotePayload;
    const IDENTIFIER: &'static str = "worth.query.tests.workflow.remote-effect.v1";
}

fn inbound(source: &str) -> ApplicationInboundOccurrenceBinding<RemoteEffect> {
    let one = NonZeroU64::new(1).unwrap();
    ApplicationInboundOccurrenceBinding::new(
        ApplicationInboundOccurrenceProtocol::new(
            BoundaryProtocolIdentity::new("worth.query.workflow.remote"),
            BoundaryProtocolVersion::new(1),
        ),
        source,
        ApplicationInboundOccurrenceLimits {
            maximum_envelope_bytes: one,
            maximum_payload_bytes: one,
            maximum_outstanding_dispatch_provenance: one,
            maximum_accepted_occurrences: one,
            maximum_accepted_bytes: one,
            maximum_concurrent_publications: one,
            maximum_discovery_work: one,
            replay_window_milliseconds: one,
            maximum_cleanup_work: one,
        },
    )
    .unwrap()
}

fn deadline_limits() -> ApplicationWorkflowDefinitionLimits {
    limits()
        .with_total_deadline(Duration::from_secs(30))
        .unwrap()
}

#[test]
fn typed_wait_requires_a_real_prior_origin_and_total_deadline() {
    let build = |limits, future_origin| {
        let mut builder =
            ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new("remote-wait", limits)
                .unwrap();
        let operation = builder
            .operation::<ProposeChange>("operation", false)
            .unwrap();
        let wait = builder
            .await_inbound::<RemoteEffect>(
                "wait",
                &operation,
                inbound("rail"),
                ApplicationWorkflowInboundWait::UntilInstanceDeadline,
            )
            .unwrap();
        let terminal = builder.terminal("terminal").unwrap();
        if future_origin {
            builder
                .start(&wait)
                .control(
                    &wait,
                    ApplicationWorkflowControlOutcome::Completed,
                    &operation,
                )
                .control(
                    &operation,
                    ApplicationWorkflowControlOutcome::Completed,
                    &terminal,
                );
        } else {
            builder
                .start(&operation)
                .control(
                    &operation,
                    ApplicationWorkflowControlOutcome::Completed,
                    &wait,
                )
                .control(
                    &wait,
                    ApplicationWorkflowControlOutcome::Completed,
                    &terminal,
                );
        }
        builder.finish().unwrap().validate()
    };
    assert!(build(deadline_limits(), false).is_ok());
    assert_eq!(
        build(limits(), false).err().unwrap().kind(),
        ApplicationWorkflowValidationDenialKind::MissingInboundDeadline
    );
    assert_eq!(
        build(deadline_limits(), true).err().unwrap().kind(),
        ApplicationWorkflowValidationDenialKind::UnavailableInboundOrigin
    );
}

#[test]
fn source_contract_is_part_of_wait_content_identity() {
    let build = |source| {
        let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
            "remote-wait",
            deadline_limits(),
        )
        .unwrap();
        let operation = builder
            .operation::<ProposeChange>("operation", false)
            .unwrap();
        let wait = builder
            .await_inbound::<RemoteEffect>(
                "wait",
                &operation,
                inbound(source),
                ApplicationWorkflowInboundWait::UntilInstanceDeadline,
            )
            .unwrap();
        let terminal = builder.terminal("terminal").unwrap();
        builder
            .start(&operation)
            .control(
                &operation,
                ApplicationWorkflowControlOutcome::Completed,
                &wait,
            )
            .control(
                &wait,
                ApplicationWorkflowControlOutcome::Completed,
                &terminal,
            );
        builder
            .finish()
            .unwrap()
            .validate()
            .unwrap()
            .content_identity()
            .clone()
    };
    assert_ne!(build("rail-a"), build("rail-b"));
}

#[test]
fn foreign_operation_reference_cannot_become_an_inbound_origin() {
    let mut foreign =
        ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new("foreign", deadline_limits())
            .unwrap();
    let foreign_operation = foreign
        .operation::<ProposeChange>("foreign-operation", false)
        .unwrap();
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "remote-wait",
        deadline_limits(),
    )
    .unwrap();
    let operation = builder
        .operation::<ProposeChange>("operation", false)
        .unwrap();
    let wait = builder
        .await_inbound::<RemoteEffect>(
            "wait",
            &foreign_operation,
            inbound("rail"),
            ApplicationWorkflowInboundWait::UntilInstanceDeadline,
        )
        .unwrap();
    let terminal = builder.terminal("terminal").unwrap();
    builder
        .start(&operation)
        .control(
            &operation,
            ApplicationWorkflowControlOutcome::Completed,
            &wait,
        )
        .control(
            &wait,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        );
    assert_eq!(
        builder.finish().unwrap().validate().err().unwrap().kind(),
        ApplicationWorkflowValidationDenialKind::MissingInboundOrigin,
    );
}
