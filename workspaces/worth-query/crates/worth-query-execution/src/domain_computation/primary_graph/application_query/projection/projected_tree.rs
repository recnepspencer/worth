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
    ApplicationFieldUnit, OptionalApplicationFieldValue, RequiredApplicationFieldValue,
    WorthQueryInstalledGraphProjection, WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::identity::EntityId;

mod disclosure_boundary;

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
}

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationProjectedRelation
{
    result_path: Arc<str>,
    slot_type: Arc<str>,
    slot_key: Arc<ApplicationQueryResultSlotKey>,
    cardinality: ApplicationQueryCardinality,
    rows: Vec<WorthQueryApplicationProjectionNode>,
}

pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationProjectionNode
{
    entity_id: EntityId,
    fields: Vec<WorthQueryApplicationProjectedField>,
    relations: Vec<WorthQueryApplicationProjectedRelation>,
}

impl WorthQueryApplicationProjectionNode {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        entity_id: EntityId,
        fields: Vec<WorthQueryApplicationProjectedField>,
        relations: Vec<WorthQueryApplicationProjectedRelation>,
    ) -> Self {
        Self {
            entity_id,
            fields,
            relations,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) const fn entity_id(
        &self,
    ) -> EntityId {
        self.entity_id
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

    pub(in crate::domain_computation::primary_graph::application_query) fn retained_bytes(
        &self,
    ) -> usize {
        let fields = self
            .fields
            .iter()
            .map(WorthQueryApplicationProjectedField::retained_bytes)
            .fold(
                self.fields
                    .capacity()
                    .saturating_mul(std::mem::size_of::<WorthQueryApplicationProjectedField>()),
                usize::saturating_add,
            );
        self.relations
            .iter()
            .map(WorthQueryApplicationProjectedRelation::retained_bytes)
            .fold(
                fields.saturating_add(
                    self.relations
                        .capacity()
                        .saturating_mul(
                            std::mem::size_of::<WorthQueryApplicationProjectedRelation>(),
                        ),
                ),
                usize::saturating_add,
            )
    }
}

impl WorthQueryApplicationProjectedField {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        projection: &WorthQueryInstalledGraphProjection,
        value: AspectValue,
    ) -> Self {
        Self {
            result_path: projection.result_path_identity(),
            slot_type: projection.slot_type_identity(),
            slot_key: projection.slot_key_identity(),
            value,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn result_path(
        &self,
    ) -> &str {
        &self.result_path
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn value(
        &self,
    ) -> &AspectValue {
        &self.value
    }

    fn retained_bytes(&self) -> usize {
        self.value.owned_allocation_capacity_bytes()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn matches<
        Query,
        Slot,
        Schema,
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        &self,
        selector: &ApplicationQueryResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> bool
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.slot_key.as_ref() == &selector.slot_key()
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn matches_optional<
        Query,
        Slot,
        Schema,
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        &self,
        selector: &ApplicationQueryOptionalResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> bool
    where
        Field: OptionalApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.slot_key.as_ref() == &selector.slot_key()
    }
}

impl WorthQueryApplicationProjectedRelation {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        relation: &WorthQueryInstalledGraphRelation,
        rows: Vec<WorthQueryApplicationProjectionNode>,
    ) -> Self {
        Self {
            result_path: relation.result_path_identity(),
            slot_type: relation.slot_type_identity(),
            slot_key: relation.slot_key_identity(),
            cardinality: relation.cardinality(),
            rows,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn result_path(
        &self,
    ) -> &str {
        &self.result_path
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn rows(
        &self,
    ) -> &[WorthQueryApplicationProjectionNode] {
        &self.rows
    }

    fn retained_bytes(&self) -> usize {
        self.rows
            .iter()
            .map(WorthQueryApplicationProjectionNode::retained_bytes)
            .fold(
                self.rows
                    .capacity()
                    .saturating_mul(std::mem::size_of::<WorthQueryApplicationProjectionNode>()),
                usize::saturating_add,
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
