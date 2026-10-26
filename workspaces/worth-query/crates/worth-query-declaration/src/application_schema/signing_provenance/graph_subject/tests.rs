//! Codec/correspondence evidence only; none of these descriptive values authorize signing.
use super::*;
use crate::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationSigningSubjectProvenance,
    ApplicationSigningSubjectProvenanceBinding, ApplicationSigningSubjectProvenanceParts,
};
use ApplicationSigningGraphSubjectProvenance as Value;
use ApplicationSigningGraphSubjectProvenanceBinding as Binding;
use ApplicationSigningSubjectObservedFieldVersion as Observed;

fn parts() -> ApplicationSigningGraphSubjectProvenanceParts {
    let authentication = super::super::tests::parts();
    let selected = ApplicationSigningEntityCoordinates {
        slot: 2,
        ..authentication.subject
    };
    let captured = ApplicationFieldBindingLocus::new("Evidence", "Facts", "Quantity");
    let shared = ApplicationSigningGraphSubjectDependency {
        source: selected,
        source_field: ApplicationFieldBindingLocus::new("Lot", "Facts", "Quantity"),
        captured_field: captured.clone(),
        value_binding: "worth.rust.u64".into(),
        observed_version: Observed::Present { native_revision: 7 },
    };
    let absent = ApplicationSigningGraphSubjectDependency {
        source: authentication.subject,
        source_field: ApplicationFieldBindingLocus::new("Row", "Facts", "Comment"),
        captured_field: ApplicationFieldBindingLocus::new("Evidence", "Facts", "Comment"),
        value_binding: "worth.rust.string".into(),
        observed_version: Observed::Absent {
            native_revision: None,
        },
    };
    let selection = ApplicationSigningSubjectSelectionDependency {
        captured_field: captured,
        anchor: authentication.subject,
        selected,
        relation: "Lot".into(),
        direction: RelationDirection::Forward,
        native_revision: 9,
    };
    ApplicationSigningGraphSubjectProvenanceParts {
        authentication,
        dependencies: vec![shared, absent],
        selections: vec![selection],
    }
}

#[test]
fn v3_preserves_foreign_selection_and_optional_absence_without_reinterpreting_v2() {
    let original = parts();
    let value = Value::from_parts(original.clone()).unwrap();
    let encoded = Binding::encode(&value).unwrap();
    assert_eq!(Binding::decode(&encoded).unwrap(), value);
    assert!(ApplicationSigningSubjectProvenanceBinding::decode(&encoded).is_err());
    for observed in [
        Observed::Absent {
            native_revision: Some(10),
        },
        Observed::Present {
            native_revision: 11,
        },
    ] {
        let mut changed = original.clone();
        changed.dependencies[1].observed_version = observed;
        let changed = Value::from_parts(changed).unwrap();
        assert_ne!(Binding::encode(&changed).unwrap(), encoded);
        assert_eq!(
            Binding::decode(&Binding::encode(&changed).unwrap()).unwrap(),
            changed
        );
    }
    let old =
        ApplicationSigningSubjectProvenance::from_parts(ApplicationSigningSubjectProvenanceParts {
            authentication: original.authentication,
            dependencies: vec![],
        })
        .unwrap();
    assert!(
        Binding::decode(&ApplicationSigningSubjectProvenanceBinding::encode(&old).unwrap())
            .is_err()
    );
}

#[test]
fn v3_refuses_a_broken_source_chain_or_orphaned_capture_selection() {
    for attack in ["anchor", "terminal", "orphan"] {
        let mut bad = parts();
        match attack {
            "anchor" => bad.selections[0].anchor.slot += 1,
            "terminal" => bad.dependencies[0].source.slot += 1,
            _ => {
                bad.selections[0].captured_field =
                    ApplicationFieldBindingLocus::new("Evidence", "Facts", "Other")
            }
        }
        assert!(Value::from_parts(bad).is_err(), "{attack}");
    }
}
