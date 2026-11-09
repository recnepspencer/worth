//! Provider owners describe authored parameters and rebuild them from fresh services.
use super::{BridgeConditionalProviderSemantics, BridgeConditionalProviderSet};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
pub use worth_foundational::facade::{
    BridgeAuthoredProviderDescription, BridgeAuthoredProviderDescriptions,
    BridgeConditionalProviderRole,
};

/// An implementation must describe all behavior-bearing selection/parameters.
/// Reconstruction uses this fresh installed instance's services, not its
/// original authored parameters and never an old executable snapshot.
pub trait BridgeAuthoredConditionalProvider: BridgeConditionalProviderSemantics + Sized {
    fn authored_description(&self) -> BridgeAuthoredProviderDescription;
    fn admit_authored_source_version(
        &self,
        description: &BridgeAuthoredProviderDescription,
    ) -> Result<(), BridgeAuthoredProviderDenial> {
        let expected = self.authored_description();
        if expected.protocol() != description.protocol() {
            return Err(BridgeAuthoredProviderDenial::InvalidPayload);
        }
        worth_foundational::facade::BoundaryProtocolCompatibilityWindow::inclusive(
            expected.version(),
            expected.version(),
        )
        .admit(description.version())
        .map_err(BridgeAuthoredProviderDenial::UnsupportedVersion)?;
        Ok(())
    }
    fn readmit_authored_provider(
        &self,
        description: &BridgeAuthoredProviderDescription,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial>;
}

#[derive(Debug)]
pub enum BridgeAuthoredProviderDenial {
    MemoryOnlyRole(BridgeConditionalProviderRole),
    MissingDecoder(BridgeConditionalProviderRole),
    AmbiguousDecoder(BridgeConditionalProviderRole),
    DescriptionMismatch(BridgeConditionalProviderRole),
    UnsupportedVersion(worth_foundational::facade::BoundaryProtocolUnsupportedVersion),
    InvalidPayload,
    Owner(worth_foundational::facade::AuthoredProviderReconstructionDenial),
    Request(worth_execution::WorkCeilingDenial),
}

type ProviderDecoder = dyn Fn(
        &BridgeAuthoredProviderDescription,
        &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredProviderDenial>
    + Send
    + Sync;

#[derive(Clone)]
pub(super) struct BridgeAuthoredProviderBinding {
    pub(super) role: BridgeConditionalProviderRole,
    pub(super) description: BridgeAuthoredProviderDescription,
    pub(super) decoder: Arc<ProviderDecoder>,
    pub(super) provider_address: usize,
    pub(super) provider_arc_bytes: u64,
    pub(super) provider_heap: Result<u64, super::BridgeConditionalProviderRetentionOverflow>,
}
impl BridgeAuthoredProviderBinding {
    pub(super) fn retained_bytes(&self) -> Result<u64, super::retention::BridgeRetentionDenial> {
        super::retention::sum(&[
            super::retention::arc_value_charge(self.decoder.as_ref())?,
            super::retention::arc_value_charge(self.description.payload())?,
            super::retention::arc_value_charge(self.description.protocol())?,
            u64::try_from(
                self.description
                    .protocol()
                    .owned_allocation_capacity_bytes(),
            )
            .map_err(|_| super::retention::BridgeRetentionDenial::BytesExhausted)?,
        ])
    }
}

impl BridgeConditionalProviderSet {
    /// Adding decoder families is an installation decision. Durable successors
    /// retain this original set and cannot replace it with their own factories.
    pub fn with_authored_provider_decoders(
        mut self,
        prototypes: Self,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        for binding in prototypes.decoder_sources {
            if self.decoder_sources.iter().any(|known| {
                known.role == binding.role
                    && known.description.protocol() == binding.description.protocol()
            }) {
                return Err(BridgeAuthoredProviderDenial::AmbiguousDecoder(binding.role));
            }
            self.decoder_sources.push(binding);
        }
        Ok(self)
    }

    /// Every actual live role must carry its complete authored source.
    pub fn authored_descriptions(
        &self,
    ) -> Result<BridgeAuthoredProviderDescriptions, BridgeAuthoredProviderDenial> {
        if self.condition.is_some() && self.authored.condition.is_none() {
            return Err(BridgeAuthoredProviderDenial::MemoryOnlyRole(
                BridgeConditionalProviderRole::Condition,
            ));
        }
        if self.dependency_comparator.is_some() && self.authored.dependency_comparator.is_none() {
            return Err(BridgeAuthoredProviderDenial::MemoryOnlyRole(
                BridgeConditionalProviderRole::DependencyComparator,
            ));
        }
        if self.output_comparator.is_some() && self.authored.output_comparator.is_none() {
            return Err(BridgeAuthoredProviderDenial::MemoryOnlyRole(
                BridgeConditionalProviderRole::OutputComparator,
            ));
        }
        if self.reuse_comparator.is_some() && self.authored.reuse_comparator.is_none() {
            return Err(BridgeAuthoredProviderDenial::MemoryOnlyRole(
                BridgeConditionalProviderRole::ArtifactReuseComparator,
            ));
        }
        if self.trigger.is_some() && self.authored.trigger.is_none() {
            return Err(BridgeAuthoredProviderDenial::MemoryOnlyRole(
                BridgeConditionalProviderRole::Trigger,
            ));
        }
        if self.wake.is_some() && self.authored.wake.is_none() {
            return Err(BridgeAuthoredProviderDenial::MemoryOnlyRole(
                BridgeConditionalProviderRole::Wake,
            ));
        }
        if self.compute.is_some() && self.authored.compute.is_none() {
            return Err(BridgeAuthoredProviderDenial::MemoryOnlyRole(
                BridgeConditionalProviderRole::Compute,
            ));
        }
        Ok(self.authored.clone())
    }

    pub(super) fn reconstruct_authored(
        &self,
        descriptions: &BridgeAuthoredProviderDescriptions,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        let mut result = Self::new();
        result.decoder_sources = self.decoder_sources.clone();
        for role in BridgeConditionalProviderRole::ALL {
            let Some(description) = descriptions.get(role) else {
                continue;
            };
            let mut matches = self.decoder_sources.iter().filter(|binding| {
                binding.role == role && binding.description.protocol() == description.protocol()
            });
            let binding = matches
                .next()
                .ok_or(BridgeAuthoredProviderDenial::MissingDecoder(role))?;
            if matches.next().is_some() {
                return Err(BridgeAuthoredProviderDenial::AmbiguousDecoder(role));
            }
            execution
                .consult()
                .map_err(BridgeAuthoredProviderDenial::Request)?;
            let mut rebound = (binding.decoder)(description, execution)?;
            if rebound.authored_descriptions()?.get(role) != Some(description) {
                return Err(BridgeAuthoredProviderDenial::DescriptionMismatch(role));
            }
            match role {
                BridgeConditionalProviderRole::Condition => {
                    result.condition = rebound.condition.take();
                    result.semantic_contracts.condition =
                        rebound.semantic_contracts.condition.take();
                    result.authored.condition = rebound.authored.condition.take();
                }
                BridgeConditionalProviderRole::DependencyComparator => {
                    result.dependency_comparator = rebound.dependency_comparator.take();
                    result.semantic_contracts.dependency_comparator =
                        rebound.semantic_contracts.dependency_comparator.take();
                    result.authored.dependency_comparator =
                        rebound.authored.dependency_comparator.take();
                }
                BridgeConditionalProviderRole::OutputComparator => {
                    result.output_comparator = rebound.output_comparator.take();
                    result.semantic_contracts.output_comparator =
                        rebound.semantic_contracts.output_comparator.take();
                    result.authored.output_comparator = rebound.authored.output_comparator.take();
                }
                BridgeConditionalProviderRole::ArtifactReuseComparator => {
                    result.reuse_comparator = rebound.reuse_comparator.take();
                    result.semantic_contracts.reuse_comparator =
                        rebound.semantic_contracts.reuse_comparator.take();
                    result.authored.reuse_comparator = rebound.authored.reuse_comparator.take();
                }
                BridgeConditionalProviderRole::Trigger => {
                    result.trigger = rebound.trigger.take();
                    result.semantic_contracts.trigger = rebound.semantic_contracts.trigger.take();
                    result.authored.trigger = rebound.authored.trigger.take();
                }
                BridgeConditionalProviderRole::Wake => {
                    result.wake = rebound.wake.take();
                    result.semantic_contracts.wake = rebound.semantic_contracts.wake.take();
                    result.authored.wake = rebound.authored.wake.take();
                }
                BridgeConditionalProviderRole::Compute => {
                    result.compute = rebound.compute.take();
                    result.semantic_contracts.compute = rebound.semantic_contracts.compute.take();
                    result.authored.compute = rebound.authored.compute.take();
                }
            }
        }
        Ok(result)
    }

    pub(super) fn reconstruct_authored_successor(
        &self,
        candidate: &Self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeAuthoredProviderDenial> {
        let descriptions = candidate.authored_descriptions()?;
        // Executable successors come from the installed decoder and recorded
        // parameters, never from candidate-supplied callback/factory objects.
        self.reconstruct_authored(&descriptions, execution)
    }
}
