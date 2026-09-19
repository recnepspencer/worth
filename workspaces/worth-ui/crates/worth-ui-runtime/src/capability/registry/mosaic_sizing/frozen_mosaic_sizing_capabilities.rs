use crate::capability::MosaicSizingContractId;

use super::{MosaicSizingAcceptedRegistrationProof, MosaicSizingContractDescriptor};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenMosaicSizingCapabilities {
    descriptors: Vec<MosaicSizingContractDescriptor>,
}

impl FrozenMosaicSizingCapabilities {
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }

    pub(crate) fn from_accepted_descriptors(
        mut descriptors: Vec<MosaicSizingContractDescriptor>,
        accepted_contracts: &MosaicSizingAcceptedRegistrationProof,
    ) -> Self {
        descriptors.retain(|descriptor| accepted_contracts.admits(descriptor));
        descriptors.sort_by(|left, right| left.id().cmp(right.id()));
        Self { descriptors }
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn descriptors(&self) -> &[MosaicSizingContractDescriptor] {
        &self.descriptors
    }

    pub fn get(&self, id: &MosaicSizingContractId) -> Option<&MosaicSizingContractDescriptor> {
        self.descriptors
            .binary_search_by(|descriptor| descriptor.id().cmp(id))
            .ok()
            .map(|index| &self.descriptors[index])
    }

    pub(crate) fn digest_basis(&self) -> u64 {
        self.descriptors
            .iter()
            .fold(0x8fc2_18a6_b47d_0e35, fold_mosaic_sizing_descriptor)
    }
}

impl MosaicSizingContractDescriptor {
    /// The same complete descriptor basis used by the frozen capability catalog.
    pub(crate) fn digest_basis(&self) -> u64 {
        fold_mosaic_sizing_descriptor(0x8fc2_18a6_b47d_0e35, self)
    }
}

fn fold_mosaic_sizing_descriptor(
    accumulator: u64,
    descriptor: &MosaicSizingContractDescriptor,
) -> u64 {
    let with_id = fold_bytes(accumulator, descriptor.id().as_str().as_bytes());
    let with_kind = fold_bytes(with_id, descriptor.kind().digest_basis().as_bytes());
    let with_measurement = fold_optional_string(
        with_kind,
        descriptor
            .named_measurement()
            .map(|measurement| measurement.digest_basis()),
    );
    let with_authority = fold_optional_str(
        with_measurement,
        descriptor
            .measurement_authority()
            .map(|authority| authority.digest_basis()),
    );
    let with_resize = fold_optional_str(
        with_authority,
        descriptor
            .resize_permission()
            .map(|permission| permission.digest_basis()),
    );
    let with_persistence = fold_optional_str(
        with_resize,
        descriptor
            .persistence()
            .map(|persistence| persistence.digest_basis()),
    );
    let with_overflow = fold_optional_str(
        with_persistence,
        descriptor
            .overflow_behavior()
            .map(|overflow| overflow.digest_basis()),
    );
    let with_growth = fold_optional_str(
        with_overflow,
        descriptor
            .parent_growth_behavior()
            .map(|growth| growth.digest_basis()),
    );
    let with_viewport = fold_optional_str(
        with_growth,
        descriptor
            .viewport_constraint()
            .map(|viewport| viewport.digest_basis()),
    );
    let with_raw = descriptor
        .raw_measurements_for_diagnostics()
        .iter()
        .fold(with_viewport, |basis, raw| {
            fold_bytes(basis, raw.digest_basis().as_bytes())
        });
    fold_optional_str(with_raw, descriptor.label())
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
