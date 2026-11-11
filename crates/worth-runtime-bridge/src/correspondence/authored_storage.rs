//! Cold-installation backing bounds come from the concrete derived index owners.
use super::{BridgeSemanticCorrespondenceRegistration, InstalledCorrespondenceTarget};
use std::mem::size_of;

pub(super) fn tree(slot: usize, count: usize) -> Option<usize> {
    // One node per entry bounds BTree occupancy; include parent/edge storage.
    slot.checked_mul(11)?
        .checked_add(size_of::<usize>().checked_mul(13)?)?
        .checked_add(size_of::<u16>().checked_mul(2)?)?
        .checked_add(std::mem::align_of::<usize>().checked_mul(5)?)?
        .checked_mul(count.checked_add(1)?)
}
pub(crate) fn authored_registration_storage_bytes(
    rows: usize,
    targets: usize,
    text_bytes: usize,
) -> Option<u64> {
    let registration = tree(
        size_of::<worth_foundational::facade::BridgeSemanticClaimReference>()
            .checked_add(size_of::<BridgeSemanticCorrespondenceRegistration>())?,
        rows,
    )?;
    let string_index = tree(size_of::<String>().checked_add(size_of::<usize>())?, rows)?;
    let binding_slots = rows.checked_mul(2)?.checked_add(16)?;
    let currentness = size_of::<(super::BridgeInstalledBindingKey, u64)>()
        .checked_add(1)?
        .checked_mul(binding_slots)?;
    let target_slot = size_of::<super::BridgeSignalAspectTargetDeclaration>()
        .checked_add(size_of::<worth_signal::facade::InstalledSignalNodeCapability>())?
        .checked_add(size_of::<
            worth_signal::facade::InstalledSignalAspectCapability,
        >())?
        .checked_add(size_of::<usize>().checked_mul(4)?)?;
    let target_bytes = target_slot.checked_mul(targets)?;
    // Original registrations, freeze input and sort/resolve materializations can
    // overlap. Text fields are length-delimited in the enclosing authored bytes.
    registration
        .checked_mul(3)?
        .checked_add(string_index.checked_mul(3)?)?
        .checked_add(currentness)?
        .checked_add(target_bytes.checked_mul(4)?)?
        .checked_add(text_bytes.checked_mul(4)?)?
        .try_into()
        .ok()
}
pub(crate) fn authored_binding_key_storage_bytes(
    candidate: &super::BridgeSemanticDependencyCandidate,
    encoded_meaning: usize,
) -> Option<u64> {
    let mut text = encoded_meaning;
    for field in [
        &candidate.source_installation_identity,
        &candidate.source_basis,
        &candidate.source_authority_binding_identity,
        &candidate.graph_participation_identity,
        &candidate.graph_adapter_identity,
    ] {
        text = text.checked_add(field.len())?;
    }
    // Twenty length-delimited fields, each with a possible full-width decimal
    // scalar and length prefix. Record identities have four such scalars each.
    text = text
        .checked_add(20usize.checked_mul(41)?)?
        .checked_add(8usize.checked_mul(21)?)?;
    // Four retained index keys plus six simultaneously formatted/sorted keys;
    // String's geometric growth is bounded separately from final UTF-8 length.
    text.checked_mul(4usize.checked_add(6)?)?
        .checked_mul(2)?
        .try_into()
        .ok()
}
pub(crate) fn authored_performed_storage_bytes(
    targets: usize,
    sources: usize,
    text_bytes: usize,
) -> Option<u64> {
    size_of::<InstalledCorrespondenceTarget>()
        .checked_mul(targets)?
        .checked_mul(4)?
        .checked_add(
            size_of::<super::BridgeSemanticClaimOrigin>()
                .checked_mul(sources)?
                .checked_mul(4)?,
        )?
        .checked_add(
            size_of::<super::BridgeSignalAspectTargetDeclaration>()
                .checked_mul(targets)?
                .checked_mul(4)?,
        )?
        .checked_add(text_bytes.checked_mul(4)?)?
        .try_into()
        .ok()
}
