use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{
    PhysicalMutationFingerprintInput, PhysicalMutationOperationFamily,
    PhysicalMutationPayloadDigest, PhysicalMutationRequestScope, PhysicalMutationSecurityBasis,
};
use crate::physical_runtime::record_serving::publication::record_append_scope_identity;
use crate::physical_runtime::PhysicalMutationRequestFingerprint;

impl RecordPublicationDirector {
    pub(super) fn derive_record_append_fingerprint(
        &self,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
        manifest_capacity_transition: crate::physical_runtime::PhysicalManifestCapacityTransition,
        payload_digest: [u8; 32],
        durability_request: crate::physical_runtime::durability::PhysicalMutationDurabilityRequest,
        family: PhysicalMutationOperationFamily,
    ) -> Result<PhysicalMutationRequestFingerprint, ()> {
        let scope =
            record_append_scope_identity(self.format, placement, manifest_capacity_transition);
        let security = [PhysicalMutationSecurityBasis::from_admitted_security(
            self.security_basis,
        )];
        PhysicalMutationRequestFingerprint::derive(PhysicalMutationFingerprintInput {
            store: self.durability.store_identity(),
            durability_policy: self.durability.policy_identity(),
            scope: PhysicalMutationRequestScope::record_append(scope),
            payload: PhysicalMutationPayloadDigest::from_validated_payload(payload_digest),
            durability_request,
            operation_family: family,
            security_bases: &security,
        })
        .map_err(|_| ())
    }
}
