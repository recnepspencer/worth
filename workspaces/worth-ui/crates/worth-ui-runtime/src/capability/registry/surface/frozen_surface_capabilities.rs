use crate::capability::SurfaceId;

use super::{SurfaceAcceptedRegistrationProof, SurfaceDescriptor};

/// Canonical frozen surface capability index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenSurfaceCapabilities {
    descriptors: Vec<SurfaceDescriptor>,
}

impl FrozenSurfaceCapabilities {
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }

    pub(crate) fn from_accepted_descriptors(
        mut descriptors: Vec<SurfaceDescriptor>,
        accepted_surfaces: &SurfaceAcceptedRegistrationProof,
    ) -> Self {
        descriptors.retain(|descriptor| accepted_surfaces.admits(descriptor));
        descriptors.sort_by(|left, right| left.id().cmp(right.id()));
        Self { descriptors }
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn descriptors(&self) -> &[SurfaceDescriptor] {
        &self.descriptors
    }

    pub fn get(&self, id: &SurfaceId) -> Option<&SurfaceDescriptor> {
        self.descriptors
            .binary_search_by(|descriptor| descriptor.id().cmp(id))
            .ok()
            .map(|index| &self.descriptors[index])
    }

    pub(crate) fn digest_basis(&self) -> u64 {
        self.descriptors
            .iter()
            .fold(0x1c04_5d89_e9f3_27ab, fold_surface_descriptor)
    }
}

impl SurfaceDescriptor {
    /// The complete surface basis shared with the frozen capability catalog.
    pub(crate) fn digest_basis(&self) -> u64 {
        fold_surface_descriptor(0x1c04_5d89_e9f3_27ab, self)
    }
}

fn fold_surface_descriptor(accumulator: u64, descriptor: &SurfaceDescriptor) -> u64 {
    let with_id = fold_bytes(accumulator, descriptor.id().as_str().as_bytes());
    let with_kind = fold_bytes(with_id, descriptor.kind().digest_basis().as_bytes());
    let with_component = fold_bytes(with_kind, descriptor.component_id().as_str().as_bytes());
    let with_placement = fold_bytes(
        with_component,
        descriptor.placement_class().digest_basis().as_bytes(),
    );
    let with_state = fold_bytes(
        with_placement,
        descriptor.state_class().digest_basis().as_bytes(),
    );
    let with_command_slots = descriptor.command_slots().iter().fold(
        fold_bytes(with_state, b"command_slots"),
        |accumulator, command_id| fold_list_item(accumulator, command_id.as_str()),
    );
    let with_label = fold_optional_str(with_command_slots, descriptor.label().map(str::to_owned));
    fold_optional_str(
        with_label,
        descriptor
            .view_binding()
            .map(|view_binding| view_binding.as_str().to_owned()),
    )
}

fn fold_list_item(accumulator: u64, value: &str) -> u64 {
    fold_bytes(fold_bytes(accumulator, b"item"), value.as_bytes())
}

fn fold_optional_str(accumulator: u64, value: Option<String>) -> u64 {
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
