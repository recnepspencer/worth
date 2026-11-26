//! Persistent-name reads on an applied candidate: a coupled producer or a
//! later edit-envelope stage deciding against the envelope's unpublished
//! candidate.
//!
//! The candidate has no name index of its own; names are published only when
//! the envelope commits. A read therefore runs against the committed name
//! index exactly at the commit basis the candidate was proposed on, borrowed
//! through the reader's committed-runtime access only after the candidate is
//! proven aligned with this reader: proposed on the same runtime instance, on
//! exactly this reader's branch observation, with `before` pinned at this
//! reader's runtime, branch and version. Anything else refuses `CandidateBasis`; no other
//! runtime or basis is ever read.
//!
//! The basis answer is the candidate's answer by Relational's posture law
//! (commit finalization): `Create` and `Restore` claim only records the
//! commit created; `Retire`, `Split` and a non-keeper `Merge` parent require
//! the record retired in the commit; `Preserve` and an in-place merge keeper
//! keep the record's own lineage. So a record live at the basis and still
//! live at the same identity in the candidate carries its basis name (or
//! stays unnamed), and a resolution whose every live footprint record is
//! such a record gains no new live descendant. Where that premise fails the
//! read refuses typed and vetoes sealing, never answering from stale state:
//!
//! - a record not live at the basis was created by the candidate. When it is
//!   live in the candidate as read, an earlier envelope member may have
//!   named it: its name and birth are read from the names the envelope gives
//!   its records (`pending_naming`, #426). Otherwise, or when no member named
//!   it, it has no name yet (`UnpublishedCandidateRecord`);
//! - a footprint record the candidate ended (deleted, retired, split or
//!   merged away) refuses `EndedInCandidate`; it is never filtered out, so an
//!   ambiguous sibling set never collapses to the survivor;
//! - an answer resting on an ended name or an unborn held name refuses
//!   `CandidateNameHistory`: the candidate may restore, regenerate or bear it
//!   on a created record. (A committed read of such an answer never seals
//!   either.)
//!
//! A successor round reads each record and relation its own chain wrote as it
//! was before the chain (`projection_of`, `projection_of_relation`, §1.5,
//! #277), and admits its identity there too. Owner chains are read on the candidate
//! itself (`name_owner`), so a same-candidate re-owning is what `resolve_in`
//! sees and is tracked as an adjacency fact.

use worth_query_execution_naming::facade::{ExecutionNameReader, NameReadDependencies};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::lineage::LineageRecordTarget;
use worth_relational::facade::runtime::{
    ProjectionAspectScope, RelationalRuntime, VisibilityProjectionView,
};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::RelationalExecutionDenialCause;

use super::current_output::{WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind};
use super::naming::{PersistentNameReadVeto, WorthQueryDecisionNameDenial};
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::NativeProjectionRead;

type Denial = WorthQueryDecisionNameDenial;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Runs `read` over the committed name index at this reader's basis; on
    /// an applied candidate only once the candidate is proven aligned with
    /// it. A refusal vetoes sealing.
    pub(super) fn with_name_reader<T>(
        &mut self,
        read: impl FnOnce(&ExecutionNameReader<'_>) -> T,
    ) -> Result<T, Denial> {
        let (execution, observation) = (self.reader.execution, &self.reader.observation);
        let read = self.aligned_runtime_read(|runtime| {
            read(&ExecutionNameReader::at_observation(
                runtime,
                execution,
                observation,
            ))
        });
        if read.is_err() {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
        }
        read
    }

    /// Runs `read` over the committed runtime at this reader's basis
    /// snapshot, under the same alignment proof as `with_name_reader`.
    pub(super) fn aligned_runtime_read<T>(
        &self,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> Result<T, Denial> {
        let reader = &*self.reader;
        let candidate = match &reader.native {
            NativeProjectionRead::Committed(runtime) => return Ok(read(runtime)),
            NativeProjectionRead::Applied { candidate, .. } => candidate,
        };
        self.require_candidate_aligned()?;
        let instance = candidate
            .candidate_identity()
            .proposal()
            .runtime_instance_id();
        let access = self.source_query.ok_or(Denial::CandidateBasis)?;
        let mut read = Some(read);
        let mut answer = None;
        access.with_committed_runtime_dyn(reader.execution, &mut |runtime| {
            if runtime.runtime_instance_id() != instance {
                return;
            }
            if let Some(read) = read.take() {
                answer = Some(read(runtime));
            }
        });
        answer.ok_or(Denial::CandidateBasis)
    }

    /// Proves an applied candidate aligned with this reader: proposed on the
    /// same runtime instance, on exactly this reader's branch observation,
    /// with `before` pinned at this reader's runtime, branch and version.
    pub(super) fn require_candidate_aligned(&self) -> Result<(), Denial> {
        let reader = &*self.reader;
        let NativeProjectionRead::Applied {
            before, candidate, ..
        } = &reader.native
        else {
            return Ok(());
        };
        let proposal = candidate.candidate_identity().proposal();
        let instance = proposal.runtime_instance_id();
        // `before` is an immutable pin, so it is aligned by identity: the
        // same runtime, branch and version as this reader's own snapshot,
        // which is this reader's observation.
        let pinned = before.snapshot();
        let snapshot = reader.snapshot;
        if proposal.branch_observation() != reader.observation.reference()
            || pinned.runtime_instance_id() != instance
            || snapshot.runtime_instance_id() != instance
            || pinned.branch_id() != snapshot.branch_id()
            || pinned.version_id() != snapshot.version_id()
            || snapshot.version_id() != reader.observation.version_id()
        {
            return Err(Denial::CandidateBasis);
        }
        Ok(())
    }

    /// What the applied candidate did to `record`: refuses, vetoing sealing,
    /// a record it ended or one it created that is not live as read; admits
    /// every record of a committed read as unaltered.
    pub(super) fn candidate_record(
        &mut self,
        record: LineageRecordTarget,
    ) -> Result<CandidateRecord, Denial> {
        let classified = self.candidate_alteration(record);
        if classified.is_err() {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
        }
        classified
    }

    /// Refuses, vetoing sealing, a read of `record` the applied candidate
    /// created or ended: a group member read rests on the basis grouping.
    pub(super) fn require_candidate_unaltered(
        &mut self,
        record: LineageRecordTarget,
    ) -> Result<(), Denial> {
        let unaltered = self
            .candidate_alteration(record)
            .and_then(|altered| match altered {
                CandidateRecord::Unaltered => Ok(()),
                CandidateRecord::Created => Err(Denial::UnpublishedCandidateRecord(record)),
            });
        self.vetoed(unaltered)
    }

    /// Refuses, vetoing sealing, a resolution on an applied candidate that
    /// rested on an ended or unborn name, or on a live record the candidate
    /// ended. Every footprint record is checked; none is filtered out.
    pub(super) fn require_candidate_footprint(
        &mut self,
        dependencies: &NameReadDependencies,
        branch: &BranchId,
    ) -> Result<(), Denial> {
        if !matches!(self.reader.native, NativeProjectionRead::Applied { .. }) {
            return Ok(());
        }
        let footprint = dependencies
            .live_records_on(branch)
            .into_iter()
            .try_for_each(|record| match self.candidate_alteration(record)? {
                CandidateRecord::Unaltered => Ok(()),
                CandidateRecord::Created => Err(Denial::UnpublishedCandidateRecord(record)),
            })
            .and_then(|()| {
                if dependencies.ended_lineages_on(branch).is_empty()
                    && !dependencies.awaits_births_on(branch)
                {
                    Ok(())
                } else {
                    Err(Denial::CandidateNameHistory)
                }
            });
        self.vetoed(footprint)
    }

    fn vetoed(&mut self, outcome: Result<(), Denial>) -> Result<(), Denial> {
        if outcome.is_err() {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
        }
        outcome
    }

    /// How the applied candidate changed `record` since its commit basis:
    /// `Unaltered` when it is live at the same identity at both (or the read
    /// is committed), `Created` when only the candidate as read has it live.
    fn candidate_alteration(
        &mut self,
        record: LineageRecordTarget,
    ) -> Result<CandidateRecord, Denial> {
        let reader = &mut *self.reader;
        let NativeProjectionRead::Applied { before, .. } = &reader.native else {
            return Ok(CandidateRecord::Unaltered);
        };
        let execution = reader.execution;
        let at_candidate = match record {
            LineageRecordTarget::Entity(entity) => {
                reader.native.projection_of(reader.snapshot, entity)
            }
            LineageRecordTarget::Relation(relation) => reader
                .native
                .projection_of_relation(reader.snapshot, relation),
        };
        let live = |view: Option<VisibilityProjectionView>| -> Result<bool, _> {
            let Some(view) = view else {
                return Ok(false);
            };
            let lifecycle = match record {
                LineageRecordTarget::Entity(entity) => view.entity_record_with_projection_scope(
                    execution,
                    entity,
                    ProjectionAspectScope::empty(),
                    |found| Some(found.lifecycle()),
                ),
                LineageRecordTarget::Relation(relation) => view
                    .relation_record_with_projection_scope(
                        execution,
                        relation,
                        ProjectionAspectScope::empty(),
                        |found| Some(found.lifecycle()),
                    ),
            }?;
            Ok::<_, RelationalExecutionDenialCause>(lifecycle == Some(RecordLifecycleState::Live))
        };
        let states = live(Some(before.project_read()))
            .and_then(|basis| live(at_candidate).map(|candidate| (basis, candidate)));
        match states {
            Ok((true, true)) => Ok(CandidateRecord::Unaltered),
            Ok((false, true)) => Ok(CandidateRecord::Created),
            Ok((false, false)) => Err(Denial::UnpublishedCandidateRecord(record)),
            Ok((true, false)) => Err(Denial::EndedInCandidate(record)),
            Err(cause) => {
                reader.native_read_stop.get_or_insert(cause);
                Err(Denial::Identity(WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::ExecutionRequest(cause),
                    "persistent name",
                )))
            }
        }
    }
}

/// A record an applied candidate's read admits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CandidateRecord {
    /// Live at the same identity at the basis and in the candidate.
    Unaltered,
    /// Not live at the basis, live in the candidate as read: an earlier
    /// envelope member created it.
    Created,
}
