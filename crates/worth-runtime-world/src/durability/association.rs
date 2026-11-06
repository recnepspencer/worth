use super::{WorldDurableFrameDenial, WorldDurableMediumDenial};
use worth_execution::{ExecutionArray, ExecutionRequest};
use worth_foundational::facade::{CompositeAttemptAssociation, HomeIdentity};
mod home_transition;
pub(crate) use home_transition::encode_home_transition_association;
use worth_foundational::facade::HomeProgramTransitionCaller;
const MAGIC: &[u8; 8] = b"WWASSOC1";
pub(crate) fn is_world_association(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC) || HomeProgramTransitionCaller::recognizes(bytes)
}
pub(crate) fn encode_association(
    execution: &ExecutionRequest<'_, '_>,
    association: &CompositeAttemptAssociation,
) -> Result<ExecutionArray<u8>, WorldDurableMediumDenial> {
    super::codec::encode_payload(MAGIC, execution, &mut |out| {
        super::codec::write_association(out, association)
    })
}
pub(crate) fn decode_association(
    bytes: &[u8],
    home: HomeIdentity,
) -> Result<CompositeAttemptAssociation, WorldDurableFrameDenial> {
    let bytes = if HomeProgramTransitionCaller::recognizes(bytes) {
        let described =
            HomeProgramTransitionCaller::read(bytes).map_err(home_transition::frame_denial)?;
        if described.step().transition().home() != home {
            return Err(WorldDurableFrameDenial::WrongHome);
        }
        described.original()
    } else {
        bytes
    };
    let payload = super::codec::payload(bytes, MAGIC)?;
    let mut read = super::codec::Read::new(payload, home);
    let association = read.association()?;
    read.finished()?;
    Ok(association)
}
