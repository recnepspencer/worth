use worth_foundational::facade::ScalarAspectType;

use super::{
    canonical_identity, ApplicationSchemaCanonicalHeader, ApplicationSchemaContributionProvenance,
    ApplicationSchemaMember,
};
use crate::facade::application_schema::{ApplicationFieldPresence, ApplicationRelationIntegrity};

#[test]
fn schema_axes_use_the_current_complete_identity_preimage() {
    let framed = ApplicationSchemaMember::Field {
        entity: "Entity".to_owned(),
        aspect: "Aspect".to_owned(),
        field: "Field".to_owned(),
        presence: ApplicationFieldPresence::Required,
        scalar_family: ScalarAspectType::UInt64,
        value_type: "worth.rust.u64".to_owned(),
        unit: None,
        frame: Some("worth.tests.frame.v1".to_owned()),
        writable: false,
        equality_queryable: false,
    };
    let no_self_edges_relation = ApplicationSchemaMember::Relation {
        relation: "Relation".to_owned(),
        from: "From".to_owned(),
        to: "To".to_owned(),
        integrity:
            ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling(),
    };
    let provenance = ApplicationSchemaContributionProvenance::from_untrusted_parts(
        "worth.tests.contribution.v1".to_owned(),
        Vec::new(),
    );
    for identity in [
        canonical_identity(header(), &[framed], &[]),
        canonical_identity(header(), &[no_self_edges_relation], &[]),
        canonical_identity(header(), &[], &[provenance]),
    ] {
        assert_eq!(
            identity.canonical_basis().payload().version().as_str(),
            "worth-query-application-schema-v13"
        );
    }
}

#[test]
fn relation_endpoint_policy_changes_current_canonical_meaning() {
    let relation = |integrity| ApplicationSchemaMember::Relation {
        relation: "Relation".to_owned(),
        from: "Entity".to_owned(),
        to: "Entity".to_owned(),
        integrity,
    };
    let unchanged = canonical_identity(
        header(),
        &[relation(
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )],
        &[],
    );
    let no_self_edges = canonical_identity(
        header(),
        &[relation(
            ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling(),
        )],
        &[],
    );

    assert!(
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
            .endpoints
            .self_edges_allowed
    );
    assert_eq!(
        unchanged.canonical_basis().payload().version().as_str(),
        "worth-query-application-schema-v13"
    );
    assert_eq!(
        no_self_edges.canonical_basis().payload().version().as_str(),
        "worth-query-application-schema-v13"
    );
    assert_ne!(unchanged, no_self_edges);
}

fn header() -> ApplicationSchemaCanonicalHeader<'static> {
    ApplicationSchemaCanonicalHeader {
        owner: "owner",
        name: "Schema",
        major: 1,
        minor: 0,
    }
}
