//! Declaration validation and canonical identity tests for related-record
//! absence/existence predicates and internal-relation disclosure selectors.
use super::*;
use crate::application_query::{
    validate_portable_application_query_freshly, ApplicationQueryBasisSupport,
    ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDefinitionDenial, ApplicationQueryDisclosureContract,
    ApplicationQueryInfluenceContract, ApplicationQueryLaneEligibility,
    ApplicationQueryObservableInfluence, ApplicationQueryParameterDefinition,
    ApplicationQueryResultShapeBuilder, ApplicationQueryWhereFilterOperand,
    ApplicationQueryWhereFilterProgram, ApplicationQueryWhereFilterSource,
    ErasedApplicationQueryDefinition, WorthQueryPortableApplicationQueryParts,
};
use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationEntityRef, ApplicationFieldPresence,
    ApplicationRelationIntegrity, ApplicationRelationRef, StringApplicationValueBinding,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use crate::{worth_query_application_query, worth_query_structured_value_binding};
use worth_foundational::expression_api::expressions;
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType, Symbol};

struct Schema;
struct Task;
struct Project;
struct BlockedBy;
struct Row;
worth_query_portable_type!(Row => "worth.tests.related.row.v1");
worth_query_structured_value_binding!(Params for () { identity: "worth.rust.unit" });
worth_query_structured_value_binding!(Rows for Row { identity: "worth.tests.related.row.v1" });
worth_query_application_query!(Query for Schema, identity "worth.tests.related.query.v1",
    parameters Params, result Rows, scope Task => "Task", name "related_test");

fn base_parts() -> WorthQueryPortableApplicationQueryParts {
    let task = ApplicationEntityRef::<Schema, Task>::from_schema_identifier("Task");
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(task)
        .scope(task)
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Task, Row, Rows>::new(task).build(),
        )
        .cardinality(ApplicationQueryCardinality::OptionalOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
        .into_erased()
        .into_parts()
}

fn declared_param(name: &str, scalar: ScalarAspectType) -> ApplicationQueryParameterDefinition {
    ApplicationQueryParameterDefinition::from_untrusted_fields(
        name.to_owned(),
        scalar,
        WorthQueryPortableTypeIdentity::declared("worth.tests.related.param.v1"),
        None,
        None,
    )
}

fn blocked_by(
    direction: RelationDirection,
    polarity: ApplicationQueryRelatedPolarity,
    target_predicates: Vec<ApplicationQueryRelatedTargetPredicate>,
    target_where_filter: Option<ApplicationQueryWhereFilterProgram>,
) -> ApplicationQueryRelatedPredicateMeaning {
    ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryRelatedPredicateParts {
            relation: "BlockedBy".to_owned(),
            from: "Task".to_owned(),
            to: "Task".to_owned(),
            direction,
            polarity,
            target_predicates,
            target_where_filter,
        },
    )
}

fn open_status_target() -> ApplicationQueryRelatedTargetPredicate {
    ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
        "Task".to_owned(),
        "TaskFacts".to_owned(),
        "Status".to_owned(),
        AspectValue::String(InternedString::Raw("open".into())),
        ScalarAspectType::String,
        ApplicationFieldPresence::Required,
    )
}

#[test]
fn canonical_basis_distinguishes_polarity_direction_and_target_predicates() {
    let plain = ErasedApplicationQueryDefinition::from_untrusted_parts(base_parts());
    let mut absent_parts = base_parts();
    absent_parts.related_predicates = vec![blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Absent,
        vec![open_status_target()],
        None,
    )];
    let absent = ErasedApplicationQueryDefinition::from_untrusted_parts(absent_parts);

    let mut exists_parts = base_parts();
    exists_parts.related_predicates = vec![blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Exists,
        vec![open_status_target()],
        None,
    )];
    let exists = ErasedApplicationQueryDefinition::from_untrusted_parts(exists_parts);

    let mut rev_parts = base_parts();
    rev_parts.related_predicates = vec![blocked_by(
        RelationDirection::Reverse,
        ApplicationQueryRelatedPolarity::Absent,
        vec![open_status_target()],
        None,
    )];
    let reverse = ErasedApplicationQueryDefinition::from_untrusted_parts(rev_parts);

    let mut unfiltered_parts = base_parts();
    unfiltered_parts.related_predicates = vec![blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Absent,
        vec![],
        None,
    )];
    let unfiltered = ErasedApplicationQueryDefinition::from_untrusted_parts(unfiltered_parts);

    for (a, b) in [
        (&plain, &absent),
        (&absent, &exists),
        (&absent, &reverse),
        (&absent, &unfiltered),
    ] {
        assert_ne!(a.canonical_basis(), b.canonical_basis());
    }
    assert_eq!(
        validate_portable_application_query_freshly(absent.parts()),
        Ok(())
    );
    assert_eq!(
        validate_portable_application_query_freshly(exists.parts()),
        Ok(())
    );
    assert_eq!(
        validate_portable_application_query_freshly(reverse.parts()),
        Ok(())
    );
}

#[test]
fn invalid_related_predicates_refuse_with_exact_typed_denials() {
    // Wrong root endpoint refuses with RelatedPredicateEndpointMismatch.
    let mut bad_endpoint = base_parts();
    bad_endpoint.related_predicates = vec![
        ApplicationQueryRelatedPredicateMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRelatedPredicateParts {
                relation: "Contains".into(),
                from: "Project".into(),
                to: "Task".into(),
                direction: RelationDirection::Forward,
                polarity: ApplicationQueryRelatedPolarity::Absent,
                target_predicates: vec![],
                target_where_filter: None,
            },
        ),
    ];
    assert_eq!(
        validate_portable_application_query_freshly(&bad_endpoint),
        Err(ApplicationQueryDefinitionDenial::RelatedPredicateEndpointMismatch)
    );

    // Foreign target field refuses with RelatedPredicateTargetFieldMismatch.
    let mut bad_target = base_parts();
    bad_target.related_predicates = vec![blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Absent,
        vec![
            ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
                "Project".into(),
                "ProjectFacts".into(),
                "ProjectKey".into(),
                AspectValue::UInt64(1),
                ScalarAspectType::UInt64,
                ApplicationFieldPresence::Required,
            ),
        ],
        None,
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&bad_target),
        Err(ApplicationQueryDefinitionDenial::RelatedPredicateTargetFieldMismatch)
    );

    // Oversized (> 64 KiB) or Symbol target literal refuses with InvalidPredicateValue.
    for value in [
        AspectValue::String(InternedString::Raw("x".repeat(64 * 1024 + 1))),
        AspectValue::String(InternedString::Symbol(Symbol(1))),
        AspectValue::Int64(7),
    ] {
        let mut bad_lit = base_parts();
        bad_lit.related_predicates = vec![blocked_by(
            RelationDirection::Forward,
            ApplicationQueryRelatedPolarity::Absent,
            vec![
                ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
                    "Task".into(),
                    "TaskFacts".into(),
                    "Status".into(),
                    value,
                    ScalarAspectType::String,
                    ApplicationFieldPresence::Required,
                ),
            ],
            None,
        )];
        assert_eq!(
            validate_portable_application_query_freshly(&bad_lit),
            Err(ApplicationQueryDefinitionDenial::InvalidPredicateValue)
        );
    }

    // Optional parameter in structured target equality refuses with PredicateParameterMismatch.
    let mut opt_target = base_parts();
    opt_target.parameters =
        vec![declared_param("st", ScalarAspectType::String).with_optional_presence()];
    opt_target.related_predicates = vec![blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Absent,
        vec![
            ApplicationQueryRelatedTargetPredicate::from_untrusted_parameter(
                "Task".into(),
                "TaskFacts".into(),
                "Status".into(),
                "st".into(),
                ScalarAspectType::String,
                ApplicationFieldPresence::Required,
            ),
        ],
        None,
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&opt_target),
        Err(ApplicationQueryDefinitionDenial::PredicateParameterMismatch)
    );

    // Duplicate related predicates refuse with InvalidCanonicalOrdering.
    let mut dup = base_parts();
    let one = blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Absent,
        vec![open_status_target()],
        None,
    );
    dup.related_predicates = vec![one.clone(), one.clone()];
    assert_eq!(
        validate_portable_application_query_freshly(&dup),
        Err(ApplicationQueryDefinitionDenial::InvalidCanonicalOrdering)
    );

    // Contradictory Absent + Exists over identical matcher refuses with RelatedPredicateContradictory.
    let mut contra_pol = base_parts();
    let exists_same = blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Exists,
        vec![open_status_target()],
        None,
    );
    contra_pol.related_predicates = vec![one.clone(), exists_same];
    assert_eq!(
        validate_portable_application_query_freshly(&contra_pol),
        Err(ApplicationQueryDefinitionDenial::RelatedPredicateContradictory)
    );

    // Contradictory unequal literals on the same target field refuses with RelatedPredicateContradictory.
    let done_status_target = ApplicationQueryRelatedTargetPredicate::from_untrusted_value(
        "Task".into(),
        "TaskFacts".into(),
        "Status".into(),
        AspectValue::String(InternedString::Raw("done".into())),
        ScalarAspectType::String,
        ApplicationFieldPresence::Required,
    );
    let mut contra_lit = base_parts();
    contra_lit.related_predicates = vec![blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Exists,
        vec![done_status_target.clone(), open_status_target()],
        None,
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&contra_lit),
        Err(ApplicationQueryDefinitionDenial::RelatedPredicateContradictory)
    );

    // Distinct filters on the same relation with opposite polarities remain valid!
    let mut distinct_valid = base_parts();
    distinct_valid.related_predicates = vec![
        one,
        blocked_by(
            RelationDirection::Forward,
            ApplicationQueryRelatedPolarity::Exists,
            vec![done_status_target],
            None,
        ),
    ];
    assert!(validate_portable_application_query_freshly(&distinct_valid).is_ok());

    // Target where_filter field outside target entity refuses.
    let draft = expressions().parse("k == k").unwrap().encode();
    let foreign_filter = ApplicationQueryWhereFilterProgram::from_untrusted(
        vec![ApplicationQueryWhereFilterOperand::from_untrusted(
            "k".into(),
            ApplicationQueryWhereFilterSource::Field {
                entity: "Project".into(),
                aspect: "ProjectFacts".into(),
                field: "ProjectKey".into(),
                scalar_family: ScalarAspectType::UInt64,
                presence: ApplicationFieldPresence::Required,
            },
        )],
        &draft,
    )
    .unwrap();
    let mut bad_filter = base_parts();
    bad_filter.related_predicates = vec![blocked_by(
        RelationDirection::Forward,
        ApplicationQueryRelatedPolarity::Exists,
        vec![],
        Some(foreign_filter),
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&bad_filter),
        Err(ApplicationQueryDefinitionDenial::RelatedPredicateTargetFieldMismatch)
    );

    // InternalRelation disclosure rule changes disclosure canonical basis.
    let rel = ApplicationRelationRef::<Schema, BlockedBy, Task, Task>::from_schema_identifiers(
        "BlockedBy",
        "Task",
        "Task",
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    );
    let _ = ApplicationEntityRef::<Schema, Project>::from_schema_identifier("Project");
    let allow =
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("ok".into())
            .unwrap();
    let d1 = ApplicationQueryDisclosureContract::public();
    let d2 = d1.clone().use_relation_by(
        rel,
        RelationDirection::Forward,
        allow,
        ApplicationQueryInfluenceContract::permit([
            ApplicationQueryObservableInfluence::RowPresence,
        ]),
    );
    assert_ne!(d1.canonical_basis(), d2.canonical_basis());
}
