//! Native custody for the surviving writers that consume an authored retirement.
use super::*;
use crate::execution::request_stage::checkpoint;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{CommitId, SelectedAuthoredRetirement},
    identity::data::{EntityId, RelationId},
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    runtime::RelationalRuntime,
    transactions::data::RecordRef,
};
use std::{collections::BTreeSet, sync::Arc};
use worth_execution::ExecutionRequest;

/// Canonical predecessor facts, issued only after the shared closure selection.
/// A submitted connection roster cannot construct or enlarge this owner set.
#[derive(Clone)]
pub struct RelationalAuthoredCoupledOwners {
    pub(crate) before: AdmittedRelationalBranchBasis,
    pub(crate) anchor: EntityId,
    pub(crate) birth: CommitId,
    pub(crate) co_anchors: Vec<(EntityId, CommitId)>,
    pub(crate) roles: Vec<RequiredOutputRoleTarget>,
    pub(crate) retirements: Vec<RelationalRequiredOutputPublication>,
    pub(crate) owners: Vec<RelationalRequiredOutputPublication>,
    pub(crate) current: Vec<crate::history::data::SelectedOutputPublication>,
    pub(crate) deferred_edges: BTreeSet<RelationId>,
    pub(crate) restoration: Option<Arc<RelationalAuthoredRestoration>>,
}

impl std::fmt::Debug for RelationalAuthoredCoupledOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalAuthoredCoupledOwners")
            .field("anchor", &self.anchor)
            .field("owner_count", &self.owners.len())
            .field("retirement_count", &self.retirements.len())
            .finish_non_exhaustive()
    }
}

/// Native classification of an installed declared producer route.
pub enum RelationalDeclaredCoupledOwner<'a> {
    Surviving(&'a RelationalRequiredOutputPublication),
    Retiring,
}

impl RelationalAuthoredCoupledOwners {
    /// Independent publication is legal only when canonical history requires no surviving writer.
    pub fn require_independent(&self) -> Result<(), TransactionCommitError> {
        if let Some(owner) = self.owners.first() {
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(
                    Cause::MissingCoupledOwner,
                    RecordRef::Entity(owner.meaning().source_scope),
                ),
            ));
        }
        Ok(())
    }
    /// Select the recorded route before any producer handler executes.
    /// A closure member is already retired by the authored actor; it cannot
    /// also be submitted as a surviving writer.
    pub(super) fn recorded_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: EntityId,
        partition: &[u8; 32],
    ) -> Result<Option<RelationalDeclaredCoupledOwner<'_>>, TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                for (publications, retiring) in [(&self.owners, false), (&self.retirements, true)] {
                    for owner in publications {
                        checkpoint(
                            context,
                            (family.len() + owner.origin.meaning.family.len() + 65) as u64,
                        )
                        .map_err(custody::execution_stop)?;
                        let meaning = &owner.origin.meaning;
                        if meaning.family == family
                            && meaning.source_scope == source
                            && &meaning.source_partition == partition
                        {
                            return Ok(Some(if retiring {
                                RelationalDeclaredCoupledOwner::Retiring
                            } else {
                                RelationalDeclaredCoupledOwner::Surviving(owner)
                            }));
                        }
                    }
                }
                Ok(None)
            })
            .map_err(custody::execution_stop)?
            .0
    }

    pub fn retirement_authorities(
        &self,
    ) -> impl Iterator<Item = &worth_proof::RequiredOutputPublicationAuthority> {
        self.retirements
            .iter()
            .map(|publication| publication.authority())
    }

    pub fn required_owner_count(&self) -> usize {
        self.owners.len()
    }
}

impl RelationalRuntime {
    /// The closure owner authenticates all anchors and current publications once.
    /// Surviving edge consumers are readmitted through the existing custody owner.
    pub fn readmit_authored_coupled_owners(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        anchor: EntityId,
        co_anchors: &[EntityId],
        roles: &[RequiredOutputRoleTarget],
    ) -> Result<RelationalAuthoredCoupledOwners, TransactionCommitError> {
        execution
            .in_scope(|_| {
                let SelectedAuthoredRetirement {
                    birth,
                    co_anchors,
                    retirements,
                    coupled_owners,
                    current,
                } = crate::history::data::select_authored_source_retirement(
                    execution,
                    self,
                    installed,
                    basis,
                    anchor,
                    co_anchors,
                    roles,
                    crate::history::data::AuthoredRetirementSelection::ClosureAndLinkedOwners,
                )?;
                let readmit = |selected: &crate::history::data::SelectedOutputPublication| {
                    self.readmit_canonical_publication(
                        execution,
                        installed,
                        basis,
                        &selected.origin,
                    )
                    .map_err(|error| match error {
                        TransactionCommitError::Publication { error, .. } => {
                            let original_detail = error.detail;
                            let mut error = PublicationError::authored_retirement(
                                Cause::CorruptPublication,
                                RecordRef::Entity(selected.origin.meaning.source_scope),
                            );
                            error.detail.push_str(": ");
                            error.detail.push_str(&original_detail);
                            TransactionCommitError::publication(error)
                        }
                        _ => error,
                    })
                };
                let retirements = retirements
                    .iter()
                    .map(readmit)
                    .collect::<Result<Vec<_>, _>>()?;
                let owners = coupled_owners
                    .iter()
                    .map(readmit)
                    .collect::<Result<Vec<_>, _>>()?;
                let deferred_edges = owners
                    .iter()
                    .flat_map(|owner| owner.origin.relations.iter().map(|edge| edge.id))
                    .collect();
                Ok(RelationalAuthoredCoupledOwners {
                    before: basis.clone(),
                    anchor,
                    birth,
                    co_anchors,
                    roles: roles.to_vec(),
                    retirements,
                    owners,
                    current,
                    deferred_edges,
                    restoration: None,
                })
            })
            .map_err(custody::execution_stop)?
    }
}
