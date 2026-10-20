//! Version and data-integrity boundaries; these carriers do not mint signing authority.
use super::*;
use crate::application_schema::{
    ApplicationReadableScalarValueBinding, ApplicationSigningProvenance,
    ApplicationSigningProvenanceBinding,
};

fn parts() -> ApplicationSigningSubjectProvenanceParts {
    let authentication = super::super::tests::parts();
    let dependency = ApplicationSigningSubjectDependency {
        source: authentication.subject,
        source_field: ApplicationFieldBindingLocus::new("Document", "Facts", "Description"),
        captured_field: ApplicationFieldBindingLocus::new("Evidence", "Facts", "Description"),
        value_binding: "worth.rust.string".into(),
        native_revision: 7,
    };
    ApplicationSigningSubjectProvenanceParts {
        authentication,
        dependencies: vec![dependency],
    }
}
#[test]
fn v1_and_v2_never_decode_as_each_other_and_v2_retains_source_correspondence() {
    let value = ApplicationSigningSubjectProvenance::from_parts(parts()).unwrap();
    let encoded = ApplicationSigningSubjectProvenanceBinding::encode(&value).unwrap();
    assert_eq!(
        ApplicationSigningSubjectProvenanceBinding::decode(&encoded).unwrap(),
        value
    );
    assert!(ApplicationSigningProvenanceBinding::decode(&encoded).is_err());
    let old = ApplicationSigningProvenance::from_parts(super::super::tests::parts()).unwrap();
    let old_encoded = ApplicationSigningProvenanceBinding::encode(&old).unwrap();
    assert_eq!(
        ApplicationSigningProvenanceBinding::decode(&old_encoded).unwrap(),
        old
    );
    assert!(ApplicationSigningSubjectProvenanceBinding::decode(&old_encoded).is_err());
}
#[test]
fn duplicate_captures_are_refused_without_confusing_descriptive_coordinates_with_authority() {
    let mut duplicate = parts();
    duplicate
        .dependencies
        .push(duplicate.dependencies[0].clone());
    assert!(ApplicationSigningSubjectProvenance::from_parts(duplicate).is_err());
    let mut foreign = parts();
    foreign.dependencies[0].source.slot += 1;
    // The data codec cannot decide graph affinity. Only the native producer can
    // establish which source was admitted; these data cannot reconstruct it.
    assert!(ApplicationSigningSubjectProvenance::from_parts(foreign).is_ok());
    let mut repeated = parts();
    let mut second = repeated.dependencies[0].clone();
    second.captured_field =
        ApplicationFieldBindingLocus::new("Evidence", "Facts", "OtherDescription");
    repeated.dependencies.push(second);
    assert!(ApplicationSigningSubjectProvenance::from_parts(repeated).is_ok());
}
