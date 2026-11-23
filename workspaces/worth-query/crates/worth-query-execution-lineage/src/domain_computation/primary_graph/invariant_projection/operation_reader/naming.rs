//! Tracked persistent-name reads on an operation's decision reader.
//!
//! Resolution, kind verification and dependency recording belong to
//! `worth-query-execution-naming`; this module only admits the read against
//! the operation's declared contract and records what the answer rested on
//! as decision facts. A live record's name and every live descendant a
//! resolution found are keyed as that record's entity fact, so committing a
//! split, merge, replacement or retirement that ends one of them stales the
//! decision through ordinary entity-lifecycle invalidation. A read that also
//! walked through ended names, found a held name not yet born, or rested on
//! a relation record has no Query fact key for that part and is recorded as
//! an unkeyed read.

use std::fmt;

use worth_query_declaration::facade::application_naming::{
    NameSelection, ResolvedName, WorthQueryName,
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

use super::decision_plan::WorthQueryInvariantDecisionPlanDenial;
use super::decision_reads::DecisionReadOutcome;
use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;

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
            Self::Read(denial) => fmt::Display::fmt(denial, formatter),
        }
    }
}

impl std::error::Error for WorthQueryDecisionNameDenial {}

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
        let mut dependencies = NameReadDependencies::new();
        let (named, branch) = {
            let reader = self.name_reader()?;
            let named =
                reader.entity_name_of::<Schema, Entity>(identity.entity_id, &mut dependencies);
            (named, reader.branch().clone())
        };
        self.record_name_dependencies(&dependencies, &branch, Entity::IDENTIFIER, named.is_err());
        named.map_err(WorthQueryDecisionNameDenial::Read)
    }

    /// Resolves a held entity name at this reader's basis under `selection`,
    /// as a tracked decision read. The outer result is the decision admission;
    /// the inner one is resolution's own typed outcome.
    pub fn resolve<Entity>(
        &mut self,
        held: &WorthQueryName<Entity>,
        selection: &NameSelection,
    ) -> Result<
        Result<ResolvedName<EntityId>, ExecutionNameResolveDenial<Entity>>,
        WorthQueryDecisionNameDenial,
    >
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
        let failed = matches!(resolved, Err(ExecutionNameResolveDenial::Read(_)));
        self.record_name_dependencies(&dependencies, &branch, Entity::IDENTIFIER, failed);
        Ok(resolved)
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

    /// Keys every live record the reads rested on as its entity fact; marks
    /// the parts no Query fact key names as an unkeyed read.
    fn record_name_dependencies(
        &mut self,
        dependencies: &NameReadDependencies,
        branch: &worth_relational::facade::history::BranchId,
        entity: &str,
        read_failed: bool,
    ) {
        let mut unkeyed = read_failed
            || !dependencies.ended_lineages_on(branch).is_empty()
            || dependencies.awaits_births_on(branch);
        for record in dependencies.live_records_on(branch) {
            match record {
                LineageRecordTarget::Entity(entity_id) => self.decision_facts.record(
                    WorthQueryApplicationFactKey::Entity {
                        entity: entity.to_owned(),
                        entity_id,
                    },
                    DecisionReadOutcome::Observed,
                ),
                LineageRecordTarget::Relation(_) => unkeyed = true,
            }
        }
        if unkeyed {
            self.decision_facts.record_unkeyed_read();
        }
    }
}
