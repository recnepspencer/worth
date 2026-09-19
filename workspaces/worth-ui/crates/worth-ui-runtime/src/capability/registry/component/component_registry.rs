use super::{ComponentAcceptedRegistrationProof, ComponentDescriptor, FrozenComponentCapabilities};

/// Builder-owned component registry lane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ComponentRegistry {
    descriptors: Vec<ComponentDescriptor>,
}

impl ComponentRegistry {
    pub(crate) fn empty() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, descriptor: ComponentDescriptor) {
        self.descriptors.push(descriptor);
    }

    #[allow(
        dead_code,
        reason = "Gate 0 theme admission inspects component descriptors in tests"
    )]
    pub(crate) fn descriptors(&self) -> &[ComponentDescriptor] {
        &self.descriptors
    }

    pub(crate) fn freeze(
        self,
        accepted_components: &ComponentAcceptedRegistrationProof,
    ) -> FrozenComponentCapabilities {
        FrozenComponentCapabilities::from_accepted_descriptors(
            self.descriptors,
            accepted_components,
        )
    }
}
