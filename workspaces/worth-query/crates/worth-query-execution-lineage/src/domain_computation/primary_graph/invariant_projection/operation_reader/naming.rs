//! Tracked persistent-name reads on an operation's decision reader.
//!
//! Resolution, kind verification and dependency recording belong to
//! `worth-query-execution-naming`; this module admits the read against the
//! operation's declared entity contract and turns what the answer rested on
//! into decision facts (`relation_naming` does the same for relations). A
//! decision never seals a name dependency it cannot track:
//!
//! - a live record's name, and every live descendant a resolution found, is
//!   admitted into the realized read scope and keyed as that record's fact,
//!   so a split, merge, replacement or retirement ending it stales the
//!   decision through ordinary lifecycle invalidation;
//! - an answer that also rested on ended names on the walked path (which a
//!   later `Restore` revives), on a held name or carrier not yet born, or on
//!   a record of the other subject kind is keyed as a `PersistentNameHistory`
//!   fact naming the read itself (`track_name_history`): the seal re-observes
//!   it, and a committed event touching a lineage it rests on, or birthing
//!   the awaited name, re-verifies it, so the decision seals and stales
//!   exactly when that history changes;
//! - on an applied candidate that history is the unpublished candidate's,
//!   which no fact can re-observe, so the projection refuses to seal with
//!   [`FailedDecisionRead::PersistentName`]; a name read that failed vetoes
//!   sealing the same way, even when the handler swallowed the error.
//!
//! On an applied candidate the reads run against the committed name index at
//! the candidate's commit basis and are admitted only where the candidate left
//! every record the answer rests on live at the same identity
//! (`candidate_naming`); a record an earlier envelope member created reads the
//! name the envelope gives it (`pending_naming`).

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::{
    NameOwnerScope, NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
    WorthQueryNamedKind,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_execution_naming::facade::{
    candidate_entity_name_of, ExecutionNameReadDenial, ExecutionNameResolveDenial,
    NameReadDependencies,
};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema, OperationReads,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::{LineageRecordTarget, PersistentNameSubjectKind};

use super::candidate_naming::CandidateRecord;
use super::decision_reads::{DecisionReadOutcome, FailedDecisionRead};
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::fact::persistent_name_history::{
    PersistentNameHistoryQuestion as Question, PersistentNameHistoryRead,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::NativeProjectionRead;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;

mod denial;
pub use denial::WorthQueryDecisionNameDenial;

/// Why a decision that read a persistent name refuses to seal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistentNameReadVeto {
    /// A name read failed; its outcome rests on nothing a fact retains.
    ReadFailed,
    /// On an applied candidate, the answer rested on name history the
    /// unpublished candidate may change and no fact can re-observe: an ended
    /// name a later `Restore` revives, an unborn held name, or a record of
    /// another subject kind than the one resolved.
    UntrackedDependency,
}

impl PersistentNameReadVeto {
    /// The veto a decision's `name_of` outcome needs: only an answer or the
    /// stable `RecordNotNamed` rests on the record's entity fact alone.
    pub fn for_name_of<K>(
        named: &Result<WorthQueryName<K>, ExecutionNameReadDenial>,
    ) -> Option<Self> {
        match named {
            Ok(_) | Err(ExecutionNameReadDenial::RecordNotNamed { .. }) => None,
            Err(_) => Some(Self::ReadFailed),
        }
    }

    /// The veto a decision's `resolve` outcome needs: every read refusal
    /// and a pruned basis are failed reads. Every other answer rests on
    /// facts the read keys (`track_name_history` for its name history).
    pub fn for_resolve<H, K, O>(
        resolved: &Result<ResolvedName<H>, ExecutionNameResolveDenial<K, O>>,
    ) -> Option<Self> {
        match resolved {
            Err(ExecutionNameResolveDenial::Read(_))
            | Err(ExecutionNameResolveDenial::Resolution(NameResolutionDenial::BasisPruned)) => {
                Some(Self::ReadFailed)
            }
            _ => None,
        }
    }
}

/// The owning entity kind `Entity`'s declared owner scope reaches.
pub type NameOwnerOf<Entity> =
    <<Entity as WorthQueryNamedKind>::OwnerScope as NameOwnerScope<Entity>>::Owner;

/// Resolution's own typed outcome: local identities of the resolved records,
/// or a refusal whose candidates carry their owners' local identities.
pub type WorthQueryDecisionNameResolution<Schema, Entity> = Result<
    ResolvedName<WorthQueryInvariantEntityIdentity<Schema, Entity>>,
    ExecutionNameResolveDenial<
        Entity,
        WorthQueryInvariantEntityIdentity<Schema, NameOwnerOf<Entity>>,
    >,
>;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// The admitted `WorthQueryName<Entity>` the live entity carries, as a
    /// tracked decision read keyed on the entity's lifecycle.
    pub fn name_of<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryName<Entity>, WorthQueryDecisionNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.admit_name_read(Entity::IDENTIFIER)?;
        if !self.reader.identity_is_local(identity, Entity::IDENTIFIER) {
            return Err(WorthQueryDecisionNameDenial::ForeignIdentity);
        }
        let (entity_id, record) = (
            identity.entity_id,
            LineageRecordTarget::Entity(identity.entity_id),
        );
        if self.candidate_record(record)? == CandidateRecord::Created {
            let named = self.pending_name_of(record, |names| {
                candidate_entity_name_of::<Schema, Entity>(names, entity_id)
            })?;
            self.live_identity::<Entity>(entity_id)?;
            return Ok(named);
        }
        let named = self.with_name_reader(|reader| {
            reader.entity_name_of::<Schema, Entity>(entity_id, &mut NameReadDependencies::new())
        })?;
        if let Some(veto) = PersistentNameReadVeto::for_name_of(&named) {
            self.decision_facts.veto_persistent_name(veto);
        }
        // A record's name changes only when its lineage ends, which ends the
        // record: the entity fact tracks the answer and `RecordNotNamed`.
        self.live_identity::<Entity>(entity_id)?;
        named.map_err(WorthQueryDecisionNameDenial::Read)
    }

    /// Resolves a held entity name at this reader's basis under `selection`,
    /// as a tracked decision read. Resolved records come back as local
    /// identities the decision can read further. The outer result is the
    /// decision admission; the inner one is resolution's own typed outcome.
    ///
    /// Several live candidates under a selection that names one refuse
    /// `Ambiguous`, each candidate with the owners its kind's declared owner
    /// chain reaches (see `resolve_in`); resolution never picks one.
    pub fn resolve<Entity>(
        &mut self,
        held: &WorthQueryName<Entity>,
        selection: &NameSelection,
    ) -> Result<WorthQueryDecisionNameResolution<Schema, Entity>, WorthQueryDecisionNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>
            + OperationReads<Operation>
            + WorthQueryNamedKind,
    {
        self.resolve_scoped(None, held, selection)
    }

    /// Resolves `held` across every owner, or among the records `within`
    /// holds. Owner reads run after the name index is released.
    pub(super) fn resolve_scoped<Entity>(
        &mut self,
        within: Option<EntityId>,
        held: &WorthQueryName<Entity>,
        selection: &NameSelection,
    ) -> Result<WorthQueryDecisionNameResolution<Schema, Entity>, WorthQueryDecisionNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>
            + OperationReads<Operation>
            + WorthQueryNamedKind,
    {
        self.admit_name_read(Entity::IDENTIFIER)?;
        let mut dependencies = NameReadDependencies::new();
        let (candidates, branch) = self.with_name_reader(|reader| {
            let candidates =
                reader.entity_candidates::<Schema, Entity>(held, selection, &mut dependencies);
            (candidates, reader.branch().clone())
        })?;
        let candidates = match candidates {
            Ok(candidates) => candidates,
            Err(read) => {
                self.decision_facts
                    .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
                return Ok(Err(ExecutionNameResolveDenial::Read(read)));
            }
        };
        self.require_candidate_footprint(&dependencies, &branch)?;
        let mut identities = BTreeMap::new();
        let mut other_kind = false;
        for record in dependencies.live_records_on(&branch) {
            match record {
                LineageRecordTarget::Entity(entity_id) => {
                    identities.insert(entity_id, self.live_identity::<Entity>(entity_id)?);
                }
                LineageRecordTarget::Relation(_) => other_kind = true,
            }
        }
        let mut owners = BTreeMap::new();
        let mut owners_of = |entity: &EntityId| {
            let candidate = identities
                .get(entity)
                .expect("every matched record is a live footprint record admitted above");
            let reached = self.name_owners::<Entity>(candidate).inspect_err(|_| {
                self.decision_facts
                    .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
            })?;
            let held_by = reached.keys().copied().collect::<Vec<_>>();
            owners.extend(reached);
            Ok::<_, WorthQueryDecisionNameDenial>(held_by)
        };
        let resolved = match within {
            Some(owner) => candidates.resolve_in(&owner, &mut owners_of),
            None => candidates.resolve(&mut owners_of),
        }?;
        let resolved = resolved.map_err(ExecutionNameResolveDenial::Resolution);
        if let Some(veto) = PersistentNameReadVeto::for_resolve(&resolved) {
            self.decision_facts.veto_persistent_name(veto);
        }
        if other_kind || rests_on_name_history(&resolved, &dependencies, &branch) {
            let subject = PersistentNameSubjectKind::Entity;
            self.track_name_history(Entity::IDENTIFIER, subject, &dependencies, &branch);
        }
        Ok(resolved
            .map(|resolved| local_handles(resolved, &identities))
            .map_err(|denial| owner_handles(denial, &owners)))
    }

    pub(super) fn admit_name_read(&self, entity: &str) -> Result<(), WorthQueryDecisionNameDenial> {
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Entity {
            entity: entity.to_owned(),
        })
        .map_err(WorthQueryDecisionNameDenial::UndeclaredDecisionTarget)
    }

    /// Admits a live record into the realized read scope as a local identity
    /// and keys it as the entity's decision fact; a successor round's own
    /// record is admitted at its chain head, where its liveness was read.
    fn live_identity<Entity>(
        &mut self,
        entity_id: EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryDecisionNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        let execution = *self.reader.execution;
        self.live_own_view_identity(&execution, Entity::IDENTIFIER, entity_id)
            .map_err(|denial| {
                self.decision_facts
                    .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
                WorthQueryDecisionNameDenial::Identity(denial)
            })
    }
}

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Keys what a read's answer rested on in name history, beyond its live
    /// records, as `PersistentNameHistory` decision facts admitted under the
    /// declared read of `kind`: every resolution it ran, under its exact
    /// Relational selection, and every name or carrier it found unborn. On
    /// an applied candidate that history is the candidate's own, which no
    /// fact re-observes, so the decision refuses to seal instead.
    pub(super) fn track_name_history(
        &mut self,
        kind: &str,
        subject: PersistentNameSubjectKind,
        dependencies: &NameReadDependencies,
        branch: &BranchId,
    ) {
        if matches!(self.reader.native, NativeProjectionRead::Applied { .. }) {
            self.decision_facts
                .veto_persistent_name(PersistentNameReadVeto::UntrackedDependency);
            return;
        }
        let resolutions = dependencies.resolutions_on(branch).into_iter();
        let questions = resolutions
            .map(|(held, selection)| Question::Resolution { held, selection })
            .chain(
                dependencies
                    .awaited_births_on(branch)
                    .into_iter()
                    .map(Question::Birth),
            );
        for question in questions {
            let read = PersistentNameHistoryRead {
                kind: kind.to_owned(),
                subject,
                question,
            };
            self.decision_facts.record(
                WorthQueryApplicationFactKey::PersistentNameHistory(read),
                DecisionReadOutcome::Observed,
            );
        }
    }
}

impl FailedDecisionRead {
    pub const fn persistent_name(veto: PersistentNameReadVeto) -> Self {
        Self::PersistentName(veto)
    }
}

/// Whether a resolution's answer rested on name history no record lifecycle
/// carries: an ended name on the walked path, or a held name or carrier not
/// yet born.
pub(super) fn rests_on_name_history<H, K, O>(
    resolved: &Result<ResolvedName<H>, ExecutionNameResolveDenial<K, O>>,
    dependencies: &NameReadDependencies,
    branch: &BranchId,
) -> bool {
    !dependencies.ended_lineages_on(branch).is_empty()
        || dependencies.awaits_births_on(branch)
        || matches!(
            resolved,
            Err(ExecutionNameResolveDenial::Resolution(
                NameResolutionDenial::UnknownName
            ))
        )
}

/// The resolved records as the local handles they were admitted under.
pub(super) fn local_handles<Record: Ord, Handle: Clone>(
    resolved: ResolvedName<Record>,
    handles: &BTreeMap<Record, Handle>,
) -> ResolvedName<Handle> {
    let local = |record: Record| {
        handles
            .get(&record)
            .cloned()
            .expect("every resolved record is a live footprint record admitted above")
    };
    match resolved {
        ResolvedName::Exact(record) => ResolvedName::Exact(local(record)),
        ResolvedName::Plural(records) => {
            ResolvedName::Plural(records.into_iter().map(local).collect())
        }
        ResolvedName::Retired { at_commit } => ResolvedName::Retired { at_commit },
    }
}

/// A resolution refusal with each candidate's owners as the local identities
/// the owner chain admitted them under.
fn owner_handles<K, Schema, Owner>(
    denial: ExecutionNameResolveDenial<K, EntityId>,
    owners: &BTreeMap<EntityId, WorthQueryInvariantEntityIdentity<(), ()>>,
) -> ExecutionNameResolveDenial<K, WorthQueryInvariantEntityIdentity<Schema, Owner>> {
    match denial {
        ExecutionNameResolveDenial::Read(read) => ExecutionNameResolveDenial::Read(read),
        ExecutionNameResolveDenial::Resolution(denial) => {
            ExecutionNameResolveDenial::Resolution(denial.map_owners(|owner| {
                owners
                    .get(&owner)
                    .cloned()
                    .expect("every reported owner was reached and admitted by the owner chain")
                    .into_typed()
            }))
        }
    }
}
