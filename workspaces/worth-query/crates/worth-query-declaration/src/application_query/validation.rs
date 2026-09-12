use std::collections::BTreeSet;

use super::{ApplicationQueryResultShape, WorthQueryPortableApplicationQueryParts};

mod canonical_ordering;
mod disclosure;
mod result_shape;
use result_shape::{
    contains_entity as shape_contains_entity, continuation_parent_path_is_exact,
    counts as count_shape, field_by_slot as shape_field_by_slot,
    many_relation_count as count_many_relations, query_is_consistent as shape_query_is_valid,
    relation_by_slot as shape_relation_by_slot, relation_parent_entity,
    slots_are_unique as shape_slots_are_unique,
};

mod portable_identity;
use portable_identity::{portable_identity_is_valid, shape_portable_identities_are_valid};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationQueryDefinitionDenial {
    EmptyIdentifier,
    InvalidPortableIdentity,
    ResultRootMismatch,
    ResultQueryMismatch,
    DuplicateResultSlot,
    DuplicateParameter,
    InvalidCanonicalOrdering,
    InvalidDisclosureContract,
    DisclosureSelectorMismatch,
    UnknownPredicateParameter,
    UnknownOrderingResultSlot,
    OrderingResultFieldMismatch,
    UnknownContinuationResultSlot,
    ContinuationResultRelationMismatch,
    ContinuationOrderingMissing,
    ContinuationOrderingOutsideTarget,
    ContinuationRequiresPinnedBasis,
    ContinuationRequiresExactlyOneParentPath,
    ContinuationRequiresSingleRoot,
    ContinuationRequiresSingleManyCollection,
    InvalidRootPath,
    DependencyCeilingExceeded,
    PreviewLaneWithoutBasisSupport,
    LiveLaneWithoutCauseContract,
    LiveCauseContractWithoutLane,
    LiveCauseRequiresContinuation,
    LiveCauseScopeSelectorMismatch,
    LiveCauseTargetSelectorMismatch,
    InvalidLiveResourceContract,
}

pub(crate) fn validate_portable_application_query_freshly(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    if definition.name().trim().is_empty() {
        return Err(ApplicationQueryDefinitionDenial::EmptyIdentifier);
    }
    validate_portable_identities(definition)?;
    validate_result_and_root_selection(definition)?;
    validate_parameter_names(definition)?;
    if !canonical_ordering::is_canonical(definition) {
        return Err(ApplicationQueryDefinitionDenial::InvalidCanonicalOrdering);
    }
    disclosure::validate(definition)?;
    validate_predicates(definition)?;
    validate_ordering(definition)?;
    validate_continuation(definition)?;
    validate_live_cause(definition)?;
    validate_dependency_ceiling(definition)?;
    if definition.lanes().preview_enabled() && !definition.basis_support().preview() {
        return Err(ApplicationQueryDefinitionDenial::PreviewLaneWithoutBasisSupport);
    }
    Ok(())
}

fn validate_portable_identities(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    if !portable_identity_is_valid(definition.query_type())
        || !portable_identity_is_valid(definition.parameter_type())
        || !portable_identity_is_valid(definition.result_type())
        || !portable_identity_is_valid(definition.scope_type())
        || definition.parameters().iter().any(|parameter| {
            !portable_identity_is_valid(parameter.value_type())
                || parameter
                    .unit()
                    .is_some_and(|unit| !portable_identity_is_valid(unit))
                || parameter
                    .frame()
                    .is_some_and(|frame| !portable_identity_is_valid(frame))
        })
        || definition
            .root_paths()
            .iter()
            .flat_map(|path| path.guards())
            .any(|guard| !portable_identity_is_valid(guard.value_type()))
        || !shape_portable_identities_are_valid(definition.result_shape())
        || definition.live_cause().is_some_and(|live| {
            [
                live.binding_type(),
                live.payload_type(),
                live.scope_slot_type(),
                live.scope_value_type(),
                live.target_slot_type(),
                live.target_value_type(),
            ]
            .into_iter()
            .any(|identity| !portable_identity_is_valid(identity))
        })
        || definition
            .disclosure()
            .capability_type()
            .is_some_and(|identity| !portable_identity_is_valid(identity))
        || definition
            .disclosure()
            .rules()
            .iter()
            .any(|rule| !rule.selector().portable_identities_are_valid())
    {
        return Err(ApplicationQueryDefinitionDenial::InvalidPortableIdentity);
    }
    Ok(())
}

fn validate_result_and_root_selection(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    if definition.root_entity() != definition.result_shape().root_entity() {
        return Err(ApplicationQueryDefinitionDenial::ResultRootMismatch);
    }
    if definition.root_paths().iter().any(|path| {
        path.steps().is_empty()
            || path.start_entity() != definition.scope_entity()
            || path.terminal_entity() != definition.root_entity()
            || path
                .steps()
                .windows(2)
                .any(|pair| pair[0].child_entity() != pair[1].parent_entity())
            || path.guards().iter().any(|guard| {
                guard.after_step() > path.steps().len()
                    || root_path_entity_after_step(path, guard.after_step()) != Some(guard.entity())
            })
    }) {
        return Err(ApplicationQueryDefinitionDenial::InvalidRootPath);
    }
    if definition.result_shape().query_type() != definition.query_type()
        || !shape_query_is_valid(definition.result_shape(), definition.query_type())
    {
        return Err(ApplicationQueryDefinitionDenial::ResultQueryMismatch);
    }
    let mut slots = BTreeSet::new();
    if !shape_slots_are_unique(definition.result_shape(), &mut slots) {
        return Err(ApplicationQueryDefinitionDenial::DuplicateResultSlot);
    }
    Ok(())
}

fn validate_parameter_names(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    let mut parameter_names = BTreeSet::new();
    if definition
        .parameters()
        .iter()
        .any(|parameter| !parameter_names.insert(parameter.name()))
    {
        return Err(ApplicationQueryDefinitionDenial::DuplicateParameter);
    }
    Ok(())
}

fn validate_predicates(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    if definition.predicates().iter().any(|predicate| {
        !definition
            .parameters()
            .iter()
            .any(|parameter| parameter.name() == predicate.parameter())
    }) {
        return Err(ApplicationQueryDefinitionDenial::UnknownPredicateParameter);
    }
    if definition
        .predicates()
        .iter()
        .any(|predicate| !shape_contains_entity(definition.result_shape(), predicate.field().0))
    {
        return Err(ApplicationQueryDefinitionDenial::ResultRootMismatch);
    }
    Ok(())
}

fn validate_ordering(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    for ordering in definition.ordering() {
        let Some(field) = shape_field_by_slot(definition.result_shape(), ordering.slot_type())
        else {
            return Err(ApplicationQueryDefinitionDenial::UnknownOrderingResultSlot);
        };
        if field.query_type() != ordering.query_type()
            || (field.entity(), field.aspect(), field.field()) != ordering.field()
            || field.output_name() != ordering.output_name()
            || field.scalar_family() != ordering.scalar_family()
            || field.value_type() != ordering.value_type()
        {
            return Err(ApplicationQueryDefinitionDenial::OrderingResultFieldMismatch);
        }
    }
    Ok(())
}

fn validate_dependency_ceiling(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    let shape_counts = count_shape(definition.result_shape());
    let root_path_depth = definition
        .root_paths()
        .iter()
        .map(|path| path.steps().len())
        .max()
        .unwrap_or(0);
    let root_path_relations = definition
        .root_paths()
        .iter()
        .map(|path| path.steps().len())
        .sum::<usize>();
    let ceiling = definition.dependency_ceiling();
    if shape_counts.0.max(root_path_depth) > ceiling.maximum_traversal_depth()
        || shape_counts.1.saturating_add(root_path_relations) > ceiling.maximum_relation_count()
        || shape_counts.2 > ceiling.maximum_projected_field_count()
    {
        return Err(ApplicationQueryDefinitionDenial::DependencyCeilingExceeded);
    }
    Ok(())
}

fn root_path_entity_after_step(
    path: &super::ApplicationQueryRootPathMeaning,
    after_step: usize,
) -> Option<&str> {
    if after_step == 0 {
        Some(path.start_entity())
    } else {
        path.steps()
            .get(after_step - 1)
            .map(super::ApplicationQueryRootPathStep::child_entity)
    }
}

fn validate_live_cause(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    let Some(live) = definition.live_cause() else {
        return if definition.lanes().live_enabled() {
            Err(ApplicationQueryDefinitionDenial::LiveLaneWithoutCauseContract)
        } else {
            Ok(())
        };
    };
    if !definition.lanes().live_enabled() {
        return Err(ApplicationQueryDefinitionDenial::LiveCauseContractWithoutLane);
    }
    let Some(continuation) = definition.continuation() else {
        return Err(ApplicationQueryDefinitionDenial::LiveCauseRequiresContinuation);
    };
    let scope_matches = definition.result_shape().fields().iter().any(|field| {
        field.slot_type() == live.scope_slot_type()
            && (field.entity(), field.aspect(), field.field()) == live.scope_field()
            && field.value_type() == live.scope_value_type()
            && field.entity() == definition.scope_entity()
    });
    if !scope_matches {
        return Err(ApplicationQueryDefinitionDenial::LiveCauseScopeSelectorMismatch);
    }
    let target_matches =
        shape_relation_by_slot(definition.result_shape(), continuation.slot_type()).is_some_and(
            |relation| {
                relation.nested_shape().fields().iter().any(|field| {
                    field.slot_type() == live.target_slot_type()
                        && (field.entity(), field.aspect(), field.field()) == live.target_field()
                        && field.value_type() == live.target_value_type()
                        && field.entity() == continuation.child_entity()
                })
            },
        );
    if !target_matches {
        return Err(ApplicationQueryDefinitionDenial::LiveCauseTargetSelectorMismatch);
    }
    let resources = live.resources();
    if resources.maximum_buffered_causes() == 0
        || resources.maximum_work_per_delivery() == 0
        || resources.maximum_retained_payload_bytes() == 0
    {
        return Err(ApplicationQueryDefinitionDenial::InvalidLiveResourceContract);
    }
    Ok(())
}

fn validate_continuation(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), ApplicationQueryDefinitionDenial> {
    let Some(continuation) = definition.continuation() else {
        return Ok(());
    };
    if definition.cardinality() != super::ApplicationQueryCardinality::ExactlyOne {
        return Err(ApplicationQueryDefinitionDenial::ContinuationRequiresSingleRoot);
    }
    if !definition.basis_support().pinned() {
        return Err(ApplicationQueryDefinitionDenial::ContinuationRequiresPinnedBasis);
    }
    let Some(relation) =
        shape_relation_by_slot(definition.result_shape(), continuation.slot_type())
    else {
        return Err(ApplicationQueryDefinitionDenial::UnknownContinuationResultSlot);
    };
    if continuation.query_type() != definition.query_type()
        || relation.query_type() != continuation.query_type()
        || relation.relation() != continuation.relation()
        || relation.cardinality() != continuation.cardinality()
        || relation.direction() != continuation.direction()
        || relation.nested_shape().root_entity() != continuation.child_entity()
        || relation_parent_entity(relation) != continuation.parent_entity()
    {
        return Err(ApplicationQueryDefinitionDenial::ContinuationResultRelationMismatch);
    }
    if count_many_relations(definition.result_shape()) != 1 {
        return Err(ApplicationQueryDefinitionDenial::ContinuationRequiresSingleManyCollection);
    }
    if continuation_parent_path_is_exact(definition.result_shape(), continuation.slot_type())
        != Some(true)
    {
        return Err(ApplicationQueryDefinitionDenial::ContinuationRequiresExactlyOneParentPath);
    }
    let ordering = definition.ordering();
    if ordering.is_empty() {
        return Err(ApplicationQueryDefinitionDenial::ContinuationOrderingMissing);
    }
    if ordering.iter().any(|term| {
        relation
            .nested_shape()
            .fields()
            .iter()
            .all(|field| field.slot_type() != term.slot_type())
    }) {
        return Err(ApplicationQueryDefinitionDenial::ContinuationOrderingOutsideTarget);
    }
    Ok(())
}
