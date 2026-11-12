//! One recorded semantic claim and its complete declared target set.
use super::{fields::Reader, BridgeAuthoredSectionDenial as D};
use worth_foundational::facade::BridgeSemanticClaimReference;

pub(super) struct RegistrationView<'a> {
    pub(super) reference: BridgeSemanticClaimReference,
    pub(super) meaning: &'a [u8],
    pub(super) targets: &'a [u8],
}
impl<'a> RegistrationView<'a> {
    pub(super) fn read(input: &mut Reader<'a>) -> Result<Self, D> {
        Ok(Self {
            reference: input.claim()?,
            meaning: input.blob()?,
            targets: input.blob()?,
        })
    }
}
