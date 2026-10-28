use serde_json::json;
use worth_foundational::{
    aspects, compatibility, AspectLocator, AspectValue, BoundarySourceLocator, LocatorAuthority,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use super::json_lowering_fixtures::task_summary_contract;

#[test]
fn compatibility_front_doors_keep_json_lowering_explicit_and_parity_honest() {
    let contract = task_summary_contract();
    let source = BoundarySourceLocator::aspect(AspectLocator::new(
        LocatorAuthority::SupportOnly,
        aspects()
            .vocabulary()
            .key("task.summary")
            .expect("valid aspect key"),
    ));

    let json_lane = compatibility().json();
    let TransitionOutcome::Success(json_state) = json_lane.lower_state([json_lane.input(
        contract.clone(),
        source.clone(),
        json!({
            "title": "Ship it",
            "done": true,
            "note": "compatibility"
        }),
    )]) else {
        panic!("expected lowered JSON state");
    };

    let struct_value = aspects()
        .vocabulary()
        .struct_value()
        .with_field("title", AspectValue::String("Ship it".into()))
        .with_field("done", AspectValue::Bool(true))
        .with_field("note", AspectValue::String("compatibility".into()))
        .finish()
        .expect("native struct value");
    let TransitionOutcome::Success(validated) =
        aspects().validate().against(&contract).value(struct_value)
    else {
        panic!("expected native validation");
    };
    let TransitionOutcome::Success(native_state) =
        aspects().authoritative_state().admit([validated])
    else {
        panic!("expected native state admission");
    };

    assert_eq!(json_state.payload(), native_state.payload());
}

#[test]
fn compatibility_front_doors_lower_scalar_values_without_becoming_native_authority() {
    let contract = aspects()
        .contract()
        .for_key(
            aspects()
                .vocabulary()
                .key("retry.count")
                .expect("valid aspect key"),
        )
        .identified_by(aspects().vocabulary().identity(9))
        .at_revision(aspects().vocabulary().revision(1))
        .scalar(ScalarAspectType::Int64);
    let source = BoundarySourceLocator::aspect(AspectLocator::new(
        LocatorAuthority::SupportOnly,
        contract.key().clone(),
    ));

    let TransitionOutcome::Success(lowered) =
        compatibility()
            .json()
            .lower_value(&contract, source, &json!(3))
    else {
        panic!("expected lowered compatibility value");
    };

    assert_eq!(lowered.payload().key(), contract.key());
}

#[test]
fn compatibility_front_doors_reject_empty_state_lowering_requests() {
    let outcome = compatibility().json().lower_state([]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(
            worth_foundational::JsonCompatibilityLoweringDenial::StateAdmissionDenied(
                worth_foundational::AuthoritativeStateAdmissionDenial::EmptyAdmission
            )
        )
    );
}
