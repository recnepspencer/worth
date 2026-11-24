//! What a probed scope's Source read at one candidate image: every record
//! and adjacency list its membership depended on, including the lists it
//! found empty, and every row it produced. A member whose records touch
//! none of them cannot have changed that Source, whatever their kinds.
use super::*;
use crate::identity::data::EntityId;
use crate::validation::candidate_source::ManagedIndexAppliedSourceRead;

#[derive(Default)]
pub(super) struct Reach {
    /// Membership dependencies and rows.
    pub(super) entities: BTreeSet<EntityId>,
    /// `(anchor, relation kind, outgoing)` of every list membership read.
    pub(super) anchors: BTreeSet<(EntityId, KindId, bool)>,
}

impl RelationalAuthoredCoupledCandidate<'_> {
    /// The reach of `program` at `scope` in the retained prefix `earlier`, or
    /// `None` where it cannot be read there (the kind-level answer stands).
    /// Cost: one membership acquisition of that Source.
    pub(super) fn source_reach(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
        earlier: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<Reach>, Denial> {
        use crate::derived_relation::DerivedRelationReadDenial as Read;
        let request = |cause| match cause {
            Denial::Source(Read::Request(_)) => Err(cause),
            _ => Ok(None),
        };
        let loan = match ManagedIndexAppliedSourceRead::issue(
            execution,
            self.identity,
            earlier.clone(),
            program,
            scope,
        ) {
            Ok(loan) => loan,
            Err(cause) => return request(cause),
        };
        let read = (|| {
            let rows = loan.rows(*execution)?;
            let read = rows.acquire_all()?;
            let membership = rows.membership_support(&read)?;
            let mut reach = Reach::default();
            reach
                .entities
                .extend(membership.entity_dependencies().iter().map(|(entity, _)| *entity));
            reach.anchors.extend(
                membership
                    .adjacency_dependencies()
                    .iter()
                    .map(|edge| (edge.anchor, edge.relation, edge.outgoing)),
            );
            reach
                .entities
                .extend(rows.occurrences(&read)?.iter().map(|row| row.entity()));
            Ok::<_, Denial>(reach)
        })();
        match read {
            Ok(reach) => Ok(Some(reach)),
            Err(cause) => request(cause),
        }
    }
}
