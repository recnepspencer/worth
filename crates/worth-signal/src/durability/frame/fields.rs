use super::SignalDurableFrameDenial as D;
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableCompositePublicationAttemptIdentity, DurableIdentity,
    DurableProductBranchIdentity, DurableProductBranchIncarnation, DurableSignalBranchIdentity,
    DurableSignalOwnerIdentity, DurableWorldOwnerIdentity, HomeIdentity,
    SignalDefinitionTransitionIdentity, SignalDefinitionTransitionReference,
    SignalOwnerSlotIdentity, WorldOwnerSlotIdentity,
};
pub(in crate::durability) struct Writer<'a> {
    emit: &'a mut dyn FnMut(&[u8]) -> Result<(), D>,
}
impl<'a> Writer<'a> {
    pub(super) fn new(emit: &'a mut dyn FnMut(&[u8]) -> Result<(), D>) -> Self {
        Self { emit }
    }
    pub(in crate::durability) fn fixed(&mut self, bytes: &[u8]) -> Result<(), D> {
        (self.emit)(bytes)
    }
    pub(in crate::durability) fn number(&mut self, value: u64) -> Result<(), D> {
        self.fixed(&value.to_le_bytes())
    }
    pub(in crate::durability) fn byte(&mut self, value: u8) -> Result<(), D> {
        self.fixed(&[value])
    }
    pub(in crate::durability) fn blob(&mut self, bytes: &[u8]) -> Result<(), D> {
        self.number(bytes.len() as u64)?;
        self.fixed(bytes)
    }
    pub(in crate::durability) fn owner(
        &mut self,
        owner: DurableSignalOwnerIdentity,
    ) -> Result<(), D> {
        durable(owner);
        self.fixed(&owner.home().bytes())?;
        self.number(owner.slot().get())
    }
    pub(in crate::durability) fn branch(
        &mut self,
        branch: DurableSignalBranchIdentity,
    ) -> Result<(), D> {
        durable(branch);
        self.owner(branch.owner())?;
        self.number(branch.ordinal())
    }
    pub(in crate::durability) fn definition(
        &mut self,
        reference: SignalDefinitionTransitionReference,
    ) -> Result<(), D> {
        durable(reference);
        self.owner(reference.identity().owner())?;
        self.number(reference.identity().ordinal())?;
        self.fixed(&reference.digest())
    }
    pub(in crate::durability) fn association(
        &mut self,
        value: Option<&CompositeAttemptAssociation>,
    ) -> Result<(), D> {
        self.byte(u8::from(value.is_some()))?;
        if let Some(value) = value {
            durable(value.clone());
            let owner = value.attempt().owner();
            self.fixed(&owner.home().bytes())?;
            self.number(owner.slot().get())?;
            self.number(value.attempt().ordinal())?;
            self.blob(value.branch().name().as_bytes())?;
            self.number(value.incarnation().ordinal())?;
        }
        Ok(())
    }
}
fn durable<T: DurableIdentity>(_: T) {}
pub(in crate::durability) struct Reader<'a> {
    pub(in crate::durability) bytes: &'a [u8],
    pub(in crate::durability) position: usize,
}
impl<'a> Reader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: super::HEADER,
        }
    }
    pub(in crate::durability) fn fixed<const N: usize>(&mut self) -> Result<[u8; N], D> {
        let (head, tail) = self.bytes.split_first_chunk::<N>().ok_or(D::Malformed)?;
        self.bytes = tail;
        self.position += N;
        Ok(*head)
    }
    pub(in crate::durability) fn number(&mut self) -> Result<u64, D> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }
    pub(in crate::durability) fn byte(&mut self) -> Result<u8, D> {
        Ok(self.fixed::<1>()?[0])
    }
    pub(in crate::durability) fn blob(&mut self) -> Result<&'a [u8], D> {
        let length: usize = self.number()?.try_into().map_err(|_| D::SizeLimit)?;
        let (head, tail) = self.bytes.split_at_checked(length).ok_or(D::Malformed)?;
        self.bytes = tail;
        self.position += length;
        Ok(head)
    }
    pub(in crate::durability) fn owner(&mut self) -> Result<DurableSignalOwnerIdentity, D> {
        let home = HomeIdentity::from_observed_bytes(self.fixed()?).map_err(|_| D::Malformed)?;
        Ok(DurableSignalOwnerIdentity::from_recorded(
            home,
            SignalOwnerSlotIdentity::from_recorded(self.number()?),
        ))
    }
    pub(in crate::durability) fn branch(&mut self) -> Result<DurableSignalBranchIdentity, D> {
        Ok(DurableSignalBranchIdentity::from_recorded(
            self.owner()?,
            self.number()?,
        ))
    }
    pub(in crate::durability) fn definition(
        &mut self,
    ) -> Result<SignalDefinitionTransitionReference, D> {
        let id = SignalDefinitionTransitionIdentity::from_recorded(self.owner()?, self.number()?);
        if id.ordinal() == 0 {
            return Err(D::Malformed);
        }
        Ok(SignalDefinitionTransitionReference::from_recorded(
            id,
            self.fixed()?,
        ))
    }
    pub(in crate::durability) fn association(
        &mut self,
    ) -> Result<Option<CompositeAttemptAssociation>, D> {
        match self.byte()? {
            0 => Ok(None),
            1 => {
                let home =
                    HomeIdentity::from_observed_bytes(self.fixed()?).map_err(|_| D::Malformed)?;
                let owner = DurableWorldOwnerIdentity::from_recorded(
                    home,
                    WorldOwnerSlotIdentity::from_recorded(self.number()?),
                );
                let attempt = DurableCompositePublicationAttemptIdentity::from_recorded(
                    owner,
                    self.number()?,
                );
                let name = self.blob()?;
                if name.len() > 256 {
                    return Err(D::Malformed);
                }
                let name = std::str::from_utf8(name)
                    .map_err(|_| D::Malformed)?
                    .to_owned();
                let branch = DurableProductBranchIdentity::from_recorded(owner, name)
                    .map_err(|_| D::Malformed)?;
                let incarnation =
                    DurableProductBranchIncarnation::from_recorded(owner, self.number()?);
                Ok(Some(
                    CompositeAttemptAssociation::from_recorded(attempt, branch, incarnation)
                        .map_err(|_| D::Malformed)?,
                ))
            }
            _ => Err(D::Malformed),
        }
    }
    pub(in crate::durability) fn finish(self) -> Result<(), D> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(D::Malformed)
        }
    }
}
