//! Visibility participates in reuse identity; only Native names components.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::domain_computation::primary_graph) enum ComponentSelectionSubject {
    Of(EntityId),
    Census,
    DerivedOf(worth_relational::facade::schema::NativeMemberKey),
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct ComponentSelectionKey {
    query: bool,
    derived: bool,
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
            derived: false,
            scope,
            definition,
            root,
            subject,
        }
    }
    pub(super) fn for_derived(
        scope: &worth_query_installation::facade::WorthQueryOperationDerivedSourceReadScope,
        definition: AggregateDefinitionIdentity,
        root: EntityId,
        member: worth_relational::facade::schema::NativeMemberKey,
    ) -> Self {
        Self {
            query: false,
            derived: true,
            scope: scope.association_identity(),
            definition,
            root,
            subject: ComponentSelectionSubject::DerivedOf(member),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn matches_derived_scope(
        &self,
        scope: &worth_query_installation::facade::WorthQueryOperationDerivedSourceReadScope,
    ) -> bool {
        !self.query && self.derived && self.scope == scope.association_identity()
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
            derived: false,
            scope: digest.finalize().into(),
            definition,
            root,
            subject,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn subject(
        &self,
    ) -> ComponentSelectionSubject {
        self.subject.clone()
    }
    pub(super) fn checkpoint_subject<E>(
        &self,
        poll: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), E> {
        if let ComponentSelectionSubject::DerivedOf(member) = &self.subject {
            crate::domain_computation::primary_graph::aggregate_observation::member_work::checkpoint_names(member,poll)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn root(&self) -> EntityId {
        self.root
    }
    pub(in crate::domain_computation::primary_graph) fn matches_scope(
        &self,
        scope: &WorthQueryOperationComponentReadScope,
    ) -> bool {
        !self.query && !self.derived && self.scope == scope.meaning_identity()
    }
    pub(in crate::domain_computation::primary_graph) fn write_locator(
        &self,
        output: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        write!(
            output,
            "{}",
            if self.derived {
                "derived-component-selection:v1:"
            } else if self.query {
                "component-query-selection:v1:"
            } else {
                "component-selection:v1:"
            }
        )?;
        for byte in self.scope.iter().chain(self.definition.bytes()) {
            write!(output, "{byte:02x}")?;
        }
        write_entity(output, self.root)?;
        match &self.subject {
            ComponentSelectionSubject::Of(member) => {
                write!(output, ":of")?;
                write_entity(output, *member)
            }
            ComponentSelectionSubject::Census => write!(output, ":census"),
            ComponentSelectionSubject::DerivedOf(member) => {
                write!(output, ":derived-of:")?;
                write_member(output, member)
            }
        }
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

pub(super) fn write_member(
    output: &mut dyn std::fmt::Write,
    key: &worth_relational::facade::schema::NativeMemberKey,
) -> std::fmt::Result {
    use worth_relational::facade::schema::NativeMemberKey;
    match key {
        NativeMemberKey::Entity(entity) => {
            write!(output, "entity")?;
            write_entity(output, *entity)
        }
        NativeMemberKey::Projected(member) => {
            write!(
                output,
                "projected:{}#{}:{}:{}#{}:{}#{}",
                member.domain.schema.0.len(),
                member.domain.schema.0,
                member.domain.version.0,
                member.domain.name.len(),
                member.domain.name,
                member.case.len(),
                member.case
            )?;
            write_entity(output, member.carrier)
        }
    }
}
