//! Installed model declaration, interpreted over selected immutable native facts.
use super::read_denial::RelationalModelOwnershipDenial;
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
    pub domains: Vec<RelationalModelOwnershipDomain>,
    pub relations: Vec<KindId>,
}

impl ModelOwnershipReadPlan {
    pub fn new(mut domains: Vec<RelationalModelOwnershipDomain>) -> Self {
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
    pub fn combine<'a>(owners: impl IntoIterator<Item = &'a Self>) -> Self {
        Self::new(
            owners
                .into_iter()
                .flat_map(|owner| owner.domains.iter().cloned())
                .collect(),
        )
    }

    pub fn entity_kinds(&self) -> Vec<KindId> {
        self.domains
            .iter()
            .flat_map(|domain| domain.entities.iter().copied().chain([domain.root]))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn relation_kinds(&self) -> Vec<KindId> {
        self.relations.clone()
    }
    pub fn declares_relation(&self, kind: KindId) -> bool {
        self.relations.binary_search(&kind).is_ok()
    }
    pub fn relation_lookup_work(&self) -> usize {
        self.relations.len().checked_ilog2().unwrap_or(0) as usize + 1
    }
}

impl ModelOwnershipReadPlan {
    pub fn observe_source_boundary(
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
