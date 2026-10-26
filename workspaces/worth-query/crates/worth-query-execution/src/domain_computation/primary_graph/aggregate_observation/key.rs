//! A descriptive member selection, never permission to read an aggregate.
use std::sync::Arc;
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_relational::facade::identity::{EntityId, KindId};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct AggregateKey {
    relation: Arc<str>,
    entity_kind: KindId,
    relation_kind: KindId,
    member: EntityId,
    definition: AggregateDefinitionIdentity,
}
impl AggregateKey {
    pub(super) fn new(
        relation: &str,
        entity_kind: KindId,
        relation_kind: KindId,
        member: EntityId,
        definition: AggregateDefinitionIdentity,
    ) -> Self {
        Self {
            relation: Arc::from(relation),
            entity_kind,
            relation_kind,
            member,
            definition,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn relation(&self) -> &str {
        &self.relation
    }
    pub(in crate::domain_computation::primary_graph) fn entity_kind(&self) -> KindId {
        self.entity_kind
    }
    pub(in crate::domain_computation::primary_graph) fn relation_kind(&self) -> KindId {
        self.relation_kind
    }
    pub(in crate::domain_computation::primary_graph) fn member(&self) -> EntityId {
        self.member
    }
    pub(in crate::domain_computation::primary_graph) fn definition(
        &self,
    ) -> AggregateDefinitionIdentity {
        self.definition
    }
    pub(in crate::domain_computation::primary_graph) fn write_locator(
        &self,
        output: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        write!(
            output,
            "application-aggregate:{}#{}:{}:{}:{}:{}:{}:",
            self.relation.len(),
            self.relation,
            self.entity_kind.as_u32(),
            self.relation_kind.as_u32(),
            self.member.partition_value(),
            self.member.local_slot_value(),
            self.member.generation_value()
        )?;
        for byte in self.definition.bytes() {
            write!(output, "{byte:02x}")?;
        }
        Ok(())
    }
}
