use super::{WorldOpenAttemptRecord, WorldRecord};
use crate::identity::{RuntimeWorldIdentityExhaustion, RuntimeWorldIdentityFamily};
/// Recovered descriptive maxima, never an issuer or authority by themselves.
#[derive(Debug, Default, Clone)]
pub(crate) struct WorldIdentityFrontiers {
    pub(crate) branch: u64,
    pub(crate) commit: u64,
    pub(crate) bootstrap: u64,
    pub(crate) attempt: u64,
    pub(crate) custody: u64,
}
fn include(
    next: &mut u64,
    value: u64,
    family: RuntimeWorldIdentityFamily,
) -> Result<(), RuntimeWorldIdentityExhaustion> {
    *next = (*next).max(
        value
            .checked_add(1)
            .ok_or(RuntimeWorldIdentityExhaustion::new(family))?,
    );
    Ok(())
}
impl WorldIdentityFrontiers {
    pub(crate) fn include_record(
        &mut self,
        record: &WorldRecord,
    ) -> Result<(), RuntimeWorldIdentityExhaustion> {
        use RuntimeWorldIdentityFamily as Family;
        match record {
            WorldRecord::Owner { .. } => Ok(()),
            WorldRecord::Branch {
                head,
                bootstrap,
                components: _,
                program_revision: _,
            } => {
                include(
                    &mut self.branch,
                    head.incarnation.ordinal(),
                    Family::ProductBranchIncarnation,
                )?;
                include(
                    &mut self.commit,
                    head.commit.ordinal(),
                    Family::CompositeCommit,
                )?;
                include(
                    &mut self.bootstrap,
                    bootstrap.ordinal(),
                    Family::BootstrapAttempt,
                )
            }
            WorldRecord::CreationIntent(intent) => self.include_creation(intent),
            WorldRecord::BranchCreated(created) => self.include_creation(&created.intent),
            WorldRecord::OpenAttempt(open) | WorldRecord::Occurrence { open, .. } => {
                self.include_open(open)
            }
            WorldRecord::Released {
                association,
                custody,
                ..
            }
            | WorldRecord::Continued {
                association,
                custody,
            }
            | WorldRecord::CreationReleased {
                association,
                custody,
            } => {
                include(
                    &mut self.attempt,
                    association.attempt().ordinal(),
                    Family::PublicationAttempt,
                )?;
                include(
                    &mut self.branch,
                    association.incarnation().ordinal(),
                    Family::ProductBranchIncarnation,
                )?;
                include(
                    &mut self.custody,
                    custody.ordinal(),
                    Family::ProductUnpublishedOwnerEffects,
                )
            }
        }
    }
    fn include_creation(
        &mut self,
        intent: &super::artifact::WorldCreationIntentRecord,
    ) -> Result<(), RuntimeWorldIdentityExhaustion> {
        use RuntimeWorldIdentityFamily as Family;
        include(
            &mut self.attempt,
            intent.association.attempt().ordinal(),
            Family::PublicationAttempt,
        )?;
        include(
            &mut self.custody,
            intent.custody.ordinal(),
            Family::ProductUnpublishedOwnerEffects,
        )?;
        for head in [&intent.source, &intent.destination] {
            include(
                &mut self.branch,
                head.incarnation.ordinal(),
                Family::ProductBranchIncarnation,
            )?;
            include(
                &mut self.commit,
                head.commit.ordinal(),
                Family::CompositeCommit,
            )?;
        }
        Ok(())
    }

    fn include_open(
        &mut self,
        open: &WorldOpenAttemptRecord,
    ) -> Result<(), RuntimeWorldIdentityExhaustion> {
        use RuntimeWorldIdentityFamily as Family;
        include(
            &mut self.attempt,
            open.association.attempt().ordinal(),
            Family::PublicationAttempt,
        )?;
        include(
            &mut self.commit,
            open.commit.ordinal(),
            Family::CompositeCommit,
        )?;
        include(
            &mut self.commit,
            open.expected.commit.ordinal(),
            Family::CompositeCommit,
        )?;
        include(
            &mut self.custody,
            open.custody.ordinal(),
            Family::ProductUnpublishedOwnerEffects,
        )?;
        include(
            &mut self.branch,
            open.association.incarnation().ordinal(),
            Family::ProductBranchIncarnation,
        )
    }
}
