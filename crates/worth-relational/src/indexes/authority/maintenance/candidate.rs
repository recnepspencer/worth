use std::sync::Arc;

use super::{basis, work::MaintenanceWork};
use crate::branch::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexGeneration, DerivedIndexId,
    DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenial,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexMaintenanceWork,
};
use crate::mvcc::PreparedRelationalCommitCandidate;
use crate::runtime::VisibilityProjectionView;
use crate::snapshots::data::SnapshotHandle;
use crate::visibility::snapshot_states::{SnapshotStateBasis, VisibilitySnapshotBasis};

use super::super::{prepared_generation, IndexAuthority};

/// Fully derived payload owned by one single-use Relational candidate. Only
/// catalog installation remains after its canonical branch movement.
pub(crate) struct PreparedCandidateIndexPublication {
    prepared: CandidateIndexPreparation,
    funding: worth_execution::ExecutionMemoryReservation,
    metadata: Option<worth_execution::ExecutionMemoryReservation>,
}

struct CandidateIndexPreparation {
    root_id: u64,
    branch_id: BranchId,
    commit_id: crate::history::data::CommitId,
    version_id: crate::identity::data::VersionId,
    schema_version: crate::schema::data::SchemaVersionId,
    publication: crate::runtime::PreparedGenerationBatch,
}

impl PreparedCandidateIndexPublication {
    pub(crate) fn matches_root(&self, root: &Arc<RelationalBranchRoot>) -> bool {
        root.id() == self.prepared.root_id
            && root.commit_id() == Some(self.prepared.commit_id)
            && root.canonical_envelope().is_some_and(|envelope| {
                envelope.branch_context == self.prepared.branch_id
                    && envelope.commit.version_id == self.prepared.version_id
            })
            && root.schema_authority().schema_version() == self.prepared.schema_version
    }

    pub(crate) fn covers_home_definitions(
        &self,
        definitions: &crate::runtime::PreparedRelationalIndexDefinitions,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, worth_execution::WorkCeilingDenial> {
        self.prepared
            .publication
            .covers_home_definitions(definitions, execution)
    }

    pub(crate) fn admit_catalog(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        expires_at: std::time::Instant,
        maximum_lifetime_millis: u64,
    ) -> Result<(), crate::mvcc::RelationalPublicationOutcome> {
        use crate::indexes::data::SelectedIndexGenerationAdmissionStop as Stop;
        use crate::mvcc::{RelationalPublicationDeferred, RelationalPublicationOutcome as Outcome};
        let mut metadata = execution
            .reserve_retained_memory(0)
            .map_err(Outcome::RequestStopped)?;
        let mut total = 0u64;
        let admission = self
            .prepared
            .publication
            .admit_actual_geometry(|units, bytes| {
                execution.consult().map_err(Outcome::RequestStopped)?;
                if std::time::Instant::now() >= expires_at {
                    return Err(Outcome::deferred(
                        RelationalPublicationDeferred::CandidateLifetimeExpired {
                            maximum_lifetime_millis,
                        },
                    ));
                }
                pay_catalog_work(execution, units).map_err(Outcome::RequestStopped)?;
                total = total.checked_add(bytes).ok_or(Outcome::RequestStopped(
                    worth_execution::WorkCeilingDenial::Admission(
                        worth_execution::LeaseDenial::ChargedBytesOverflow,
                    ),
                ))?;
                if let Some(memory) = metadata.as_mut() {
                    memory.resize(total).map_err(|denial| {
                        Outcome::RequestStopped(worth_execution::WorkCeilingDenial::Admission(
                            worth_execution::LeaseDenial::MemoryExhausted(denial),
                        ))
                    })?;
                }
                Ok(())
            });
        match admission {
            Ok(()) => {
                self.metadata = metadata;
                Ok(())
            }
            Err(Stop::Admission(outcome)) => Err(outcome),
            Err(Stop::AccountingOverflow) => Err(Outcome::RequestStopped(
                worth_execution::WorkCeilingDenial::Admission(
                    worth_execution::LeaseDenial::ChargedBytesOverflow,
                ),
            )),
        }
    }

    pub(crate) fn publish_admitted(&mut self) {
        self.prepared.publication.publish_admitted();
    }

    pub(crate) fn retire(self) {
        let Self {
            prepared,
            funding,
            metadata,
        } = self;
        assert!(
            prepared.publication.is_published(),
            "settlement retires the publication already installed by its cutover"
        );
        // Payload, empty Vec backing and lookup keys all die before their loans.
        drop(prepared);
        drop(metadata);
        drop(funding);
    }
}

impl IndexAuthority<'_> {
    /// Budget and derive exact native index deltas from a prepared candidate
    /// before World can perform its publication. No generation is made visible
    /// until its admitted Native cutover installs them. Settlement retires the payload.
    pub fn prepare_for_candidate(
        &self,
        candidate: &mut PreparedRelationalCommitCandidate,
        index_ids: &[DerivedIndexId],
        before: Option<&SnapshotHandle>,
        budget: DerivedIndexMaintenanceBudget,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        let (prepared, counts) = super::request_payment::run(execution, budget, |work| {
            self.prepare_candidate_indexes(candidate, index_ids, before, work, execution)
        })?;
        let (prepared, funding) = prepared.into_funded_parts();
        let publication = PreparedCandidateIndexPublication {
            prepared,
            funding,
            metadata: None,
        };
        candidate
            .attach_prepared_indexes(publication)
            .map(|()| counts)
            .map_err(|kind| DerivedIndexMaintenanceDenial { kind, work: counts })
    }

    /// Prepares selected generations from the authentic candidate predecessor.
    /// The original request funds all work and memory; missing predecessors refuse
    /// rather than entering population reconstruction. Publication stays candidate-owned.
    pub fn prepare_incremental_indexes_for_candidate(
        &self,
        candidate: &mut PreparedRelationalCommitCandidate,
        index_ids: &[DerivedIndexId],
        before: &SnapshotHandle,
        budget: DerivedIndexMaintenanceBudget,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        let (prepared, counts) = super::request_payment::run(execution, budget, |work| {
            work.restrict_to_incremental_indexes();
            self.prepare_candidate_indexes(candidate, index_ids, Some(before), work, execution)
        })?;
        let (prepared, funding) = prepared.into_funded_parts();
        let publication = PreparedCandidateIndexPublication {
            prepared,
            funding,
            metadata: None,
        };
        candidate
            .attach_prepared_indexes(publication)
            .map(|()| counts)
            .map_err(|kind| DerivedIndexMaintenanceDenial { kind, work: counts })
    }

    fn prepare_candidate_indexes(
        &self,
        candidate: &mut PreparedRelationalCommitCandidate,
        index_ids: &[DerivedIndexId],
        before: Option<&SnapshotHandle>,
        work: &mut MaintenanceWork,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<CandidateIndexPreparation, Denial> {
        if candidate.runtime_instance_id() != self.runtime.runtime_instance_id() {
            return Err(Denial::ForeignCandidate);
        }
        if candidate.lifetime_expired() {
            return Err(Denial::CandidateLifetimeExpired {
                maximum_lifetime_millis: candidate.maximum_lifetime_millis,
            });
        }
        if candidate.indexes_already_prepared() {
            return Err(Denial::CandidateIndexesAlreadyPrepared);
        }
        work.charge(index_ids.len())?;
        let (expected_root, prepared_root) = candidate
            .index_preparation_roots()
            .ok_or(Denial::CandidateUnavailable)?;
        work.prepare(0, (candidate.branch().0.len() * 3) as u64)?;
        work.array::<DerivedIndexId>(index_ids.len())?;
        let branch_id = candidate.branch().clone();
        let envelope = prepared_root
            .canonical_envelope()
            .ok_or(Denial::CommitMismatch)?;
        if envelope.branch_context != branch_id {
            return Err(Denial::CommitMismatch);
        }
        let request = DerivedIndexBuildRequest {
            source_commit_id: envelope.commit.commit_id,
            branch_id: branch_id.clone(),
            index_ids: index_ids.to_vec(),
        };
        // The prepared root may contain unpublished text. Its authenticated,
        // already-built dictionary stays candidate-owned until acceptance.
        let symbols = candidate
            .index_preparation_symbols()
            .ok_or(Denial::CandidateUnavailable)?;
        let after = prepared_projection(
            self.runtime,
            branch_id.clone(),
            Arc::clone(&prepared_root),
            symbols,
        );
        let before = before
            .map(|snapshot| {
                if snapshot.branch_id() != &request.branch_id {
                    return Err(Denial::BeforeRootMismatch);
                }
                self.runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .ok_or(Denial::SnapshotUnavailable)
            })
            .transpose()?;
        validate_expected_root(&expected_root, before.as_ref())?;
        basis::validate_prepared(&request, before.as_ref(), &after)?;
        let definitions = candidate.home_index_definitions();
        let prepared = self.prepare_refresh(
            execution,
            &request,
            before.as_ref(),
            definitions.as_deref(),
            &after,
            work,
        )?;
        work.charge(prepared.entries.len())?;
        work.reserve_generation_publications(prepared.entries.len());
        work.array::<DerivedIndexGeneration>(prepared.entries.len())?;
        work.array::<crate::indexes::data::DerivedIndexGenerationId>(prepared.entries.len())?;
        work.prepare(
            0,
            (prepared.publication.branch_id.0.len() * 2 * prepared.entries.len()) as u64,
        )?;
        let generation_ids = self
            .runtime
            .indexes
            .reserve_generation_ids(prepared.entries.len())
            .ok_or(Denial::GenerationIdentityExhausted)?;
        let generations = prepared
            .entries
            .into_iter()
            .zip(generation_ids)
            .map(|((index_id, entries), generation_id)| {
                prepared_generation(&prepared.publication, generation_id, index_id, entries)
            })
            .collect();
        let publication = self
            .runtime
            .indexes
            .prepare_generation_batch(generations, |units, bytes| work.prepare(units, bytes))
            .map_err(|stop| match stop {
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                    Denial::WorkBudgetExceeded
                }
            })?;
        Ok(CandidateIndexPreparation {
            root_id: prepared_root.id(),
            branch_id,
            commit_id: request.source_commit_id,
            version_id: after.version_id(),
            schema_version: prepared_root.schema_authority().schema_version(),
            publication,
        })
    }
}

fn prepared_projection(
    runtime: &crate::runtime::RelationalRuntime,
    branch_id: BranchId,
    root: Arc<RelationalBranchRoot>,
    symbols: crate::runtime::RuntimeSymbolTable,
) -> VisibilityProjectionView {
    VisibilityProjectionView::with_symbols(
        runtime,
        SnapshotStateBasis::Exact(VisibilitySnapshotBasis::from_prepared_root(branch_id, root)),
        symbols,
    )
}

fn validate_expected_root(
    expected: &Arc<RelationalBranchRoot>,
    before: Option<&VisibilityProjectionView>,
) -> Result<(), Denial> {
    if let Some(before) = before {
        let selected = before.selected_root().ok_or(Denial::BeforeRootMismatch)?;
        if selected.id() != expected.id() {
            return Err(Denial::BeforeRootMismatch);
        }
    }
    Ok(())
}

// This native metadata preparation shares the original request and preserves
// its actual first stop instead of replacing it with a nested-stop wrapper.
fn pay_catalog_work(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    units: u64,
) -> Result<(), worth_execution::WorkCeilingDenial> {
    let mut answer = None;
    let scope = execution.run_native_with_checkpoint(|context| {
        answer = Some(crate::execution::request_stage::checkpoint(context, units));
    });
    match (scope, answer) {
        (_, Some(Err(stop))) => Err(worth_execution::WorkCeilingDenial::Stopped(stop)),
        (Err(stop), _) => Err(stop),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed metadata admission executes its checkpoint"),
    }
}
