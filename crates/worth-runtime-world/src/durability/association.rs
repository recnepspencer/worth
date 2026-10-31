use super::{WorldDurableFrameDenial, WorldDurableMediumDenial};
use worth_execution::{ExecutionArray, ExecutionRequest};
use worth_foundational::facade::{CompositeAttemptAssociation, HomeIdentity};
const MAGIC: &[u8; 8] = b"WWASSOC1";
pub(crate) fn is_world_association(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
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
    let payload = super::codec::payload(bytes, MAGIC)?;
    let mut read = super::codec::Read::new(payload, home);
    let association = read.association()?;
    read.finished()?;
    Ok(association)
}
