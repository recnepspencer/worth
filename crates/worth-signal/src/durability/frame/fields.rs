use super::SignalDurableFrameDenial as D;
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableCompositePublicationAttemptIdentity, DurableIdentity,
    DurableProductBranchIdentity, DurableProductBranchIncarnation, DurableSignalBranchIdentity,
    DurableSignalConditionalDefinitionOccurrence, DurableSignalConditionalOccurrence,
    DurableSignalNodeIdentity, DurableSignalOwnerIdentity, DurableWorldOwnerIdentity, HomeIdentity,
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
    pub(in crate::durability) fn conditional_definition(
        &mut self,
        row: DurableSignalConditionalDefinitionOccurrence,
    ) -> Result<(), D> {
        durable(row);
        self.branch(row.node().branch())?;
        self.number(row.node().index() as u64)?;
        self.number(row.node().generation() as u64)?;
        self.owner(row.occurrence().owner())?;
        self.number(row.occurrence().ordinal())
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
    pub(in crate::durability) fn conditional_definition(
        &mut self,
    ) -> Result<DurableSignalConditionalDefinitionOccurrence, D> {
        let branch = self.branch()?;
        let index = self.number()?.try_into().map_err(|_| D::Malformed)?;
        let generation = self.number()?.try_into().map_err(|_| D::Malformed)?;
        let node = DurableSignalNodeIdentity::from_recorded(branch, index, generation);
        let owner = self.owner()?;
        let ordinal = self.number()?;
        if ordinal == 0 {
            return Err(D::Malformed);
        }
        DurableSignalConditionalDefinitionOccurrence::from_recorded(
            node,
            DurableSignalConditionalOccurrence::from_recorded(owner, ordinal),
        )
        .map_err(|_| D::ForeignOwner)
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
    pub(in crate::durability) fn association_fields(
        &mut self,
    ) -> Result<Option<AssociationFields<'a>>, D> {
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
                let name = std::str::from_utf8(name).map_err(|_| D::Malformed)?;
                if name.trim().is_empty() {
                    return Err(D::Malformed);
                }
                let incarnation =
                    DurableProductBranchIncarnation::from_recorded(owner, self.number()?);
                Ok(Some(AssociationFields {
                    attempt,
                    name,
                    incarnation,
                }))
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
pub(in crate::durability) struct AssociationFields<'a> {
    pub(in crate::durability) attempt: DurableCompositePublicationAttemptIdentity,
    pub(in crate::durability) name: &'a str,
    incarnation: DurableProductBranchIncarnation,
}
impl AssociationFields<'_> {
    pub(in crate::durability) fn materialize(self) -> Result<CompositeAttemptAssociation, D> {
        let branch =
            DurableProductBranchIdentity::from_recorded(self.attempt.owner(), self.name.to_owned())
                .map_err(|_| D::Malformed)?;
        CompositeAttemptAssociation::from_recorded(self.attempt, branch, self.incarnation)
            .map_err(|_| D::Malformed)
    }
}
