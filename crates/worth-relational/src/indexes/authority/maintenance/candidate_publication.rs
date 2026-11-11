//! A fully funded, single-use batch bound to its actual prepared canonical root.
use super::super::prepared_generation;
use super::{publication::PreparedRefresh, work::MaintenanceWork};
use crate::branch::RelationalBranchRoot;
use crate::history::data::BranchId;
use crate::indexes::data::{DerivedIndexGeneration, DerivedIndexMaintenanceDenialKind as Denial};
use std::sync::Arc;
/// Fully derived payload owned by one single-use Relational candidate. Only
/// catalog installation remains after its canonical branch movement.
pub(crate) struct PreparedCandidateIndexPublication {
    pub(super) prepared: CandidateIndexPreparation,
    pub(super) funding: worth_execution::ExecutionMemoryReservation,
    pub(super) metadata: Option<worth_execution::ExecutionMemoryReservation>,
    pub(super) work: crate::indexes::data::DerivedIndexMaintenanceWork,
}

pub(super) struct CandidateIndexPreparation {
    root_id: u64,
    branch_id: BranchId,
    commit_id: crate::history::data::CommitId,
    version_id: crate::identity::data::VersionId,
    schema_version: crate::schema::data::SchemaVersionId,
    publication: crate::runtime::PreparedGenerationBatch,
}

impl PreparedCandidateIndexPublication {
    pub(crate) fn work(&self) -> crate::indexes::data::DerivedIndexMaintenanceWork {
        self.work
    }

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

    pub(crate) fn install_admitted(&mut self) {
        self.prepared.publication.install_admitted();
    }

    pub(crate) fn publish(mut self, _runtime: &crate::runtime::RelationalRuntime) {
        self.prepared.publication.settle_artifacts();
        // Native payload was installed at cutover. Retire the retained backing
        // and keys before releasing either original-request reservation.
        let Self {
            prepared,
            funding,
            metadata,
            work: _,
        } = self;
        drop(prepared);
        drop(metadata);
        drop(funding);
    }
}

impl CandidateIndexPreparation {
    pub(super) fn selection_state(&self) -> Option<Arc<crate::runtime::IndexSelectionState>> {
        self.publication.selection_state()
    }

    pub(super) fn prepare(
        indexes: &crate::runtime::IndexingSubsystem,
        prepared_root: &Arc<RelationalBranchRoot>,
        mut prepared: PreparedRefresh,
        work: &mut MaintenanceWork,
    ) -> Result<Self, Denial> {
        work.charge(prepared.entries.len())?;
        work.reserve_generation_publications(prepared.entries.len());
        work.array::<DerivedIndexGeneration>(prepared.entries.len())?;
        work.array::<crate::indexes::data::DerivedIndexGenerationId>(prepared.entries.len())?;
        work.prepare(
            0,
            (prepared.publication.branch_id.0.len() * 2 * prepared.entries.len()) as u64,
        )?;
        let generation_ids = indexes
            .reserve_generation_ids(prepared.entries.len())
            .ok_or(Denial::GenerationIdentityExhausted)?;
        let generations: Vec<DerivedIndexGeneration> = prepared
            .entries
            .into_iter()
            .zip(generation_ids)
            .map(|((index_id, entries), generation_id)| {
                prepared_generation(&prepared.publication, generation_id, index_id, entries)
            })
            .collect();
        if let Some(basis) = &mut prepared.basis {
            for generation in &generations {
                let width = generation.source_branch_id.0.len()
                    + generation.applicability.branch_id.0.len();
                work.prepare(
                    width as u64 + 1,
                    (width
                        + std::mem::size_of::<DerivedIndexGeneration>()
                        + 2 * std::mem::size_of::<usize>()) as u64,
                )?;
                basis.replace_admitted(Arc::new(generation.clone()), &mut |units, bytes| {
                    work.prepare(units, bytes)
                })?;
            }
        }
        let publication = indexes
            .prepare_generation_batch(generations, prepared.basis, |units, bytes| {
                work.prepare(units, bytes)
            })
            .map_err(|stop| match stop {
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                crate::indexes::data::SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                    Denial::WorkBudgetExceeded
                }
            })?;
        Ok(CandidateIndexPreparation {
            root_id: prepared_root.id(),
            branch_id: prepared_root
                .canonical_envelope()
                .ok_or(Denial::CommitMismatch)?
                .branch_context
                .clone(),
            commit_id: prepared_root.commit_id().ok_or(Denial::CommitMismatch)?,
            version_id: prepared_root
                .axes()
                .map(|axes| crate::identity::data::VersionId(axes.storage_version))
                .ok_or(Denial::CommitMismatch)?,
            schema_version: prepared_root.schema_authority().schema_version(),
            publication,
        })
    }
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
