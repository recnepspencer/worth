//! Retained fresh decoders keep their real prototype providers alive after a successor.
use super::retention::{self, BridgeRetentionDenial as D};
use super::{BridgeConditionalProviderRole, BridgeConditionalProviderSet};
use std::sync::Arc;
impl BridgeConditionalProviderSet {
    pub(super) fn authored_retained_arc_bytes(&self) -> Result<u64, D> {
        let mut bytes = retention::array_charge::<
            super::authored_provider::BridgeAuthoredProviderBinding,
        >(self.decoder_sources.capacity())?;
        for binding in &self.decoder_sources {
            bytes = bytes
                .checked_add(binding.retained_bytes()?)
                .ok_or(D::BytesExhausted)?;
            let active = match binding.role {
                BridgeConditionalProviderRole::Condition => self
                    .condition
                    .as_ref()
                    .map(|p| Arc::as_ptr(p) as *const () as usize),
                BridgeConditionalProviderRole::DependencyComparator => self
                    .dependency_comparator
                    .as_ref()
                    .map(|p| Arc::as_ptr(p) as *const () as usize),
                BridgeConditionalProviderRole::OutputComparator => self
                    .output_comparator
                    .as_ref()
                    .map(|p| Arc::as_ptr(p) as *const () as usize),
                BridgeConditionalProviderRole::ArtifactReuseComparator => self
                    .reuse_comparator
                    .as_ref()
                    .map(|p| Arc::as_ptr(p) as *const () as usize),
                BridgeConditionalProviderRole::Trigger => self
                    .trigger
                    .as_ref()
                    .map(|p| Arc::as_ptr(p) as *const () as usize),
                BridgeConditionalProviderRole::Wake => self
                    .wake
                    .as_ref()
                    .map(|p| Arc::as_ptr(p) as *const () as usize),
                BridgeConditionalProviderRole::Compute => self
                    .compute
                    .as_ref()
                    .map(|p| Arc::as_ptr(p) as *const () as usize),
            };
            if active != Some(binding.provider_address) {
                bytes = retention::sum(&[
                    bytes,
                    binding.provider_arc_bytes,
                    binding.provider_heap.map_err(|_| D::BytesExhausted)?,
                ])?;
            }
        }
        for role in BridgeConditionalProviderRole::ALL {
            let Some(source) = self.authored.get(role) else {
                continue;
            };
            if !self
                .decoder_sources
                .iter()
                .any(|binding| std::ptr::eq(source.protocol(), binding.description.protocol()))
            {
                bytes = retention::sum(&[
                    bytes,
                    retention::arc_value_charge(source.protocol())?,
                    source.protocol().owned_allocation_capacity_bytes() as u64,
                ])?;
            }
            if !self
                .decoder_sources
                .iter()
                .any(|binding| std::ptr::eq(source.payload(), binding.description.payload()))
            {
                bytes = bytes
                    .checked_add(retention::arc_value_charge(source.payload())?)
                    .ok_or(D::BytesExhausted)?;
            }
        }
        Ok(bytes)
    }
}
