//! A relation predicate closes only over a declared, equality-queryable field.
use super::super::shape_is_closed;
use super::*;

#[test]
fn relation_predicate_field_must_be_declared_and_equality_queryable() {
    for absent in [false, true] {
        let shape = relation_predicate_shape(absent);
        assert!(!shape_is_closed(
            &relation_predicate_dependencies(false),
            "PredicateQuery",
            &shape
        ));
        assert!(shape_is_closed(
            &relation_predicate_dependencies(true),
            "PredicateQuery",
            &shape
        ));
    }
}

fn relation_predicate_dependencies(equality_queryable: bool) -> Vec<ApplicationSchemaMember> {
    let mut members = dependencies();
    members.extend([
        ApplicationSchemaMember::Entity {
            entity: "Activity".to_owned(),
        },
        ApplicationSchemaMember::Aspect {
            absence: worth_foundational::facade::AbsenceLaw::Required,
            entity: "Activity".to_owned(),
            aspect: "ActivityFacts".to_owned(),
            identity: worth_foundational::facade::AspectIdentity(0x91613006),
            revision: worth_foundational::facade::AspectContractRevision(1),
        },
        ApplicationSchemaMember::Field {
            content_format: None,
            entity: "Activity".to_owned(),
            aspect: "ActivityFacts".to_owned(),
            field: "ActivityId".to_owned(),
            presence: crate::application_schema::ApplicationFieldPresence::Required,
            scalar_family: ScalarAspectType::UInt64,
            canonical: None,
            value_type:
                <u64 as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY
                    .as_str()
                    .to_owned(),
            unit: None,
            frame: None,
            writable: false,
            equality_queryable,
            unique: false,
        },
        ApplicationSchemaMember::Relation {
            relation: "AccountAllActivity".to_owned(),
            from: "Account".to_owned(),
            to: "Activity".to_owned(),
            integrity: crate::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().into(),
        },
    ]);
    members
}

fn relation_predicate_shape(absent: bool) -> ApplicationQueryResultShape {
    let identity = |value: &str| {
        WorthQueryPortableTypeIdentity::from_untrusted(ARCHIVE_DECODE, value.to_owned())
    };
    let nested = ApplicationQueryResultShape::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: identity("PredicateQuery"),
            root_entity: "Activity".to_owned(),
            result_type: identity("ActivityResult"),
            fields: Vec::new(),
            relations: Vec::new(),
        },
    );
    let relation = ApplicationQueryResultRelation::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryResultRelationParts {
            query_type: identity("PredicateQuery"),
            slot_type: identity("ActivitySlot"),
            relation: "AccountAllActivity".to_owned(),
            from: "Account".to_owned(),
            to: "Activity".to_owned(),
            direction: ApplicationQueryResultTraversalDirection::Forward,
            output_name: "activity".to_owned(),
            cardinality: ApplicationQueryCardinality::ExactlyOne,
            predicate: Some(if absent {
                crate::application_query::ApplicationQueryRelationPredicate::FieldAbsent {
                    entity: "Activity".to_owned(),
                    aspect: "ActivityFacts".to_owned(),
                    field: "ActivityId".to_owned(),
                    scalar_family: ScalarAspectType::UInt64,
                }
            } else {
                crate::application_query::ApplicationQueryRelationPredicate::EqualParameter(
                    ApplicationQueryPredicate::from_untrusted_fields(
                        ARCHIVE_DECODE,
                        "Activity".to_owned(),
                        "ActivityFacts".to_owned(),
                        "ActivityId".to_owned(),
                        "activity".to_owned(),
                        ScalarAspectType::UInt64,
                    ),
                )
            }),
            nested_shape: nested,
        },
    );
    ApplicationQueryResultShape::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: identity("PredicateQuery"),
            root_entity: "Account".to_owned(),
            result_type: identity("AccountResult"),
            fields: Vec::new(),
            relations: vec![relation],
        },
    )
}
