//! Shared logical output of the same transition that authored private consequences.
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::AggregateIdentity;

use super::retention_extent as extent;
use super::transition::AggregateTransition;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::schema::data::AggregateIdentityContractDeclaration;

pub(super) struct ChangedProjection {
    pub(super) transition: AggregateTransition,
    pub(super) member_assignments: BTreeMap<EntityId, usize>,
    // Complete prior membership is the proof that an absent assignment is
    // affected/deleted rather than permission to fall back to old identity.
    pub(super) prior_memberships: BTreeMap<EntityId, (RelationId, AggregateIdentity)>,
}

pub(super) struct DefinitionProjection {
    pub(super) contract: AggregateIdentityContractDeclaration,
    // None is a proven absence of affected cohorts, not an uninitialized cache.
    pub(super) changed: Option<ChangedProjection>,
}

#[derive(Default)]
pub(super) struct ProjectionData {
    pub(super) definitions: BTreeMap<KindId, DefinitionProjection>,
}

pub(super) struct AggregateProjection {
    pub(super) data: ProjectionData,
    // One conservative preparation grant, shared with the write admission.
    // It survives final seal while any retained read still depends on it.
    pub(super) _retention: Option<Arc<ExecutionMemoryReservation>>,
}

impl ProjectionData {
    pub(super) fn owned_bytes(&self) -> u64 {
        self.definitions.values().fold(
            self.definitions.len() as u64 * extent::tree_entry::<KindId, DefinitionProjection>(),
            |bytes, definition| {
                bytes.saturating_add(
                    definition
                        .changed
                        .as_ref()
                        .map_or(0, ChangedProjection::owned_bytes),
                )
            },
        )
    }
}

impl ChangedProjection {
    fn owned_bytes(&self) -> u64 {
        self.transition.assignments.iter().fold(
            (self.transition.assignments.capacity()
                * std::mem::size_of::<super::AggregateAssignment>()) as u64,
            |bytes, group| {
                bytes.saturating_add(
                    group.members.len() as u64 * extent::tree_entry::<EntityId, ()>(),
                )
            },
        ) + self.transition.retired.len() as u64 * extent::tree_entry::<AggregateIdentity, ()>()
            + self.transition.lineage.len() as u64
                * extent::tree_entry::<super::AggregateLineageEvent, ()>()
            + self.member_assignments.len() as u64 * extent::tree_entry::<EntityId, usize>()
            + self.prior_memberships.len() as u64 * extent::membership()
    }
}
