pub(super) mod content_storage;
use std::sync::Arc;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryMarkerIdentity,
    ApplicationQueryOptionalResultFieldRef, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationCardinality, ApplicationQueryResultRelationRef,
    ApplicationQueryResultSlotKey, ApplicationQueryResultTraversal,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableType;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphProjection, WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::identity::{EntityId, RelationId};

pub(in crate::domain_computation::primary_graph::application_query::projection) mod comparison;
pub(in crate::domain_computation::primary_graph::application_query) mod content;
mod content_comparison;
pub(in crate::domain_computation::primary_graph) use content_comparison::ProjectionContentComparisonFault;
mod disclosure_boundary;
mod field;
mod owned_rows;
pub(in crate::domain_computation::primary_graph::application_query) use owned_rows::{
    OwnedProjectionRows, ProjectionDisposal,
};

pub(in crate::domain_computation::primary_graph::application_query) use disclosure_boundary::{
    WorthQueryApplicationDisclosedProjectionNode, WorthQueryApplicationDisclosedProjectionTree,
    WorthQueryApplicationWorkingProjectionTree,
};

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationProjectedField
{
    result_path: Arc<str>,
    slot_type: Arc<str>,
    slot_key: Arc<ApplicationQueryResultSlotKey>,
    value: AspectValue,
    content: Option<worth_relational::facade::content::RelationalContentFrame>,
}

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationProjectedRelation
{
    result_path: Arc<str>,
    slot_type: Arc<str>,
    slot_key: Arc<ApplicationQueryResultSlotKey>,
    cardinality: ApplicationQueryCardinality,
    predicate_sources: Vec<EntityId>,
    rows: Vec<WorthQueryApplicationProjectionNode>,
}

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationProjectionNode
{
    entity_id: EntityId,
    incoming_relation: Option<RelationId>,
    source_path: Option<Arc<str>>,
    source_dependencies_complete: bool,
    fields: Vec<WorthQueryApplicationProjectedField>,
    relations: Vec<WorthQueryApplicationProjectedRelation>,
}

impl WorthQueryApplicationProjectionNode {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        entity_id: EntityId,
        source_path: Option<Arc<str>>,
        source_dependencies_complete: bool,
        fields: Vec<WorthQueryApplicationProjectedField>,
        relations: Vec<WorthQueryApplicationProjectedRelation>,
    ) -> Self {
        Self {
            entity_id,
            incoming_relation: None,
            source_path,
            source_dependencies_complete,
            fields,
            relations,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) const fn entity_id(
        &self,
    ) -> EntityId {
        self.entity_id
    }

    pub(in crate::domain_computation::primary_graph::application_query) const fn incoming_relation(
        &self,
    ) -> Option<RelationId> {
        self.incoming_relation
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn attach_incoming_relation(
        &mut self,
        relation: RelationId,
    ) -> bool {
        if self.incoming_relation.is_some() {
            return false;
        }
        self.incoming_relation = Some(relation);
        true
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn result_path(
        &self,
    ) -> &str {
        self.source_path.as_deref().unwrap_or("root")
    }

    pub(in crate::domain_computation::primary_graph::application_query) const fn source_dependencies_complete(
        &self,
    ) -> bool {
        self.source_dependencies_complete
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn relations(
        &self,
    ) -> &[WorthQueryApplicationProjectedRelation] {
        &self.relations
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn field(
        &self,
        slot_type: &str,
    ) -> Option<&WorthQueryApplicationProjectedField> {
        self.fields
            .iter()
            .find(|field| field.slot_type.as_ref() == slot_type)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn relation(
        &self,
        slot_type: &str,
    ) -> Option<&WorthQueryApplicationProjectedRelation> {
        self.relations
            .iter()
            .find(|relation| relation.slot_type.as_ref() == slot_type)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn insert_relation(
        &mut self,
        relation: WorthQueryApplicationProjectedRelation,
    ) -> bool {
        if self
            .relations
            .iter()
            .any(|installed| installed.slot_type == relation.slot_type)
        {
            false
        } else {
            self.relations.push(relation);
            true
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn owned_shallow_bytes(
        &self,
    ) -> usize {
        self.fields
            .iter()
            .fold(
                self.fields
                    .capacity()
                    .saturating_mul(size_of::<WorthQueryApplicationProjectedField>()),
                |bytes, field| bytes.saturating_add(field.retained_bytes()),
            )
            .saturating_add(
                self.relations
                    .capacity()
                    .saturating_mul(size_of::<WorthQueryApplicationProjectedRelation>()),
            )
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn retained_bytes(
        &self,
    ) -> usize {
        let Self {
            entity_id: _entity_id,
            incoming_relation: _incoming_relation,
            source_path: _source_path,
            source_dependencies_complete: _source_dependencies_complete,
            fields,
            relations,
        } = self;
        let fields = fields
            .iter()
            .map(WorthQueryApplicationProjectedField::retained_bytes)
            .fold(
                fields
                    .capacity()
                    .saturating_mul(std::mem::size_of::<WorthQueryApplicationProjectedField>()),
                usize::saturating_add,
            );
        relations
            .iter()
            .map(WorthQueryApplicationProjectedRelation::retained_bytes)
            .fold(
                fields.saturating_add(
                    relations.capacity().saturating_mul(std::mem::size_of::<
                        WorthQueryApplicationProjectedRelation,
                    >()),
                ),
                usize::saturating_add,
            )
    }
}

impl WorthQueryApplicationProjectedRelation {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        relation: &WorthQueryInstalledGraphRelation,
        predicate_sources: Vec<EntityId>,
        rows: Vec<WorthQueryApplicationProjectionNode>,
    ) -> Self {
        Self {
            result_path: relation.result_path_identity(),
            slot_type: relation.slot_type_identity(),
            slot_key: relation.slot_key_identity(),
            cardinality: relation.cardinality(),
            predicate_sources,
            rows,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn result_path(
        &self,
    ) -> &str {
        &self.result_path
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn slot_type(
        &self,
    ) -> &str {
        &self.slot_type
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn slot_key(
        &self,
    ) -> &ApplicationQueryResultSlotKey {
        self.slot_key.as_ref()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn rows(
        &self,
    ) -> &[WorthQueryApplicationProjectionNode] {
        &self.rows
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn predicate_sources(
        &self,
    ) -> &[EntityId] {
        &self.predicate_sources
    }

    fn retained_bytes(&self) -> usize {
        let Self {
            result_path: _result_path,
            slot_type: _slot_type,
            slot_key: _slot_key,
            cardinality: _cardinality,
            predicate_sources,
            rows,
        } = self;
        predicate_sources
            .capacity()
            .saturating_mul(std::mem::size_of::<EntityId>())
            .saturating_add(
                rows.iter()
                    .map(WorthQueryApplicationProjectionNode::retained_bytes)
                    .fold(
                        rows.capacity().saturating_mul(std::mem::size_of::<
                            WorthQueryApplicationProjectionNode,
                        >()),
                        usize::saturating_add,
                    ),
            )
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn matches<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        Direction,
        Cardinality,
    >(
        &self,
        selector: &ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            Cardinality,
        >,
    ) -> bool
    where
        Direction: ApplicationQueryResultTraversal,
        Cardinality: ApplicationQueryResultRelationCardinality,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.slot_key.as_ref() == &selector.slot_key() && self.cardinality == selector.cardinality()
    }
}

pub(in crate::domain_computation::primary_graph::application_query::projection) mod materialization;
