//! Tracked persistent-name reads on an operation's decision reader.
//!
//! Resolution, kind verification and dependency recording belong to
//! `worth-query-execution-naming`; this module admits the read against the
//! operation's declared entity contract and turns what the answer rested on
//! into decision facts. A decision never seals a name dependency it cannot
//! track:
//!
//! - a live record's name, and every live descendant a resolution found, is
//!   admitted into the realized read scope and keyed as that entity's fact,
//!   so a split, merge, replacement or retirement ending it stales the
//!   decision through ordinary entity-lifecycle invalidation;
//! - an answer that also rested on ended names on the walked path (which a
//!   later `Restore` revives), on a held name not yet born, or on a relation
//!   record has no Query fact key for that part, so the projection refuses to
//!   seal with [`FailedDecisionRead::PersistentName`];
//! - a name read that failed vetoes sealing the same way, even when the
//!   handler swallowed the error.

use std::collections::BTreeMap;
use std::fmt;

use worth_query_declaration::facade::application_naming::{
    NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_execution_naming::facade::{
    ExecutionNameReadDenial, ExecutionNameReader, ExecutionNameResolveDenial, NameReadDependencies,
};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationSchema, OperationReads,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::LineageRecordTarget;

use super::current_output::WorthQueryCurrentOutputDenial;
use super::decision_plan::WorthQueryInvariantDecisionPlanDenial;
use super::decision_reads::FailedDecisionRead;
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;

/// Why a decision that read a persistent name refuses to seal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistentNameReadVeto {
    /// A name read failed; its outcome rests on nothing a fact retains.
    ReadFailed,
    /// The answer rested on a dependency no Query fact key tracks: an ended
    /// name a later `Restore` revives, an unborn held name, or a relation.
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
    /// and a pruned basis are failed reads; otherwise an answer resting on
    /// what no fact key tracks (`untracked`) is an untracked dependency.
    pub fn for_resolve<H, K>(
        resolved: &Result<ResolvedName<H>, ExecutionNameResolveDenial<K>>,
        untracked: bool,
    ) -> Option<Self> {
        match resolved {
            Err(ExecutionNameResolveDenial::Read(_))
            | Err(ExecutionNameResolveDenial::Resolution(NameResolutionDenial::BasisPruned)) => {
                Some(Self::ReadFailed)
            }
            _ if untracked => Some(Self::UntrackedDependency),
            _ => None,
        }
    }
}

/// Why a decision reader refused a persistent-name read before reading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryDecisionNameDenial {
    /// The operation did not declare a decision read of the entity kind.
    UndeclaredDecisionTarget(WorthQueryInvariantDecisionPlanDenial),
    /// The identity was issued by another reader, phase or entity kind.
    ForeignIdentity,
    /// Names are read only at a committed basis; an applied candidate has no
    /// published lineage to resolve against.
    CandidateBasis,
    /// A resolved live record could not be admitted as a local identity.
    Identity(WorthQueryCurrentOutputDenial),
    /// The name read itself refused.
    Read(ExecutionNameReadDenial),
}

impl fmt::Display for WorthQueryDecisionNameDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndeclaredDecisionTarget(denial) => write!(formatter, "{denial:?}"),
            Self::ForeignIdentity => {
                formatter.write_str("entity identity is foreign to this reader")
            }
            Self::CandidateBasis => {
                formatter.write_str("persistent names are read only at a committed basis")
            }
            Self::Identity(denial) => write!(formatter, "{denial:?}"),
            Self::Read(denial) => fmt::Display::fmt(denial, formatter),
        }
    }
}

impl std::error::Error for WorthQueryDecisionNameDenial {}

type Resolution<Schema, Entity> = Result<
    ResolvedName<WorthQueryInvariantEntityIdentity<Schema, Entity>>,
    ExecutionNameResolveDenial<Entity>,
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
        let named = {
            let reader = self.name_reader()?;
            reader.entity_name_of::<Schema, Entity>(
                identity.entity_id,
                &mut NameReadDependencies::new(),
            )
        };
        if let Some(veto) = PersistentNameReadVeto::for_name_of(&named) {
            self.decision_facts.veto_persistent_name(veto);
        }
        // A record's name changes only when its lineage ends, which ends the
        // record: the entity fact tracks the answer and `RecordNotNamed`.
        self.live_identity::<Entity>(identity.entity_id)?;
        named.map_err(WorthQueryDecisionNameDenial::Read)
    }

    /// Resolves a held entity name at this reader's basis under `selection`,
    /// as a tracked decision read. Resolved records come back as local
    /// identities the decision can read further. The outer result is the
    /// decision admission; the inner one is resolution's own typed outcome.
    pub fn resolve<Entity>(
        &mut self,
        held: &WorthQueryName<Entity>,
        selection: &NameSelection,
    ) -> Result<Resolution<Schema, Entity>, WorthQueryDecisionNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        self.admit_name_read(Entity::IDENTIFIER)?;
        let mut dependencies = NameReadDependencies::new();
        let (resolved, branch) = {
            let reader = self.name_reader()?;
            let resolved =
                reader.resolve_entity::<Schema, Entity>(held, selection, &mut dependencies);
            (resolved, reader.branch().clone())
        };
        let untracked = !dependencies.ended_lineages_on(&branch).is_empty()
            || dependencies.awaits_births_on(&branch)
            || matches!(
                resolved,
                Err(ExecutionNameResolveDenial::Resolution(
                    NameResolutionDenial::UnknownName
                ))
            );
        if let Some(veto) = PersistentNameReadVeto::for_resolve(&resolved, untracked) {
            self.decision_facts.veto_persistent_name(veto);
        }
        let mut identities = BTreeMap::new();
        for record in dependencies.live_records_on(&branch) {
            match record {
                LineageRecordTarget::Entity(entity_id) => {
                    identities.insert(entity_id, self.live_identity::<Entity>(entity_id)?);
                }
                LineageRecordTarget::Relation(_) => {
                    self.decision_facts
                        .veto_persistent_name(PersistentNameReadVeto::UntrackedDependency);
                }
            }
        }
        Ok(resolved.map(|resolved| local_identities(resolved, &identities)))
    }

    fn admit_name_read(&self, entity: &str) -> Result<(), WorthQueryDecisionNameDenial> {
        self.admit_decision_target(&ApplicationOperationDecisionReadTarget::Entity {
            entity: entity.to_owned(),
        })
        .map_err(WorthQueryDecisionNameDenial::UndeclaredDecisionTarget)
    }

    fn name_reader(&self) -> Result<ExecutionNameReader<'_>, WorthQueryDecisionNameDenial> {
        let runtime = self
            .reader
            .native
            .committed_runtime()
            .ok_or(WorthQueryDecisionNameDenial::CandidateBasis)?;
        Ok(ExecutionNameReader::at_observation(
            runtime,
            self.reader.execution,
            &self.reader.observation,
        ))
    }

    /// Admits a live record into the realized read scope as a local identity
    /// and keys it as the entity's decision fact.
    fn live_identity<Entity>(
        &mut self,
        entity_id: EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryDecisionNameDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        let execution = *self.reader.execution;
        self.live_current_identity(&execution, Entity::IDENTIFIER, entity_id)
            .map_err(|denial| {
                self.decision_facts
                    .veto_persistent_name(PersistentNameReadVeto::ReadFailed);
                WorthQueryDecisionNameDenial::Identity(denial)
            })
    }
}

impl FailedDecisionRead {
    pub const fn persistent_name(veto: PersistentNameReadVeto) -> Self {
        Self::PersistentName(veto)
    }
}

fn local_identities<Schema, Entity>(
    resolved: ResolvedName<EntityId>,
    identities: &BTreeMap<EntityId, WorthQueryInvariantEntityIdentity<Schema, Entity>>,
) -> ResolvedName<WorthQueryInvariantEntityIdentity<Schema, Entity>> {
    let local = |entity: EntityId| {
        identities
            .get(&entity)
            .cloned()
            .expect("every resolved record is a live footprint record admitted above")
    };
    match resolved {
        ResolvedName::Exact(entity) => ResolvedName::Exact(local(entity)),
        ResolvedName::Plural(entities) => {
            ResolvedName::Plural(entities.into_iter().map(local).collect())
        }
        ResolvedName::Retired { at_commit } => ResolvedName::Retired { at_commit },
    }
}
