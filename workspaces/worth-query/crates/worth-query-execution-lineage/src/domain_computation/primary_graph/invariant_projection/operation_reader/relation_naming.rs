//! Tracked persistent-name reads of relations on an operation's decision
//! reader.
//!
//! A relation is read through the handle the decision reader's own
//! traversals return, never a raw relation identity. The read is admitted
//! against the operation's declared relation read, and every live relation
//! an answer rested on is admitted with its endpoints into the realized read
//! scope and keyed as that relation's decision fact: the exact relations
//! between its endpoint pair. Unlinking, splitting, merging or retiring it
//! (or deleting an endpoint) changes that fact and stales the decision.
//!
//! What no fact key tracks vetoes sealing exactly as for entities
//! ([`PersistentNameReadVeto`]), and an applied candidate admits the read
//! only as `candidate_naming` states.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use worth_query_declaration::facade::application_naming::{
    NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_declaration::facade::application_schema::fragment;
use worth_query_execution_naming::facade::{ExecutionNameResolveDenial, NameReadDependencies};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationRelationRef, ApplicationSchema,
    OperationReads,
};
use worth_relational::facade::identity::{KindId, RelationId};
use worth_relational::facade::lineage::LineageRecordTarget;
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_relational::facade::storage::RecordLifecycleState;

use super::current_output::{WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind};
use super::decision_reads::DecisionReadOutcome;
use super::naming::{
    local_handles, untracked_resolution, PersistentNameReadVeto, WorthQueryDecisionNameDenial,
};
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey;
use crate::domain_computation::primary_graph::WorthQueryInvariantRelation;

type RelationResolution<Schema, Relation, From, To> = Result<
    ResolvedName<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
    ExecutionNameResolveDenial<Relation>,
>;

/// The installed relation a name read is about, with its endpoint kinds.
pub(super) struct NamedRelation {
    name: String,
    kind: KindId,
    from: (KindId, String),
    to: (KindId, String),
}

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// The admitted `WorthQueryName<Relation>` the live relation carries, as
    /// a tracked decision read keyed on the relation between its endpoints.
    ///
    /// `relation` is a handle this reader's own traversal returned.
    pub fn relation_name_of<Relation, From, To>(
        &mut self,
        relation: &WorthQueryInvariantRelation<Schema, Relation, From, To>,
    ) -> Result<WorthQueryName<Relation>, WorthQueryDecisionNameDenial>
    where
        Relation: fragment::Relation + OperationReads<Operation>,
    {
        let subject = self.admit_relation_name_read(Relation::NAME)?;
        if !self
            .reader
            .identity_is_local(relation.from(), &subject.from.1)
            || !self.reader.identity_is_local(relation.to(), &subject.to.1)
        {
            return Err(WorthQueryDecisionNameDenial::ForeignIdentity);
        }
        self.require_candidate_unaltered(LineageRecordTarget::Relation(relation.relation_id()))?;
        let named = self.with_name_reader(|reader| {
            reader.relation_name_of::<Relation>(
                relation.relation_id(),
                &mut NameReadDependencies::new(),
            )
        })?;
        if let Some(veto) = PersistentNameReadVeto::for_name_of(&named) {
            self.decision_facts.veto_persistent_name(veto);
        }
        // A relation's name changes only when its lineage ends, which ends
        // the relation: its fact tracks the answer and `RecordNotNamed`.
        let live = self.live_relation::<Relation, From, To>(&subject, relation.relation_id())?;
        if live != *relation {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
            return Err(WorthQueryDecisionNameDenial::ForeignIdentity);
        }
        named.map_err(WorthQueryDecisionNameDenial::Read)
    }

    /// Resolves a held relation name at this reader's basis under
    /// `selection`, as a tracked decision read. Resolved relations come back
    /// as local handles with their endpoints. The outer result is the
    /// decision admission; the inner one is resolution's own typed outcome.
    pub fn resolve_relation<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        held: &WorthQueryName<Relation>,
        selection: &NameSelection,
    ) -> Result<RelationResolution<Schema, Relation, From, To>, WorthQueryDecisionNameDenial>
    where
        Relation: fragment::Relation + OperationReads<Operation>,
    {
        let subject = self.admit_relation_name_read(relation.name())?;
        let mut dependencies = NameReadDependencies::new();
        let (resolved, branch) = self.with_name_reader(|reader| {
            let resolved = reader.resolve_relation::<Relation>(held, selection, &mut dependencies);
            (resolved, reader.branch().clone())
        })?;
        let resolved = match resolved {
            Err(ExecutionNameResolveDenial::Read(read)) => {
                self.decision_facts
                    .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
                return Ok(Err(ExecutionNameResolveDenial::Read(read)));
            }
            resolved => resolved,
        };
        self.require_candidate_footprint(&dependencies, &branch)?;
        let untracked = untracked_resolution(&resolved, &dependencies, &branch);
        if let Some(veto) = PersistentNameReadVeto::for_resolve(&resolved, untracked) {
            self.decision_facts.veto_persistent_name(veto);
        }
        let mut relations = BTreeMap::new();
        for record in dependencies.live_records_on(&branch) {
            match record {
                LineageRecordTarget::Relation(relation_id) => {
                    relations.insert(relation_id, self.live_relation(&subject, relation_id)?);
                }
                LineageRecordTarget::Entity(_) => {
                    self.decision_facts
                        .veto_persistent_name(PersistentNameReadVeto::UntrackedDependency);
                }
            }
        }
        Ok(resolved.map(|resolved| local_handles(resolved, &relations)))
    }

    /// Admits a name read of the relation named `name` against the
    /// operation's declared relation read and returns the installed relation
    /// it is about.
    pub(super) fn admit_relation_name_read(
        &self,
        name: &str,
    ) -> Result<NamedRelation, WorthQueryDecisionNameDenial> {
        let layout = self.reader.layout;
        let installed = layout.relation(name).and_then(|relation| {
            let endpoint = |kind| Some((kind, layout.entity_name(kind)?.to_owned()));
            Some(NamedRelation {
                name: name.to_owned(),
                kind: relation.kind,
                from: endpoint(relation.from)?,
                to: endpoint(relation.to)?,
            })
        });
        // A relation the schema does not install is declared by no operation.
        let (from, to) = installed
            .as_ref()
            .map(|subject| (subject.from.1.clone(), subject.to.1.clone()))
            .unwrap_or_default();
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Relation {
            relation: name.to_owned(),
            from,
            to,
        })
        .map_err(WorthQueryDecisionNameDenial::UndeclaredDecisionTarget)?;
        installed.ok_or(WorthQueryDecisionNameDenial::Identity(unavailable(
            WorthQueryCurrentOutputDenialKind::FamilyUnavailable,
            name,
        )))
    }

    /// Admits a live relation with its endpoints into the realized read
    /// scope as a local handle and keys it as the relation's decision fact.
    fn live_relation<Relation, From, To>(
        &mut self,
        subject: &NamedRelation,
        relation_id: RelationId,
    ) -> Result<WorthQueryInvariantRelation<Schema, Relation, From, To>, WorthQueryDecisionNameDenial>
    {
        let admitted = self.admit_live_relation(subject, relation_id);
        if admitted.is_err() {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
        }
        admitted
    }

    fn admit_live_relation<Relation, From, To>(
        &mut self,
        subject: &NamedRelation,
        relation_id: RelationId,
    ) -> Result<WorthQueryInvariantRelation<Schema, Relation, From, To>, WorthQueryDecisionNameDenial>
    {
        let execution = *self.reader.execution;
        let record = self
            .reader
            .native
            .projection_of_relation(self.reader.snapshot, relation_id)
            .map(|view| {
                view.relation_record_with_projection_scope(
                    &execution,
                    relation_id,
                    ProjectionAspectScope::empty(),
                    |record| {
                        Some((
                            record.kind_id(),
                            record.lifecycle(),
                            record.source(),
                            record.target(),
                        ))
                    },
                )
            })
            .transpose()
            .map_err(|cause| {
                self.reader.native_read_stop.get_or_insert(cause);
                WorthQueryDecisionNameDenial::Identity(unavailable(
                    WorthQueryCurrentOutputDenialKind::ExecutionRequest(cause),
                    &subject.name,
                ))
            })?
            .flatten()
            .filter(|(kind, lifecycle, _, _)| {
                *kind == subject.kind && *lifecycle == RecordLifecycleState::Live
            });
        let Some((_, _, source, target)) = record else {
            return Err(WorthQueryDecisionNameDenial::Identity(unavailable(
                WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                &subject.name,
            )));
        };
        let (from_kind, from_name) = &subject.from;
        let (to_kind, to_name) = &subject.to;
        let from = self
            .reader
            .endpoint::<From>(&execution, source, *from_kind, from_name)
            .map_err(WorthQueryDecisionNameDenial::Endpoint)?;
        let to = self
            .reader
            .endpoint::<To>(&execution, target, *to_kind, to_name)
            .map_err(WorthQueryDecisionNameDenial::Endpoint)?;
        self.reader.realized_scope.record_relation(source, target);
        self.decision_facts.record(
            WorthQueryApplicationFactKey::Relation {
                relation: subject.name.clone(),
                from: source,
                to: target,
            },
            DecisionReadOutcome::Observed,
        );
        Ok(WorthQueryInvariantRelation {
            relation_id,
            from,
            to,
            _relation: PhantomData,
        })
    }
}

fn unavailable(
    kind: WorthQueryCurrentOutputDenialKind,
    relation: &str,
) -> WorthQueryCurrentOutputDenial {
    WorthQueryCurrentOutputDenial::new(kind, relation)
}
