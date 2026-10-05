use std::sync::Arc;

use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

use crate::domain_computation::primary_graph::provider::{
    RebaseVerificationReason, WorthQueryPrimaryGraphCommittedApplication,
};

use super::super::{RecordedSettlementIdentity, WorthQueryApplicationOutputLineage};
use super::{
    FullVerificationReason, SettlementRegistration, SettlementRegistrationStop,
    SourceInvalidationOwner,
};

/// Both performed and stable publications retain the same exact upstream
/// identities through the invalidation owner's admitted ordered set.
pub(in crate::domain_computation::primary_graph) fn collect_consumed_output_upstream<'a>(
    consumed_outputs: impl IntoIterator<
        Item = &'a crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence,
    >,
    admission: &mut super::InvalidationEditAdmission,
) -> Result<
    im::OrdSet<Arc<RecordedSettlementIdentity>>,
    worth_relational::facade::mvcc::CompanionPreflightStop,
> {
    use super::admission::IndexAdmission;
    let mut upstream = im::OrdSet::new();
    for consumed in consumed_outputs {
        admission.work(1)?;
        admission.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(upstream.len())?;
        upstream.insert(Arc::clone(consumed.identity()));
    }
    Ok(upstream)
}

/// Successful World effect is already authoritative here. Failure to retain
/// derived evidence records its typed full-verification requirement; it cannot
/// turn that performed effect into a no-effect refusal. A commit whose facts
/// could not be rebased retains none, so no settlement is registered for it.
pub(in crate::domain_computation::primary_graph) fn register_completed(
    owner: &SourceInvalidationOwner,
    application: &WorthQueryPrimaryGraphCommittedApplication,
    consumed_outputs: &[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence],
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    identity: Arc<RecordedSettlementIdentity>,
    work_membership: Option<Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>>,
    output_witness: Option<&super::super::SealedNativeOutputWitness>,
    admission: &mut super::InvalidationEditAdmission,
) -> Result<(), FullVerificationReason> {
    let facts = application
        .commit_evidence()
        .rebased_source_facts()
        .map_err(|failed| failed.reason)?;
    let read_basis = runtime
        .read_truth()
        .positioned_snapshot(snapshot)
        .map_err(FullVerificationReason::SelectedSourceUnavailable)?;
    let output_witness = output_witness.ok_or(FullVerificationReason::NativeRevisionUnavailable)?;
    let output_facts = owner
        .prepare_performed_output_facts(output_witness, admission)
        .map_err(FullVerificationReason::MarkingAdmissionDenied)?
        .ok_or(FullVerificationReason::NativeRevisionUnavailable)?;
    let upstream = collect_consumed_output_upstream(consumed_outputs, admission)
        .map_err(FullVerificationReason::MarkingAdmissionDenied)?;
    for consumed in consumed_outputs {
        // The reader compared a restored output in full at the basis it read.
        // That output gets its row first: a consumer registered over a missing
        // row would stay pending behind it.
        if !consumed
            .establish_restored(owner, admission)
            .map_err(registration_reason)?
        {
            return Err(FullVerificationReason::MissingSettlement);
        }
    }
    let stale_at_read_basis = own_effect_stale_ordinals(application, admission)
        .map_err(FullVerificationReason::MarkingAdmissionDenied)?;
    owner
        .register_settlement(
            SettlementRegistration {
                work_membership,
                identity,
                facts,
                output_facts: Some(output_facts),
                read_basis,
                stale_at_read_basis,
                requirement: None,
                upstream,
            },
            admission,
        )
        .map_err(registration_reason)
}

/// A republication re-creates a retained performed output at a later commit.
/// Nothing delivered between the performed read and that commit marked the
/// new row, so it registers as a retained delivery gap and its first reader
/// compares it in full. A requirement the predecessor's row already carries
/// continues unchanged, and a predecessor with no row to continue leaves the
/// republication without one.
#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph) fn register_republished(
    owner: &SourceInvalidationOwner,
    predecessor: &RecordedSettlementIdentity,
    identity: Arc<RecordedSettlementIdentity>,
    facts: Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
    consumed_outputs: &[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence],
    output_witness: &super::super::SealedNativeOutputWitness,
    read_basis: worth_relational::facade::runtime::PositionedRelationalSnapshot,
    admission: &mut super::InvalidationEditAdmission,
) -> Result<(), FullVerificationReason> {
    use FullVerificationReason as Reason;
    let requirement = match owner
        .currentness(&read_basis, predecessor, admission)
        .map_err(Reason::MarkingAdmissionDenied)?
    {
        super::SourceSettlementCurrentness::FullVerificationRequired(
            reason @ (Reason::MissingSettlement
            | Reason::CheckpointRestore
            | Reason::ForeignSource
            | Reason::DifferentBranch
            | Reason::BeforeReadBasis),
        ) => return Err(reason),
        super::SourceSettlementCurrentness::FullVerificationRequired(reason) => reason,
        _ => Reason::RetainedDeliveryGap,
    };
    let output_facts = owner
        .prepare_performed_output_facts(output_witness, admission)
        .map_err(Reason::MarkingAdmissionDenied)?
        .ok_or(Reason::NativeRevisionUnavailable)?;
    let upstream = collect_consumed_output_upstream(consumed_outputs, admission)
        .map_err(Reason::MarkingAdmissionDenied)?;
    owner
        .register_settlement(
            SettlementRegistration {
                work_membership: None,
                identity,
                facts,
                output_facts: Some(output_facts),
                read_basis,
                stale_at_read_basis: im::OrdSet::new(),
                requirement: Some(requirement),
                upstream,
            },
            admission,
        )
        .map_err(registration_reason)
}

/// Why the row of a commit that could not rebase its facts requires
/// verification.
impl From<RebaseVerificationReason> for FullVerificationReason {
    fn from(reason: RebaseVerificationReason) -> Self {
        match reason {
            RebaseVerificationReason::NativeRevisionUnavailable => Self::NativeRevisionUnavailable,
            RebaseVerificationReason::UnsupportedDecisionFact => Self::UnsupportedFact,
            RebaseVerificationReason::AdmissionDenied(stop) => Self::MarkingAdmissionDenied(stop),
        }
    }
}

fn registration_reason(stop: SettlementRegistrationStop) -> FullVerificationReason {
    match stop {
        SettlementRegistrationStop::Alignment(reason) => reason,
        SettlementRegistrationStop::Admission(stop) => {
            FullVerificationReason::MarkingAdmissionDenied(stop)
        }
        SettlementRegistrationStop::Edit(stop) => FullVerificationReason::DerivedEditPending(stop),
    }
}

/// The registration reads at the post-effect snapshot, so no later delivery
/// marks a source fact the effect itself moved. The row starts dirty there.
fn own_effect_stale_ordinals(
    application: &WorthQueryPrimaryGraphCommittedApplication,
    admission: &mut super::InvalidationEditAdmission,
) -> Result<im::OrdSet<usize>, worth_relational::facade::mvcc::CompanionPreflightStop> {
    use super::admission::IndexAdmission;
    let mut stale = im::OrdSet::new();
    for ordinal in application
        .commit_evidence()
        .source_facts_superseded_by_own_effect()
    {
        admission.work(1)?;
        admission.ordered_edit::<usize, ()>(stale.len())?;
        stale.insert(*ordinal);
    }
    Ok(stale)
}

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn require_settlement_verification(
        &mut self,
        identity: &RecordedSettlementIdentity,
        reason: FullVerificationReason,
    ) {
        let coordinate = identity.coordinate();
        if let Some(recorded) = self
            .by_source
            .get_mut(identity.source())
            .and_then(|branches| branches.get_mut(&coordinate.occurrence))
            .and_then(|history| history.get_mut(&coordinate.generation))
            .and_then(|records| records.get(identity.slot()))
            .and_then(|cell| cell.get())
        {
            recorded.require_verification(reason);
        }
    }
}
