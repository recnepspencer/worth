//! Tracked persistent group reads on an operation's decision reader.
//!
//! A group answer rests on the derived name index's group state, which no
//! record lifecycle carries: regrouping live members changes no record. Each
//! read is therefore keyed as a `PersistentGroup` decision fact naming the
//! read itself, and sealing re-observes it at the sealing basis, so a later
//! commit that splits, merges, retires or re-members a group the decision
//! read stales it, while an unrelated commit leaves it current.
//!
//! Groups are read on a committed basis only: an applied candidate's own
//! group partitions publish with its envelope, so a candidate-basis group
//! read refuses `CandidateGroupState` and vetoes sealing.

use worth_query_declaration::facade::application_naming::{
    ApplicationGroupKind, NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_execution_naming::facade::{ExecutionNameResolveDenial, NameReadDependencies};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema, OperationReads,
};

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
        self.record_group_fact::<G>(Member::IDENTIFIER, subject);
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
        self.record_group_fact::<G>(Member::IDENTIFIER, subject);
        Ok(resolved)
    }

    /// Admits a group read under the operation's declared read of the
    /// member kind, on a committed basis only.
    fn admit_group_read(&mut self, member_kind: &str) -> Result<(), Denial> {
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Entity {
            entity: member_kind.to_owned(),
        })
        .map_err(Denial::UndeclaredDecisionTarget)?;
        self.require_committed_group_basis()
    }

    fn require_committed_group_basis(&mut self) -> Result<(), Denial> {
        if matches!(self.reader.native, NativeProjectionRead::Applied { .. }) {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
            return Err(Denial::CandidateGroupState);
        }
        Ok(())
    }

    fn record_group_fact<G: ApplicationGroupKind>(
        &mut self,
        member_kind: &str,
        subject: PersistentGroupSubject,
    ) {
        let read = PersistentGroupRead {
            group_kind: G::NAME.to_owned(),
            member_kind: member_kind.to_owned(),
            subject,
        };
        self.decision_facts.record(
            WorthQueryApplicationFactKey::PersistentGroup(read),
            DecisionReadOutcome::Observed,
        );
    }
}
