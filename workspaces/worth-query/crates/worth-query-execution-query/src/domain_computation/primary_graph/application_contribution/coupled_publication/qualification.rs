//! Qualified use (PR C §1.2 R2): every append marks the retained derivations
//! its certified touches could change, and compares each exactly at that
//! append. A change base would not settle supersedes the derivation at that
//! authenticated invalidator; its task reopens before any later use, owed at
//! the later of its own demand and the invalidator (R1).
use super::ledger::{Derivation, LedgerStop};
use super::*;
mod selection;
mod work;
pub(in crate::domain_computation::primary_graph::application_contribution) use selection::ReopenTarget;
use work::{charge, ledger_stop};
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key;
use crate::domain_computation::primary_graph::provider::{ParentRead, RetainedReadRequalification};

/// The member whose append is being marked.
pub(in crate::domain_computation::primary_graph::application_contribution) enum AppendedMember {
    /// Native producer part `index` at applied `position`, owed at `owed`,
    /// deriving `meaning`; `continues` when it is an owned route that can
    /// derive its chain's next round.
    Part {
        index: usize,
        position: usize,
        owed: usize,
        meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
        continues: bool,
    },
    /// Later authored stage `physical` at applied `position`.
    Stage { physical: usize, position: usize },
}

/// A derivation qualified use reopens: a retained parent output, or a
/// selection (#238).
pub(in crate::domain_computation::primary_graph::application_contribution) struct Reopen {
    pub(in crate::domain_computation::primary_graph::application_contribution) target: ReopenTarget,
    /// The applied position and owed position of the append whose exact
    /// comparison invalidated it.
    pub(in crate::domain_computation::primary_graph::application_contribution) invalidator: usize,
    pub(in crate::domain_computation::primary_graph::application_contribution) invalidator_owed:
        usize,
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
        self.discovering.set(task.map(|task| (task, false)));
    }
    /// The wave task whose incoming prerequisite selection retains next.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn discover_incoming_for(
        &self,
        task: usize,
    ) {
        self.discovering.set(Some((task, true)));
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
    /// A wave task owns the derivation of `read`: it can derive again.
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
    /// Record the parent outputs the derivation `consumer` consumed (R3).
    pub(super) fn record_consumption(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        reads: &[ParentRead],
        consumer: ParentRead,
    ) -> Result<(), CoupledSourceReadStop> {
        let subject = "consumed parent outputs";
        let mut consumed = self.consumed.borrow_mut();
        for read in reads {
            charge(execution, subject, consumed.len().max(1).ilog2() as u64 + 1)?;
            consumed
                .insert_cloned_native_for_request(&(read.0, consumer.0), &(), *execution)
                .map_err(|cause| ledger_stop(subject, LedgerStop(cause)))?;
        }
        Ok(())
    }

    /// Register producer part `index`, deriving `read`, keyed by its facts.
    pub(super) fn register_part(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index: usize,
        read: ParentRead,
    ) -> Result<(), CoupledSourceReadStop> {
        let subject = "producer part qualification";
        let keys = {
            let writers = self.writers.borrow();
            let Some(CoupledWriterStage::Inspected(writer)) = writers[index].stage.as_ref() else {
                unreachable!("actual appended writer")
            };
            self.application
                .runtime
                .primary_provider
                .coupled_writer_fact_keys(writer.session())
        };
        charge(execution, subject, keys.len() as u64 + 1)?;
        let inserted = self
            .ledger
            .borrow_mut()
            .retain(
                execution,
                Derivation::part(read, self.applied_position(), Some(index)),
                keys,
            )
            .map_err(|stop| ledger_stop(subject, stop))?;
        charge(execution, subject, inserted)
    }

    /// A consumer of a reopened derivation reopens with it (R3): its own
    /// derivation of `read` is superseded at the same invalidator.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn supersede_consumer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: ParentRead,
        invalidator: usize,
    ) -> Result<(), CoupledSourceReadStop> {
        let retention = {
            let mut ledger = self.ledger.borrow_mut();
            charge(execution, "reopened consumer", ledger.find_work())?;
            let Some(id) = ledger.find(read) else {
                return Ok(());
            };
            ledger.supersede(id);
            ledger.entry(id).writer.is_none()
        };
        if retention {
            self.application
                .runtime
                .primary_provider
                .supersede_retained_parent(execution, &self.root_session, read, invalidator)
                .map_err(CoupledSourceReadStop::capture)?;
        }
        Ok(())
    }

    /// A successor round replaced the part that derived `read`.
    pub(super) fn retire_part(&self, read: ParentRead) {
        let mut ledger = self.ledger.borrow_mut();
        if let Some(id) = ledger.find(read) {
            ledger.supersede(id);
        }
    }

    /// Whether every derivation that consumed `read` is owned by a wave task,
    /// so reopening `read` reopens each consumer after it (R3). The part
    /// being appended (`appending`) is owned by the task preparing it.
    fn consumers_reopenable(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: ParentRead,
        appending: Option<ParentRead>,
    ) -> Result<bool, CoupledSourceReadStop> {
        let subject = "consumed parent outputs";
        let consumed = self.consumed.borrow();
        let ledger = self.ledger.borrow();
        charge(execution, subject, consumed.len().max(1).ilog2() as u64 + 1)?;
        let Some(mut rank) = consumed
            .first_at_or_after(&(read.0, 0))
            .and_then(|(first, ())| consumed.rank_of(first))
        else {
            return Ok(true);
        };
        while let Some((&(parent, consumer), ())) = consumed.entry_at(rank) {
            if parent != read.0 {
                break;
            }
            charge(execution, subject, ledger.find_work())?;
            if appending != Some(ParentRead(consumer)) && !ledger.claimed(ParentRead(consumer)) {
                return Ok(false);
            }
            rank += 1;
        }
        Ok(true)
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
        let (at, at_owed) = match &member {
            AppendedMember::Part { position, owed, .. } => (*position, *owed),
            AppendedMember::Stage { position, .. } => (*position, *position),
        };
        self.mark_selections(execution, candidate, at, at_owed)?;
        if self.ledger.borrow().is_empty() {
            return Ok(());
        }
        let appending = match &member {
            // Only a part that can derive again is reopenable as the
            // consumer of a parent its own append changed.
            AppendedMember::Part {
                index, continues, ..
            } => continues.then(|| self.writers.borrow()[*index].produces),
            AppendedMember::Stage { .. } => None,
        };
        let (touches, position, owed, route) = match member {
            AppendedMember::Part {
                index,
                position,
                owed,
                meaning,
                ..
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
            self.requalify(
                execution,
                id,
                &current,
                position,
                owed,
                route.as_ref(),
                appending,
            )?;
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
        appending: Option<ParentRead>,
    ) -> Result<(), CoupledSourceReadStop> {
        let (read, position, claimed, writer) = {
            let ledger = self.ledger.borrow();
            let entry = ledger.entry(id);
            (entry.read, entry.position, entry.claimed, entry.writer)
        };
        if !claimed || !self.consumers_reopenable(execution, read, appending)? {
            return Ok(());
        }
        if let Some(index) = writer {
            // A producer reader stays strict: any change reopens it (R2).
            let changed = {
                let writers = self.writers.borrow();
                let Some(CoupledWriterStage::Inspected(part)) = writers[index].stage.as_ref()
                else {
                    unreachable!("actual appended writer")
                };
                self.application
                    .runtime
                    .primary_provider
                    .coupled_writer_support_changed(execution, part.session(), self.before, current)
                    .map_err(CoupledSourceReadStop::capture)?
            };
            if changed {
                self.ledger.borrow_mut().supersede(id);
                self.reopens.borrow_mut().push(Reopen {
                    target: ReopenTarget::Retention(read),
                    invalidator,
                    invalidator_owed,
                });
            }
            return Ok(());
        }
        // An authored reader settles a change only by the producer of the same
        // route, owed at or after it (`settles_read_support`).
        let authored = !self.all_stage_captures().is_empty();
        let settles = |parent: &dyn worth_query_execution_attempt::facade::ports::CandidateReadSupportPort| {
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
                target: ReopenTarget::Retention(read),
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
