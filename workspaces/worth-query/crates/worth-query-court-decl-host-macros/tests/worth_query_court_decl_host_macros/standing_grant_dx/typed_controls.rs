//! W10A typed `worth_query_workflow_controls!`: the grant target is written
//! in the grant record's own field types (here the `Purpose` vocabulary), a
//! context slot selects its entity from an input field, and one `reads` list
//! serves every control operation. The request each control projects and the
//! contract target values the context builds are the same typed values.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_capability_context_entity_slot, worth_query_structured_value_binding,
    worth_query_workflow_controls,
};

use super::grants::REVIEW;
use super::schema::*;

/// The site a control step escalates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Escalate {
    pub site: String,
}

worth_query_structured_value_binding!(pub EscalateInput for Escalate {
    identity: "standing-grant.dx.escalate.input.v1"
});

worth_query_workflow_controls! {
    schema: QmsSchema,
    binding_context: Desktop,
    input: Escalate => EscalateInput,
    result: () => UnitApplicationValueBinding,
    denial: <BodyDenial as ApplicationStructuredValueBinding>::Value => BodyDenial,
    scope: Site, field SiteId = SiteFacts::site_id(), input.site,
    capability_context: pub EscalationContext = "standing-grant.dx.escalation-context.v1",
    grant: {
        action: RoleGrantFacts::role() => REVIEW,
        purpose: RoleGrantFacts::purpose() => GrantPurpose::Safety,
    },
    context_slots: [EscalationSite = SiteFacts::site_id(), input.site],
    reads: [SiteFacts::site_id(), SiteFacts::site_status()],
    controls: [
        pub EscalationOpen = "standing-grant.dx.escalation-open" {
            EscalationOpenOperation, EscalationOpenBinding, EscalationOpenIntent,
        },
        pub EscalationClose = "standing-grant.dx.escalation-close" {
            EscalationCloseOperation, EscalationCloseBinding, EscalationCloseIntent,
        },
    ],
}

worth_query_capability_context_entity_slot!(
    pub EscalationSite in QmsSchema, EscalationContext => Site
);

#[test]
fn a_typed_control_grant_projects_the_values_its_contract_target_installs() {
    let input = Escalate {
        site: "north".to_owned(),
    };
    let open =
        <Escalate as ApplicationCapabilityRequest<QmsSchema, EscalationOpen>>::capability_request(
            &input,
        )
        .expect("the open control projects");
    let close =
        <Escalate as ApplicationCapabilityRequest<QmsSchema, EscalationClose>>::capability_request(
            &input,
        )
        .expect("the close control projects");
    let action = EscalationContext::grant_action().expect("the action target binds");
    let purpose = EscalationContext::grant_purpose().expect("the purpose target binds");
    for projection in [&open, &close] {
        assert_eq!(projection.action(), action.value());
        assert_eq!(projection.purpose(), purpose.value());
        assert_eq!(projection.resource().field(), "SiteId");
        let slots: Vec<_> = projection.context_value().entities().iter().collect();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot().slot(), "EscalationSite");
    }
    // The vocabulary is stored by its durable code, not its name.
    assert_eq!(
        purpose.value(),
        &worth_foundational::facade::AspectValue::UInt64(2)
    );
    assert_eq!(action.field().field(), "Role");
    assert_eq!(purpose.field().field(), "Purpose");
}

// The literal form keeps its shape and text encoding.
worth_query_workflow_controls! {
    schema: QmsSchema,
    binding_context: Desktop,
    input: Escalate => EscalateInput,
    result: () => UnitApplicationValueBinding,
    denial: <BodyDenial as ApplicationStructuredValueBinding>::Value => BodyDenial,
    scope: Site, field SiteId = SiteFacts::site_id(), input.site,
    capability_context: pub LiteralContext = "standing-grant.dx.literal-context.v1",
    grant: "review" for "quality",
    controls: [
        pub LiteralControl = "standing-grant.dx.literal-control" {
            LiteralOperation, LiteralBinding, LiteralIntent,
        },
    ],
}

#[test]
fn a_literal_control_grant_still_projects_text_values() {
    let input = Escalate {
        site: "north".to_owned(),
    };
    let projection =
        <Escalate as ApplicationCapabilityRequest<QmsSchema, LiteralControl>>::capability_request(
            &input,
        )
        .expect("the literal control projects");
    let text = |value: &str| {
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(value.to_owned())
            .expect("text encodes")
            .as_foundational_value()
            .clone()
    };
    assert_eq!(projection.action(), &text("review"));
    assert_eq!(projection.purpose(), &text("quality"));
    assert!(projection.context_value().entities().is_empty());
}
