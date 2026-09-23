use std::collections::BTreeMap;

use worth_query_declaration::facade::application_query::{
    ApplicationQueryOrderingDirection, ApplicationQueryResultTraversalDirection,
};
use worth_query_installation::facade::{
    ApplicationSchemaMember, ErasedApplicationSchemaDeclaration,
    WorthQueryInstalledApplicationContinuationContract,
};
use worth_relational::facade::{
    identity::KindId,
    indexes::{
        DerivedIndexId, RelatedEntityEndpoint, RelatedEntityOrderingDirection,
        RelatedEntityOrderingField,
    },
};

use super::{
    planned_field_locator, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryRelationLayout,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryPrimaryContinuationOrderingLayout {
    relation: String,
    relation_kind: KindId,
    parent_endpoint: RelatedEntityEndpoint,
    child_entity: String,
    child_kind: KindId,
    ordering: Vec<RelatedEntityOrderingField>,
    index_id: DerivedIndexId,
}

pub(super) fn lower_continuation_orderings(
    schema: &ErasedApplicationSchemaDeclaration,
    entity_kinds: &BTreeMap<String, KindId>,
    relation_layouts: &BTreeMap<String, WorthQueryPrimaryRelationLayout>,
) -> Result<
    Vec<WorthQueryPrimaryContinuationOrderingLayout>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let mut layouts = Vec::new();
    for definition in schema.members().iter().filter_map(|member| match member {
        ApplicationSchemaMember::ApplicationQuery { definition } => Some(definition),
        _ => None,
    }) {
        let Some(target) = definition.continuation() else {
            continue;
        };
        let relation = relation_layouts
            .get(target.relation())
            .ok_or_else(|| super::invalid_member(target.relation()))?;
        let child_kind = entity_kinds
            .get(target.child_entity())
            .copied()
            .ok_or_else(|| super::invalid_member(target.child_entity()))?;
        let ordering = definition
            .ordering()
            .iter()
            .map(|term| {
                let (_, aspect, field) = term.field();
                Ok(RelatedEntityOrderingField::new(
                    planned_field_locator(aspect, field)?,
                    match term.direction() {
                        ApplicationQueryOrderingDirection::Ascending => {
                            RelatedEntityOrderingDirection::Ascending
                        }
                        ApplicationQueryOrderingDirection::Descending => {
                            RelatedEntityOrderingDirection::Descending
                        }
                    },
                ))
            })
            .collect::<Result<Vec<_>, WorthQueryPrimaryGraphInstallationDenial>>()?;
        let layout = WorthQueryPrimaryContinuationOrderingLayout {
            relation: target.relation().to_string(),
            relation_kind: relation.kind,
            parent_endpoint: match target.direction() {
                ApplicationQueryResultTraversalDirection::Forward => {
                    RelatedEntityEndpoint::SourceParent
                }
                ApplicationQueryResultTraversalDirection::Reverse => {
                    RelatedEntityEndpoint::TargetParent
                }
            },
            child_entity: target.child_entity().to_string(),
            child_kind,
            ordering,
            index_id: DerivedIndexId(0),
        };
        if !layouts.contains(&layout) {
            layouts.push(layout);
        }
    }
    Ok(layouts)
}

impl WorthQueryPrimaryContinuationOrderingLayout {
    pub(in crate::domain_computation::primary_graph) fn index_definition(
        &self,
        ordinal: usize,
    ) -> worth_relational::facade::indexes::DerivedIndexDefinition {
        worth_relational::facade::indexes::DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: format!("application-continuation-ordering.{ordinal}"),
            kind: worth_relational::facade::indexes::DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: self.relation_kind,
                parent_endpoint: self.parent_endpoint,
                child_kind: self.child_kind,
                ordering: self.ordering.clone(),
            },
            branch_scoped: false,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_index(
        &mut self,
        index_id: DerivedIndexId,
    ) {
        self.index_id = index_id;
    }

    pub(super) const fn index_id(&self) -> DerivedIndexId {
        self.index_id
    }

    pub(super) fn matches(
        &self,
        contract: &WorthQueryInstalledApplicationContinuationContract,
    ) -> bool {
        self.relation == contract.relation()
            && self.child_entity == contract.child_entity()
            && self.parent_endpoint
                == match contract.direction() {
                    ApplicationQueryResultTraversalDirection::Forward => {
                        RelatedEntityEndpoint::SourceParent
                    }
                    ApplicationQueryResultTraversalDirection::Reverse => {
                        RelatedEntityEndpoint::TargetParent
                    }
                }
            && self.ordering.len() == contract.ordering().len()
            && self
                .ordering
                .iter()
                .zip(contract.ordering())
                .all(|(installed, declared)| {
                    let (_, aspect, field) = declared.field();
                    installed.locator().aspect().aspect_key().as_str() == aspect
                        && installed
                            .locator()
                            .field_path()
                            .fields()
                            .first()
                            .is_some_and(|key| key.as_str() == field)
                        && installed.direction()
                            == match declared.direction() {
                                ApplicationQueryOrderingDirection::Ascending => {
                                    RelatedEntityOrderingDirection::Ascending
                                }
                                ApplicationQueryOrderingDirection::Descending => {
                                    RelatedEntityOrderingDirection::Descending
                                }
                            }
                })
    }
}

impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn register_continuation_orderings<E>(
        &mut self,
        mut register: impl FnMut(
            worth_relational::facade::indexes::DerivedIndexDefinition,
        ) -> Result<DerivedIndexId, E>,
    ) -> Result<(), E> {
        for (ordinal, continuation) in self.continuation_orderings.iter_mut().enumerate() {
            let index_id = register(continuation.index_definition(ordinal))?;
            continuation.bind_index(index_id);
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn supports_continuation_ordering(
        &self,
        contract: &WorthQueryInstalledApplicationContinuationContract,
    ) -> bool {
        self.continuation_orderings
            .iter()
            .any(|layout| layout.matches(contract))
    }

    pub(in crate::domain_computation::primary_graph) fn continuation_ordering_index_id(
        &self,
        contract: &WorthQueryInstalledApplicationContinuationContract,
    ) -> Option<DerivedIndexId> {
        self.continuation_orderings
            .iter()
            .find(|layout| layout.matches(contract))
            .map(WorthQueryPrimaryContinuationOrderingLayout::index_id)
    }

    pub(in crate::domain_computation::primary_graph) fn continuation_ordering_index_ids(
        &self,
    ) -> impl Iterator<Item = DerivedIndexId> + '_ {
        self.continuation_orderings
            .iter()
            .map(WorthQueryPrimaryContinuationOrderingLayout::index_id)
    }
}
