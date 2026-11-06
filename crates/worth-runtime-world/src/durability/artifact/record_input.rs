//! Borrowed writer inputs are copied only inside the admitted artifact owner.
use super::*;
use crate::durability::{RecordedWorldInitialComponents, WorldDurableMediumDenial as Denial};

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
        let owner = observed
            .owner_identity()
            .durable()
            .ok_or(WorldDurableFrameDenial::IdentityMismatch)?;
        Ok(Self {
            owner,
            name: observed.branch_identity().name().as_str(),
            incarnation: DurableProductBranchIncarnation::from_recorded(
                owner,
                observed.lifecycle_incarnation().ordinal(),
            ),
            generation: DurableProductReferenceGeneration::from_recorded(
                observed.reference_generation().get(),
            ),
            commit: DurableCompositeCommitIdentity::from_recorded(
                owner,
                observed.selected_commit().ordinal(),
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
    Released {
        association: &'a CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
        relational: Option<super::super::WorldRelationalReleaseEvidence>,
    },
}
impl WorldArtifactInput<'_> {
    pub(super) fn name_bytes(&self) -> Result<usize, Denial> {
        match self {
            Self::Branch { head, .. } => Ok(head.name.len()),
            Self::OpenAttempt(open) => open.name_bytes(),
            Self::Released { association, .. } | Self::Continued { association, .. } => {
                Ok(association.branch().name().len())
            }
        }
    }
    pub(super) fn materialize(self) -> Result<WorldRecord, Denial> {
        Ok(match self {
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
    DurableProductBranchIdentity::from_recorded(owner, copied)
        .map_err(|_| WorldDurableFrameDenial::IdentityMismatch.into())
}
