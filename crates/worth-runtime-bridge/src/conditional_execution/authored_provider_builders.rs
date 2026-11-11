//! Explicit authored role construction captures the actual provider and fresh decoder.
use super::authored_provider::{BridgeAuthoredConditionalProvider, BridgeAuthoredProviderBinding};
use super::provider_semantics::BridgeErasedProviderSemanticContract;
use super::*;
use std::sync::Arc;
impl BridgeConditionalProviderSet {
    pub fn authored_condition<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalConditionProvider + BridgeAuthoredConditionalProvider,
    {
        let description = provider.authored_description();
        let semantics =
            BridgeErasedProviderSemanticContract::capture_authored(&provider, description.clone());
        let provider_heap = semantics.retained_provider_heap_bytes();
        self.semantic_contracts.condition = Some(semantics);
        let provider_arc_bytes =
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(&provider);
        let provider = Arc::new(provider);
        self.condition = Some(provider.clone());
        self.authored.condition = Some(description.clone());
        self.decoder_sources.retain(|known| {
            known.role != BridgeConditionalProviderRole::Condition
                || known.description.protocol() != description.protocol()
        });
        self.decoder_sources.push(BridgeAuthoredProviderBinding {
            role: BridgeConditionalProviderRole::Condition,
            provider_address: Arc::as_ptr(&provider) as usize,
            provider_arc_bytes,
            provider_heap,
            description,
            decoder: Arc::new(move |description, execution| {
                let rebound = provider.readmit_authored_provider(description, execution)?;
                Ok(Self::new().authored_condition(rebound))
            }),
        });
        self
    }
    pub fn authored_dependency_comparator<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalComparatorProvider + BridgeAuthoredConditionalProvider,
    {
        let description = provider.authored_description();
        let semantics =
            BridgeErasedProviderSemanticContract::capture_authored(&provider, description.clone());
        let provider_heap = semantics.retained_provider_heap_bytes();
        self.semantic_contracts.dependency_comparator = Some(semantics);
        let provider_arc_bytes =
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(&provider);
        let provider = Arc::new(provider);
        self.dependency_comparator = Some(provider.clone());
        self.authored.dependency_comparator = Some(description.clone());
        self.decoder_sources.retain(|known| {
            known.role != BridgeConditionalProviderRole::DependencyComparator
                || known.description.protocol() != description.protocol()
        });
        self.decoder_sources.push(BridgeAuthoredProviderBinding {
            role: BridgeConditionalProviderRole::DependencyComparator,
            provider_address: Arc::as_ptr(&provider) as usize,
            provider_arc_bytes,
            provider_heap,
            description,
            decoder: Arc::new(move |description, execution| {
                let rebound = provider.readmit_authored_provider(description, execution)?;
                Ok(Self::new().authored_dependency_comparator(rebound))
            }),
        });
        self
    }
    pub fn authored_output_comparator<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalComparatorProvider + BridgeAuthoredConditionalProvider,
    {
        let description = provider.authored_description();
        let semantics =
            BridgeErasedProviderSemanticContract::capture_authored(&provider, description.clone());
        let provider_heap = semantics.retained_provider_heap_bytes();
        self.semantic_contracts.output_comparator = Some(semantics);
        let provider_arc_bytes =
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(&provider);
        let provider = Arc::new(provider);
        self.output_comparator = Some(provider.clone());
        self.authored.output_comparator = Some(description.clone());
        self.decoder_sources.retain(|known| {
            known.role != BridgeConditionalProviderRole::OutputComparator
                || known.description.protocol() != description.protocol()
        });
        self.decoder_sources.push(BridgeAuthoredProviderBinding {
            role: BridgeConditionalProviderRole::OutputComparator,
            provider_address: Arc::as_ptr(&provider) as usize,
            provider_arc_bytes,
            provider_heap,
            description,
            decoder: Arc::new(move |description, execution| {
                let rebound = provider.readmit_authored_provider(description, execution)?;
                Ok(Self::new().authored_output_comparator(rebound))
            }),
        });
        self
    }
    pub fn authored_reuse_comparator<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalComparatorProvider + BridgeAuthoredConditionalProvider,
    {
        let description = provider.authored_description();
        let semantics =
            BridgeErasedProviderSemanticContract::capture_authored(&provider, description.clone());
        let provider_heap = semantics.retained_provider_heap_bytes();
        self.semantic_contracts.reuse_comparator = Some(semantics);
        let provider_arc_bytes =
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(&provider);
        let provider = Arc::new(provider);
        self.reuse_comparator = Some(provider.clone());
        self.authored.reuse_comparator = Some(description.clone());
        self.decoder_sources.retain(|known| {
            known.role != BridgeConditionalProviderRole::ArtifactReuseComparator
                || known.description.protocol() != description.protocol()
        });
        self.decoder_sources.push(BridgeAuthoredProviderBinding {
            role: BridgeConditionalProviderRole::ArtifactReuseComparator,
            provider_address: Arc::as_ptr(&provider) as usize,
            provider_arc_bytes,
            provider_heap,
            description,
            decoder: Arc::new(move |description, execution| {
                let rebound = provider.readmit_authored_provider(description, execution)?;
                Ok(Self::new().authored_reuse_comparator(rebound))
            }),
        });
        self
    }
    pub fn authored_trigger<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalTriggerProvider + BridgeAuthoredConditionalProvider,
    {
        let description = provider.authored_description();
        let semantics =
            BridgeErasedProviderSemanticContract::capture_authored(&provider, description.clone());
        let provider_heap = semantics.retained_provider_heap_bytes();
        self.semantic_contracts.trigger = Some(semantics);
        let provider_arc_bytes =
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(&provider);
        let provider = Arc::new(provider);
        self.trigger = Some(provider.clone());
        self.authored.trigger = Some(description.clone());
        self.decoder_sources.retain(|known| {
            known.role != BridgeConditionalProviderRole::Trigger
                || known.description.protocol() != description.protocol()
        });
        self.decoder_sources.push(BridgeAuthoredProviderBinding {
            role: BridgeConditionalProviderRole::Trigger,
            provider_address: Arc::as_ptr(&provider) as usize,
            provider_arc_bytes,
            provider_heap,
            description,
            decoder: Arc::new(move |description, execution| {
                let rebound = provider.readmit_authored_provider(description, execution)?;
                Ok(Self::new().authored_trigger(rebound))
            }),
        });
        self
    }
    pub fn authored_wake<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalWakeProvider + BridgeAuthoredConditionalProvider,
    {
        let description = provider.authored_description();
        let semantics =
            BridgeErasedProviderSemanticContract::capture_authored(&provider, description.clone());
        let provider_heap = semantics.retained_provider_heap_bytes();
        self.semantic_contracts.wake = Some(semantics);
        let provider_arc_bytes =
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(&provider);
        let provider = Arc::new(provider);
        self.wake = Some(provider.clone());
        self.authored.wake = Some(description.clone());
        self.decoder_sources.retain(|known| {
            known.role != BridgeConditionalProviderRole::Wake
                || known.description.protocol() != description.protocol()
        });
        self.decoder_sources.push(BridgeAuthoredProviderBinding {
            role: BridgeConditionalProviderRole::Wake,
            provider_address: Arc::as_ptr(&provider) as usize,
            provider_arc_bytes,
            provider_heap,
            description,
            decoder: Arc::new(move |description, execution| {
                let rebound = provider.readmit_authored_provider(description, execution)?;
                Ok(Self::new().authored_wake(rebound))
            }),
        });
        self
    }
    pub fn authored_compute<P>(mut self, provider: P) -> Self
    where
        P: BridgeConditionalComputeProvider + BridgeAuthoredConditionalProvider,
    {
        let description = provider.authored_description();
        let semantics =
            BridgeErasedProviderSemanticContract::capture_authored(&provider, description.clone());
        let provider_heap = semantics.retained_provider_heap_bytes();
        self.semantic_contracts.compute = Some(semantics);
        let provider_arc_bytes =
            BridgeConditionalProviderHeapRetention::arc_allocation_bytes(&provider);
        let provider = Arc::new(provider);
        self.compute = Some(provider.clone());
        self.authored.compute = Some(description.clone());
        self.decoder_sources.retain(|known| {
            known.role != BridgeConditionalProviderRole::Compute
                || known.description.protocol() != description.protocol()
        });
        self.decoder_sources.push(BridgeAuthoredProviderBinding {
            role: BridgeConditionalProviderRole::Compute,
            provider_address: Arc::as_ptr(&provider) as usize,
            provider_arc_bytes,
            provider_heap,
            description,
            decoder: Arc::new(move |description, execution| {
                let rebound = provider.readmit_authored_provider(description, execution)?;
                Ok(Self::new().authored_compute(rebound))
            }),
        });
        self
    }
}
