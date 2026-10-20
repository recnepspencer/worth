//! Data integrity only; decoding never grants signing or publication authority.
use super::*;
use crate::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
};
use worth_foundational::facade::{AspectValue, InternedString};

fn parts() -> ApplicationSigningSupersessionParts {
    let coordinates = ApplicationSigningEntityCoordinates {
        partition: 1,
        slot: 2,
        generation: 3,
    };
    ApplicationSigningSupersessionParts {
        evidence: coordinates,
        subject: coordinates,
        actor: coordinates,
        operation: "Save".into(),
        logical_request: [7; 32],
        changed_dependencies: vec![ApplicationFieldBindingLocus::new(
            "Item",
            "Facts",
            "Description",
        )],
    }
}
#[test]
fn supersession_roundtrips_but_refuses_empty_duplicate_and_unknown_payloads() {
    let value = ApplicationSigningSupersession::from_parts(parts()).unwrap();
    let encoded = ApplicationSigningSupersessionBinding::encode(&value).unwrap();
    assert_eq!(
        ApplicationSigningSupersessionBinding::decode(&encoded).unwrap(),
        value
    );
    let mut duplicate = parts();
    duplicate
        .changed_dependencies
        .push(duplicate.changed_dependencies[0].clone());
    assert!(ApplicationSigningSupersession::from_parts(duplicate).is_err());
    let mut empty = parts();
    empty.changed_dependencies.clear();
    assert!(ApplicationSigningSupersession::from_parts(empty).is_err());
    let mut unknown = serde_json::to_value(parts()).unwrap();
    unknown["permission"] = serde_json::json!(true);
    assert!(
        ApplicationSigningSupersessionBinding::decode(&AspectValue::String(InternedString::Raw(
            unknown.to_string()
        )))
        .is_err()
    );
}
