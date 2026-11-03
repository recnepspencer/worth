//! A pending mark may name an upstream whose equal successor already settled.
use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::InputCutoffVerificationStop;
use super::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::output_lineage::invalidation::ConsumedOutputCurrentness;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

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
) -> Result<Resolution<'edges>, InputCutoffVerificationStop> {
    for identity in edges {
        let current = owner.consumed_output_currentness(selected, identity)?;
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
