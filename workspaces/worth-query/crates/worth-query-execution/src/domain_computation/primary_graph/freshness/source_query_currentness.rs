//! Native revision dependencies of an already authenticated principal.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use worth_relational::facade::mvcc::CompanionPreflightStop;

impl WorthQueryPrincipalFreshnessEvidence {
    pub(in crate::domain_computation) fn source_query_facts(
        &self,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
    ) -> Result<Vec<Fact>, CompanionPreflightStop> {
        let locators = [
            &layout.identity_locator,
            &layout.status_locator,
            &layout.principal_identity_locator,
        ];
        let mut bytes = 6usize
            .checked_mul(std::mem::size_of::<Fact>())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        for locator in locators {
            bytes = bytes
                .checked_add(locator.owned_allocation_capacity_bytes())
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        }

        Ok(vec![
            Fact::Entity { entity_id: self.mapping.entity_id, kind: self.mapping.kind_id },
            Fact::Entity { entity_id: self.target.target, kind: self.target.principal_kind },
            Fact::SourceFieldRevision { entity_id: self.mapping.entity_id, locator: super::super::observations::authoritative(&layout.identity_locator), native_revision: Some(self.mapping.identity_revision) },
            Fact::SourceFieldRevision { entity_id: self.mapping.entity_id, locator: super::super::observations::authoritative(&layout.status_locator), native_revision: Some(self.mapping.status_revision) },
            Fact::SourceFieldRevision { entity_id: self.target.target, locator: super::super::observations::authoritative(&layout.principal_identity_locator), native_revision: Some(self.target.principal_identity_revision) },
            Fact::SourceAdjacencyRevision {
                relation_kind: self.target.relation_kind, anchor: self.mapping.entity_id,
                direction: worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing,
                native_revision: self.target.adjacency_revision, comparison_work_limit: 1,
                endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
            },
        ])
    }
}
