//! What a probed scope's Source read at one candidate image: every record
//! and adjacency list its membership depended on, including the lists it
//! found empty, and every row it produced. A member whose records touch
//! none of them cannot have changed that Source, whatever their kinds.
use std::collections::BTreeSet;
use worth_relational_contracts::internal_port::identity::data::{
    primitives::KindId, record_ids::EntityId,
};
use worth_relational_contracts::internal_port::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use worth_relational_mutation::internal_port::validation::candidate_source::preparation::rows::ManagedIndexSourceRows;

#[derive(Default)]
pub struct Reach {
    /// Membership dependencies and rows.
    pub(super) entities: BTreeSet<EntityId>,
    /// `(anchor, relation kind, outgoing)` of every list membership read.
    pub(super) anchors: BTreeSet<(EntityId, KindId, bool)>,
}

impl Reach {
    /// Acquire the Source membership support and rows from its admitted loan.
    pub fn acquire(rows: ManagedIndexSourceRows<'_>) -> Result<Self, Denial> {
        let read = rows.acquire_all()?;
        let membership = rows.membership_support(&read)?;
        let mut reach = Reach::default();
        reach.entities.extend(
            membership
                .entity_dependencies()
                .iter()
                .map(|(entity, _)| *entity),
        );
        reach.anchors.extend(
            membership
                .adjacency_dependencies()
                .iter()
                .map(|edge| (edge.anchor, edge.relation, edge.outgoing)),
        );
        reach
            .entities
            .extend(rows.occurrences(&read)?.iter().map(|row| row.entity()));
        Ok(reach)
    }
}
