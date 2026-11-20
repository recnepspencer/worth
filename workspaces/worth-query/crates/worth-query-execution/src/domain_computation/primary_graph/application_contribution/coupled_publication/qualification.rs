//! Qualified use (PR C §1.2 R2): every append marks the retained derivations
//! its certified touches could change; before the next use each suspect is
//! compared exactly. A change base would not settle reopens the derivation,
//! owed at the later of its own demand and its invalidator (R1).
use super::ledger::{Derivation, DerivationState, LedgerStop};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key;
use crate::domain_computation::primary_graph::provider::{ParentRead, RetainedReadRequalification};

/// The member whose append is being marked.
pub(in crate::domain_computation::primary_graph::application_contribution) enum AppendedMember {
    /// Native producer part `index` at applied `position`, owed at `owed`.
    Part {
        index: usize,
        position: usize,
        owed: usize,
    },
    /// Later authored stage `physical` at applied `position`.
    Stage { physical: usize, position: usize },
}

/// A retained parent output whose derivation must run again.
pub(in crate::domain_computation::primary_graph::application_contribution) struct Reopen {
    pub(in crate::domain_computation::primary_graph::application_contribution) read: ParentRead,
    pub(in crate::domain_computation::primary_graph::application_contribution) owed: usize,
    /// The applied position of the append that invalidated it.
    pub(in crate::domain_computation::primary_graph::application_contribution) invalidator: usize,
}

fn ledger_stop(subject: &'static str, LedgerStop(cause): LedgerStop) -> CoupledSourceReadStop {
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
    use worth_execution::ExecutionOrderedMapDenial as Map;
    CoupledSourceReadStop::capture(match cause {
        Map::Allocation(cause) => Attempt::allocation_denied(subject, cause),
        Map::Request(cause) => Attempt::relational_read_stopped(cause.into(), subject),
        Map::CapacityOverflow => Attempt::new(
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
            subject,
        ),
    })
}

fn charge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    subject: &'static str,
    work: u64,
) -> Result<(), CoupledSourceReadStop> {
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
    let denied = |cause: worth_execution::WorkCeilingDenial| {
        CoupledSourceReadStop::capture(Attempt::relational_read_stopped(cause.into(), subject))
    };
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
        })
        .map_err(denied)?
        .0
        .map_err(|cause| denied(worth_execution::WorkCeilingDenial::Stopped(cause)))
}

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// Whether the route read next derives again after a stale retention.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn supersede(
        &self,
        superseding: bool,
    ) {
        self.superseding.set(superseding);
    }
    /// The wave task whose discovery retains the support retained next.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn discover_for(
        &self,
        task: Option<usize>,
    ) {
        self.discovering.set(task);
    }
    /// A reopened task's own discovery evidence ends at its invalidator.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn supersede_discovery(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        task: usize,
        invalidator: usize,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.application
            .runtime
            .primary_provider
            .supersede_owned_candidate_support(execution, &self.root_session, task, invalidator)
            .map_err(execution::denial::candidate_support_denied)
    }
    /// Register a Verified parent the assembly retained, keyed by its proofs.
    pub(super) fn register_retention(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        parent: &crate::domain_computation::primary_graph::application_contribution::producer::VerifiedAppliedParent,
        read: ParentRead,
    ) -> Result<(), CoupledSourceReadStop> {
        let subject = "retained parent qualification";
        let keys = parent.facts().map(touch_key::fact_keys).collect::<Vec<_>>();
        charge(execution, subject, keys.len() as u64 + 1)?;
        self.ledger
            .borrow_mut()
            .retain(
                execution,
                Derivation {
                    read,
                    position: self.applied_position(),
                    owed: self.owing().position(),
                    state: DerivationState::Live,
                },
                keys,
            )
            .map_err(|stop| ledger_stop(subject, stop))
    }

    /// Mark what one actual append could have changed (R2, append time).
    pub(super) fn mark_append(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        member: AppendedMember,
    ) -> Result<(), CoupledSourceReadStop> {
        let subject = "append touch marking";
        if self.ledger.borrow().is_empty() {
            return Ok(());
        }
        let (touches, position, owed) = match member {
            AppendedMember::Part {
                index,
                position,
                owed,
            } => (
                candidate.part_append_touches(execution, index)?,
                position,
                owed,
            ),
            AppendedMember::Stage { physical, position } => (
                candidate.authored_stage_append_touches(execution, physical)?,
                position,
                position,
            ),
        };
        charge(execution, subject, touches.touches().len() as u64 + 1)?;
        let hits = self
            .ledger
            .borrow_mut()
            .mark(
                execution,
                touches.touches().iter().map(touch_key::touch_probe),
                position,
                owed,
            )
            .map_err(|stop| ledger_stop(subject, stop))?;
        charge(execution, subject, hits)
    }

    /// Compare each suspect exactly at the current candidate. Returns the
    /// retained parents to reopen; each is superseded at its invalidator.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn qualify(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Vec<Reopen>, CoupledSourceReadStop> {
        if self.ledger.borrow().is_quiet() {
            return Ok(Vec::new());
        }
        let subject = "retained parent qualification";
        let suspects = self.ledger.borrow_mut().take_suspects();
        charge(execution, subject, suspects.len() as u64)?;
        let current = candidate.pin_source(execution)?;
        // An authored stage's read settles only where the producer of the same
        // route, owed at or after it, derived it (`settles_read_support`).
        let authored = !self.all_stage_captures().is_empty();
        let mut reopens = Vec::new();
        for id in suspects {
            reopens.extend(self.requalify(execution, id, &current, authored)?);
        }
        Ok(reopens)
    }

    /// Requalify one suspect: unchanged, settled by base, kept for a consumer
    /// that already ran, or superseded and returned for reopen.
    fn requalify(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: usize,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        authored: bool,
    ) -> Result<Option<Reopen>, CoupledSourceReadStop> {
        let (read, position, owed, invalidator, invalidator_owed) = {
            let ledger = self.ledger.borrow();
            let entry = ledger.entry(id);
            let DerivationState::Suspect { invalidator, owed } = entry.state else {
                return Ok(None);
            };
            (entry.read, entry.position, entry.owed, invalidator, owed)
        };
        // A derivation that already consumed this output keeps it until its
        // own settlement or reopen can follow (R3): the old proof stays live,
        // so final support refuses exactly as base does.
        let consumed = self
            .writers
            .borrow()
            .iter()
            .any(|writer| writer.consumes.contains(&read));
        let writer = self
            .writers
            .borrow()
            .iter()
            .find(|writer| writer.position == invalidator)
            .map(|writer| (writer.owed.position(), writer.meaning.clone()));
        let settles = |parent: &crate::domain_computation::primary_graph::application_contribution::producer::VerifiedAppliedParent| {
            authored
                && writer
                    .as_ref()
                    .is_some_and(|(owed, meaning)| *owed >= position && parent.has_route(meaning))
        };
        let answer = if consumed {
            RetainedReadRequalification::Settled
        } else {
            self.application
                .runtime
                .primary_provider
                .requalify_retained_parent(
                    execution,
                    &self.root_session,
                    self.before,
                    read,
                    current,
                    invalidator,
                    settles,
                )
                .map_err(CoupledSourceReadStop::capture)?
        };
        if answer != RetainedReadRequalification::Superseded {
            self.ledger.borrow_mut().settle(id);
            return Ok(None);
        }
        self.ledger.borrow_mut().supersede(id);
        Ok(Some(Reopen {
            read,
            owed: owed.max(invalidator_owed),
            invalidator,
        }))
    }
}
