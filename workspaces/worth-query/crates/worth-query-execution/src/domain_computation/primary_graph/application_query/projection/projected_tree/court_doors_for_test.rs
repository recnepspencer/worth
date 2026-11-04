//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn entity_id_for_test(&self) -> &EntityId {
        &self.entity_id
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn fields_for_test(&self) -> &Vec<WorthQueryApplicationProjectedField> {
        &self.fields
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn relations_for_test(&self) -> &Vec<WorthQueryApplicationProjectedRelation> {
        &self.relations
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn entity_id_mut_for_test(&mut self) -> &mut EntityId {
        &mut self.entity_id
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn incoming_relation_mut_for_test(&mut self) -> &mut Option<RelationId> {
        &mut self.incoming_relation
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn relations_mut_for_test(&mut self) -> &mut Vec<WorthQueryApplicationProjectedRelation> {
        &mut self.relations
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn source_dependencies_complete_mut_for_test(&mut self) -> &mut bool {
        &mut self.source_dependencies_complete
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn source_path_mut_for_test(&mut self) -> &mut Option<Arc<str>> {
        &mut self.source_path
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn new_for_test(
        entity_id: EntityId,
        source_path: Option<Arc<str>>,
        source_dependencies_complete: bool,
        fields: Vec<WorthQueryApplicationProjectedField>,
        relations: Vec<WorthQueryApplicationProjectedRelation>,
    ) -> Self {
        Self::new(
            entity_id,
            source_path,
            source_dependencies_complete,
            fields,
            relations,
        )
    }
}

impl WorthQueryApplicationProjectedRelation {
    #[doc(hidden)]
    pub fn predicate_sources_for_test(&self) -> &Vec<EntityId> {
        &self.predicate_sources
    }
}

impl WorthQueryApplicationProjectedRelation {
    #[doc(hidden)]
    pub fn rows_for_test(&self) -> &Vec<WorthQueryApplicationProjectionNode> {
        &self.rows
    }
}

impl WorthQueryApplicationProjectedRelation {
    #[doc(hidden)]
    pub fn cardinality_mut_for_test(&mut self) -> &mut ApplicationQueryCardinality {
        &mut self.cardinality
    }
}

impl WorthQueryApplicationProjectedRelation {
    #[doc(hidden)]
    pub fn predicate_sources_mut_for_test(&mut self) -> &mut Vec<EntityId> {
        &mut self.predicate_sources
    }
}

impl WorthQueryApplicationProjectedField {
    #[doc(hidden)]
    pub fn result_path_mut_for_test(&mut self) -> &mut Arc<str> {
        &mut self.result_path
    }
}

impl WorthQueryApplicationProjectedRelation {
    #[doc(hidden)]
    pub fn result_path_mut_for_test(&mut self) -> &mut Arc<str> {
        &mut self.result_path
    }
}

impl WorthQueryApplicationProjectedRelation {
    #[doc(hidden)]
    pub fn rows_mut_for_test(&mut self) -> &mut Vec<WorthQueryApplicationProjectionNode> {
        &mut self.rows
    }
}

impl WorthQueryApplicationProjectedField {
    #[doc(hidden)]
    pub fn slot_key_mut_for_test(&mut self) -> &mut Arc<ApplicationQueryResultSlotKey> {
        &mut self.slot_key
    }
}

impl WorthQueryApplicationProjectedRelation {
    #[doc(hidden)]
    pub fn slot_key_mut_for_test(&mut self) -> &mut Arc<ApplicationQueryResultSlotKey> {
        &mut self.slot_key
    }
}

impl WorthQueryApplicationProjectedField {
    #[doc(hidden)]
    pub fn slot_type_mut_for_test(&mut self) -> &mut Arc<str> {
        &mut self.slot_type
    }
}

impl WorthQueryApplicationProjectedRelation {
    #[doc(hidden)]
    pub fn slot_type_mut_for_test(&mut self) -> &mut Arc<str> {
        &mut self.slot_type
    }
}

impl WorthQueryApplicationProjectedField {
    #[doc(hidden)]
    pub fn value_mut_for_test(&mut self) -> &mut AspectValue {
        &mut self.value
    }
}

impl WorthQueryApplicationProjectionNode {
    #[doc(hidden)]
    pub fn fields_mut_for_test(&mut self) -> &mut Vec<WorthQueryApplicationProjectedField> {
        &mut self.fields
    }
}
