//! The one owner of predecessor custody for a release above the checkpoint,
//! completed or pending. Its per-object chain is judged against the ordered
//! history and the checkpoint-source heads. A chain that starts at a head is
//! joined to that head's custody at the checkpoint source root: the head
//! settles only what that root already lacked, and drops above the
//! checkpoint are settled by the ordered history, per key.

use worth_store_physical_format::{
    BlobReclaimDescriptorV2, CurrentPhysicalRecordPlacement, PersistedPhysicalRecoveryOperation,
    PersistedRecordIdentity, ReleaseCustodyHeadEntryV1, ReleasedGenerationReclaimBasisV1,
};

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::super::selected_release_gate::admit_source_heads;
use super::closure_evidence::ReleasedClosureEvidence;
use super::continuation::{self, VerifiedCheckpointSourceAbsence};
use super::historical_predecessors::{self, AuthenticatedPredecessors};
use crate::progression::PlanningCustody;

/// Custody of everything a release with a predecessor may find absent.
pub(super) struct PredecessorCustody {
    retained_same_key: Vec<PersistedRecordIdentity>,
    residual: Option<VerifiedCheckpointSourceAbsence>,
}

impl PredecessorCustody {
    pub(super) fn evidence(&self) -> ReleasedClosureEvidence<'_> {
        match &self.residual {
            Some(residual) => ReleasedClosureEvidence::CheckpointHeadAnchored {
                residual,
                retained_same_key: &self.retained_same_key,
            },
            None => ReleasedClosureEvidence::RetainedHistory(&self.retained_same_key),
        }
    }
}

/// Authenticates the chain of the completed release `operation`, whose head
/// effect the ordered history replayed.
pub(super) fn authenticate_completed_chain(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    current: BlobReclaimDescriptorV2,
    source: ReleasedGenerationReclaimBasisV1,
    current_count: u16,
    operation: [u8; 32],
) -> Result<(PlanningContext, AuthenticatedPredecessors), crate::entry::PhysicalRecoveryOutcome> {
    let mut matching = basis
        .observed_pages
        .ordered_releases
        .iter()
        .flatten()
        .filter(|release| release.operation == operation);
    let (Some(release), None) = (matching.next(), matching.next()) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let prior = historical_predecessors::replayed_prior(release);
    authenticate_chain(context, basis, current, source, current_count, prior)
}

/// Authenticates the chain of the pending release `operation`. Its head
/// effect is still the WAL's claim here: `pending_wal::admit` replays the
/// same effect against the selected head media before the release is
/// admitted, and blocks when the head tree disagrees with the prior.
pub(super) fn authenticate_pending_chain(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    current: BlobReclaimDescriptorV2,
    source: ReleasedGenerationReclaimBasisV1,
    current_count: u16,
    operation: [u8; 32],
) -> Result<(PlanningContext, AuthenticatedPredecessors), crate::entry::PhysicalRecoveryOutcome> {
    let mut matching = basis
        .redo
        .projections()
        .iter()
        .filter(|projection| projection.operation() == operation);
    let (
        Some(PersistedPhysicalRecoveryOperation::RecordsDropped {
            head_effect: Some(effect),
            ..
        }),
        None,
    ) = (
        matching
            .next()
            .map(|projection| projection.materialization().operation()),
        matching.next(),
    )
    else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let prior = effect.mutation().expected_prior();
    authenticate_chain(context, basis, current, source, current_count, prior)
}

fn authenticate_chain(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    current: BlobReclaimDescriptorV2,
    source: ReleasedGenerationReclaimBasisV1,
    current_count: u16,
    current_prior: Option<ReleaseCustodyHeadEntryV1>,
) -> Result<(PlanningContext, AuthenticatedPredecessors), crate::entry::PhysicalRecoveryOutcome> {
    // The chain may start at a checkpoint-source head, so the head roster is
    // joined before the history is judged against it.
    let context = admit_source_heads(context, basis)?;
    let checkpoint_heads = match &basis.custody {
        PlanningCustody::SourceHeads(custody) => custody.selected_heads(),
        _ => &[],
    };
    match historical_predecessors::authenticate_chain(
        current,
        source,
        current_count,
        current_prior,
        basis
            .observed_pages
            .ordered_releases
            .as_deref()
            .unwrap_or(&[]),
        checkpoint_heads,
        context.limits.manifest_entries,
    ) {
        Some(predecessors) => Ok((context, predecessors)),
        None => Err(context.redo_block(basis.planning_counters(), None)),
    }
}

/// Mints the custody of an authenticated chain. `release_routes` is the
/// record-sorted inventory of the root the release was built on.
pub(super) fn observe(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    source: ReleasedGenerationReclaimBasisV1,
    predecessors: AuthenticatedPredecessors,
    release_routes: &[CurrentPhysicalRecordPlacement],
) -> Result<(PlanningContext, PredecessorCustody), crate::entry::PhysicalRecoveryOutcome> {
    let retained_same_key = predecessors.retained_same_key;
    if retained_same_key.iter().any(|record| {
        release_routes
            .binary_search_by_key(record, |route| route.record())
            .is_ok()
    }) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let Some(anchor) = predecessors.anchor else {
        return Ok((
            context,
            PredecessorCustody {
                retained_same_key,
                residual: None,
            },
        ));
    };
    let (context, residual) =
        continuation::authenticate(context, basis, anchor.descriptor, source, anchor.count)?;
    Ok((
        context,
        PredecessorCustody {
            retained_same_key,
            residual: Some(residual),
        },
    ))
}
