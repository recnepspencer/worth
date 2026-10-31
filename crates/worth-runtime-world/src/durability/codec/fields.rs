use super::super::{
    DurableWorldHead, WorldDurableFrameDenial as Denial, WorldOpenAttemptRecord,
    WorldRelationalEvidence,
};
use worth_foundational::facade::*;
pub(in crate::durability) type Emit<'a> = dyn FnMut(&[u8]) -> Result<(), Denial> + 'a;
pub(in crate::durability) fn number(out: &mut Emit<'_>, value: u64) -> Result<(), Denial> {
    out(&value.to_le_bytes())
}
pub(in crate::durability) fn owner(
    out: &mut Emit<'_>,
    owner: DurableWorldOwnerIdentity,
) -> Result<(), Denial> {
    out(&owner.home().bytes())?;
    number(out, owner.slot().get())
}
pub(in crate::durability) fn branch(
    out: &mut Emit<'_>,
    branch: &DurableProductBranchIdentity,
) -> Result<(), Denial> {
    owner(out, branch.owner())?;
    out(&(branch.name().len() as u16).to_le_bytes())?;
    out(branch.name().as_bytes())
}
pub(in crate::durability) fn association(
    out: &mut Emit<'_>,
    value: &CompositeAttemptAssociation,
) -> Result<(), Denial> {
    branch(out, value.branch())?;
    number(out, value.attempt().ordinal())?;
    number(out, value.incarnation().ordinal())
}
pub(in crate::durability) fn head(
    out: &mut Emit<'_>,
    value: &DurableWorldHead,
) -> Result<(), Denial> {
    branch(out, &value.branch)?;
    number(out, value.incarnation.ordinal())?;
    number(out, value.generation.get())?;
    number(out, value.commit.ordinal())
}
pub(in crate::durability) fn open(
    out: &mut Emit<'_>,
    value: &WorldOpenAttemptRecord,
) -> Result<(), Denial> {
    association(out, &value.association)?;
    number(out, value.commit.ordinal())?;
    number(out, value.custody.ordinal())?;
    head(out, &value.expected)?;
    out(&value.program_revision)?;
    match value.component_intent {
        super::super::WorldOpenComponentIntent::RelationalOnly => out(&[1]),
    }
}
pub(in crate::durability) fn evidence(
    out: &mut Emit<'_>,
    value: &WorldRelationalEvidence,
) -> Result<(), Denial> {
    number(out, value.commit)?;
    number(out, value.position)?;
    out(&value.digest)
}
pub(in crate::durability) struct Read<'a> {
    bytes: &'a [u8],
    home: HomeIdentity,
}
impl<'a> Read<'a> {
    pub(in crate::durability) fn new(bytes: &'a [u8], home: HomeIdentity) -> Self {
        Self { bytes, home }
    }
    pub(in crate::durability) fn fixed<const N: usize>(&mut self) -> Result<[u8; N], Denial> {
        let (next, rest) = self
            .bytes
            .split_first_chunk::<N>()
            .ok_or(Denial::Malformed)?;
        self.bytes = rest;
        Ok(*next)
    }
    pub(in crate::durability) fn number(&mut self) -> Result<u64, Denial> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }
    pub(in crate::durability) fn owner(&mut self) -> Result<DurableWorldOwnerIdentity, Denial> {
        let observed =
            HomeIdentity::from_observed_bytes(self.fixed()?).map_err(|_| Denial::Malformed)?;
        if observed != self.home {
            return Err(Denial::WrongHome);
        }
        Ok(DurableWorldOwnerIdentity::from_recorded(
            observed,
            WorldOwnerSlotIdentity::from_recorded(self.number()?),
        ))
    }
    pub(in crate::durability) fn branch(&mut self) -> Result<DurableProductBranchIdentity, Denial> {
        let owner = self.owner()?;
        let len = u16::from_le_bytes(self.fixed()?) as usize;
        let (name, rest) = self.bytes.split_at_checked(len).ok_or(Denial::Malformed)?;
        let name = std::str::from_utf8(name)
            .map_err(|_| Denial::Malformed)?
            .to_owned();
        self.bytes = rest;
        DurableProductBranchIdentity::from_recorded(owner, name).map_err(|_| Denial::Malformed)
    }
    pub(in crate::durability) fn association(
        &mut self,
    ) -> Result<CompositeAttemptAssociation, Denial> {
        let branch = self.branch()?;
        let owner = branch.owner();
        let attempt =
            DurableCompositePublicationAttemptIdentity::from_recorded(owner, self.number()?);
        let incarnation = DurableProductBranchIncarnation::from_recorded(owner, self.number()?);
        CompositeAttemptAssociation::from_recorded(attempt, branch, incarnation)
            .map_err(|_| Denial::IdentityMismatch)
    }
    pub(in crate::durability) fn head(&mut self) -> Result<DurableWorldHead, Denial> {
        let branch = self.branch()?;
        let owner = branch.owner();
        Ok(DurableWorldHead {
            branch,
            incarnation: DurableProductBranchIncarnation::from_recorded(owner, self.number()?),
            generation: DurableProductReferenceGeneration::from_recorded(self.number()?),
            commit: DurableCompositeCommitIdentity::from_recorded(owner, self.number()?),
        })
    }
    pub(in crate::durability) fn open(&mut self) -> Result<WorldOpenAttemptRecord, Denial> {
        let association = self.association()?;
        let owner = association.attempt().owner();
        let commit = DurableCompositeCommitIdentity::from_recorded(owner, self.number()?);
        let custody =
            DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(owner, self.number()?);
        let expected = self.head()?;
        if association.branch() != &expected.branch
            || association.incarnation() != expected.incarnation
        {
            return Err(Denial::IdentityMismatch);
        }
        let program_revision = self.fixed()?;
        let component_intent = match self.fixed::<1>()? {
            [1] => super::super::WorldOpenComponentIntent::RelationalOnly,
            _ => return Err(Denial::Malformed),
        };
        Ok(WorldOpenAttemptRecord {
            component_intent,
            association,
            commit,
            custody,
            expected,
            program_revision,
        })
    }
    pub(in crate::durability) fn evidence(&mut self) -> Result<WorldRelationalEvidence, Denial> {
        Ok(WorldRelationalEvidence {
            commit: self.number()?,
            position: self.number()?,
            digest: self.fixed()?,
        })
    }
    pub(in crate::durability) fn finished(self) -> Result<(), Denial> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(Denial::Malformed)
        }
    }
}
