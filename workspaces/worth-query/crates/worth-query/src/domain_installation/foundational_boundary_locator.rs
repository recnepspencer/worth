use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use crate::evidence_identity::WorthQueryEvidenceIdentity;
use worth_foundational::facade::{BoundaryArtifactId, BoundaryHandle};

#[derive(Default)]
struct WorthQueryFoundationalBoundaryLocatorRegistry {
    next_handle: u64,
    handles_by_evidence_identity: BTreeMap<String, u64>,
}

impl WorthQueryFoundationalBoundaryLocatorRegistry {
    fn locate(&mut self, evidence_identity: &WorthQueryEvidenceIdentity) -> u64 {
        if let Some(handle) = self
            .handles_by_evidence_identity
            .get(evidence_identity.as_str())
        {
            return *handle;
        }
        self.next_handle = self
            .next_handle
            .checked_add(1)
            .expect("Foundational boundary locator space exhausted");
        let handle = self.next_handle;
        self.handles_by_evidence_identity
            .insert(evidence_identity.as_str().to_owned(), handle);
        handle
    }
}

pub(crate) fn foundational_boundary_handle(
    evidence_identity: &WorthQueryEvidenceIdentity,
) -> BoundaryHandle {
    BoundaryHandle::new(locate(evidence_identity))
}

pub(crate) fn foundational_boundary_artifact_id(
    evidence_identity: &WorthQueryEvidenceIdentity,
) -> BoundaryArtifactId {
    BoundaryArtifactId::new(locate(evidence_identity))
}

fn locate(evidence_identity: &WorthQueryEvidenceIdentity) -> u64 {
    static REGISTRY: OnceLock<Mutex<WorthQueryFoundationalBoundaryLocatorRegistry>> =
        OnceLock::new();
    let mut registry = REGISTRY
        .get_or_init(|| Mutex::new(WorthQueryFoundationalBoundaryLocatorRegistry::default()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    registry.locate(evidence_identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_identity::{WorthQueryEvidenceScope, WorthQueryEvidenceTag};

    fn identity(value: &str) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::InstalledDomainExecution)
                .field_value(WorthQueryEvidenceTag::new("subject"), value)
                .seal()?,
        )
    }

    #[test]
    fn equivalent_full_identities_share_a_locator_without_digest_truncation() {
        let first = identity("first");
        let equivalent = first.clone();
        let second = identity("second");

        assert_eq!(
            foundational_boundary_handle(&first),
            foundational_boundary_handle(&equivalent)
        );
        assert_ne!(
            foundational_boundary_handle(&first),
            foundational_boundary_handle(&second)
        );
    }
}
