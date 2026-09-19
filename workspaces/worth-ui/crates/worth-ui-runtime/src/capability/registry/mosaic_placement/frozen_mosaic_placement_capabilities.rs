use crate::capability::MosaicPlacementPolicyId;

use super::{MosaicPlacementAcceptedRegistrationProof, MosaicPlacementPolicyDescriptor};

/// Canonical frozen mosaic placement policy capability index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenMosaicPlacementCapabilities {
    descriptors: Vec<MosaicPlacementPolicyDescriptor>,
}

impl FrozenMosaicPlacementCapabilities {
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }

    pub(crate) fn from_accepted_descriptors(
        mut descriptors: Vec<MosaicPlacementPolicyDescriptor>,
        accepted_policies: &MosaicPlacementAcceptedRegistrationProof,
    ) -> Self {
        descriptors.retain(|descriptor| accepted_policies.admits(descriptor));
        descriptors.sort_by(|left, right| left.id().cmp(right.id()));
        Self { descriptors }
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn descriptors(&self) -> &[MosaicPlacementPolicyDescriptor] {
        &self.descriptors
    }

    pub fn get(&self, id: &MosaicPlacementPolicyId) -> Option<&MosaicPlacementPolicyDescriptor> {
        self.descriptors
            .binary_search_by(|descriptor| descriptor.id().cmp(id))
            .ok()
            .map(|index| &self.descriptors[index])
    }

    pub(crate) fn digest_basis(&self) -> u64 {
        self.descriptors
            .iter()
            .fold(0x6c8e_9cf5_7b2d_4a31, fold_mosaic_placement_descriptor)
    }
}

impl MosaicPlacementPolicyDescriptor {
    /// The same complete descriptor basis used by the frozen capability catalog.
    pub(crate) fn digest_basis(&self) -> u64 {
        fold_mosaic_placement_descriptor(0x6c8e_9cf5_7b2d_4a31, self)
    }
}

fn fold_mosaic_placement_descriptor(
    accumulator: u64,
    descriptor: &MosaicPlacementPolicyDescriptor,
) -> u64 {
    let with_id = fold_bytes(accumulator, descriptor.id().as_str().as_bytes());
    let with_action = fold_bytes(with_id, descriptor.action().digest_basis().as_bytes());
    let with_source = fold_optional_string(
        with_action,
        descriptor.source().map(|source| source.digest_basis()),
    );
    let with_target = fold_optional_string(
        with_source,
        descriptor.target().map(|target| target.digest_basis()),
    );
    let with_persistence = fold_optional_str(
        with_target,
        descriptor
            .persistence()
            .map(|persistence| persistence.digest_basis()),
    );
    let with_identity = fold_optional_str(
        with_persistence,
        descriptor
            .stable_identity_behavior()
            .map(|identity| identity.digest_basis()),
    );
    let with_conflict = fold_optional_str(
        with_identity,
        descriptor
            .conflict_behavior()
            .map(|conflict| conflict.digest_basis()),
    );
    let with_reload = fold_optional_str(
        with_conflict,
        descriptor
            .reload_reconciliation()
            .map(|reload| reload.digest_basis()),
    );
    let with_support = fold_optional_str(
        with_reload,
        descriptor.support().map(|support| support.digest_basis()),
    );
    fold_optional_str(with_support, descriptor.label())
}

fn fold_optional_string(accumulator: u64, value: Option<String>) -> u64 {
    match value {
        Some(value) => fold_bytes(fold_bytes(accumulator, b"some"), value.as_bytes()),
        None => fold_bytes(accumulator, b"none"),
    }
}

fn fold_optional_str(accumulator: u64, value: Option<&str>) -> u64 {
    match value {
        Some(value) => fold_bytes(fold_bytes(accumulator, b"some"), value.as_bytes()),
        None => fold_bytes(accumulator, b"none"),
    }
}

fn fold_bytes(mut accumulator: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        accumulator ^= u64::from(*byte);
        accumulator = accumulator.wrapping_mul(0x0000_0100_0000_01b3);
    }
    accumulator
}
