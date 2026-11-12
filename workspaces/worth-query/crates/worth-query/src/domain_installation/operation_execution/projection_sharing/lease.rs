use std::sync::Arc;

use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::{
    WorthQueryManagedLiveWorkspaceCapability, WorthQuerySharedExecutionOwnerIdentity,
    WorthQuerySharedProjectionLeaseIdentity, WorthQuerySharedProjectionLeaseToken,
};
use worth_foundational::facade::admit_foundational_authority_identity;
use worth_proof::{
    Artifact, AssumptionBasis, AuthorityMarker, AuthorityWitness, CurrentValidity,
    FreshnessScopedBasis, NoProofs, PhaseMarker,
};

use crate::identity_authority::{
    query_subscription_authority, QuerySubscriptionAuthorityIdentity, QuerySubscriptionIdentityKind,
};

use super::super::WorthQuerySettledDomainProjection;

pub(super) struct WorthQuerySharedProjectionLeasePhase;
impl PhaseMarker for WorthQuerySharedProjectionLeasePhase {}

struct WorthQuerySharedProjectionLeaseAuthority;
impl AuthorityMarker for WorthQuerySharedProjectionLeaseAuthority {}

pub(super) struct WorthQuerySharedProjectionLeaseBasis {
    owner: WorthQuerySharedExecutionOwnerIdentity,
    lease: WorthQuerySharedProjectionLeaseIdentity,
    source_identity: String,
    binding_identity: String,
    collection_delivery_contract_identity: Option<String>,
    capability_identity: u64,
    capability_generation: crate::domain_installation::WorthQueryBoundCapabilityGeneration,
}

pub(super) struct WorthQuerySharedProjectionLeaseEvidence {
    identity: String,
    predecessor_identity: String,
}

#[derive(Clone, Copy)]
pub(crate) struct WorthQuerySharedProjectionLeaseReadmission<'a> {
    pub(crate) owner: WorthQuerySharedExecutionOwnerIdentity,
    pub(crate) lease: WorthQuerySharedProjectionLeaseIdentity,
    pub(crate) source_identity: &'a str,
    pub(crate) binding_identity: &'a str,
    pub(crate) collection_delivery_contract_identity: Option<&'a str>,
    pub(crate) capability_identity: u64,
    pub(crate) capability_generation:
        crate::domain_installation::WorthQueryBoundCapabilityGeneration,
    pub(crate) closure:
        &'a crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
}

type WorthQuerySharedProjectionLeaseProgressionProof = Artifact<
    WorthQuerySharedProjectionLeasePhase,
    WorthQuerySharedProjectionLeaseEvidence,
    NoProofs,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<WorthQuerySharedProjectionLeaseBasis>>,
>;

pub(super) struct WorthQuerySharedProjectionLeaseProof {
    proof: WorthQuerySharedProjectionLeaseProgressionProof,
    _owner_identity: QuerySubscriptionAuthorityIdentity<Arc<str>, QuerySubscriptionIdentityKind>,
}

impl WorthQuerySharedProjectionLeaseProof {
    fn payload(&self) -> &WorthQuerySharedProjectionLeaseEvidence {
        self.proof.payload()
    }

    fn strong_basis(&self) -> &WorthQuerySharedProjectionLeaseBasis {
        self.proof.strong_basis().value()
    }
}

#[must_use = "shared projection leases keep the shared owner active until disposed or dropped"]
pub struct WorthQuerySharedLiveProjectionLease<D, O, F, L: BasisOperationLane> {
    source: Option<WorthQuerySettledDomainProjection<D, O, F, L>>,
    proof: Option<WorthQuerySharedProjectionLeaseProof>,
    workspace_capability: Arc<WorthQueryManagedLiveWorkspaceCapability>,
    token: Option<WorthQuerySharedProjectionLeaseToken>,
    singleton_admission_counters: Option<super::WorthQueryProjectionLeaseAdmissionCounters>,
}

impl<D, O, F, L: BasisOperationLane> WorthQuerySharedLiveProjectionLease<D, O, F, L> {
    pub(super) fn new(
        source: WorthQuerySettledDomainProjection<D, O, F, L>,
        predecessor_identity: String,
        capability_generation: crate::domain_installation::WorthQueryBoundCapabilityGeneration,
        workspace_capability: Arc<WorthQueryManagedLiveWorkspaceCapability>,
        token: WorthQuerySharedProjectionLeaseToken,
    ) -> Self {
        let bound = source.bound_operation();
        let basis = WorthQuerySharedProjectionLeaseBasis {
            owner: token.owner(),
            lease: token.lease(),
            source_identity: source.identity().to_string(),
            binding_identity: bound.binding_identity().to_string(),
            collection_delivery_contract_identity: source.collection_delivery_contract_identity(),
            capability_identity: bound.capability_identity(),
            capability_generation,
        };
        let identity = crate::identity::hash_parts(&[
            "worth_query_shared_projection_lease_v1".into(),
            format!("runtime-authority:{}", token.owner().runtime_authority()),
            format!("owner-slot:{}", token.owner().slot()),
            format!("owner-generation:{}", token.owner().generation()),
            format!("lease-slot:{}", token.lease().slot()),
            format!("lease-generation:{}", token.lease().generation()),
            format!("source:{}", source.identity()),
        ]);
        let owner_identity = admit_foundational_authority_identity(
            Arc::<str>::from(identity.clone()),
            query_subscription_authority(),
        );
        let proof = Artifact::with_current_basis(
            WorthQuerySharedProjectionLeaseEvidence {
                identity,
                predecessor_identity,
            },
            basis,
            AuthorityWitness::from_authority_marker(WorthQuerySharedProjectionLeaseAuthority),
        );
        Self {
            source: Some(source),
            proof: Some(WorthQuerySharedProjectionLeaseProof {
                proof,
                _owner_identity: owner_identity,
            }),
            workspace_capability,
            token: Some(token),
            singleton_admission_counters: None,
        }
    }

    pub(super) fn with_singleton_admission_counters(
        mut self,
        counters: super::WorthQueryProjectionLeaseAdmissionCounters,
    ) -> Self {
        self.singleton_admission_counters = Some(counters);
        self
    }

    pub const fn singleton_admission_counters(
        &self,
    ) -> Option<super::WorthQueryProjectionLeaseAdmissionCounters> {
        self.singleton_admission_counters
    }

    pub fn identity(&self) -> &str {
        &self.proof().payload().identity
    }

    pub fn predecessor_identity(&self) -> &str {
        &self.proof().payload().predecessor_identity
    }

    pub fn owner_identity(&self) -> WorthQuerySharedExecutionOwnerIdentity {
        self.proof().strong_basis().owner
    }

    pub fn lease_identity(&self) -> WorthQuerySharedProjectionLeaseIdentity {
        self.proof().strong_basis().lease
    }

    pub fn snapshot(&self) -> &WorthQuerySettledDomainProjection<D, O, F, L> {
        self.source
            .as_ref()
            .expect("active shared projection lease must retain its source")
    }

    pub(super) fn into_parts(
        mut self,
    ) -> (
        WorthQuerySettledDomainProjection<D, O, F, L>,
        WorthQuerySharedProjectionLeaseProof,
        Arc<WorthQueryManagedLiveWorkspaceCapability>,
        WorthQuerySharedProjectionLeaseToken,
        Option<super::WorthQueryProjectionLeaseAdmissionCounters>,
    ) {
        let token = self
            .token
            .take()
            .expect("active shared projection lease must retain its token");
        let source = self
            .source
            .take()
            .expect("active shared projection lease must retain its source");
        let proof = self
            .proof
            .take()
            .expect("active shared projection lease must retain its proof");
        (
            source,
            proof,
            self.workspace_capability.clone(),
            token,
            self.singleton_admission_counters,
        )
    }

    pub(super) fn from_parts(
        source: WorthQuerySettledDomainProjection<D, O, F, L>,
        proof: WorthQuerySharedProjectionLeaseProof,
        workspace_capability: Arc<WorthQueryManagedLiveWorkspaceCapability>,
        token: WorthQuerySharedProjectionLeaseToken,
        singleton_admission_counters: Option<super::WorthQueryProjectionLeaseAdmissionCounters>,
    ) -> Self {
        Self {
            source: Some(source),
            proof: Some(proof),
            workspace_capability,
            token: Some(token),
            singleton_admission_counters,
        }
    }

    pub(crate) fn workspace_capability(&self) -> &Arc<WorthQueryManagedLiveWorkspaceCapability> {
        &self.workspace_capability
    }

    pub(crate) fn readmission(&self) -> WorthQuerySharedProjectionLeaseReadmission<'_> {
        let basis = self.proof().strong_basis();
        WorthQuerySharedProjectionLeaseReadmission {
            owner: basis.owner,
            lease: basis.lease,
            source_identity: &basis.source_identity,
            binding_identity: &basis.binding_identity,
            collection_delivery_contract_identity: basis
                .collection_delivery_contract_identity
                .as_deref(),
            capability_identity: basis.capability_identity,
            capability_generation: basis.capability_generation,
            closure: self.snapshot().semantic_aspect_dependency_closure(),
        }
    }

    fn proof(&self) -> &WorthQuerySharedProjectionLeaseProof {
        self.proof
            .as_ref()
            .expect("active shared projection lease must retain its proof")
    }
}

impl<D, O, F, L: BasisOperationLane> Drop for WorthQuerySharedLiveProjectionLease<D, O, F, L> {
    fn drop(&mut self) {
        if let Some(token) = self.token.take() {
            self.workspace_capability
                .abandon_shared_projection_lease(token);
        }
    }
}
#[cfg(test)]
#[path = "lease_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
