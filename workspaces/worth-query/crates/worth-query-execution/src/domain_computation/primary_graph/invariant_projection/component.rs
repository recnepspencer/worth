//! Complete component inspection under installation-owned projection authority.
use super::{
    WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use crate::domain_computation::primary_graph::{
    aggregate_observation::AggregateSelection, schema_layout::ComponentSetLayout,
};
use worth_query_declaration::facade::application_schema::ApplicationComponentSetRef;
use worth_query_installation::facade::ApplicationSchema;
mod physical_reads;
mod result;
pub use result::WorthQueryInvariantProjectionComponent;
#[cfg(test)]
mod tests;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Inspect a complete component at this reader's exact snapshot. This broad
    /// installation authority admits all members; no publication facts are issued.
    /// Work grows with the selected cohort and physical source adjacency, never
    /// with unrelated components or the whole member-kind inventory.
    pub fn component_of<Set, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryInvariantProjectionComponent<Schema, Set, Entity>, Denial> {
        let layout = self.component_layout(set)?;
        if !self.identity_is_local(member, set.member()) || member.kind != layout.member_kind {
            return Err(Denial::new(Kind::ForeignIdentity, set.name()));
        }
        let execution = *self.execution;
        let result = execution.run_native_with_checkpoint(|context| {
            let aggregate = self.read_aggregate_identity(
                layout.member_kind,
                AggregateSelection::ComponentDefinition(layout.definition),
                member.entity_id,
                set.name(),
                None,
            )?;
            self.admit_component_members::<Entity>(&layout, &aggregate, context, set.name())?;
            Ok(WorthQueryInvariantProjectionComponent::new(
                aggregate,
                &layout,
                self.binding_identity.clone(),
                self.authority_identity,
            ))
        });
        match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.walk_stop(stop.into(), set.name())),
        }
    }

    /// Inspect stored lineage for an entity or logical component set under this
    /// privileged projection authority. Historical selection does not require
    /// the identity or any issuing member to remain live.
    pub fn component_lineage<Set, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
        identity: &crate::domain_computation::primary_graph::WorthQueryAggregateIdentity<
            Schema,
            Set,
        >,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryAggregateLineage<Schema, Set>,
        Denial,
    > {
        let layout = self
            .layout
            .component_set(set.name())
            .ok_or_else(|| Denial::new(Kind::ComponentSetNotInstalled, set.name()))?;
        use crate::domain_computation::primary_graph::schema_layout::MemberDomainBinding;
        use worth_relational::facade::schema::NativeMemberDomain;
        let domain_matches = match (
            self.layout.member_binding(set.member_selector()),
            &layout.member_domain,
        ) {
            (Some(MemberDomainBinding::Entity(actual)), NativeMemberDomain::Entity(expected)) => {
                actual == *expected
            }
            (
                Some(MemberDomainBinding::Projected(actual)),
                NativeMemberDomain::Projected { projected },
            ) => actual.parts().identity == *projected,
            _ => false,
        };
        if layout.member.as_ref() != set.member_selector().name() || !domain_matches {
            return Err(Denial::new(Kind::ForeignIdentity, set.name()));
        }
        let scope = identity
            .component_lineage_selection(self.binding_identity, layout.definition)
            .ok_or_else(|| Denial::new(Kind::ForeignIdentity, set.name()))?;
        self.read_lineage(identity, scope, set.name())
    }

    pub(super) fn component_layout<Set, Entity>(
        &self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
    ) -> Result<ComponentSetLayout, Denial> {
        let layout = self
            .layout
            .component_set(set.name())
            .ok_or_else(|| Denial::new(Kind::ComponentSetNotInstalled, set.name()))?;
        if layout.member.as_ref() != set.member()
            || self.layout.entity_kind(set.member()) != Some(layout.member_kind)
        {
            return Err(Denial::new(Kind::ForeignIdentity, set.name()));
        }
        Ok(layout.clone())
    }
}
