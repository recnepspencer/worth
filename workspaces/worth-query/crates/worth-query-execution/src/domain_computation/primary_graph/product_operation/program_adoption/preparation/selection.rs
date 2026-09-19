use std::collections::BTreeSet;

use worth_query_declaration::facade::application_schema::ApplicationInvariantScopeTarget;
use worth_query_installation::facade::WorthQueryProgramAdoptionRequirements;
use worth_relational::facade::identity::EntityId;

use super::WorthQueryBranchAdoptionPreparationDenial;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;

pub(super) struct WorthQueryAdoptionSelection {
    pub(super) entities: Vec<EntityId>,
    pub(super) work_units: usize,
}

pub(super) fn select(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    layout: &WorthQueryPrimaryGraphLayout,
    version: worth_relational::facade::identity::VersionId,
    requirements: &WorthQueryProgramAdoptionRequirements,
    maximum_work_units: usize,
) -> Result<WorthQueryAdoptionSelection, WorthQueryBranchAdoptionPreparationDenial> {
    let mut entity_names = BTreeSet::new();
    for rule in requirements.added_rules() {
        for target in rule.validation_scope() {
            match target {
                ApplicationInvariantScopeTarget::Entity(entity) => {
                    entity_names.insert(entity.clone());
                }
                ApplicationInvariantScopeTarget::Relation(relation) => {
                    return Err(
                        WorthQueryBranchAdoptionPreparationDenial::RelationScopeUnsupported {
                            relation: relation.clone(),
                        },
                    );
                }
            }
        }
    }

    let mut entities = BTreeSet::new();
    let mut consumed = 0usize;
    for entity in entity_names {
        let kind = layout.entity_kind(&entity).ok_or_else(|| {
            WorthQueryBranchAdoptionPreparationDenial::UnknownEntityScope {
                entity: entity.clone(),
            }
        })?;
        let remaining = maximum_work_units.saturating_sub(consumed);
        let read = runtime
            .read_truth()
            .bounded_visible_entities_of_kind(kind, version, remaining)
            .map_err(|denial| {
                WorthQueryBranchAdoptionPreparationDenial::SelectionLimitExceeded {
                    maximum_work_units,
                    consumed_work_units: consumed.saturating_add(denial.consumed_work_units()),
                }
            })?;
        consumed = consumed.saturating_add(read.work_units());
        entities.extend(
            read.into_records()
                .into_iter()
                .map(|record| record.entity_id),
        );
    }
    Ok(WorthQueryAdoptionSelection {
        entities: entities.into_iter().collect(),
        work_units: consumed,
    })
}
