//! Visibility participates in reuse identity; only Native names components.
use super::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum ComponentSelectionSubject {
    Of(EntityId),
    Census,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct ComponentSelectionKey {
    query: bool,
    scope: [u8; 32],
    definition: AggregateDefinitionIdentity,
    root: EntityId,
    subject: ComponentSelectionSubject,
}
impl ComponentSelectionKey {
    pub(super) fn new(
        scope: [u8; 32],
        definition: AggregateDefinitionIdentity,
        root: EntityId,
        subject: ComponentSelectionSubject,
    ) -> Self {
        Self {
            query: false,
            scope,
            definition,
            root,
            subject,
        }
    }
    pub(super) fn for_query(
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphReadContract,
        definition: AggregateDefinitionIdentity,
        root: EntityId,
        subject: ComponentSelectionSubject,
    ) -> Self {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        digest.update(b"worth-query:component-query-scope:v1");
        digest.update(graph.schema_basis_digest().bytes());
        digest.update(graph.digest().bytes());
        Self {
            query: true,
            scope: digest.finalize().into(),
            definition,
            root,
            subject,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn subject(
        &self,
    ) -> ComponentSelectionSubject {
        self.subject
    }
    pub(in crate::domain_computation::primary_graph) fn root(&self) -> EntityId {
        self.root
    }
    pub(in crate::domain_computation::primary_graph) fn matches_scope(
        &self,
        scope: &WorthQueryOperationComponentReadScope,
    ) -> bool {
        !self.query && self.scope == scope.meaning_identity()
    }
    pub(in crate::domain_computation::primary_graph) fn write_locator(
        &self,
        output: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        write!(
            output,
            "{}",
            if self.query {
                "component-query-selection:v1:"
            } else {
                "component-selection:v1:"
            }
        )?;
        for byte in self.scope.iter().chain(self.definition.bytes()) {
            write!(output, "{byte:02x}")?;
        }
        write_entity(output, self.root)?;
        match self.subject {
            ComponentSelectionSubject::Of(member) => {
                write!(output, ":of")?;
                write_entity(output, member)
            }
            ComponentSelectionSubject::Census => write!(output, ":census"),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn locator(&self) -> String {
        let mut result = String::new();
        self.write_locator(&mut result).expect("String write");
        result
    }
}

fn write_entity(output: &mut dyn std::fmt::Write, entity: EntityId) -> std::fmt::Result {
    write!(
        output,
        ":{}:{}:{}",
        entity.partition_value(),
        entity.local_slot_value(),
        entity.generation_value()
    )
}
