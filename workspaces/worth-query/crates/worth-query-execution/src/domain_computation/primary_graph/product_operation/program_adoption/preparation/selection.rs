use std::collections::BTreeSet;

use worth_query_declaration::facade::application_schema::ApplicationInvariantScopeTarget;
use worth_query_installation::facade::WorthQueryProgramAdoptionRequirements;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::{RelationalRuntime, VisibilityProjectionView};

use super::WorthQueryBranchAdoptionPreparationDenial;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;

pub(super) struct WorthQueryAdoptionSelection {
    pub(super) entities: Vec<EntityId>,
    pub(super) work_units: usize,
}

/// The selected branch's own root. Adoption reads the branch's writes and
/// never another branch's, so a fork decides only what the fork holds.
pub(super) fn branch_view<'runtime>(
    runtime: &'runtime RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<VisibilityProjectionView<'runtime>, WorthQueryBranchAdoptionPreparationDenial> {
    runtime
        .read_truth()
        .project_observation(&basis.observation())
        .map_err(WorthQueryBranchAdoptionPreparationDenial::BranchBasisUnavailable)
}

pub(super) fn select(
    branch: &VisibilityProjectionView<'_>,
    layout: &WorthQueryPrimaryGraphLayout,
    requirements: &WorthQueryProgramAdoptionRequirements,
    maximum_work_units: usize,
) -> Result<WorthQueryAdoptionSelection, WorthQueryBranchAdoptionPreparationDenial> {
    let mut entity_names = BTreeSet::new();
    for scope in requirements.validation_scopes() {
        for target in scope.targets() {
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
        let read = branch
            .bounded_entities_of_kind(kind, remaining)
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
