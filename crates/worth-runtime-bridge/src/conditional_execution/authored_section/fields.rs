//! Primitive fields of Bridge's authored format; identities stay home-scoped.
use super::BridgeAuthoredSectionDenial as D;
use worth_foundational::facade::{
    BridgeSemanticClaimReference, DurableIdentity, DurableSignalBranchIdentity,
    DurableSignalConditionalDefinitionOccurrence, DurableSignalConditionalOccurrence,
    DurableSignalNodeIdentity, DurableSignalOwnerIdentity, HomeIdentity,
    SignalDefinitionTransitionIdentity, SignalOwnerSlotIdentity,
};
mod json;
#[cfg(test)]
mod tests;
pub(super) const CLAIM_BYTES: usize = 40;

pub(super) struct Writer<'a> {
    emit: &'a mut dyn FnMut(&[u8]) -> Result<(), D>,
    charge: &'a dyn Fn(usize) -> Result<(), D>,
}
impl<'a> Writer<'a> {
    pub(super) fn new(
        emit: &'a mut dyn FnMut(&[u8]) -> Result<(), D>,
        charge: &'a dyn Fn(usize) -> Result<(), D>,
    ) -> Self {
        Self { emit, charge }
    }
    pub(super) fn charge_bytes(&self, bytes: usize) -> Result<(), D> {
        (self.charge)(bytes)
    }
    pub(super) fn fixed(&mut self, bytes: &[u8]) -> Result<(), D> {
        self.charge_bytes(bytes.len())?;
        (self.emit)(bytes)
    }
    pub(super) fn number(&mut self, value: u64) -> Result<(), D> {
        self.fixed(&value.to_le_bytes())
    }
    pub(super) fn byte(&mut self, value: u8) -> Result<(), D> {
        self.fixed(&[value])
    }
    pub(super) fn blob(&mut self, value: &[u8]) -> Result<(), D> {
        self.number(value.len() as u64)?;
        self.fixed(value)
    }
    pub(super) fn text(&mut self, value: &str) -> Result<(), D> {
        self.blob(value.as_bytes())
    }
    pub(super) fn record(
        &mut self,
        emit: &mut dyn FnMut(&mut Writer<'_>) -> Result<(), D>,
    ) -> Result<(), D> {
        let mut length = 0usize;
        emit(&mut Writer::new(
            &mut |bytes| {
                length = length
                    .checked_add(bytes.len())
                    .filter(|length| *length <= super::framing::MAXIMUM_BYTES)
                    .ok_or(D::SizeLimit)?;
                Ok(())
            },
            self.charge,
        ))?;
        self.number(length as u64)?;
        emit(self)
    }
    pub(super) fn json<T: serde::Serialize>(&mut self, value: &T) -> Result<(), D> {
        json::encode(self, value)
    }
    pub(super) fn branch(&mut self, branch: DurableSignalBranchIdentity) -> Result<(), D> {
        durable(branch);
        self.fixed(&branch.owner().home().bytes())?;
        self.number(branch.owner().slot().get())?;
        self.number(branch.ordinal())
    }
    pub(super) fn claim(&mut self, claim: BridgeSemanticClaimReference) -> Result<(), D> {
        durable(claim);
        let definition = claim.definition();
        if definition.ordinal() == 0 {
            return Err(D::Malformed);
        }
        self.fixed(&definition.owner().home().bytes())?;
        self.number(definition.owner().slot().get())?;
        self.number(definition.ordinal())?;
        self.number(claim.ordinal() as u64)
    }
    pub(super) fn node(&mut self, node: DurableSignalNodeIdentity) -> Result<(), D> {
        durable(node);
        self.branch(node.branch())?;
        self.number(node.index() as u64)?;
        self.number(node.generation() as u64)
    }
    pub(super) fn occurrence(
        &mut self,
        occurrence: DurableSignalConditionalDefinitionOccurrence,
    ) -> Result<(), D> {
        durable(occurrence);
        self.node(occurrence.node())?;
        self.number(occurrence.occurrence().ordinal())
    }
}
fn durable<T: DurableIdentity>(_: T) {}

pub(super) struct Reader<'a> {
    remaining: &'a [u8],
}
impl<'a> Reader<'a> {
    pub(super) fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }
    pub(super) fn fixed<const N: usize>(&mut self) -> Result<[u8; N], D> {
        let (value, remaining) = self
            .remaining
            .split_first_chunk::<N>()
            .ok_or(D::Malformed)?;
        self.remaining = remaining;
        Ok(*value)
    }
    pub(super) fn number(&mut self) -> Result<u64, D> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }
    pub(super) fn byte(&mut self) -> Result<u8, D> {
        Ok(self.fixed::<1>()?[0])
    }
    pub(super) fn blob(&mut self) -> Result<&'a [u8], D> {
        let length = self.number()?.try_into().map_err(|_| D::SizeLimit)?;
        let (value, remaining) = self
            .remaining
            .split_at_checked(length)
            .ok_or(D::Malformed)?;
        self.remaining = remaining;
        Ok(value)
    }
    pub(super) fn text(&mut self) -> Result<&'a str, D> {
        std::str::from_utf8(self.blob()?).map_err(|_| D::Malformed)
    }
    pub(super) fn branch(&mut self) -> Result<DurableSignalBranchIdentity, D> {
        let home = HomeIdentity::from_observed_bytes(self.fixed()?).map_err(|_| D::Malformed)?;
        let owner = DurableSignalOwnerIdentity::from_recorded(
            home,
            SignalOwnerSlotIdentity::from_recorded(self.number()?),
        );
        Ok(DurableSignalBranchIdentity::from_recorded(
            owner,
            self.number()?,
        ))
    }
    pub(super) fn node(&mut self) -> Result<DurableSignalNodeIdentity, D> {
        let branch = self.branch()?;
        let index = self.number()?.try_into().map_err(|_| D::Malformed)?;
        let generation = self.number()?.try_into().map_err(|_| D::Malformed)?;
        Ok(DurableSignalNodeIdentity::from_recorded(
            branch, index, generation,
        ))
    }
    pub(super) fn claim(&mut self) -> Result<BridgeSemanticClaimReference, D> {
        let home = HomeIdentity::from_observed_bytes(self.fixed()?).map_err(|_| D::Malformed)?;
        let owner = DurableSignalOwnerIdentity::from_recorded(
            home,
            SignalOwnerSlotIdentity::from_recorded(self.number()?),
        );
        let definition = SignalDefinitionTransitionIdentity::from_recorded(owner, self.number()?);
        if definition.ordinal() == 0 {
            return Err(D::Malformed);
        }
        let ordinal = self.number()?.try_into().map_err(|_| D::SizeLimit)?;
        Ok(BridgeSemanticClaimReference::from_recorded(
            definition, ordinal,
        ))
    }
    pub(super) fn occurrence(&mut self) -> Result<DurableSignalConditionalDefinitionOccurrence, D> {
        let node = self.node()?;
        let ordinal = self.number()?;
        if ordinal == 0 {
            return Err(D::Malformed);
        }
        DurableSignalConditionalDefinitionOccurrence::from_recorded(
            node,
            DurableSignalConditionalOccurrence::from_recorded(node.branch().owner(), ordinal),
        )
        .map_err(|_| D::Malformed)
    }
    pub(super) fn finish(self) -> Result<(), D> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(D::Malformed)
        }
    }
}
