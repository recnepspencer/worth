//! Native custody for the surviving writers that consume an authored retirement.
use super::*;
use crate::publication::ownership::authored_retirement::SelectedAuthoredRetirement;
use contracts::execution::request_stage::checkpoint;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::record_ids::EntityId;
use contracts::identity::data::record_ids::RelationId;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::publication::data::publication_error::PublicationError;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::primitives::RecordRef;
use std::{collections::BTreeSet, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_contracts::internal_port as contracts;
pub mod corrective_root;
pub mod edit;
pub mod member_edit;
pub mod stages;

// Request-local correspondence only. It never admits a retirement or publication.
pub(crate) type AuthoredRootCorrespondence = (
    CommitId,
    Arc<
        worth_execution::ExecutionArray<(
            EntityId,
            contracts::transactions::data::primitives::CreatedEntityRef,
        )>,
    >,
);

/// Canonical predecessor facts, issued only after the shared closure selection.
/// A submitted connection roster cannot construct or enlarge this owner set.
#[derive(Clone)]
pub struct RelationalAuthoredCoupledOwners {
    pub(crate) before: AdmittedRelationalBranchBasis,
    pub(crate) source: CoupledSource,
    pub(crate) retirements: Vec<RelationalRequiredOutputPublication>,
    pub(crate) owners: Vec<RelationalRequiredOutputPublication>,
    /// Current owners of later edit stages' sources under a Retirement or
    /// Restoration prefix. Like a lone edit's, they survive and may republish,
    /// but no later stage is required to (public #348).
    pub(crate) survivors: Vec<RelationalRequiredOutputPublication>,
    pub(crate) current: Vec<worth_relational_branch::internal_port::history::data::codec_retirement_closure::output_correspondence::SelectedOutputPublication>,
    pub(crate) deferred_edges: BTreeSet<RelationId>,
    /// Sources of authored stages 2..k in envelope order, also when the
    /// genuine prefix is a Retirement/Restoration. Empty for a single writer.
    pub(crate) stages: Vec<EntityId>,
    pub(crate) corrective_root: Option<AuthoredRootCorrespondence>,
}

/// The authored prefix has an explicit meaning. Ordinary edits cannot supply
/// retirement evidence, and a restoration retains its actual inverse closure.
#[derive(Clone)]
pub enum CoupledSource {
    Edit(EntityId),
    Retirement {
        anchor: EntityId,
        birth: CommitId,
        co_anchors: Vec<(EntityId, CommitId)>,
        roles: Vec<RequiredOutputRoleTarget>,
    },
    Restoration {
        anchor: EntityId,
        plan: Arc<RelationalAuthoredRestoration>,
    },
    MemberEditRetirement {
        source: EntityId,
        anchor: EntityId,
        birth: CommitId,
        co_anchors: Vec<(EntityId, CommitId)>,
        roles: Vec<RequiredOutputRoleTarget>,
    },
    MemberEditRestoration {
        source: EntityId,
        plan: Arc<RelationalAuthoredRestoration>,
    },
}
impl CoupledSource {
    pub fn subject(&self) -> EntityId {
        match self {
            Self::Edit(source) => *source,
            Self::Retirement { anchor, .. } => *anchor,
            Self::Restoration { anchor, .. } => *anchor,
            Self::MemberEditRetirement { source, .. }
            | Self::MemberEditRestoration { source, .. } => *source,
        }
    }
    pub fn restoration(&self) -> Option<&Arc<RelationalAuthoredRestoration>> {
        match self {
            Self::Restoration { plan, .. } | Self::MemberEditRestoration { plan, .. } => Some(plan),
            _ => None,
        }
    }
    pub fn is_edit(&self) -> bool {
        matches!(
            self,
            Self::Edit(_) | Self::MemberEditRetirement { .. } | Self::MemberEditRestoration { .. }
        )
    }
    pub(crate) fn is_member_edit(&self) -> bool {
        matches!(
            self,
            Self::MemberEditRetirement { .. } | Self::MemberEditRestoration { .. }
        )
    }
}

impl std::fmt::Debug for RelationalAuthoredCoupledOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalAuthoredCoupledOwners")
            .field("source", &self.source.subject())
            .field("owner_count", &self.owners.len())
            .field("survivor_count", &self.survivors.len())
            .field("retirement_count", &self.retirements.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy)]
enum Recorded {
    Owner,
    Survivor,
    Retiring,
}

/// Native classification of an installed declared producer route.
pub enum RelationalDeclaredCoupledOwner<'a> {
    Surviving(&'a RelationalRequiredOutputPublication),
    RequiredSurviving(&'a RelationalRequiredOutputPublication),
    Restored(RelationalRequiredOutputInput),
    Retiring,
}

impl RelationalAuthoredCoupledOwners {
    /// Independent publication is legal only when canonical history requires no surviving writer.
    pub fn require_independent(&self) -> Result<(), TransactionCommitError> {
        if let Some(owner) = self.required_owners().first() {
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
                for (publications, posture) in [
                    (&self.owners, Recorded::Owner),
                    (&self.survivors, Recorded::Survivor),
                    (&self.retirements, Recorded::Retiring),
                ] {
                    for owner in publications {
                        checkpoint(
                            context,
                            (family.len() + owner.origin.meaning.family.len() + 65) as u64,
                        )
                        .map_err(custody::execution_stop)?;
                        let meaning = &owner.origin.meaning;
                        if meaning.family == family
                            && meaning.source_scope == source
                            && meaning.source_partition.bytes() == partition
                        {
                            return Ok(Some(match posture {
                                Recorded::Retiring => RelationalDeclaredCoupledOwner::Retiring,
                                Recorded::Owner if !matches!(self.source, CoupledSource::Edit(_)) => {
                                    RelationalDeclaredCoupledOwner::RequiredSurviving(owner)
                                }
                                Recorded::Owner | Recorded::Survivor => {
                                    RelationalDeclaredCoupledOwner::Surviving(owner)
                                }
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
        self.required_owners().len()
    }

    /// An ordinary edit dispatches its declared synchronous graph even when no
    /// output exists yet. Retirement keeps its established exact-owner trigger.
    pub fn requires_coupled_assembly(&self) -> bool {
        self.source.is_edit() || !self.owners.is_empty()
    }

    pub(crate) fn required_owners(&self) -> &[RelationalRequiredOutputPublication] {
        if matches!(self.source, CoupledSource::Edit(_)) {
            &[]
        } else {
            &self.owners
        }
    }
}

impl AuthoredPublicationReads<'_> {
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
                } = crate::publication::ownership::authored_retirement::select_authored_source_retirement(
                    execution,
                    self.history,
                    installed,
                    basis,
                    anchor,
                    co_anchors,
                    roles,
                    crate::publication::ownership::authored_retirement::AuthoredRetirementSelection::ClosureAndLinkedOwners,
                )?;
                let readmit = |selected: &worth_relational_branch::internal_port::history::data::codec_retirement_closure::output_correspondence::SelectedOutputPublication| {
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
                    source: CoupledSource::Retirement {
                        anchor,
                        birth,
                        co_anchors,
                        roles: roles.to_vec(),
                    },
                    retirements,
                    owners,
                    survivors: Vec::new(),
                    current,
                    deferred_edges,
                    stages: Vec::new(),
                    corrective_root: None,
                })
            })
            .map_err(custody::execution_stop)?
    }
}

#[doc(hidden)]
pub trait RelationalAuthoredCoupledOwnersFieldsPort: Sized {
    fn corrective_root(&self) -> &Option<AuthoredRootCorrespondence>;
    fn stages(&self) -> &Vec<EntityId>;

    fn before(&self) -> &AdmittedRelationalBranchBasis;
    fn retirements(&self) -> &Vec<RelationalRequiredOutputPublication>;
    fn source(&self) -> &CoupledSource;
}

impl RelationalAuthoredCoupledOwnersFieldsPort for RelationalAuthoredCoupledOwners {
    fn corrective_root(&self) -> &Option<AuthoredRootCorrespondence> {
        &self.corrective_root
    }

    fn stages(&self) -> &Vec<EntityId> {
        &self.stages
    }

    fn before(&self) -> &AdmittedRelationalBranchBasis {
        &self.before
    }
    fn retirements(&self) -> &Vec<RelationalRequiredOutputPublication> {
        &self.retirements
    }
    fn source(&self) -> &CoupledSource {
        &self.source
    }
}

#[doc(hidden)]
pub trait RelationalAuthoredCoupledOwnersOwnerPort: Sized {
    fn bind_restoration(
        before: AdmittedRelationalBranchBasis,
        source: CoupledSource,
        retirements: Vec<RelationalRequiredOutputPublication>,
        owners: Vec<RelationalRequiredOutputPublication>,
        current: Vec<worth_relational_branch::internal_port::history::data::codec_retirement_closure::output_correspondence::SelectedOutputPublication>,
        deferred_edges: BTreeSet<RelationId>,
        stages: Vec<EntityId>,
        corrective_root: Option<AuthoredRootCorrespondence>,
    ) -> Self;

    fn recorded_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: EntityId,
        partition: &[u8; 32],
    ) -> Result<Option<RelationalDeclaredCoupledOwner<'_>>, TransactionCommitError>;
}

impl RelationalAuthoredCoupledOwnersOwnerPort for RelationalAuthoredCoupledOwners {
    fn bind_restoration(
        before: AdmittedRelationalBranchBasis,
        source: CoupledSource,
        retirements: Vec<RelationalRequiredOutputPublication>,
        owners: Vec<RelationalRequiredOutputPublication>,
        current: Vec<worth_relational_branch::internal_port::history::data::codec_retirement_closure::output_correspondence::SelectedOutputPublication>,
        deferred_edges: BTreeSet<RelationId>,
        stages: Vec<EntityId>,
        corrective_root: Option<AuthoredRootCorrespondence>,
    ) -> Self {
        Self {
            before,
            source,
            retirements,
            owners,
            survivors: Vec::new(),
            current,
            deferred_edges,
            stages,
            corrective_root,
        }
    }

    fn recorded_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: EntityId,
        partition: &[u8; 32],
    ) -> Result<Option<RelationalDeclaredCoupledOwner<'_>>, TransactionCommitError> {
        self.recorded_owner(execution, family, source, partition)
    }
}
