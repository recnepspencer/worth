use super::canonical_components::{
    append_field, append_relation, append_value_binding, text, unsigned, unsigned_64,
    ApplicationCapabilityCanonicalComponent,
};
use super::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityElevationRule,
    ApplicationCapabilityOperationBinding, ApplicationCapabilityTransitionBinding,
    ErasedApplicationCapabilityContract,
};

pub(super) fn append_elevation(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    contract: &ErasedApplicationCapabilityContract,
) {
    let ApplicationCapabilityElevationRule::Governed(elevation) = contract.elevation() else {
        text(components, "elevation.posture", "not-applicable");
        return;
    };
    text(components, "elevation.posture", "governed");
    append_field(components, "elevation.identity", elevation.identity());
    append_field(components, "elevation.reason", elevation.reason());
    append_field(components, "elevation.status", elevation.status());
    append_field(components, "elevation.closed-at", elevation.closed_at());
    for (name, state) in [
        ("requested", elevation.states().requested()),
        ("approved", elevation.states().approved()),
        ("expired", elevation.states().expired()),
        ("revoked", elevation.states().revoked()),
    ] {
        append_value_binding(components, &format!("elevation.state.{name}"), state);
    }
    text(
        components,
        "elevation.validity.timeline",
        elevation.validity().timeline().canonical_name(),
    );
    append_field(
        components,
        "elevation.validity.not-before",
        elevation.validity().not_before(),
    );
    append_field(
        components,
        "elevation.validity.not-after",
        elevation.validity().not_after(),
    );
    unsigned_64(
        components,
        "elevation.validity.maximum-duration-seconds",
        elevation.maximum_duration().as_secs(),
    );
    unsigned(
        components,
        "elevation.validity.maximum-duration-subsecond-nanos",
        elevation.maximum_duration().subsec_nanos(),
    );
    append_relation(components, "elevation.requester", elevation.requester());
    append_relation(components, "elevation.approver", elevation.approver());
    append_relation(components, "elevation.grant", elevation.grant());
    if let Some(resource_relation) = elevation.resource_relation() {
        text(
            components,
            "elevation.resource-relation.posture",
            "governed",
        );
        append_relation(components, "elevation.resource-relation", resource_relation);
    } else {
        text(
            components,
            "elevation.resource-relation.posture",
            "not-applicable",
        );
    }
    append_lifecycle(components, elevation.lifecycle());
    let review = elevation.review();
    append_relation(components, "elevation.review.relation", review.relation());
    append_field(components, "elevation.review.identity", review.identity());
    append_value_binding(components, "elevation.review.kind", review.kind());
    append_relation(components, "elevation.review.scope", review.scope());
    append_relation(components, "elevation.review.reviewer", review.reviewer());
    append_field(components, "elevation.review.status", review.status());
    append_field(
        components,
        "elevation.review.reviewed-at",
        review.reviewed_at(),
    );
    append_value_binding(components, "elevation.review.required", review.required());
    append_value_binding(components, "elevation.review.completed", review.completed());
}

fn append_lifecycle(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    lifecycle: &super::ApplicationCapabilityElevationLifecycleDefinition,
) {
    append_slot(
        components,
        "elevation.lifecycle.elevation-slot",
        lifecycle.elevation_slot(),
    );
    append_slot(
        components,
        "elevation.lifecycle.review-slot",
        lifecycle.review_slot(),
    );
    for (role, transition) in [
        ("request", lifecycle.request()),
        ("approve", lifecycle.approve()),
        ("revoke", lifecycle.revoke()),
        ("complete-review", lifecycle.complete_review()),
    ] {
        append_transition(
            components,
            &format!("elevation.lifecycle.{role}"),
            transition,
        );
    }
}

fn append_transition(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    transition: &ApplicationCapabilityTransitionBinding,
) {
    text(
        components,
        format!("{prefix}.capability"),
        transition.capability(),
    );
    text(
        components,
        format!("{prefix}.capability-type"),
        transition.capability_type(),
    );
    append_operation(components, prefix, transition.operation());
    if let Some(effect) = transition.lifecycle_effect() {
        text(components, format!("{prefix}.effect.posture"), "derived");
        text(components, format!("{prefix}.effect"), effect.effect());
        text(
            components,
            format!("{prefix}.effect-type"),
            effect.effect_type(),
        );
        text(
            components,
            format!("{prefix}.effect-payload-type"),
            effect.payload_type(),
        );
    } else {
        text(
            components,
            format!("{prefix}.effect.posture"),
            "not-applicable",
        );
    }
}

fn append_slot(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    slot: &ApplicationCapabilityContextEntitySlotBinding,
) {
    text(components, format!("{prefix}.context"), slot.context());
    text(
        components,
        format!("{prefix}.context-type"),
        slot.context_identity().as_str(),
    );
    text(components, format!("{prefix}.slot"), slot.slot());
    text(
        components,
        format!("{prefix}.slot-type"),
        slot.slot_identity().as_str(),
    );
    text(components, format!("{prefix}.entity"), slot.entity());
}

pub(super) fn append_operation(
    components: &mut Vec<ApplicationCapabilityCanonicalComponent>,
    prefix: &str,
    operation: &ApplicationCapabilityOperationBinding,
) {
    text(
        components,
        format!("{prefix}.operation"),
        operation.operation(),
    );
    text(
        components,
        format!("{prefix}.operation-type"),
        operation.operation_type(),
    );
    text(
        components,
        format!("{prefix}.input-type"),
        operation.input_type(),
    );
}
