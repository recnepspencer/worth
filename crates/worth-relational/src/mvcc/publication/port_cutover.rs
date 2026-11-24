use std::sync::Arc;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::runtime::operation_control::RelationalOperationControlOwnerPort as _;
use worth_relational_publication::internal_port::mvcc::publication::outcome::{
    PerformedRelationalCommitOwnerPort as _, RelationalPublicationOutcomeOwnerPort as _,
};

use super::super::movement::cutover_deadline_elapsed;

use super::{
    PerformedRelationalCommit, PreparedBranchPublicationPreflight, RelationalPublicationOutcome,
};

impl PreparedBranchPublicationPreflight {
    pub(super) fn perform_cutover(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        deadline: Option<std::time::Instant>,
        allow_effect: Option<&mut dyn FnMut() -> bool>,
    ) -> Option<RelationalPublicationOutcome> {
        let _publication_lifecycle = self
            .movement
            .canonical_publication_route
            .enter_publication();
        if let Some(deadline) = deadline {
            if let Err(stop) = crate::authority::commit::pipeline::consult_stage(
                execution,
                crate::authority::commit::pipeline::CommitStage::PublicationCutover,
            ) {
                return Some(RelationalPublicationOutcome::RequestStopped(stop));
            }
            if std::time::Instant::now() >= deadline {
                return Some(cutover_deadline_elapsed());
            }
        }
        if let Some(outcome) = self.before_critical_section_stop() {
            return Some(outcome);
        }
        let _critical_section =
            crate::mvcc::publication::current_read::PublicationReadCoordination::enter(
                &self.publication_cell,
                self.current_read.as_ref(),
            );
        if let Some(outcome) = self.before_critical_section_stop() {
            return Some(outcome);
        }
        self.control.pause_inside_critical_section_for_test();
        let (positioned_commit, head_retirement) =
            match (super::super::movement::BranchMovementCutover {
                coordination: &_critical_section,
                publication_cell: &self.publication_cell,
                expected: &self.expected,
                next_state: &mut self.next_state,
                head_retirement: &mut self.head_retirement,
                companion: self.companion.as_mut(),
                companion_binding: self.companion_binding.as_ref(),
                medium: self.medium.as_deref(),
                branch_head_versions: &self.branch_head_versions,
            })
            .compare_and_move(
                &self.movement.canonical_publication_route,
                &self.movement.root,
                &mut self.movement.symbol_meanings,
                execution,
                deadline,
                allow_effect,
                || {
                    self.control
                        .pause_holding_patch_position_reservation_for_test()
                },
                self.settlement_reservation
                    .as_mut()
                    .expect("the same cutover owns its reserved completion"),
            ) {
                Ok(performed) => performed,
                Err(outcome) => return outcome,
            };
        let _critical_section = _critical_section.into_target_guard();
        self.control.pause_after_linearization_for_test();
        drop(_critical_section);
        let settlement_retention = head_retirement.replace_head_for_publication(
            self.candidate_retention
                .take()
                .expect("the consuming cutover owns candidate retention"),
            Arc::clone(&self.movement.root),
        );
        drop(_publication_lifecycle);
        self.movement
            .record_allocations
            .take()
            .expect("the consuming cutover owns prepared record allocations")
            .commit();
        let late_interruption = self
            .control
            .observe(crate::runtime::RelationalInterruptionBoundary::AfterLinearization);
        if let Some(interruption) = late_interruption {
            self.next_basis
                .owner_inner()
                .retention_binding
                .record_interruption(interruption);
        }
        // Authorizing the already-installed record is what disarms pre-effect
        // cleanup. From here the commit is recoverable by identity alone, and
        // the returned witness is only a witness.
        let record = self
            .settlement_reservation
            .take()
            .expect("one actual cutover authorizes its original settlement reservation")
            .authorize(
                Arc::clone(&positioned_commit),
                settlement_retention,
                late_interruption,
            );
        Some(RelationalPublicationOutcome::performed(
            PerformedRelationalCommit::record(
                positioned_commit,
                self.next_basis.clone(),
                record,
                late_interruption,
            ),
        ))
    }

    fn before_critical_section_stop(&self) -> Option<RelationalPublicationOutcome> {
        let event = self
            .control
            .observe(crate::runtime::RelationalInterruptionBoundary::BeforeCriticalSection)?;
        self.next_basis
            .owner_inner()
            .retention_binding
            .record_interruption(event);
        Some(RelationalPublicationOutcome::interrupted(event))
    }
}
