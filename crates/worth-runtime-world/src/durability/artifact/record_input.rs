//! Borrowed writer inputs are copied only inside the admitted artifact owner.
use super::*;
mod creation;
use crate::durability::{RecordedWorldInitialComponents, WorldDurableMediumDenial as Denial};
pub(crate) use creation::{WorldCreationIntentInput, WorldRelationalCreationInput};

pub(crate) struct WorldHeadInput<'a> {
    pub(crate) owner: DurableWorldOwnerIdentity,
    pub(crate) name: &'a str,
    pub(crate) incarnation: DurableProductBranchIncarnation,
    pub(crate) generation: DurableProductReferenceGeneration,
    pub(crate) commit: DurableCompositeCommitIdentity,
}
impl<'a> WorldHeadInput<'a> {
    pub(crate) fn from_observation(
        observed: &'a crate::branch::ProductBranchObservation,
    ) -> Result<Self, Denial> {
        Self::from_snapshot(observed.snapshot())
    }
    pub(crate) fn from_snapshot(
        snapshot: &'a crate::branch::ProductBranchReferenceSnapshot,
    ) -> Result<Self, Denial> {
        let owner = snapshot
            .owner()
            .durable()
            .ok_or(WorldDurableFrameDenial::IdentityMismatch)?;
        Ok(Self {
            owner,
            name: snapshot.branch().name().as_str(),
            incarnation: DurableProductBranchIncarnation::from_recorded(
                owner,
                snapshot.lifecycle().ordinal(),
            ),
            generation: DurableProductReferenceGeneration::from_recorded(
                snapshot.generation().get(),
            ),
            commit: DurableCompositeCommitIdentity::from_recorded(
                owner,
                snapshot.selected_commit().ordinal(),
            ),
        })
    }
    fn materialize(self) -> Result<DurableWorldHead, Denial> {
        Ok(DurableWorldHead {
            branch: branch(self.owner, self.name)?,
            incarnation: self.incarnation,
            generation: self.generation,
            commit: self.commit,
        })
    }
}

pub(crate) struct WorldOpenAttemptInput<'a> {
    pub(crate) attempt: worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
    pub(crate) name: &'a str,
    pub(crate) incarnation: DurableProductBranchIncarnation,
    pub(crate) commit: DurableCompositeCommitIdentity,
    pub(crate) custody: DurableProductUnpublishedOwnerEffectsIdentity,
    pub(crate) expected: WorldHeadInput<'a>,
    pub(crate) program_revision: [u8; 32],
    pub(crate) component_intent: WorldOpenComponentIntent,
}
impl<'a> WorldOpenAttemptInput<'a> {
    pub(super) fn from_record(open: &'a WorldOpenAttemptRecord) -> Self {
        Self {
            attempt: open.association.attempt(),
            name: open.association.branch().name(),
            incarnation: open.association.incarnation(),
            commit: open.commit,
            custody: open.custody,
            expected: WorldHeadInput {
                owner: open.expected.branch.owner(),
                name: open.expected.branch.name(),
                incarnation: open.expected.incarnation,
                generation: open.expected.generation,
                commit: open.expected.commit,
            },
            program_revision: open.program_revision,
            component_intent: open.component_intent,
        }
    }
    pub(super) fn name_bytes(&self) -> Result<usize, Denial> {
        self.name
            .len()
            .checked_add(self.expected.name.len())
            .ok_or(WorldDurableFrameDenial::SizeLimit.into())
    }
    pub(super) fn materialize(self) -> Result<WorldOpenAttemptRecord, Denial> {
        let association = CompositeAttemptAssociation::from_recorded(
            self.attempt,
            branch(self.attempt.owner(), self.name)?,
            self.incarnation,
        )
        .map_err(|_| WorldDurableFrameDenial::IdentityMismatch)?;
        Ok(WorldOpenAttemptRecord {
            association,
            commit: self.commit,
            custody: self.custody,
            expected: self.expected.materialize()?,
            program_revision: self.program_revision,
            component_intent: self.component_intent,
        })
    }
}

pub(crate) enum WorldArtifactInput<'a> {
    PlannedSignalLeg {
        association: &'a CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
        leg: &'a worth_runtime_bridge::facade::BridgePlannedDefinitionLeg,
    },
    Continued {
        association: &'a CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
    },
    Branch {
        head: WorldHeadInput<'a>,
        bootstrap: worth_foundational::facade::DurableWorldBootstrapAttemptIdentity,
        program_revision: [u8; 32],
        components: RecordedWorldInitialComponents,
    },
    OpenAttempt(WorldOpenAttemptInput<'a>),
    CreationIntent(WorldCreationIntentInput<'a>),
    BranchCreated {
        intent: WorldCreationIntentInput<'a>,
        components: RecordedWorldInitialComponents,
        owners: WorldCreatedBranchOwnerEvidence,
        publishing_incarnation: HomeIncarnation,
    },
    CreationReleased {
        association: &'a CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
    },
    BranchRetired {
        head: WorldHeadInput<'a>,
        retiring_incarnation: HomeIncarnation,
    },
    HistoryRetired {
        owner: DurableWorldOwnerIdentity,
        commits: &'a [DurableCompositeCommitIdentity],
        retiring_incarnation: HomeIncarnation,
    },
    Released {
        association: &'a CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
        relational: Option<super::super::WorldRelationalReleaseEvidence>,
    },
}
impl WorldArtifactInput<'_> {
    pub(super) fn name_bytes(&self) -> Result<usize, Denial> {
        match self {
            Self::Branch { head, .. } | Self::BranchRetired { head, .. } => Ok(head.name.len()),
            Self::HistoryRetired { commits, .. } => commits
                .len()
                .checked_mul(std::mem::size_of::<DurableCompositeCommitIdentity>())
                .ok_or(WorldDurableFrameDenial::SizeLimit.into()),
            Self::OpenAttempt(open) => open.name_bytes(),
            Self::CreationIntent(intent) | Self::BranchCreated { intent, .. } => {
                intent.name_bytes()
            }
            Self::CreationReleased { association, .. } => Ok(association.branch().name().len()),
            Self::Released { association, .. }
            | Self::Continued { association, .. }
            | Self::PlannedSignalLeg { association, .. } => Ok(association.branch().name().len()),
        }
    }
    pub(super) fn materialize(self) -> Result<WorldRecord, Denial> {
        Ok(match self {
            Self::PlannedSignalLeg {
                association,
                custody,
                leg,
            } => WorldRecord::PlannedSignalLeg(super::WorldPlannedSignalLegRecord {
                association: association.clone(),
                custody,
                leg: leg.clone(),
            }),
            Self::Continued {
                association,
                custody,
            } => WorldRecord::Continued {
                association: association.clone(),
                custody,
            },
            Self::Branch {
                head,
                bootstrap,
                program_revision,
                components,
            } => WorldRecord::Branch {
                head: head.materialize()?,
                bootstrap,
                program_revision,
                components,
            },
            Self::OpenAttempt(open) => WorldRecord::OpenAttempt(open.materialize()?),
            Self::CreationIntent(intent) => WorldRecord::CreationIntent(intent.materialize()?),
            Self::BranchCreated {
                intent,
                components,
                owners,
                publishing_incarnation,
            } => {
                let created = WorldBranchCreatedRecord {
                    intent: intent.materialize()?,
                    components,
                    owners,
                    publishing_incarnation,
                };
                created.validate_description()?;
                WorldRecord::BranchCreated(created)
            }
            Self::CreationReleased {
                association,
                custody,
            } => WorldRecord::CreationReleased {
                association: creation::copy_association(association)?,
                custody,
            },
            Self::BranchRetired {
                head,
                retiring_incarnation,
            } => WorldRecord::BranchRetired {
                head: head.materialize()?,
                retiring_incarnation,
            },
            Self::HistoryRetired {
                owner,
                commits,
                retiring_incarnation,
            } => {
                if commits.is_empty() || commits.iter().any(|commit| commit.owner() != owner) {
                    return Err(WorldDurableFrameDenial::IdentityMismatch.into());
                }
                let mut copied = Vec::new();
                copied
                    .try_reserve_exact(commits.len())
                    .map_err(|_| Denial::NameAllocation)?;
                copied.extend_from_slice(commits);
                WorldRecord::HistoryRetired {
                    owner,
                    commits: copied,
                    retiring_incarnation,
                }
            }
            Self::Released {
                association,
                custody,
                relational,
            } => WorldRecord::Released {
                association: CompositeAttemptAssociation::from_recorded(
                    association.attempt(),
                    branch(association.branch().owner(), association.branch().name())?,
                    association.incarnation(),
                )
                .map_err(|_| WorldDurableFrameDenial::IdentityMismatch)?,
                custody,
                relational,
            },
        })
    }
}

fn branch(
    owner: DurableWorldOwnerIdentity,
    name: &str,
) -> Result<DurableProductBranchIdentity, Denial> {
    let copied = copy_name(name)?;
    DurableProductBranchIdentity::from_recorded(owner, copied)
        .map_err(|_| WorldDurableFrameDenial::IdentityMismatch.into())
}

fn copy_name(name: &str) -> Result<String, Denial> {
    // The caller already admitted this exact copy and its containing Arc.
    // The pinned Global/RawVec exact-growth contract is guarded by owner tests.
    let mut copied = String::new();
    copied
        .try_reserve_exact(name.len())
        .map_err(|_| Denial::NameAllocation)?;
    if copied.capacity() != name.len() {
        return Err(Denial::NameAllocation);
    }
    copied.push_str(name);
    Ok(copied)
}
