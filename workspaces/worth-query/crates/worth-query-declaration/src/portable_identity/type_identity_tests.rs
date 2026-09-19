use std::collections::{BTreeSet, HashSet};

use crate::application_schema::{
    validate_portable_application_schema_freshly, ApplicationSchemaMember,
    WorthQueryPortableApplicationSchemaParts, WorthQueryPortableApplicationSchemaRecord,
};

use super::WorthQueryPortableTypeIdentity;

#[test]
fn reconstructed_identity_owns_dynamic_text() {
    let identity = {
        let source = format!("worth.tests.dynamic.{}", 7);
        WorthQueryPortableTypeIdentity::from_untrusted(source)
    };

    assert_eq!(identity.as_str(), "worth.tests.dynamic.7");
    assert!(identity.is_valid());
}

#[test]
fn declared_and_reconstructed_identity_have_equal_semantics() {
    let declared = WorthQueryPortableTypeIdentity::declared("worth.tests.equal.v1");
    let reconstructed =
        WorthQueryPortableTypeIdentity::from_untrusted("worth.tests.equal.v1".to_owned());

    assert_eq!(declared, reconstructed);
    assert_eq!(
        HashSet::from([declared.clone()]),
        HashSet::from([reconstructed.clone()])
    );
    assert_eq!(BTreeSet::from([declared]), BTreeSet::from([reconstructed]));
}

#[test]
fn reconstructed_identity_remains_unvalidated_description() {
    for invalid in ["", " leading", "trailing ", "contains space", "line\nbreak"] {
        let identity = WorthQueryPortableTypeIdentity::from_untrusted(invalid.to_owned());
        assert!(!identity.is_valid(), "{invalid:?} must fail validation");
    }
}

#[test]
fn reconstructed_identity_reenters_through_fresh_schema_validation() {
    let record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "WORTH.tests".to_owned(),
            name: "owned_identity_readmission".to_owned(),
            major: 1,
            minor: 0,
            members: vec![ApplicationSchemaMember::ApplicationCapabilityContext {
                context: "review".to_owned(),
                context_type: WorthQueryPortableTypeIdentity::from_untrusted(format!(
                    "worth.tests.context.{}",
                    1
                )),
            }],
            contributions: Vec::new(),
        },
    );

    let declaration = validate_portable_application_schema_freshly(record).unwrap();
    let ApplicationSchemaMember::ApplicationCapabilityContext { context_type, .. } =
        &declaration.members()[0]
    else {
        panic!("fresh validation changed the record family");
    };
    assert_eq!(context_type.as_str(), "worth.tests.context.1");
}
