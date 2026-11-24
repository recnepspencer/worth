//! Tracked persistent group reads on an operation's decision reader.
//!
//! A group answer rests on the derived name index's group state, which no
//! record lifecycle carries: regrouping live members changes no record. Each
//! read is therefore keyed as a `PersistentGroup` decision fact naming the
//! read itself, and sealing re-observes it at the sealing basis, so a later
//! commit that splits, merges, retires or re-members a group the decision
//! read stales it, while an unrelated commit leaves it current.
//!
//! On an applied candidate (a coupled producer or a later edit-envelope
//! stage) a group is read from the committed name index at the candidate's
//! commit basis, under `candidate_naming`'s alignment proof. Commit
//! finalization names a scheme's groups against the basis from the commit's
//! partition claims and from every group losing a member the commit ends, so
//! the basis answer is the candidate's only while that holds for the
//! candidate. Otherwise the read refuses typed and vetoes sealing, never
//! answering from stale state:
//!
//! - the envelope already claims a partition or dissolve of the scheme
//!   (`CandidateGroupState`); the read is retained with the envelope's owner,
//!   so a member that claims the scheme later refuses the envelope;
//! - the answer rests on name history: an unknown or ended member name, or
//!   a held group that is not itself live (retired, split, merged, unborn),
//!   which the candidate may revive or regroup (`CandidateNameHistory`);
//! - the candidate ended the member read or any member of a group the answer
//!   rests on (`EndedInCandidate`); siblings are never filtered out, so an
//!   ambiguous answer never collapses.
//!
//! Final coupled support compares those member records between images.

use worth_query_declaration::facade::application_naming::{
    ApplicationGroupKind, NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_execution_naming::facade::{ExecutionNameResolveDenial, NameReadDependencies};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema, OperationReads,
};
use worth_relational::facade::lineage::LineageRecordTarget;

use super::decision_reads::DecisionReadOutcome;
use super::naming::{PersistentNameReadVeto, WorthQueryDecisionNameDenial as Denial};
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::fact::persistent_group::{
    PersistentGroupRead, PersistentGroupSubject,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::NativeProjectionRead;

/// A decision's group resolution: the live groups' names, or resolution's
/// own typed refusal.
pub type WorthQueryDecisionGroupResolution<G> =
    Result<ResolvedName<WorthQueryName<G>>, ExecutionNameResolveDenial<G>>;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// The group of kind `G` that holds `member`'s name, or `None`, as a
    /// tracked decision read.
    pub fn group_name_of<G: ApplicationGroupKind, Member>(
        &mut self,
        member: &WorthQueryName<Member>,
    ) -> Result<Option<WorthQueryName<G>>, Denial>
    where
        Member: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.admit_group_read(Member::IDENTIFIER)?;
        let read = self.with_name_reader(|reader| {
            reader.group_name_of::<G, Member>(member, &mut NameReadDependencies::new())
        })?;
        let answer = read.inspect_err(|_| {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
        });
        let subject = PersistentGroupSubject::Member(*member.as_bytes());
        let fact = group_read::<G>(Member::IDENTIFIER, subject);
        if answer.is_ok() {
            self.require_candidate_group_support(&fact)?;
        }
        self.record_group_fact(fact);
        answer.map_err(Denial::Read)
    }

    /// Resolves a held group name of kind `G`, whose members are names of
    /// entity kind `Member`, under `selection`, as a tracked decision read.
    /// `Exact` on several live successors refuses `Ambiguous`; resolution
    /// never picks one.
    pub fn resolve_group<G: ApplicationGroupKind, Member>(
        &mut self,
        held: &WorthQueryName<G>,
        selection: &NameSelection,
    ) -> Result<WorthQueryDecisionGroupResolution<G>, Denial>
    where
        Member: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.admit_group_read(Member::IDENTIFIER)?;
        let resolved = self.with_name_reader(|reader| {
            reader.resolve_group::<G>(held, selection, &mut NameReadDependencies::new())
        })?;
        if let Some(veto) = PersistentNameReadVeto::for_resolve(&resolved, false) {
            self.decision_facts.veto_persistent_name(veto);
        }
        let subject = PersistentGroupSubject::Resolution(*held.as_bytes());
        let fact = group_read::<G>(Member::IDENTIFIER, subject);
        if !matches!(resolved, Err(ExecutionNameResolveDenial::Read(_))) {
            self.require_candidate_group_support(&fact)?;
        }
        self.record_group_fact(fact);
        Ok(resolved)
    }

    /// Admits a group read under the operation's declared read of the
    /// member kind.
    fn admit_group_read(&mut self, member_kind: &str) -> Result<(), Denial> {
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Entity {
            entity: member_kind.to_owned(),
        })
        .map_err(Denial::UndeclaredDecisionTarget)
    }

    /// On an applied candidate, admits a group read only where its basis
    /// answer is the candidate's (module docs); a refusal vetoes sealing.
    fn require_candidate_group_support(
        &mut self,
        read: &PersistentGroupRead,
    ) -> Result<(), Denial> {
        if !matches!(self.reader.native, NativeProjectionRead::Applied { .. }) {
            return Ok(());
        }
        let snapshot = self.reader.snapshot;
        let members = self
            .aligned_runtime_read(|runtime| read.candidate_support(runtime, snapshot))
            .and_then(|observed| observed.map_err(|_| Denial::CandidateBasis))
            .and_then(|support| {
                let claimed = self
                    .source_query
                    .and_then(|access| access.retain_applied_group_read(&support.scheme))
                    .ok_or(Denial::CandidateBasis)?;
                if claimed {
                    return Err(Denial::CandidateGroupState);
                }
                support.members.ok_or(Denial::CandidateNameHistory)
            })
            .inspect_err(|_| {
                self.decision_facts
                    .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
            })?;
        members.into_iter().try_for_each(|entity| {
            self.require_candidate_unaltered(LineageRecordTarget::Entity(entity))
        })
    }

    fn record_group_fact(&mut self, read: PersistentGroupRead) {
        self.decision_facts.record(
            WorthQueryApplicationFactKey::PersistentGroup(read),
            DecisionReadOutcome::Observed,
        );
    }
}

fn group_read<G: ApplicationGroupKind>(
    member_kind: &str,
    subject: PersistentGroupSubject,
) -> PersistentGroupRead {
    PersistentGroupRead {
        group_kind: G::NAME.to_owned(),
        member_kind: member_kind.to_owned(),
        subject,
    }
}
