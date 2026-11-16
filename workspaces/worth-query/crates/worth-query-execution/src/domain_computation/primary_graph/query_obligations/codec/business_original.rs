//! Distinct World attempt and unpublished references, with their original owner.
use super::super::{QueryManagedCleanupOriginal as Original, QueryObligationDenial as Denial};
use super::Reader;
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, DurableProductUnpublishedOwnerEffectsIdentity,
    DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity,
};
pub(in crate::domain_computation::primary_graph::query_obligations) fn encode(
    bytes: &mut Vec<u8>,
    original: Option<Original>,
) {
    let Some(original) = original else {
        bytes.push(0);
        return;
    };
    let (tag, owner, ordinal) = match original {
        Original::Attempt(original) => (1, original.owner(), original.ordinal()),
        Original::Unpublished(original) => (2, original.owner(), original.ordinal()),
    };
    bytes.push(tag);
    bytes.extend_from_slice(&owner.home().bytes());
    bytes.extend_from_slice(&owner.slot().get().to_le_bytes());
    bytes.extend_from_slice(&ordinal.to_le_bytes());
}
pub(in crate::domain_computation::primary_graph::query_obligations) fn decode(
    reader: &mut Reader<'_>,
) -> Result<Option<Original>, Denial> {
    let tag = reader.take::<1>()?[0];
    if tag == 0 {
        return Ok(None);
    }
    if tag != 1 && tag != 2 {
        return Err(Denial::MalformedFrame);
    }
    let owner = DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes(reader.take()?).map_err(|_| Denial::MalformedFrame)?,
        WorldOwnerSlotIdentity::from_recorded(u64::from_le_bytes(reader.take()?)),
    );
    let ordinal = u64::from_le_bytes(reader.take()?);
    Ok(Some(match tag {
        1 => Original::Attempt(DurableCompositePublicationAttemptIdentity::from_recorded(
            owner, ordinal,
        )),
        2 => Original::Unpublished(
            DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(owner, ordinal),
        ),
        _ => unreachable!("checked original tag"),
    }))
}
