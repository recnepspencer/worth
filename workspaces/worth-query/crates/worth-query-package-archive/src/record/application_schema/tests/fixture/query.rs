use worth_foundational::facade::*;
use worth_query_declaration::facade::application_query::*;
use worth_query_declaration::facade::application_schema::ApplicationFieldPresence;

use super::{text, type_id};

pub(super) fn application_query() -> ErasedApplicationQueryDefinition {
    let query_type = type_id("query");
    let result_type = type_id("result");
    let nested_shape = ApplicationQueryResultShape::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: query_type.clone(),
            root_entity: text("Other"),
            result_type: type_id("nested-result"),
            fields: Vec::new(),
            relations: Vec::new(),
        },
    );
    let result_shape = ApplicationQueryResultShape::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: query_type.clone(),
            root_entity: text("Entity"),
            result_type: result_type.clone(),
            fields: vec![ApplicationQueryResultField::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultFieldParts {
                    query_type: query_type.clone(),
                    slot_type: type_id("field-slot"),
                    entity: text("Entity"),
                    aspect: text("Aspect"),
                    field: text("field"),
                    output_name: text("value"),
                    scalar_family: ScalarAspectType::UInt64,
                    value_type: type_id("u64"),
                    presence: ApplicationFieldPresence::Required,
                },
            )],
            relations: vec![ApplicationQueryResultRelation::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultRelationParts {
                    query_type: query_type.clone(),
                    slot_type: type_id("relation-slot"),
                    relation: text("relates"),
                    from: text("Entity"),
                    to: text("Other"),
                    direction: ApplicationQueryResultTraversalDirection::Forward,
                    output_name: text("other"),
                    cardinality: ApplicationQueryCardinality::Many,
                    nested_shape,
                },
            )],
        },
    );
    ErasedApplicationQueryDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationQueryParts {
            name: text("ArchiveQuery"),
            query_type: query_type.clone(),
            parameter_type: type_id("parameters"),
            result_type,
            scope_type: type_id("scope"),
            root_entity: text("Entity"),
            scope_entity: text("Entity"),
            parameters: vec![ApplicationQueryParameterDefinition::from_untrusted_fields(
                text("expected"),
                ScalarAspectType::UInt64,
                type_id("u64"),
                Some(text("worth.units.metre.v1")),
                Some(text("worth.frames.model.v1")),
            )],
            result_shape,
            root_paths: vec![root_path()],
            cardinality: ApplicationQueryCardinality::Many,
            predicates: vec![ApplicationQueryPredicate::from_untrusted_fields(
                text("Entity"),
                text("Aspect"),
                text("field"),
                text("expected"),
                ScalarAspectType::UInt64,
            )],
            ordering: vec![ApplicationQueryOrderingTerm::from_untrusted_parts(
                WorthQueryPortableApplicationQueryOrderingParts {
                    query_type: query_type.clone(),
                    slot_type: type_id("field-slot"),
                    entity: text("Entity"),
                    aspect: text("Aspect"),
                    field: text("field"),
                    output_name: text("value"),
                    scalar_family: ScalarAspectType::UInt64,
                    value_type: type_id("u64"),
                    direction: ApplicationQueryOrderingDirection::Descending,
                },
            )],
            continuation: Some(ApplicationQueryContinuationTarget::from_untrusted_parts(
                WorthQueryPortableApplicationQueryContinuationParts {
                    query_type: query_type.clone(),
                    slot_type: type_id("relation-slot"),
                    relation: text("relates"),
                    parent_entity: text("Entity"),
                    child_entity: text("Other"),
                    direction: ApplicationQueryResultTraversalDirection::Forward,
                },
            )),
            live_cause: Some(live_cause()),
            dependency_ceiling: ApplicationQueryDependencyCeiling::bounded(4, 3, 2),
            disclosure: disclosure(),
            authorization: ApplicationQueryAuthorizationRequirement::Ability {
                ability: text("Read"),
                scope_entity: text("Entity"),
            },
            basis_support: ApplicationQueryBasisSupport::current_and_pinned().with_preview(),
            lanes: ApplicationQueryLaneEligibility::one_shot()
                .with_historical()
                .with_live()
                .with_preview(),
        },
    )
}

fn root_path() -> ApplicationQueryRootPathMeaning {
    ApplicationQueryRootPathMeaning::from_untrusted_parts(
        WorthQueryPortableApplicationQueryRootPathParts {
            start_entity: text("Entity"),
            terminal_entity: text("Other"),
            steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                text("relates"),
                text("Entity"),
                text("Other"),
                ApplicationQueryRootPathDirection::Forward,
            )],
            guards: vec![ApplicationQueryRootPathGuard::from_untrusted_parts(
                WorthQueryPortableApplicationQueryRootPathGuardParts {
                    after_step: 1,
                    entity: text("Other"),
                    aspect: text("Aspect"),
                    field: text("field"),
                    scalar_family: ScalarAspectType::UInt64,
                    value_type: type_id("u64"),
                    expected: AspectValue::UInt64(7),
                },
            )],
        },
    )
}

fn live_cause() -> ApplicationQueryLiveCauseContract {
    ApplicationQueryLiveCauseContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryLiveCauseParts {
            binding_type: type_id("live-binding"),
            effect: text("Changed"),
            payload_type: type_id("effect-payload"),
            scope_slot_type: type_id("scope-slot"),
            scope_entity: text("Entity"),
            scope_aspect: text("Aspect"),
            scope_field: text("field"),
            scope_value_type: type_id("u64"),
            target_slot_type: type_id("target-slot"),
            target_entity: text("Other"),
            target_aspect: text("Aspect"),
            target_field: text("field"),
            target_value_type: type_id("u64"),
            resources: ApplicationQueryLiveResourceContract::bounded(8, 16, 1024),
        },
    )
}

fn disclosure() -> ApplicationQueryDisclosureContract {
    let field = FieldKey::new("field").unwrap();
    let projection = AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(field.clone())]);
    let diagnostic = AspectMask::<DiagnosticMask>::new([CanonicalFieldPath::single(field)]);
    ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture: ApplicationQueryDisclosurePosture::Governed,
            classification: text("restricted"),
            capability_name: Some(text("Capability")),
            capability_type: Some(type_id("capability")),
            rules: vec![ApplicationQueryDisclosureRule::from_untrusted_fields(
                ApplicationQueryDisclosureSelector::InternalField {
                    entity: text("Entity"),
                    aspect: text("Aspect"),
                    field: text("field"),
                    projection_mask: projection,
                    diagnostic_mask: diagnostic,
                },
                AspectValue::Bool(true),
                ApplicationQueryInfluenceContract::permit([
                    ApplicationQueryObservableInfluence::Ordering,
                    ApplicationQueryObservableInfluence::Pagination,
                ]),
            )],
        },
    )
}
