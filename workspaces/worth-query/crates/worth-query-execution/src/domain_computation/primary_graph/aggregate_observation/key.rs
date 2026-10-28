//! A descriptive member selection, never permission to read an aggregate.
use super::AggregateSelection;
use std::sync::Arc;
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_relational::facade::identity::{EntityId, KindId};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct AggregateKey<Member = EntityId> {
    pub(super) subject: Arc<str>,
    pub(super) entity_kind: KindId,
    pub(super) selection: AggregateSelection,
    pub(super) member: Member,
    pub(super) definition: AggregateDefinitionIdentity,
}
impl AggregateKey {
    pub(super) fn new(
        subject: &str,
        entity_kind: KindId,
        selection: AggregateSelection,
        member: EntityId,
        definition: AggregateDefinitionIdentity,
    ) -> Self {
        Self {
            subject: Arc::from(subject),
            entity_kind,
            selection,
            member,
            definition,
        }
    }
    pub(super) fn native_scope(
        &self,
    ) -> worth_relational::facade::runtime::PinnedAggregateReadScope<'static> {
        self.selection.scope(self.member)
    }
    pub(in crate::domain_computation::primary_graph) fn member(&self) -> EntityId {
        self.member
    }
    pub(in crate::domain_computation::primary_graph) fn write_locator(
        &self,
        output: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        if let Some(relation) = self.orbit_relation() {
            write!(
                output,
                "application-aggregate:{}#{}:{}:{}:{}:{}:{}:",
                self.subject.len(),
                self.subject,
                self.entity_kind.as_u32(),
                relation.as_u32(),
                self.member.partition_value(),
                self.member.local_slot_value(),
                self.member.generation_value()
            )?;
        } else {
            write!(
                output,
                "application-component:{}#{}:{}:{}:{}:{}:",
                self.subject.len(),
                self.subject,
                self.entity_kind.as_u32(),
                self.member.partition_value(),
                self.member.local_slot_value(),
                self.member.generation_value()
            )?;
        }
        for byte in self.definition.bytes() {
            write!(output, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl<Member> AggregateKey<Member> {
    pub(in crate::domain_computation::primary_graph) fn subject(&self) -> &str {
        &self.subject
    }
    pub(in crate::domain_computation::primary_graph) fn entity_kind(&self) -> KindId {
        self.entity_kind
    }
    pub(in crate::domain_computation::primary_graph) fn orbit_relation(&self) -> Option<KindId> {
        self.selection.orbit_relation()
    }
    pub(in crate::domain_computation::primary_graph) fn definition(
        &self,
    ) -> AggregateDefinitionIdentity {
        self.definition
    }
}
