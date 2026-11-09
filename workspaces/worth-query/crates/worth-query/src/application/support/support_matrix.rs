use super::{
    WorthQueryCapabilityDescriptor, WorthQueryCapabilityFamily, WorthQueryCapabilityRegistry,
    WorthQueryCapabilityStatus,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySupportMatrix {
    registry: WorthQueryCapabilityRegistry,
    support_matrix_digest: String,
}

impl WorthQuerySupportMatrix {
    pub(crate) fn new(
        registry: WorthQueryCapabilityRegistry,
    ) -> Result<Self, IdentityDerivationDenial> {
        let admitted = count_status(&registry, WorthQueryCapabilityStatus::Admitted);
        let deferred = count_status(&registry, WorthQueryCapabilityStatus::DeferredDebt);
        let unsupported = count_status(&registry, WorthQueryCapabilityStatus::Unsupported);
        let support_matrix_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
                .field_shape(WorthQueryEvidenceTag::new("role"), "support-matrix")
                .field_value(
                    WorthQueryEvidenceTag::new("registry"),
                    registry.registry_digest(),
                )
                .field_usize(WorthQueryEvidenceTag::new("admitted"), admitted)
                .field_usize(WorthQueryEvidenceTag::new("deferred"), deferred)
                .field_usize(WorthQueryEvidenceTag::new("unsupported"), unsupported)
                .seal()?
                .as_str()
                .to_string();
        Ok(Self {
            registry,
            support_matrix_digest,
        })
    }

    pub fn descriptor(
        &self,
        family: WorthQueryCapabilityFamily,
    ) -> Option<&WorthQueryCapabilityDescriptor> {
        self.registry.descriptor(family)
    }

    pub fn capability_registry(&self) -> &WorthQueryCapabilityRegistry {
        &self.registry
    }

    pub fn support_matrix_digest(&self) -> &str {
        &self.support_matrix_digest
    }

    pub fn admitted_capability_count(&self) -> usize {
        count_status(&self.registry, WorthQueryCapabilityStatus::Admitted)
    }

    pub fn deferred_capability_count(&self) -> usize {
        count_status(&self.registry, WorthQueryCapabilityStatus::DeferredDebt)
    }

    pub fn unsupported_capability_count(&self) -> usize {
        count_status(&self.registry, WorthQueryCapabilityStatus::Unsupported)
    }
}

fn count_status(
    registry: &WorthQueryCapabilityRegistry,
    status: WorthQueryCapabilityStatus,
) -> usize {
    registry
        .descriptors()
        .iter()
        .filter(|descriptor| descriptor.status() == status)
        .count()
}
