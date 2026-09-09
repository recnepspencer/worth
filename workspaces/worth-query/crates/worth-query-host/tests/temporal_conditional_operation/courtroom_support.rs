use worth_query_host::facade::primary_graph;

use super::world::{request_scope, CourtroomWorld};

pub(super) fn observe(
    world: &mut CourtroomWorld,
) -> primary_graph::WorthQueryConditionalClockObservationReceipt<super::adapters::CourtroomClock> {
    let outcome = raw_observe(world);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) = outcome
    else {
        panic!(
            "due temporal observation was not accepted: {}",
            outcome_kind(&outcome)
        )
    };
    receipt
}

pub(super) fn outcome_kind<Clock>(
    outcome: &primary_graph::WorthQueryConditionalClockObservationOutcome<Clock>,
) -> String {
    match outcome {
        primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(_) => {
            "accepted".into()
        }
        primary_graph::WorthQueryConditionalClockObservationOutcome::Duplicate(_) => {
            "duplicate".into()
        }
        primary_graph::WorthQueryConditionalClockObservationOutcome::Stale => "stale".into(),
        primary_graph::WorthQueryConditionalClockObservationOutcome::Reordered => {
            "reordered".into()
        }
        primary_graph::WorthQueryConditionalClockObservationOutcome::Closed => "closed".into(),
        primary_graph::WorthQueryConditionalClockObservationOutcome::Failed(failure) => {
            format!("failed {:?}: {}", failure.kind(), failure.detail())
        }
    }
}

pub(super) fn raw_observe(
    world: &mut CourtroomWorld,
) -> primary_graph::WorthQueryConditionalClockObservationOutcome<super::adapters::CourtroomClock> {
    world.conditional_clock().observe()
}

pub(super) fn assert_authoritative_value<Entity, Aspect, Field, Value, Write, Unit>(
    world: &CourtroomWorld,
    field: worth_query_host::facade::declaration::application_schema::ApplicationFieldRef<
        super::schema::TemporalHostSchema,
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        worth_query_host::facade::declaration::application_schema::EqualityPredicate,
        Unit,
    >,
    value: Value,
) where
    Value: worth_query_host::facade::declaration::application_schema::TypedApplicationValue,
    Write: worth_query_host::facade::declaration::application_schema::WritePosture,
    Unit: worth_query_host::facade::declaration::application_schema::ApplicationFieldUnit,
{
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            field,
            value,
            &request_scope(),
            primary_graph::WorthQueryPrincipalResolutionMode::Certification,
        )
        .expect("expected value must remain authoritative");
}

pub(super) fn wake_evidence<Clock>(
    receipt: &primary_graph::WorthQueryConditionalClockObservationReceipt<Clock>,
) -> String {
    format!(
        "wake evidence: due={}, eligible={}, suppressed={}, deferred={}, failed={}, operation_failed={}, indeterminate={}",
        receipt.retained_due_wake_count(),
        receipt.retained_eligible_wake_count(),
        receipt.retained_suppressed_wake_count(),
        receipt.retained_deferred_wake_count(),
        receipt.retained_failed_wake_count(),
        receipt.failed_operation_count(),
        receipt.indeterminate_operation_count(),
    )
}
