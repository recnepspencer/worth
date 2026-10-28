use super::super::{
    BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeDenial, BridgeCausalEvidenceBinding,
    BridgeCausalEvidenceFamily, BridgeCausalEvidenceReference,
};
use super::source::{ResolvedBinding, SourceContext};
use super::BridgeCausalEnvelopeAssemblyRequest;

pub(super) struct BridgeCausalEnvelopeAssemblyProgress {
    evidence_reference_count: usize,
    bridge_retained_lookup_count: usize,
    retained_bridge_binding_count: usize,
    external_authority_reference_count: usize,
    retained_bridge_route_binding_count: usize,
    retained_native_binding_count: usize,
    native: bool,
    bindings: Vec<BridgeCausalEvidenceBinding>,
}

impl BridgeCausalEnvelopeAssemblyProgress {
    pub(super) fn query_observation_reporting(
        request: &BridgeCausalEnvelopeAssemblyRequest,
        storage: Vec<BridgeCausalEvidenceBinding>,
    ) -> Self {
        let mut progress = Self::for_request(request, false);
        progress.bindings = storage;
        progress
    }

    pub(super) fn record_query_report(&mut self, binding: BridgeCausalEvidenceBinding) {
        self.external_authority_reference_count += 1;
        self.bindings.push(binding);
    }

    pub(super) fn query_reporting_counts(&self) -> [usize; 8] {
        [
            self.evidence_reference_count,
            self.lower_runtime_family_count(),
            self.bridge_retained_lookup_count,
            self.retained_bridge_binding_count,
            self.external_authority_reference_count,
            self.bindings.len(),
            0,
            0,
        ]
    }

    pub(super) fn for_request(request: &BridgeCausalEnvelopeAssemblyRequest, native: bool) -> Self {
        Self {
            evidence_reference_count: request.references().len(),
            bridge_retained_lookup_count: 0,
            retained_bridge_binding_count: 0,
            external_authority_reference_count: 0,
            retained_bridge_route_binding_count: 0,
            retained_native_binding_count: 0,
            native,
            bindings: Vec::new(),
        }
    }

    pub(super) fn bind_reference(
        &mut self,
        source: &SourceContext<'_>,
        reference: &BridgeCausalEvidenceReference,
    ) -> Result<(), BridgeCausalEnvelopeDenial> {
        let lookup = source.needs_registry_lookup(reference);
        if lookup {
            self.bridge_retained_lookup_count += 1;
        }
        let binding = match source.resolve(reference, self.counters(usize::from(lookup)))? {
            ResolvedBinding::LegacyRetained(identity) => {
                self.retained_bridge_binding_count += 1;
                if reference.family() == BridgeCausalEvidenceFamily::BridgeRoute {
                    self.retained_bridge_route_binding_count += 1;
                }
                BridgeCausalEvidenceBinding::retained(reference, identity)
            }
            ResolvedBinding::NativeRetained => {
                self.retained_native_binding_count += 1;
                BridgeCausalEvidenceBinding::native(reference)
            }
            ResolvedBinding::External => {
                self.external_authority_reference_count += 1;
                BridgeCausalEvidenceBinding::external(reference)
            }
        };
        self.bindings.push(binding);
        Ok(())
    }

    pub(super) fn has_required_bridge_route_evidence(&self) -> bool {
        self.retained_bridge_route_binding_count > 0
    }

    pub(super) fn success_counters(&self) -> BridgeCausalEnvelopeCounters {
        self.counters(0)
    }

    fn counters(&self, missing_bridge_record_count: usize) -> BridgeCausalEnvelopeCounters {
        let counters = BridgeCausalEnvelopeCounters::new(
            self.evidence_reference_count,
            self.lower_runtime_family_count(),
            self.bridge_retained_lookup_count,
            self.retained_bridge_binding_count,
            self.external_authority_reference_count,
            self.bindings.len(),
            missing_bridge_record_count,
        );
        if self.native {
            counters.with_native_bindings(self.retained_native_binding_count)
        } else {
            counters
        }
    }

    pub(super) fn into_bindings(self) -> Vec<BridgeCausalEvidenceBinding> {
        self.bindings
    }

    fn lower_runtime_family_count(&self) -> usize {
        let mut families = Vec::new();
        for binding in &self.bindings {
            if binding.family() == BridgeCausalEvidenceFamily::QueryObservation {
                continue;
            }
            if !families.contains(&binding.family()) {
                families.push(binding.family());
            }
        }
        families.len()
    }
}
