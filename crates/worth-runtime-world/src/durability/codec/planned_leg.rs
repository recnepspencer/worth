//! World transports Bridge's planned-leg section; only Bridge admits its bytes.
use super::fields::{association, number, Emit, Read};
use crate::durability::{
    WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Medium,
    WorldPlannedSignalLegRecord,
};
use worth_execution::ExecutionRequest;
use worth_runtime_bridge::facade::{BridgeAuthoredSectionDenial, BridgePlannedDefinitionLeg};

pub(super) fn write(out: &mut Emit<'_>, planned: &WorldPlannedSignalLegRecord) -> Result<(), Frame> {
    association(out, &planned.association)?;
    number(out, planned.custody.ordinal())?;
    number(out, planned.leg.as_bytes().len() as u64)?;
    out(planned.leg.as_bytes())
}

pub(super) fn read(
    input: &mut Read<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<WorldPlannedSignalLegRecord, Medium> {
    let association = input.association()?;
    let custody =
        worth_foundational::facade::DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
            association.attempt().owner(),
            input.number()?,
        );
    let length = usize::try_from(input.number()?).map_err(|_| Frame::SizeLimit)?;
    let leg = BridgePlannedDefinitionLeg::readmit_bytes(request, input.take(length)?)
        .map_err(bridge_denial)?;
    Ok(WorldPlannedSignalLegRecord {
        association,
        custody,
        leg,
    })
}

/// Bridge's own admission refusal, classified as World frame damage.
fn bridge_denial(cause: BridgeAuthoredSectionDenial) -> Medium {
    use BridgeAuthoredSectionDenial as D;
    match cause {
        D::Request(cause) => Medium::Request(cause),
        D::Backing(cause) => Medium::Backing(cause),
        D::UnsupportedVersion(version) => Frame::UnsupportedVersion(version).into(),
        D::ChecksumMismatch => Frame::DigestMismatch.into(),
        D::SizeLimit => Frame::SizeLimit.into(),
        D::ForeignBranch => Frame::IdentityMismatch.into(),
        D::Malformed
        | D::SignalCarrier(_)
        | D::SignalImage(_)
        | D::SemanticContractEncoding
        | D::Provider(_)
        | D::SignalDefinitionMismatch
        | D::SourceProfileRequired => Frame::Malformed.into(),
    }
}
