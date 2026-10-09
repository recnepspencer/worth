use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_signal::facade::{Aspect, AspectMask, PartitionSubscription, ScopeCoverage};
use worth_store_aspect_native::{
    StoreAspectBindingStamp, StoreAspectContractAdmission, StoreAspectIdentity,
};

use super::{
    declaration::PhysicalWorkProfileParts, PhysicalSignalAspectDeclaration,
    PhysicalSignalAspectRole, PhysicalSignalProfileIdentity, PhysicalWorkCapacity,
    PhysicalWorkProfileDeclaration, PhysicalWorkSignalFamily, PhysicalWorkSignalFamilySet,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalSignalAspectBindingDigest([u8; 32]);

impl PhysicalSignalAspectBindingDigest {
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalSignalBindingDenial {
    ProjectionMaskAbsent,
}

#[derive(Debug)]
pub struct PhysicalSignalAspectBindingSet {
    profile: PhysicalSignalProfileIdentity,
    security_authorities: Box<[[u8; 32]]>,
    capacity: PhysicalWorkCapacity,
    bindings: Box<[PhysicalSignalAspectBinding]>,
}

#[derive(Debug)]
pub struct PhysicalSignalAspectBinding {
    contract: StoreAspectContractAdmission,
    signal_aspect: Aspect,
    signal_mask: AspectMask,
    role: PhysicalSignalAspectRole,
    families: PhysicalWorkSignalFamilySet,
    partition: Option<PartitionSubscription>,
    digest: PhysicalSignalAspectBindingDigest,
    capability: Arc<()>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalSignalAspectBindingObservation {
    identity: StoreAspectIdentity,
    role: PhysicalSignalAspectRole,
    families: PhysicalWorkSignalFamilySet,
    partition: Option<PartitionSubscription>,
    digest: PhysicalSignalAspectBindingDigest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalSignalAspectSubscription {
    signal_mask: AspectMask,
    partition: Option<PartitionSubscription>,
    binding: PhysicalSignalAspectBindingDigest,
}

impl PhysicalSignalAspectBindingSet {
    pub fn from_profile(declaration: PhysicalWorkProfileDeclaration) -> Self {
        Self::install(declaration)
    }

    pub(in crate::physical_runtime) fn install(
        declaration: PhysicalWorkProfileDeclaration,
    ) -> Self {
        let profile = declaration.identity();
        let PhysicalWorkProfileParts {
            security_authorities,
            aspects,
            capacity,
        } = declaration.into_parts();
        let bindings = aspects
            .into_vec()
            .into_iter()
            .enumerate()
            .map(|(slot, declaration)| binding(slot, declaration))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            profile,
            security_authorities,
            capacity,
            bindings,
        }
    }

    pub const fn profile(&self) -> PhysicalSignalProfileIdentity {
        self.profile
    }

    pub const fn capacity(&self) -> PhysicalWorkCapacity {
        self.capacity
    }

    pub const fn len(&self) -> usize {
        self.bindings.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn binding_for_identity(
        &self,
        identity: &StoreAspectIdentity,
    ) -> Option<&PhysicalSignalAspectBinding> {
        self.bindings
            .binary_search_by(|binding| binding.contract.identity().cmp(identity))
            .ok()
            .map(|index| &self.bindings[index])
    }

    pub fn binding_for_slot(&self, slot: usize) -> Option<&PhysicalSignalAspectBinding> {
        self.bindings.get(slot)
    }

    pub(in crate::physical_runtime) fn bindings(&self) -> &[PhysicalSignalAspectBinding] {
        &self.bindings
    }

    pub(in crate::physical_runtime) fn observations(
        &self,
    ) -> Box<[PhysicalSignalAspectBindingObservation]> {
        self.bindings
            .iter()
            .map(PhysicalSignalAspectBindingObservation::from)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    pub(in crate::physical_runtime) fn admits(
        &self,
        identity: &StoreAspectIdentity,
        binding: StoreAspectBindingStamp,
    ) -> bool {
        self.binding_for_identity(identity)
            .is_some_and(|installed| installed.contract.binding_stamp() == binding)
    }

    pub(in crate::physical_runtime) fn admits_security(
        &self,
        receipt: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
    ) -> bool {
        self.security_authorities
            .binary_search(&receipt.authority_identity().fingerprint())
            .is_ok()
    }
}

impl PhysicalSignalAspectBinding {
    pub const fn identity(&self) -> &StoreAspectIdentity {
        self.contract.identity()
    }

    pub const fn role(&self) -> PhysicalSignalAspectRole {
        self.role
    }

    pub const fn families(&self) -> PhysicalWorkSignalFamilySet {
        self.families
    }

    pub const fn serves_family(&self, family: PhysicalWorkSignalFamily) -> bool {
        self.families.contains(family)
    }

    pub const fn partition(&self) -> Option<&PartitionSubscription> {
        self.partition.as_ref()
    }

    pub const fn digest(&self) -> PhysicalSignalAspectBindingDigest {
        self.digest
    }

    pub fn projection_subscription(
        &self,
    ) -> Result<PhysicalSignalAspectSubscription, PhysicalSignalBindingDenial> {
        self.contract
            .projection_mask()
            .ok_or(PhysicalSignalBindingDenial::ProjectionMaskAbsent)?;
        Ok(PhysicalSignalAspectSubscription {
            signal_mask: self.signal_mask,
            partition: self.partition.clone(),
            binding: self.digest,
        })
    }

    pub(in crate::physical_runtime) const fn signal_aspect(&self) -> Aspect {
        self.signal_aspect
    }

    pub(in crate::physical_runtime) const fn signal_mask(&self) -> AspectMask {
        self.signal_mask
    }

    pub(in crate::physical_runtime) const fn contract(&self) -> &StoreAspectContractAdmission {
        &self.contract
    }

    pub(in crate::physical_runtime) fn installs(&self, capability: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.capability, capability)
    }

    pub(in crate::physical_runtime) fn capability(&self) -> Arc<()> {
        Arc::clone(&self.capability)
    }
}

impl PhysicalSignalAspectBindingObservation {
    pub const fn identity(&self) -> &StoreAspectIdentity {
        &self.identity
    }

    pub const fn role(&self) -> PhysicalSignalAspectRole {
        self.role
    }

    pub const fn families(&self) -> PhysicalWorkSignalFamilySet {
        self.families
    }

    pub const fn partition(&self) -> Option<&PartitionSubscription> {
        self.partition.as_ref()
    }

    pub const fn digest(&self) -> PhysicalSignalAspectBindingDigest {
        self.digest
    }
}

impl From<&PhysicalSignalAspectBinding> for PhysicalSignalAspectBindingObservation {
    fn from(binding: &PhysicalSignalAspectBinding) -> Self {
        Self {
            identity: binding.identity().clone(),
            role: binding.role(),
            families: binding.families(),
            partition: binding.partition().cloned(),
            digest: binding.digest(),
        }
    }
}

impl PhysicalSignalAspectSubscription {
    pub const fn binding(&self) -> PhysicalSignalAspectBindingDigest {
        self.binding
    }

    pub const fn is_partitioned(&self) -> bool {
        self.partition.is_some()
    }
}

fn binding(
    slot: usize,
    declaration: PhysicalSignalAspectDeclaration,
) -> PhysicalSignalAspectBinding {
    let signal_aspect = Aspect::try_new(slot as u8)
        .expect("profile construction already enforced Signal aspect capacity");
    let (contract, role, families, partition) = declaration.into_parts();
    let digest = binding_digest(&contract, role, families, partition.as_ref());
    PhysicalSignalAspectBinding {
        contract,
        signal_aspect,
        signal_mask: AspectMask::from_aspect(signal_aspect),
        role,
        families,
        partition,
        digest,
        capability: Arc::new(()),
    }
}

fn binding_digest(
    contract: &StoreAspectContractAdmission,
    role: PhysicalSignalAspectRole,
    families: PhysicalWorkSignalFamilySet,
    partition: Option<&PartitionSubscription>,
) -> PhysicalSignalAspectBindingDigest {
    let mut digest = Sha256::new();
    digest.update(b"worth-store.physical-signal-aspect-binding.v5");
    digest.update(contract.binding_stamp().as_bytes());
    digest.update([role_code(role)]);
    digest.update(families.bits().to_le_bytes());
    if let Some(partition) = partition {
        digest.update([1]);
        let segments = partition.path().segments();
        digest.update((segments.len() as u64).to_le_bytes());
        for segment in segments {
            digest.update((segment.len() as u64).to_le_bytes());
            digest.update(segment.as_bytes());
        }
        digest.update([coverage_code(partition.coverage())]);
    } else {
        digest.update([0]);
    }
    PhysicalSignalAspectBindingDigest(digest.finalize().into())
}

pub(super) const fn coverage_code(coverage: ScopeCoverage) -> u8 {
    match coverage {
        ScopeCoverage::Exact => 1,
        ScopeCoverage::Subtree => 2,
    }
}

const fn role_code(role: PhysicalSignalAspectRole) -> u8 {
    match role {
        PhysicalSignalAspectRole::Dependency => 1,
        PhysicalSignalAspectRole::Output => 2,
        PhysicalSignalAspectRole::DependencyAndOutput => 3,
    }
}
