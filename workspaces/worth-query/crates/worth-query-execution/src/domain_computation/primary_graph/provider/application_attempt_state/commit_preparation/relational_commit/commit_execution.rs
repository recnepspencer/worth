//! The single authoritative Relational commit transition.

mod consumed_output;
mod managed_unpublished;
mod managed_views;
pub(in crate::domain_computation::primary_graph::provider) use managed_unpublished::ManagedUnpublishedAttempt;
mod precommit_snapshot;
mod product_publication;
mod stops;
mod touched_records;
use stops::{failure, index_preparation_stop, native_output_witness_stop, transaction_commit_stop};
pub(in crate::domain_computation::primary_graph::provider) use touched_records::PreparedTouchedRecords;
mod publication;
pub(in crate::domain_computation::primary_graph) use publication::WorthQueryPrimaryGraphCommittedApplication;

use super::super::WorthQueryPreparedApplicationCommit;
use crate::domain_computation::primary_graph::provider::{
    mutation_work::WorthQueryPrimaryMutationWorkCounters,
    session_commit::{
        provider_failure, snapshot_admission_failure, WorthQueryPreImageRetentionWork,
    },
    WorthQueryPrimaryGraphApplicationAttempt, WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};

pub(in crate::domain_computation::primary_graph::provider) struct WorthQueryCommittedApplicationSession {
    attempt: WorthQueryPrimaryGraphApplicationAttempt,
    work: WorthQueryPrimaryMutationWorkCounters,
    index_maintenance_work: worth_relational::facade::indexes::DerivedIndexMaintenanceWork,
    retained_preimage:
        Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>,
    preimage_retention_work: WorthQueryPreImageRetentionWork,
    before: worth_relational::facade::snapshots::SnapshotHandle,
    next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    committed: std::sync::Arc<worth_relational::facade::transactions::CommitResult>,
    published_snapshot_custody: PublishedSnapshotCustody,
    prepared_touched_records: Option<PreparedTouchedRecords>,
    product_publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
    managed_views: Option<managed_views::PreparedViewPublication>,
    source_fact_rebase: Option<super::PreparedSourceFactRebase>,
    /// The request's meter from consumed-output verification through the
    /// source-fact rebase to completed registration.
    publication_admission: crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    required_prerequisites:
        Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
    prepared_lineage_slot:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
    prepared_output_witness:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness>,
}

#[derive(Clone, Copy)]
enum PublishedSnapshotCustody {
    LiveOwned,
    SettledReleased,
}

pub(super) fn commit(
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
    mint: super::WorthQueryCommitProgressionMint,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    WorthQueryCommittedApplicationSession,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let WorthQueryPreparedApplicationCommit {
        mut attempt,
        candidate,
        work,
        retained_preimage,
        preimage_retention_work,
        source_fact_rebase,
        _completion,
    } = prepared;
    let _ = mint;
    let product = attempt.affinity().product_publication().clone();
    let before = precommit_snapshot::WorthQueryPrecommitSnapshot::acquire(
        provider.graph.clone(),
        product.observation().basis().relational_basis(),
    )
    .map_err(|denial| {
        snapshot_admission_failure(
            WorthQueryProviderSessionProtocolStage::Commit,
            denial.into(),
            "application publication could not retain its exact pre-commit basis",
        )
    })
    .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::from)?;
    let consumed_output::VerifiedConsumedOutputPublication {
        mut publication_admission,
        mut required_prerequisites,
    } = consumed_output::verify(provider, &mut attempt, before.as_snapshot())?;
    let prepared_output_witness =
        crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness::prepare(
            &attempt,
            &provider.graph.layout,
            &provider.graph.source_owner.invalidation_owner,
            &mut publication_admission,
        )
        .map_err(native_output_witness_stop)?;
    let mut candidate = provider
        .graph
        .with_runtime_mut(|runtime| runtime.prepare_validated_proposal(candidate))
        .map_err(transaction_commit_stop)?;
    let prepared_touched_records = touched_records::PreparedTouchedRecords::prepare(
        &candidate,
        &mut publication_admission,
        allocation_policy,
    )?;
    let ordinary_index_budget =
        crate::domain_computation::primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget();
    #[cfg(test)]
    let ordinary_index_budget = if provider.take_tight_index_maintenance_budget() {
        worth_relational::facade::indexes::DerivedIndexMaintenanceBudget {
            maximum_work_units: 1,
            ..ordinary_index_budget
        }
    } else {
        ordinary_index_budget
    };
    let index_maintenance_work = provider
        .graph
        .with_runtime(|runtime| {
            crate::domain_computation::primary_graph::index_maintenance_budget::prepare_candidate_with_cold_fallback(
                runtime,
                &mut candidate,
                &provider.graph.primary_index_ids,
                before.as_snapshot(),
                ordinary_index_budget,
            )
        })
        .map_err(index_preparation_stop)?;
    let managed_views =
        managed_views::prepare(provider, &product, before.as_snapshot(), &candidate);
    #[cfg(feature = "test-world-operation-control")]
    provider.after_application_candidate_preparation_for_test();
    let performed = product_publication::publish(
        provider,
        &mut attempt,
        candidate,
        &mut required_prerequisites,
        &mut publication_admission,
    )?;
    let performed = match performed {
        product_publication::WorthQueryApplicationProductPublicationOutcome::Performed(
            performed,
        ) => performed,
        product_publication::WorthQueryApplicationProductPublicationOutcome::Unpublished {
            unpublished,
            reserved_terminal,
            prepared_lineage_slot,
            reservation,
        } => {
            let retained = ManagedUnpublishedAttempt {
                publication_mode: if unpublished.retains_conditional_definition() {
                    managed_unpublished::ManagedUnpublishedPublicationMode::ConditionalDefinition
                } else {
                    managed_unpublished::ManagedUnpublishedPublicationMode::Ordinary
                },
                attempt,
                work,
                index_maintenance_work,
                retained_preimage,
                preimage_retention_work,
                before,
                managed_views,
                source_fact_rebase: Some(source_fact_rebase),
                required_prerequisites,
                prepared_lineage_slot,
                prepared_output_witness,
                prepared_touched_records,
                lineage_metadata: None,
                publication_admission,
                reserved_terminal,
            };
            reservation.retain_managed(unpublished.recovery_handle(), retained);
            return Err(
                crate::domain_computation::WorthQueryProviderSessionCommitStop::ProductUnpublished(
                    unpublished,
                ),
            );
        }
    };
    let prepared_lineage_slot = performed.prepared_lineage_slot;
    let performed = performed.publication;
    let next_basis = performed
        .publication()
        .commit()
        .basis()
        .relational_basis()
        .clone();
    let committed = performed
        .publication()
        .component_results()
        .retain_relational_commit_result()
        .expect("World performed a prepared Relational application candidate");
    Ok(WorthQueryCommittedApplicationSession {
        attempt,
        work,
        index_maintenance_work,
        retained_preimage,
        preimage_retention_work,
        before: before.into_publication(),
        next_basis,
        committed,
        published_snapshot_custody: PublishedSnapshotCustody::LiveOwned,
        prepared_touched_records: Some(prepared_touched_records),
        product_publication: performed,
        managed_views,
        source_fact_rebase: Some(source_fact_rebase),
        publication_admission,
        required_prerequisites,
        prepared_lineage_slot,
        prepared_output_witness,
    })
}

fn world_no_effect(
    no_effect: worth_runtime_world::facade::NoEffectCompositePublication,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped, WorthQueryProviderSessionCommitStop as Stop,
        WorthQueryProviderSessionControlStopKind,
    };
    use worth_runtime_world::facade::NoEffectCause;
    match no_effect.cause() {
        NoEffectCause::StaleExpectedProductHead => Stop::ProductStale(
            crate::domain_computation::WorthQueryProductStaleApplication::new(no_effect),
        ),
        NoEffectCause::CancelledBeforeEffect => {
            Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(
                WorthQueryProviderSessionControlStopKind::Cancelled,
                "World publication cancelled before effect",
            ))
        }
        NoEffectCause::DeadlineBeforeEffect => {
            Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(
                WorthQueryProviderSessionControlStopKind::TimedOut,
                "World publication deadline elapsed before effect",
            ))
        }
        _ => Stop::NoEffect(no_effect),
    }
}
impl WorthQueryCommittedApplicationSession {
    /// What the source-fact rebase reads and the request meter it spends.
    pub(super) fn rebase_parts(
        &mut self,
    ) -> (
        &worth_relational::facade::transactions::CommitResult,
        usize,
        &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ){
        (
            self.committed.as_ref(),
            self.attempt.indexed_rebase_work_budget(),
            &mut self.publication_admission,
        )
    }
    pub(super) fn take_prepared_touched_records(&mut self) -> PreparedTouchedRecords {
        self.prepared_touched_records
            .take()
            .expect("pre-effect native touched-record receipt storage follows publication")
    }
    pub(super) fn take_source_fact_rebase(&mut self) -> super::PreparedSourceFactRebase {
        self.source_fact_rebase
            .take()
            .expect("pre-effect source-fact rebase capacity follows the committed candidate")
    }
    pub(super) const fn attempt(&self) -> &WorthQueryPrimaryGraphApplicationAttempt {
        &self.attempt
    }

    pub(super) const fn work(&self) -> WorthQueryPrimaryMutationWorkCounters {
        self.work
    }

    pub(super) const fn index_maintenance_work(
        &self,
    ) -> worth_relational::facade::indexes::DerivedIndexMaintenanceWork {
        self.index_maintenance_work
    }

    pub(super) const fn retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }

    pub(super) const fn preimage_retention_work(&self) -> WorthQueryPreImageRetentionWork {
        self.preimage_retention_work
    }

    pub(super) fn committed(&self) -> &worth_relational::facade::transactions::CommitResult {
        self.committed.as_ref()
    }

    pub(super) fn publish_and_encode(
        self,
        provider: &WorthQueryPrimaryGraphProvider,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        evidence: super::WorthQueryPrimaryGraphCommitEvidence,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        let published = publication::publish(provider, runtime, self, evidence)?;
        publication::encode(provider, published)
    }
}

#[cfg(test)]
mod index_preparation_tests;
