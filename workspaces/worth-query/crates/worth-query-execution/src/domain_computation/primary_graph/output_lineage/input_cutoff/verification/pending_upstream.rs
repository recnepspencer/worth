//! A pending mark may name an upstream whose equal successor already settled.
use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::{InputCutoffVerificationStop, InvalidationEditAdmission, SourceInvalidationOwner};
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::{ConsumedOutputCurrentness, SourceSettlementCurrentness},
    RecordedSettlementIdentity,
};

/// Scheduling proof only: the cutoff must still verify handler facts and
/// native consumed evidence before it may certify input reuse.
pub(super) struct ResolvedPendingUpstreams(());
pub(super) enum Resolution<'edges> {
    Resolved(ResolvedPendingUpstreams),
    Awaiting(&'edges Arc<RecordedSettlementIdentity>),
}

pub(super) fn resolve<'edges>(
    edges: &'edges OrdSet<Arc<RecordedSettlementIdentity>>,
    selected: &PositionedRelationalSnapshot,
    owner: &SourceInvalidationOwner,
    admission: &mut InvalidationEditAdmission,
) -> Result<Resolution<'edges>, InputCutoffVerificationStop> {
    for identity in edges {
        let current = owner.consumed_output_currentness(selected, identity, admission)?;
        if !upstream_settled(current) {
            return Ok(Resolution::Awaiting(identity));
        }
    }
    Ok(Resolution::Resolved(ResolvedPendingUpstreams(())))
}

fn upstream_settled(current: ConsumedOutputCurrentness) -> bool {
    match current {
        ConsumedOutputCurrentness::Direct(SourceSettlementCurrentness::Clean)
        | ConsumedOutputCurrentness::CanonicallyEqualClean(_) => true,
        ConsumedOutputCurrentness::Direct(
            SourceSettlementCurrentness::Dirty(_)
            | SourceSettlementCurrentness::PendingUpstream(_)
            | SourceSettlementCurrentness::FullVerificationRequired(_)
            | SourceSettlementCurrentness::Foreign,
        )
        | ConsumedOutputCurrentness::PendingEqualSuccessor
        | ConsumedOutputCurrentness::FullVerificationRequired => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;

    #[test]
    fn only_a_settled_upstream_can_resolve_a_pending_mark() {
        with_committed_own_write(|_, _, cell, _| {
            let identity = Arc::clone(&cell.get().unwrap().settlement_identity);
            assert!(upstream_settled(
                ConsumedOutputCurrentness::CanonicallyEqualClean(identity)
            ));
            assert!(upstream_settled(ConsumedOutputCurrentness::Direct(
                SourceSettlementCurrentness::Clean
            )));
            assert!(!upstream_settled(
                ConsumedOutputCurrentness::PendingEqualSuccessor
            ));
            assert!(!upstream_settled(ConsumedOutputCurrentness::Direct(
                SourceSettlementCurrentness::Dirty(OrdSet::new())
            )));
        });
    }
}
