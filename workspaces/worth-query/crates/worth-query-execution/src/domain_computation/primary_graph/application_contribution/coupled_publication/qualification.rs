//! Qualified use (PR C §1.2 R2): every append marks the retained derivations
//! its certified touches could change, and compares each exactly at that
//! append. A change base would not settle supersedes the derivation at that
//! authenticated invalidator; its task reopens before any later use, owed at
//! the later of its own demand and the invalidator (R1).
use super::ledger::{Derivation, LedgerStop};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key;
use crate::domain_computation::primary_graph::provider::{ParentRead, RetainedReadRequalification};

/// The member whose append is being marked.
pub(in crate::domain_computation::primary_graph::application_contribution) enum AppendedMember {
    /// Native producer part `index` at applied `position`, owed at `owed`,
    /// deriving `meaning`.
    Part {
        index: usize,
        position: usize,
        owed: usize,
        meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    },
    /// Later authored stage `physical` at applied `position`.
    Stage { physical: usize, position: usize },
}

/// A retained parent output whose derivation must run again.
pub(in crate::domain_computation::primary_graph::application_contribution) struct Reopen {
    pub(in crate::domain_computation::primary_graph::application_contribution) read: ParentRead,
    /// The applied position and owed position of the append whose exact
    /// comparison invalidated it.
    pub(in crate::domain_computation::primary_graph::application_contribution) invalidator: usize,
    pub(in crate::domain_computation::primary_graph::application_contribution) invalidator_owed:
        usize,
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

/// Charge qualification work; it is the wave's scheduling share, observed so.
fn charge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    subject: &'static str,
    work: u64,
) -> Result<(), CoupledSourceReadStop> {
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
    synchronous_outputs::observe_scheduling_work(work);
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
    /// A reopened task's discovery evidence retained before `invalidator`
    /// ends there, once its re-discovery proved it dropped nothing.
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
        let inserted = self
            .ledger
            .borrow_mut()
            .retain(
                execution,
                Derivation::new(read, self.applied_position()),
                keys,
            )
            .map_err(|stop| ledger_stop(subject, stop))?;
        charge(execution, subject, inserted)
    }
    /// A wave task owns the parent retained as `read`: it can derive again.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn claim_retention(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: ParentRead,
    ) -> Result<(), CoupledSourceReadStop> {
        let mut ledger = self.ledger.borrow_mut();
        if ledger.is_empty() {
            return Ok(());
        }
        charge(execution, "retained parent claim", ledger.find_work())?;
        ledger.claim(read);
        Ok(())
    }
    /// Record the parent outputs a derivation consumed, for R3 (indexed).
    pub(super) fn record_consumption(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        reads: &[ParentRead],
    ) -> Result<(), CoupledSourceReadStop> {
        let subject = "consumed parent outputs";
        let mut consumed = self.consumed.borrow_mut();
        for read in reads {
            charge(execution, subject, consumed.len().max(1).ilog2() as u64 + 1)?;
            consumed
                .insert_cloned_native_for_request(&read.0, &(), *execution)
                .map_err(|cause| ledger_stop(subject, LedgerStop(cause)))?;
        }
        Ok(())
    }

    /// Mark what one actual append could have changed and compare each marked
    /// retention exactly at that append (R2): the append is its authenticated
    /// invalidator, so `window_end` and the reopened owed position are exact.
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
        let (touches, position, owed, route) = match member {
            AppendedMember::Part {
                index,
                position,
                owed,
                meaning,
            } => (
                candidate.part_append_touches(execution, index)?,
                position,
                owed,
                Some(meaning),
            ),
            AppendedMember::Stage { physical, position } => (
                candidate.authored_stage_append_touches(execution, physical)?,
                position,
                position,
                None,
            ),
        };
        charge(execution, subject, touches.touches().len() as u64 + 1)?;
        let (hits, work) = self.ledger.borrow_mut().mark(
            touches.touches().iter().map(touch_key::touch_probe),
            position,
        );
        charge(execution, subject, work)?;
        if hits.is_empty() {
            return Ok(());
        }
        let current = candidate.pin_source(execution)?;
        for id in hits {
            self.requalify(execution, id, &current, position, owed, route.as_ref())?;
        }
        Ok(())
    }

    /// Compare one marked retention exactly at the current image. It stays
    /// live (checked in full by final support) when no wave task owns it or
    /// a derivation already consumed it (R3); base settlement keeps it too.
    fn requalify(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: usize,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        invalidator: usize,
        invalidator_owed: usize,
        route: Option<&worth_relational::facade::mvcc::RequiredOutputPublicationMeaning>,
    ) -> Result<(), CoupledSourceReadStop> {
        let (read, position, claimed) = {
            let ledger = self.ledger.borrow();
            let entry = ledger.entry(id);
            (entry.read, entry.position, entry.claimed)
        };
        let consumed = {
            let consumed = self.consumed.borrow();
            charge(
                execution,
                "consumed parent outputs",
                consumed.len().max(1).ilog2() as u64 + 1,
            )?;
            consumed.contains_key(&read.0)
        };
        if !claimed || consumed {
            return Ok(());
        }
        // An authored reader settles a change only by the producer of the same
        // route, owed at or after it (`settles_read_support`).
        let authored = !self.all_stage_captures().is_empty();
        let settles = |parent: &crate::domain_computation::primary_graph::application_contribution::producer::VerifiedAppliedParent| {
            authored
                && invalidator_owed >= position
                && route.is_some_and(|meaning| parent.has_route(meaning))
        };
        let answer = self
            .application
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
            .map_err(CoupledSourceReadStop::capture)?;
        if answer == RetainedReadRequalification::Superseded {
            self.ledger.borrow_mut().supersede(id);
            self.reopens.borrow_mut().push(Reopen {
                read,
                invalidator,
                invalidator_owed,
            });
        }
        Ok(())
    }

    /// The retained parents whose exact comparison superseded them since the
    /// last call; their tasks derive again before any later use.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn qualify(
        &self,
    ) -> Vec<Reopen> {
        std::mem::take(&mut *self.reopens.borrow_mut())
    }
}
