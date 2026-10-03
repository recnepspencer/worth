use std::sync::Arc;

use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication;

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
/// turn that performed effect into a no-effect refusal or an empty fact set.
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
    let evidence = application.commit_evidence();
    let Some(facts) = evidence
        .retain_observed_source_facts()
        .or_else(|| evidence.retain_verification_source_facts())
    else {
        return Err(FullVerificationReason::NativeRevisionUnavailable);
    };
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
    let requirement = evidence.source_fact_verification_requirement().map(|reason| match reason {
        crate::domain_computation::primary_graph::provider::RebaseVerificationReason::NativeRevisionUnavailable => FullVerificationReason::NativeRevisionUnavailable,
        crate::domain_computation::primary_graph::provider::RebaseVerificationReason::UnsupportedDecisionFact => FullVerificationReason::UnsupportedFact,
        crate::domain_computation::primary_graph::provider::RebaseVerificationReason::AdmissionDenied(stop) => FullVerificationReason::MarkingAdmissionDenied(stop),
    });
    owner
        .register_settlement(
            SettlementRegistration {
                work_membership,
                identity,
                facts,
                output_facts: Some(output_facts),
                read_basis,
                requirement,
                upstream,
            },
            admission,
        )
        .map_err(|stop| match stop {
            SettlementRegistrationStop::Alignment(reason) => reason,
            SettlementRegistrationStop::Admission(stop) => {
                FullVerificationReason::MarkingAdmissionDenied(stop)
            }
            SettlementRegistrationStop::Edit(stop) => {
                FullVerificationReason::DerivedEditPending(stop)
            }
        })
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
