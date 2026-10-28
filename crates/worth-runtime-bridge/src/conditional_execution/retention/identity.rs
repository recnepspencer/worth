//! Retained storage of exact Bridge identities, including typed payload backing.
use super::{arc_slice_charge, sum, BridgeRetentionDenial};

pub(super) fn identity<Tag>(
    identity: &crate::identity::BridgeIdentity<Tag>,
) -> Result<u64, BridgeRetentionDenial> {
    let size = arc_slice_charge::<u8>(identity.as_str().len())?;
    match identity.payload() {
        crate::identity::BridgeIdentityPayload::RelationalBranch { branch_id } => {
            sum(&[size, arc_slice_charge::<u8>(branch_id.len())?])
        }
        _ => Ok(size),
    }
}
