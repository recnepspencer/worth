//! Installed model declaration, interpreted over selected immutable native facts.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::{EntityId, KindId};
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;

/// Schema meaning only. Constructing it grants no read or mutation authority.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelationalModelOwnershipDomain {
    pub root: KindId,
    pub relations: Vec<KindId>,
    pub entities: Vec<KindId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelOwnershipReadPlan {
    pub(super) domains: Vec<RelationalModelOwnershipDomain>,
    pub(super) relations: Vec<KindId>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelationalModelOwnershipConflict {
    pub subject: EntityId,
}

impl RelationalModelOwnershipConflict {
    pub const VALUE_IDENTITY: &'static str = "worth.model-ownership.conflict.v1";
    pub fn encode(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }
}

#[derive(Debug)]
pub enum RelationalModelOwnershipDenial {
    ForeignBasis,
    Custody(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError),
    TargetNotCurrent,
    SchemaChanged,
    ModelRootKind { model: EntityId, actual: KindId },
    EntityUnavailable(EntityId),
    Conflict(RelationalModelOwnershipConflict),
    Read(crate::validation::data::structural_read::StructuralReadError),
    Adjacency(crate::validation::data::custom_rule::errors::CustomInvariantTraversalError),
    Traversal(crate::validation::data::structural_walk_denial::StructuralWalkDenialKind),
    Request(worth_execution::WorkCeilingDenial),
    Native(crate::execution::RelationalExecutionDenialCause),
}

impl ModelOwnershipReadPlan {
    pub(super) fn new(mut domains: Vec<RelationalModelOwnershipDomain>) -> Self {
        for domain in &mut domains {
            domain.relations.sort_unstable();
            domain.relations.dedup();
            domain.entities.sort_unstable();
            domain.entities.dedup();
        }
        domains.sort_unstable();
        domains.dedup();
        let relations = domains
            .iter()
            .flat_map(|domain| domain.relations.iter().copied())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Self { domains, relations }
    }
    pub(super) fn combine<'a>(owners: impl IntoIterator<Item = &'a Self>) -> Self {
        Self::new(
            owners
                .into_iter()
                .flat_map(|owner| owner.domains.iter().cloned())
                .collect(),
        )
    }

    pub(super) fn entity_kinds(&self) -> Vec<KindId> {
        self.domains
            .iter()
            .flat_map(|domain| domain.entities.iter().copied().chain([domain.root]))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub(super) fn relation_kinds(&self) -> Vec<KindId> {
        self.relations.clone()
    }
    pub(super) fn declares_relation(&self, kind: KindId) -> bool {
        self.relations.binary_search(&kind).is_ok()
    }
    pub(super) fn relation_lookup_work(&self) -> usize {
        self.relations.len().checked_ilog2().unwrap_or(0) as usize + 1
    }
}

impl RelationalModelOwnershipDenial {
    /// Keep custody-owner interruptions visible without flattening its typed refusal.
    pub fn kernel_stop(&self) -> Option<worth_execution::MapKernelStop> {
        match self {
            Self::Request(worth_execution::WorkCeilingDenial::Stopped(stop)) => Some(*stop),
            Self::Native(cause) => cause.as_kernel_stop(),
            Self::Custody(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
                denial,
                ..
            }) => {
                let crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause) =
                    denial.kind;
                cause.as_kernel_stop()
            }
            _ => None,
        }
    }
}

impl ModelOwnershipReadPlan {
    pub(crate) fn observe_source_boundary(
        &self,
        runtime_identity: u64,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        subject: EntityId,
        expected_kind: KindId,
        boundary_work: u64,
        contains: &mut dyn FnMut(EntityId, KindId) -> bool,
    ) -> Result<Option<(KindId, Option<EntityId>, bool)>, RelationalModelOwnershipDenial> {
        let (observed, _memory) =
            super::basis::observe(runtime_identity, execution, basis, |read| {
                let Some(kind) = read.kind(subject)? else {
                    return Ok(None);
                };
                if kind != expected_kind {
                    return Ok(Some((kind, None, false)));
                }
                let (owner, within) =
                    self.owner_with_boundary(read, subject, Some((boundary_work, contains)))?;
                Ok(Some((kind, owner, within)))
            })?;
        Ok(observed)
    }
}
