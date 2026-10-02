//! Exact operation, descriptor and admitted WAL joins for one ordered release.

use sha2::{Digest, Sha256};
use worth_store_physical_format::ReleasedDropWalFateWitnessV1;
use worth_store_recovery_physics::{VerifiedOrderedRootEdge, VerifiedOrderedRootHistory};

use super::{Denial, PlanningContext, ResolvedPlanningBasis};
use crate::entry::PhysicalRecoveryOrderedReleaseJoin as Join;
use crate::orchestration::planning::page_observation::OrderedReleasedObservation;

pub(super) struct JoinedOrderedReleaseMember {
    edge_index: usize,
    wal_fate: ReleasedDropWalFateWitnessV1,
}

impl JoinedOrderedReleaseMember {
    pub(super) fn edge_index(&self) -> usize {
        self.edge_index
    }

    pub(super) fn wal_fate(&self) -> ReleasedDropWalFateWitnessV1 {
        self.wal_fate
    }
}

pub(super) fn join(
    context: &PlanningContext,
    basis: &ResolvedPlanningBasis,
    history: &VerifiedOrderedRootHistory,
    release: &OrderedReleasedObservation,
) -> Result<JoinedOrderedReleaseMember, Denial> {
    let failure = |join| Denial::OperationJoin {
        operation: release.operation,
        join,
    };
    if !basis
        .verified_historical_release_operations
        .contains(&release.operation)
    {
        return Err(failure(Join::HistoricalOperation));
    }
    let mut edges = history.edges().iter().enumerate().filter(|(_, edge)| {
        matches!(edge, VerifiedOrderedRootEdge::Released(value)
            if value.operation() == release.operation)
    });
    let Some((edge_index, VerifiedOrderedRootEdge::Released(edge))) = edges.next() else {
        return Err(failure(Join::ReleasedEdge));
    };
    if edges.next().is_some() {
        return Err(failure(Join::ReleasedEdge));
    }
    if edge.descriptor_record() != release.descriptor_frame.record()
        || edge.descriptor_frame_sha256() != release.descriptor_frame.payload_sha256()
    {
        return Err(failure(Join::Descriptor));
    }
    let mut members = basis
        .sample
        .wal_members()
        .iter()
        .filter(|member| member.operation_identity() == release.operation);
    let Some(member) = members.next() else {
        return Err(failure(Join::SampledWalMember));
    };
    if members.next().is_some()
        || member.lsn_range() != edge.lsn()
        || <[u8; 32]>::from(Sha256::digest(member.canonical_redo())) != edge.redo_sha256()
    {
        return Err(failure(Join::SampledWalMember));
    }
    let frame_binding = {
        let mut frames = context
            .integrity
            .admitted_wal()
            .recoverable_frame_iter(context.selection.wal_tail())
            .filter(|frame| {
                frame.lsn_start() == edge.lsn().start().get()
                    && frame.lsn_end() == edge.lsn().end_exclusive().get()
            });
        let binding = frames.next().map(|frame| {
            (
                frame.lsn_start(),
                frame.lsn_end(),
                frame.identity_digest(),
                frame.payload_digest(),
            )
        });
        (binding, frames.next().is_some())
    };
    let (Some((lsn_start, lsn_end, identity_digest, payload_digest)), false) = frame_binding else {
        return Err(failure(Join::AdmittedWalFrame));
    };
    let wal_fate =
        ReleasedDropWalFateWitnessV1::new(lsn_start, lsn_end, identity_digest, payload_digest)
            .map_err(|cause| Denial::WalFate {
                operation: release.operation,
                cause,
            })?;
    Ok(JoinedOrderedReleaseMember {
        edge_index,
        wal_fate,
    })
}
