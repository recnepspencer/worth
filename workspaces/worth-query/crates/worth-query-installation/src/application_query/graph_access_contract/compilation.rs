use std::sync::Arc;
use worth_foundational::facade::{
    AspectKey, CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
    FieldKey,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryOrderingTerm, ApplicationQueryResultShape,
    ErasedApplicationQueryDefinition,
};

use super::{
    WorthQueryInstalledGraphOrdering, WorthQueryInstalledGraphPlanningPreparation,
    WorthQueryInstalledGraphPredicate, WorthQueryInstalledGraphProjection,
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphReadMeaning,
    WorthQueryInstalledGraphRelation,
};
use crate::application_query::canonical_basis::{prepare_graph_basis, prepare_planning_basis};
use crate::application_query::{
    WorthQueryInstalledRootPath, WorthQueryInstalledRootPathGuard, WorthQueryInstalledRootPathStep,
};

impl WorthQueryInstalledGraphReadContract {
    pub(in crate::application_query) fn compile(
        definition: &ErasedApplicationQueryDefinition,
        schema_basis_digest: &CanonicalDigestId,
        budget: CanonicalDigestWorkBudget,
    ) -> Result<Self, CanonicalDigestDerivationDenial> {
        let mut projections = Vec::new();
        let mut relations = Vec::new();
        flatten_shape(
            definition.result_shape(),
            "root",
            0,
            &mut projections,
            &mut relations,
        );
        let predicates = definition
            .predicates()
            .iter()
            .map(|predicate| {
                let (entity, aspect, field) = predicate.field();
                WorthQueryInstalledGraphPredicate {
                    entity: entity.to_string(),
                    aspect: admitted_aspect_key(aspect),
                    field: admitted_field_key(field),
                    parameter: predicate.parameter().to_string(),
                    scalar_family: predicate.scalar_family(),
                }
            })
            .collect();
        let root_paths = definition
            .root_paths()
            .iter()
            .map(|path| {
                WorthQueryInstalledRootPath::new(
                    path.steps()
                        .iter()
                        .enumerate()
                        .map(|(index, step)| {
                            WorthQueryInstalledRootPathStep::new(
                                step.relation(),
                                step.from(),
                                step.to(),
                                step.direction(),
                                index + 1,
                            )
                        })
                        .collect(),
                    path.guards()
                        .iter()
                        .map(WorthQueryInstalledRootPathGuard::new)
                        .collect(),
                )
            })
            .collect();
        let ordering = definition
            .ordering()
            .iter()
            .map(|term| install_ordering(term, &projections))
            .collect();
        let maximum_result_count = match definition.cardinality() {
            ApplicationQueryCardinality::OptionalOne | ApplicationQueryCardinality::ExactlyOne => 1,
            ApplicationQueryCardinality::Many => usize::MAX,
        };
        let meaning = WorthQueryInstalledGraphReadMeaning {
            schema_basis_digest: *schema_basis_digest,
            root_entity: definition.root_entity().to_string(),
            cardinality: definition.cardinality(),
            projections,
            relations,
            root_paths,
            predicates,
            ordering,
            maximum_traversal_depth: definition.dependency_ceiling().maximum_traversal_depth(),
            maximum_result_count,
        };
        let canonical = prepare_graph_basis(&meaning, budget)?;
        let planning = prepare_planning_basis(
            &WorthQueryInstalledGraphPlanningPreparation { meaning: &meaning },
            budget,
        )?;
        Ok(Self {
            canonical,
            planning,
            meaning,
        })
    }
}

fn install_ordering(
    term: &ApplicationQueryOrderingTerm,
    projections: &[WorthQueryInstalledGraphProjection],
) -> WorthQueryInstalledGraphOrdering {
    let projection = projections
        .iter()
        .find(|projection| projection.slot_type.as_str() == term.slot_type())
        .expect("validated ordering selectors resolve to one result projection");
    debug_assert_eq!(projection.query_type.as_str(), term.query_type());
    debug_assert_eq!(
        (
            projection.entity.as_str(),
            projection.aspect.as_str(),
            projection.field.as_str()
        ),
        term.field()
    );
    debug_assert_eq!(projection.output_name, term.output_name());
    debug_assert_eq!(projection.scalar_family, term.scalar_family());
    debug_assert_eq!(projection.value_type.as_str(), term.value_type());
    WorthQueryInstalledGraphOrdering {
        result_path: projection.result_path.to_string(),
        collection_path: projection.parent_path().to_string(),
        query_type: projection.query_type.clone(),
        slot_type: projection.slot_type.clone(),
        entity: projection.entity.clone(),
        aspect: projection.aspect.clone(),
        field: projection.field.clone(),
        output_name: projection.output_name.clone(),
        direction: term.direction(),
        scalar_family: projection.scalar_family,
        value_type: projection.value_type.clone(),
    }
}

fn flatten_shape(
    shape: &ApplicationQueryResultShape,
    parent_path: &str,
    depth: usize,
    projections: &mut Vec<WorthQueryInstalledGraphProjection>,
    relations: &mut Vec<WorthQueryInstalledGraphRelation>,
) {
    projections.extend(shape.fields().iter().enumerate().map(|(index, field)| {
        WorthQueryInstalledGraphProjection {
            slot_key: Arc::new(field.slot_key()),
            parent_path: Arc::from(parent_path),
            result_path: format!("{parent_path}/field[{index}]").into(),
            query_type: field.query_identity(),
            slot_type: field.slot_identity(),
            entity: field.entity().to_string(),
            aspect: admitted_aspect_key(field.aspect()),
            field: admitted_field_key(field.field()),
            output_name: field.output_name().to_string(),
            scalar_family: field.scalar_family(),
            value_type: field.value_identity(),
            presence: field.presence(),
        }
    }));
    for (index, relation) in shape.relations().iter().enumerate() {
        let result_path = format!("{parent_path}/relation[{index}]");
        relations.push(WorthQueryInstalledGraphRelation {
            slot_key: Arc::new(relation.slot_key()),
            parent_path: Arc::from(parent_path),
            result_path: result_path.clone().into(),
            query_type: relation.query_identity(),
            slot_type: relation.slot_identity(),
            relation: relation.relation().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
            direction: relation.direction(),
            output_name: relation.output_name().to_string(),
            cardinality: relation.cardinality(),
            depth: depth + 1,
        });
        flatten_shape(
            relation.nested_shape(),
            &result_path,
            depth + 1,
            projections,
            relations,
        );
    }
}

fn admitted_aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("installed application-query aspects are schema-admitted")
}

fn admitted_field_key(value: &str) -> FieldKey {
    FieldKey::new(value).expect("installed application-query fields are schema-admitted")
}
