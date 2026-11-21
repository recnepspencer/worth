use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use crate::domain_computation::primary_graph::application_checkpoint::facts::NativeFactPayloadBuilder;
use std::collections::BTreeSet;
use std::sync::Arc;

use worth_relational::facade::identity::EntityId;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::fact_verification_observation;

mod applied_support;
pub(in crate::domain_computation::primary_graph::provider) mod coupled_deletions;
pub use worth_query_execution_lineage::facade::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::{
    diagnostic, failure, FactlessCurrentness, FailedRebase, OwnEffect, OwnEffectOnReads,
    PreparedRebaseDenial, RebaseVerificationFailure, RebaseVerificationReason, RebasedSourceFacts,
};

mod executed_adoption;
pub(crate) mod adjacency;
mod predecessor_output;
mod resolution;
pub(crate) mod retirement;

mod full_preparation;
use full_preparation::FullSourceFactRebase;
mod preparation;
pub(crate) mod selected;
use preparation::Preparation;
pub struct PreparedSourceFactRebase {
    preparation: Preparation,
    performed_input_facts: Option<FactPayload>,
    retain_output_field_evidence: bool,
    predecessor: Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
}

pub(super) trait OwnEffectOnReadsUnansweredExt {
    fn with_unanswered(
        self,
        unanswered: &[WorthQueryApplicationObservedFact],
        producer_output: bool,
        effect: OwnEffect,
    ) -> Self;
}

impl OwnEffectOnReadsUnansweredExt for OwnEffectOnReads {
    fn with_unanswered(
        self,
        unanswered: &[WorthQueryApplicationObservedFact],
        producer_output: bool,
        effect: OwnEffect,
    ) -> Self {
        if producer_output
            && unanswered
                .iter()
                .inspect(|_| fact_verification_observation::rebase_unanswered())
                .any(resolution::reads_source)
        {
            self.join(effect)
        } else {
            self
        }
    }
}

/// Rebase the candidate's facts onto the snapshot its own commit selected.
/// An output entity this commit retired is read as that retirement; it is not
/// a source the effect moved.
#[allow(clippy::too_many_arguments)]
pub fn rebase_output(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    correspondence: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    changed_records: &[worth_relational::facade::transactions::RecordRef],
    coupled_commit: Option<(
        &worth_relational::facade::transactions::CommitResult,
        &coupled_deletions::CoupledCommitDeletions,
    )>,
    producer_output: bool,

    control: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
) -> RebasedSourceFacts {
    let changed_entities = changed_records
        .iter()
        .filter_map(|record| match record {
            worth_relational::facade::transactions::RecordRef::Entity(entity) => Some(*entity),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut own: BTreeSet<_> = correspondence
        .retired_entity_ids()
        .filter(|entity| changed_entities.contains(entity))
        .collect();
    let deleted;
    let retired = match coupled_commit {
        None => &own,
        // Every participant settled this same admitted atomic commit. Business
        // correspondence remains writer-local; receipt dependencies also retain
        // the genuine retirements performed by its other writers.
        Some((commit, deletions)) => {
            deleted = match deletions.collect(execution, commit) {
                Ok(deleted) => deleted,
                Err(stop) => {
                    return RebasedSourceFacts::VerificationRequired {
                        reason: RebaseVerificationFailure::admission_denied(stop),
                        own_effect: OwnEffectOnReads(OwnEffect::Moved),
                    };
                }
            };
            if own.is_subset(&deleted) {
                &*deleted
            } else {
                own.extend(deleted.iter().copied());
                &own
            }
        }
    };
    preparation::rebase(
        execution,
        runtime,
        snapshot,
        prepared,
        retired,
        producer_output,
        control,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn rebase(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    retired: &BTreeSet<EntityId>,
    producer: bool,
) -> RebasedSourceFacts {
    preparation::rebase(execution, runtime, snapshot, prepared, retired, producer, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(execution.allocation_policy(), None))
}
mod rebase_walk;

