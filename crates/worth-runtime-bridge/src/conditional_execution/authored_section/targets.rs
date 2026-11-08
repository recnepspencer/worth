//! Declared slots and exact performed correspondence claims are separate fields.
use super::{
    fields::{Reader, Writer},
    BridgeAuthoredSectionDenial as D,
};
use crate::correspondence::{AllocationSlotIdentity, BridgeCorrespondencePrecision};
use crate::input::envelope::BridgeAspectChangeWideningCause;
use worth_foundational::facade::{BridgeSemanticClaimReference, DurableSignalNodeIdentity};
use worth_signal::facade::{Aspect, MAX_ASPECTS};

pub(super) fn write_slot(out: &mut Writer<'_>, slot: AllocationSlotIdentity) -> Result<(), D> {
    match slot {
        AllocationSlotIdentity::Allocate => out.byte(0),
        AllocationSlotIdentity::Exact(aspect) => {
            out.byte(1)?;
            out.byte(u8::try_from(aspect.index()).map_err(|_| D::Malformed)?)
        }
    }
}
fn aspect(input: &mut Reader<'_>) -> Result<Aspect, D> {
    let value = input.byte()?;
    if usize::from(value) >= MAX_ASPECTS {
        return Err(D::Malformed);
    }
    Ok(Aspect::new(value))
}
pub(super) fn read_slot(input: &mut Reader<'_>) -> Result<AllocationSlotIdentity, D> {
    match input.byte()? {
        0 => Ok(AllocationSlotIdentity::Allocate),
        1 => Ok(AllocationSlotIdentity::Exact(aspect(input)?)),
        _ => Err(D::Malformed),
    }
}
pub(super) struct DeclarationView<'a> {
    pub(super) node: DurableSignalNodeIdentity,
    pub(super) partition: &'a str,
    pub(super) mapping: &'a str,
    pub(super) slot: AllocationSlotIdentity,
}
impl<'a> DeclarationView<'a> {
    pub(super) fn read(input: &mut Reader<'a>) -> Result<Self, D> {
        Ok(Self {
            node: input.node()?,
            partition: input.text()?,
            mapping: input.text()?,
            slot: read_slot(input)?,
        })
    }
}
pub(super) struct AllocationView<'a> {
    pub(super) owner: BridgeSemanticClaimReference,
    pub(super) declaration: DeclarationView<'a>,
    pub(super) selected: Aspect,
}
impl<'a> AllocationView<'a> {
    pub(super) fn read(input: &mut Reader<'a>) -> Result<Self, D> {
        let owner = input.claim()?;
        let declaration = DeclarationView::read(input)?;
        let selected = aspect(input)?;
        Ok(Self {
            owner,
            declaration,
            selected,
        })
    }
}
pub(super) fn write_precision(
    out: &mut Writer<'_>,
    precision: BridgeCorrespondencePrecision,
) -> Result<(), D> {
    out.byte(match precision {
        BridgeCorrespondencePrecision::Exact => 0,
        BridgeCorrespondencePrecision::DeclaredWidening => 1,
    })
}
pub(super) fn write_widening(
    out: &mut Writer<'_>,
    cause: Option<BridgeAspectChangeWideningCause>,
) -> Result<(), D> {
    out.byte(match cause {
        None => 0,
        Some(BridgeAspectChangeWideningCause::FieldToWholeAspect) => 1,
        Some(BridgeAspectChangeWideningCause::AspectToEntity) => 2,
        Some(BridgeAspectChangeWideningCause::SurfaceBroadening) => 3,
        Some(BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect) => 4,
    })
}
pub(super) struct PerformedTargetView<'a> {
    pub(super) node: DurableSignalNodeIdentity,
    pub(super) partition: &'a str,
    pub(super) mapping: &'a str,
    pub(super) selected: Aspect,
    pub(super) mapping_basis: &'a str,
    pub(super) precision: BridgeCorrespondencePrecision,
    pub(super) widening: Option<BridgeAspectChangeWideningCause>,
    pub(super) sources: &'a [u8],
}
impl<'a> PerformedTargetView<'a> {
    pub(super) fn read(input: &mut Reader<'a>) -> Result<Self, D> {
        let node = input.node()?;
        let partition = input.text()?;
        let mapping = input.text()?;
        let mapping_basis = input.text()?;
        let selected = aspect(input)?;
        let precision = match input.byte()? {
            0 => BridgeCorrespondencePrecision::Exact,
            1 => BridgeCorrespondencePrecision::DeclaredWidening,
            _ => return Err(D::Malformed),
        };
        let widening = match input.byte()? {
            0 => None,
            1 => Some(BridgeAspectChangeWideningCause::FieldToWholeAspect),
            2 => Some(BridgeAspectChangeWideningCause::AspectToEntity),
            3 => Some(BridgeAspectChangeWideningCause::SurfaceBroadening),
            4 => Some(BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect),
            _ => return Err(D::Malformed),
        };
        let sources = input.blob()?;
        if sources.len() % super::fields::CLAIM_BYTES != 0 {
            return Err(D::Malformed);
        }
        Ok(Self {
            node,
            partition,
            mapping,
            mapping_basis,
            selected,
            precision,
            widening,
            sources,
        })
    }
}
