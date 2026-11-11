//! Assemble canonical template meaning and its immutable storage.
use super::*;

impl LoweredBridgeAsyncSourceDeclaration {
    pub(crate) fn from_request_response_parts(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        declaration: ResourceNodeDeclaration,
        descriptor: LoweredResourceDescriptor,
    ) -> Self {
        Self::request_response_parts(validated, declaration, descriptor, None)
    }

    pub(super) fn request_response_parts(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        declaration: ResourceNodeDeclaration,
        descriptor: LoweredResourceDescriptor,
        storage: Option<OriginalLoweringStorage>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-async-lowering|declaration={}|family=request-response|lowering_family=resource-descriptor|payload_contract_digest={}|registry_digest={}|bundle_digest={}",
            validated.digest(),
            descriptor.payload_contract_digest().as_str(),
            descriptor.lowered_policy_bundle().registry_digest().as_str(),
            descriptor.lowered_policy_bundle().bundle_digest().as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let digest: Arc<str> = Arc::from(format!("bridge-async-lowering:sha256:{digest:x}"));
        Self {
            declaration_identity: validated.declaration_identity().clone(),
            family_kind: validated.family_kind(),
            lowering_family_kind: BridgeAsyncSignalLoweringFamilyKind::ResourceDescriptor,
            lowering_identity: BridgeAsyncSourceLoweringIdentity::admit_bridge_owned(
                digest.clone(),
            ),
            canonical_basis,
            digest: Arc::clone(&digest),
            counters: BridgeAsyncSourceDeclarationCounters::request_response_lowered(),
            body: LoweringBodyStorage::retain(
                LoweredBridgeAsyncSourceDeclarationBody::RequestResponse {
                    declaration,
                    descriptor,
                },
                digest,
                storage,
            ),
        }
    }

    pub(super) fn new_subscription_backed(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        declaration: AsyncNodeCapabilityDeclaration,
        bundle: LoweredAsyncNodeCapabilityBundle,
        storage: Option<OriginalLoweringStorage>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-async-lowering|declaration={}|family=subscription-backed|lowering_family=async-node-capability|payload_contract_digest={}|registry_digest={}|bundle_digest={}",
            validated.digest(),
            bundle.payload_contract_digest().as_str(),
            bundle.registry_digest().as_str(),
            bundle.bundle_digest().as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        let digest: Arc<str> = Arc::from(format!("bridge-async-lowering:sha256:{digest:x}"));
        Self {
            declaration_identity: validated.declaration_identity().clone(),
            family_kind: validated.family_kind(),
            lowering_family_kind: BridgeAsyncSignalLoweringFamilyKind::AsyncNodeCapability,
            lowering_identity: BridgeAsyncSourceLoweringIdentity::admit_bridge_owned(
                digest.clone(),
            ),
            canonical_basis,
            digest: Arc::clone(&digest),
            counters: BridgeAsyncSourceDeclarationCounters::subscription_backed_lowered(),
            body: LoweringBodyStorage::retain(
                LoweredBridgeAsyncSourceDeclarationBody::SubscriptionBacked {
                    declaration,
                    bundle,
                },
                digest,
                storage,
            ),
        }
    }
}
