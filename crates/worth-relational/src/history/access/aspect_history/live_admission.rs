//! Admit source subjects, selections, and cursors through the installed runtime.
use super::cursor::RetainedCanonicalRecordHistory;
use super::selection::require_exact_subject;
use super::RelationalRecordHistoryDenial as Denial;
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::change_source::RelationalSelectedCommit;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::RecordRef;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectKey;
use worth_relational_branch::internal_port::history::access::aspect_history::retained_selection::RelationalRecordHistorySelectionOwnerPort as _;
use worth_relational_branch::internal_port::history::access::aspect_history::selection::RelationalRecordHistorySubjectOwnerPort as _;
use worth_relational_branch::internal_port::mvcc::observation::RelationalBranchObservationOwnerPort as _;

impl RelationalRuntime {
    /// A subject may come from an earlier retained root. The selected commit
    /// must independently be exact and owner-issued; neither proves World ancestry.
    pub fn record_aspect_history_cursor(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        selected: RelationalSelectedCommit,
        subjects: &[RelationalRecordHistorySubject],
        aspects: &[AspectKey],
    ) -> Result<RelationalCanonicalRecordHistoryCursor, Denial> {
        use std::num::NonZeroUsize;
        let selection = self.retain_record_history_selection(
            execution,
            subjects,
            aspects,
            RelationalRecordHistorySelectionLimits {
                maximum_subjects: NonZeroUsize::new(subjects.len())
                    .ok_or(Denial::EmptySubjectSelection)?,
                maximum_aspects: NonZeroUsize::new(aspects.len())
                    .ok_or(Denial::EmptyAspectSelection)?,
                maximum_owned_bytes: NonZeroUsize::new(usize::MAX).expect("positive storage bound"),
            },
        )?;
        self.record_aspect_history_cursor_from_selection(execution, selected, &selection)
    }

    /// Open another exact canonical commit while sharing the already funded
    /// sealed selection. Each returned cursor retains its own source identity.
    pub fn record_aspect_history_cursor_from_selection(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        selected: RelationalSelectedCommit,
        selection: &RelationalRecordHistorySelection,
    ) -> Result<RelationalCanonicalRecordHistoryCursor, Denial> {
        if selected.runtime_instance_id() != self.runtime_instance_id()
            || selection.runtime() != self.runtime_instance_id()
        {
            return Err(Denial::ForeignOwner);
        }
        if selected.observation().commit_id() != Some(selected.commit_id()) {
            return Err(Denial::NotExactSelectedCommit);
        }
        super::request_stage::run(execution, |checkpoint| {
            checkpoint.poll()?;
            let memory = execution
                .reserve_retained_memory(
                    RelationalCanonicalRecordHistoryCursor::owned_source_backing_bytes() as u64,
                )
                .map_err(Denial::Request)?;
            let envelope = match self.history.canonical_envelope(selected.commit_id()) {
                Some(envelope) => envelope,
                None => self
                    .history
                    .commit_artifact(selected.commit_id())
                    .ok_or(Denial::CommitUnavailable)?
                    .demand_envelope(
                        &execution,
                        crate::storage::residency::history_capture_bound::FULL_HISTORY_READ_LIMIT,
                    )
                    .map_err(|cause| Denial::Storage(cause.into()))?,
            };
            let retained = self
                .retain_observation_snapshot(&selected.observation().admitted_basis())
                .map_err(Denial::Retention)?;
            Ok(RelationalCanonicalRecordHistoryCursor {
                source: Arc::new(RetainedCanonicalRecordHistory {
                    envelope,
                    retained,
                    selection: selection.clone(),
                    _memory: memory,
                }),
                next_record: 0,
            })
        })
    }
}

impl RelationalRuntime {
    /// Limits precede walking, copying, and canonicalizing the input.
    pub fn retain_record_history_selection(
        &self,
        execution: ExecutionRequest<'_, '_>,
        subjects: &[RelationalRecordHistorySubject],
        aspects: &[AspectKey],
        limits: RelationalRecordHistorySelectionLimits,
    ) -> Result<RelationalRecordHistorySelection, Denial> {
        super::retained_selection::retain_record_history_selection(
            self.runtime_instance_id(),
            execution,
            subjects,
            aspects,
            limits,
        )
    }
}

impl RelationalRuntime {
    /// Resolve only the exact generation in this immutable root. An identifier
    /// alone cannot recreate the returned retained subject after reclamation/reuse.
    pub fn retain_record_history_subject(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        record: RecordRef,
    ) -> Result<RelationalRecordHistorySubject, Denial> {
        execution.check_live()?;
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(Denial::ForeignOwner);
        }
        require_exact_subject(&execution, &self.services.symbols, basis, &record)?;
        let memory = execution
            .reserve_retained_memory(RelationalRecordHistorySubject::owned_backing_bytes() as u64)
            .map_err(Denial::Request)?;
        let retained = self
            .retain_observation_snapshot(basis)
            .map_err(Denial::Retention)?;
        execution.check_live()?;
        Ok(RelationalRecordHistorySubject::from_retained(
            record, retained, memory,
        ))
    }
}
