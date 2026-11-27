//! A `FamilySelection` fact certified by re-resolving its family at a
//! certification basis (public #429, plan §3.4). Marks never certify it.
//!
//! The re-resolution classifies each candidate with the read's own
//! [`candidate_verdict`] and selects with the read's own [`select_family`]
//! and [`select_current`], so the two cannot disagree on what the family
//! selects at one basis. It never performs the read's retention.

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::RelationalRuntime;

use super::observed::ObservedFamilySelection;
use super::{candidate_verdict, live_selection_identity, select_current, select_family};
use super::CandidateVerdict;
use crate::equality_type_id::EqualityTypeId as TypeId;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Failure;
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::{
    ConsumedOutputVerification, ConsumedOutputVerificationStop, EvidenceView,
};
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::output_lineage::{
    CertificationBasis, CertificationScope,
};

/// A stop that means the family no longer answers here: `Moved`.
const MOVED: Result<bool, Failure> = Ok(false);

impl ObservedFamilySelection {
    /// Whether the family still selects, at `basis`, what the read selected.
    /// Every refusal and every different selection is `Moved` (`false`).
    pub fn remains_equal_at(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: CertificationBasis<'_>,
    ) -> Result<bool, Failure> {
        self.reresolve(runtime, execution, basis, &[], &[])
    }

    /// Whether the family still selects, at the post-commit `basis` of the
    /// read's own commit, what the read selected (plan §3.7). The commit's own
    /// outputs are not yet settled heads there, so the read's overlay applies
    /// again: its `replaced` partitions are masked and its `applied` members
    /// are added verbatim.
    pub fn remains_equal_rebased(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: CertificationBasis<'_>,
    ) -> Result<bool, Failure> {
        self.reresolve(runtime, execution, basis, &self.replaced, &self.applied)
    }

    /// The re-resolution with the partitions an applied overlay masks
    /// (`replaced`) and the members it supplies (`extra`).
    fn reresolve(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: CertificationBasis<'_>,
        replaced: &[(Option<TypeId>, [u8; 32])],
        extra: &[EntityId],
    ) -> Result<bool, Failure> {
        let Some(port) = self.live_port() else {
            return MOVED;
        };
        let producer = &self.producer;
        let coordinate = basis.coordinate();
        let resolution = port
            .lineage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resolve_current_family_overlaid(
                execution,
                producer.runtime_authority,
                &producer.schema,
                producer.scope,
                producer.family,
                coordinate.occurrence,
                coordinate.generation,
                replaced,
            )
            .map_err(|stop| match stop {
                CompanionPreflightStop::Execution(cause) => Failure::Execution(cause),
                _ => Failure::WorkBudgetExceeded,
            })?;
        if select_family(resolution.family_installed, resolution.ambiguous_publication).is_err() {
            return MOVED;
        }
        let Ok(selected) = runtime.read_truth().positioned_snapshot(basis.snapshot) else {
            return MOVED;
        };
        // The fact's membership in the closure memo spans its candidates.
        let (_scope, image) = match CertificationScope::enter(&port.owner, basis, &selected) {
            Ok(entered) => entered,
            Err(ConsumedOutputVerificationStop::Unavailable) => return MOVED,
            Err(stop) => return Err(Failure::OutputObservation(stop)),
        };
        let mut verdicts = Vec::with_capacity(resolution.candidates.len());
        for candidate in &resolution.candidates {
            let upstream = candidate.consumed_outputs_at(&selected);
            let verdict = candidate_verdict(
                candidate,
                producer.producer,
                |facts, witness| {
                    // A head its readmission verified as part of the consumer's
                    // chain stands in for itself (plan §3.5, S1 STOP 3).
                    if basis.readmitted(&candidate.settlement_identity) {
                        return Ok(ConsumedOutputVerification::Current);
                    }
                    ConsumedOutputEvidence::compare_mark_free(
                        execution,
                        EvidenceView {
                            identity: &candidate.settlement_identity,
                            source_facts: Some(facts),
                            upstream: &upstream,
                            verification_requirement: candidate.verification_requirement,
                            native_output_witness: Some(witness),
                            selected_native_root: &selected,
                            historical: candidate.historical(),
                        },
                        image.as_ref(),
                        runtime,
                        basis,
                        &selected,
                    )
                },
                |facts| {
                    ConsumedOutputEvidence::verify_restored_root_at(
                        execution,
                        &candidate.correspondence,
                        facts,
                        &port.layout,
                        runtime,
                        basis.snapshot,
                    )
                },
            );
            let verdict = match verdict {
                Ok(verdict) => verdict,
                Err(ConsumedOutputVerificationStop::Unavailable) => return MOVED,
                Err(stop) => return Err(Failure::OutputObservation(stop)),
            };
            let entity = if verdict.is_current() {
                match candidate
                    .correspondence
                    .current_entity_of_type(&candidate.output_role, producer.entity_type)
                {
                    Ok(entity) => entity,
                    Err(_) => return MOVED,
                }
            } else {
                None
            };
            let obsolete = matches!(verdict, CandidateVerdict::Obsolete);
            verdicts.push((entity, verdict.is_current(), obsolete));
        }
        let Ok(current) = select_current(verdicts, !extra.is_empty()) else {
            return MOVED;
        };
        let fresh: Vec<EntityId> = current.into_iter().flatten().collect();
        let view = runtime.read_truth().project_snapshot(basis.snapshot);
        for entity in &fresh {
            let live = live_selection_identity(
                view.as_ref(),
                execution,
                *entity,
                &port.layout,
                producer.entity_identifier,
            )
            .map_err(Failure::Execution)?;
            if live.is_err() {
                return MOVED;
            }
        }
        Ok(self.selects_the_same(extra, fresh))
    }
}
