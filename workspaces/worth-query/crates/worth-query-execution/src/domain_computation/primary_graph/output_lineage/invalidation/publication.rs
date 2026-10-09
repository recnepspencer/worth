use std::sync::Arc;

use worth_relational::facade::{
    mvcc::CompanionPreflightStop, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};

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
) -> Result<im::OrdSet<Arc<RecordedSettlementIdentity>>, CompanionPreflightStop> {
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
/// derived evidence records its typed full-verification requirement, and a
/// stop records that the registration is incomplete; neither can turn that
/// performed effect into a no-effect refusal. A commit whose facts
/// could not be rebased retains none, so no settlement is registered for it.
pub(in crate::domain_computation::primary_graph) fn register_completed(
    owner: &SourceInvalidationOwner,
    application: &WorthQueryPrimaryGraphCommittedApplication,
    computation_source: super::super::ComputationSourceEvidence,
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
    let facts = super::super::RetainedSourceFacts::retain(computation_source, facts);
    let read_basis = runtime
        .read_truth()
        .positioned_snapshot(snapshot)
        .map_err(|_| registration_reason(SettlementRegistrationStop::SourceUnavailable))?;
    let output_witness = output_witness.ok_or(FullVerificationReason::NativeRevisionUnavailable)?;
    let output_facts = owner
        .prepare_performed_output_facts(output_witness, admission)
        .map_err(stopped)?
        .ok_or(FullVerificationReason::NativeRevisionUnavailable)?;
    let upstream =
        collect_consumed_output_upstream(consumed_outputs, admission).map_err(stopped)?;
    // A publication may have evicted its full index. The consumed Native
    // evidence remains authoritative: compare it after the effect, then
    // establish its missing rows upstream-first before posting this consumer.
    crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence::establish_missing(
        owner, consumed_outputs, runtime, snapshot, &read_basis, admission,
    )?;
    let stale_at_read_basis = own_effect_stale_ordinals(application, admission).map_err(stopped)?;
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
    facts: super::super::RetainedSourceFacts,
    consumed_outputs: &[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence],
    output_witness: &super::super::SealedNativeOutputWitness,
    read_basis: worth_relational::facade::runtime::PositionedRelationalSnapshot,
    admission: &mut super::InvalidationEditAdmission,
) -> Result<(), FullVerificationReason> {
    use super::SourceSettlementCurrentness as Currentness;
    use FullVerificationReason as Reason;
    let requirement = match owner
        .currentness(&read_basis, predecessor, admission)
        .map_err(stopped)?
    {
        // No row here continues into the republication's. A foreign source
        // is a stop, so the registration is incomplete.
        Currentness::Foreign => return Err(Reason::RegistrationIncomplete),
        Currentness::FullVerificationRequired(reason) if reason.no_row_answers() => {
            return Err(reason)
        }
        Currentness::FullVerificationRequired(reason) => reason,
        Currentness::Clean | Currentness::Dirty(_) | Currentness::PendingUpstream(_) => {
            Reason::RetainedDeliveryGap
        }
    };
    let output_facts = owner
        .prepare_performed_output_facts(output_witness, admission)
        .map_err(stopped)?
        .ok_or(Reason::NativeRevisionUnavailable)?;
    let upstream =
        collect_consumed_output_upstream(consumed_outputs, admission).map_err(stopped)?;
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
            RebaseVerificationReason::NativeFactRevisionUnavailable(ordinal) => {
                Self::NativeFactRevisionUnavailable(ordinal)
            }
            RebaseVerificationReason::IndexedSelectionDenied(denial) => {
                Self::IndexedSelectionDenied(denial)
            }
            RebaseVerificationReason::IndexedSelectionFactDenied(ordinal, denial) => {
                Self::IndexedSelectionFactDenied(ordinal, denial)
            }
            RebaseVerificationReason::UnsupportedDecisionFact => Self::UnsupportedFact,
            RebaseVerificationReason::AdmissionDenied(stop) => stopped(stop),
        }
    }
}

fn registration_reason(stop: SettlementRegistrationStop) -> FullVerificationReason {
    match stop {
        SettlementRegistrationStop::Alignment(reason) => reason,
        SettlementRegistrationStop::Foreign
        | SettlementRegistrationStop::SourceUnavailable
        | SettlementRegistrationStop::Admission(_)
        | SettlementRegistrationStop::Edit(_) => FullVerificationReason::RegistrationIncomplete,
    }
}

/// An admission stop after the World effect cannot deny it. The row records
/// that its registration is incomplete, never the stop itself.
fn stopped(stop: CompanionPreflightStop) -> FullVerificationReason {
    registration_reason(SettlementRegistrationStop::Admission(stop))
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
