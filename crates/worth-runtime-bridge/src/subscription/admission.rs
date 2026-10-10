use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::facade::RuntimeBridge;

use super::{
    BridgeAdmittedSubscriptionIdentity, BridgeSignalStrategyDescriptor,
    BridgeSubscriptionBasisKind, BridgeSubscriptionBasisRequest,
    BridgeSubscriptionBasisResolutionFailureKind, BridgeSubscriptionCounters,
    BridgeSubscriptionDeclaration, BridgeSubscriptionDeclarationFamilyKind,
    ValidatedSubscriptionBasisBinding,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubscriptionAdmissionRejectionKind {
    BasisKindDivergence,
    BasisResolutionFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSubscriptionAdmissionRejection {
    declaration_identity: super::BridgeSubscriptionDeclarationIdentity,
    requested_family_kind: BridgeSubscriptionDeclarationFamilyKind,
    requested_basis_kind: BridgeSubscriptionBasisKind,
    rejection_kind: BridgeSubscriptionAdmissionRejectionKind,
    basis_resolution_failure_kind: Option<BridgeSubscriptionBasisResolutionFailureKind>,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeSubscriptionAdmissionRejection {
    fn basis_kind_divergence(
        declaration: &BridgeSubscriptionDeclaration,
        requested_basis_kind: BridgeSubscriptionBasisKind,
    ) -> Self {
        Self::new(
            declaration,
            requested_basis_kind,
            BridgeSubscriptionAdmissionRejectionKind::BasisKindDivergence,
            None,
            BridgeSubscriptionCounters::from_basis_kind_divergence_rejection(),
        )
    }

    fn basis_resolution_failure(
        declaration: &BridgeSubscriptionDeclaration,
        requested_basis_kind: BridgeSubscriptionBasisKind,
        failure_kind: BridgeSubscriptionBasisResolutionFailureKind,
    ) -> Self {
        Self::new(
            declaration,
            requested_basis_kind,
            BridgeSubscriptionAdmissionRejectionKind::BasisResolutionFailure,
            Some(failure_kind),
            BridgeSubscriptionCounters::from_basis_resolution_rejection(),
        )
    }

    fn new(
        declaration: &BridgeSubscriptionDeclaration,
        requested_basis_kind: BridgeSubscriptionBasisKind,
        rejection_kind: BridgeSubscriptionAdmissionRejectionKind,
        basis_resolution_failure_kind: Option<BridgeSubscriptionBasisResolutionFailureKind>,
        counters: BridgeSubscriptionCounters,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-subscription-admission-rejection|declaration={}|family={}|basis-kind={}|rejection-kind={}|basis-resolution-kind={}",
            declaration.declaration_identity().as_str(),
            declaration.requested_family_kind().as_str(),
            requested_basis_kind.as_str(),
            rejection_kind.as_str(),
            basis_resolution_failure_kind
                .map(BridgeSubscriptionBasisResolutionFailureKind::as_str)
                .unwrap_or("-"),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            declaration_identity: declaration.declaration_identity().clone(),
            requested_family_kind: declaration.requested_family_kind(),
            requested_basis_kind,
            rejection_kind,
            basis_resolution_failure_kind,
            counters,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-subscription-admission-rejection:sha256:{digest:x}"
            )),
        }
    }

    pub fn declaration_identity(&self) -> &super::BridgeSubscriptionDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn requested_family_kind(&self) -> BridgeSubscriptionDeclarationFamilyKind {
        self.requested_family_kind
    }

    pub fn requested_basis_kind(&self) -> BridgeSubscriptionBasisKind {
        self.requested_basis_kind
    }

    pub fn rejection_kind(&self) -> BridgeSubscriptionAdmissionRejectionKind {
        self.rejection_kind
    }

    pub fn basis_resolution_failure_kind(
        &self,
    ) -> Option<BridgeSubscriptionBasisResolutionFailureKind> {
        self.basis_resolution_failure_kind
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

impl BridgeSubscriptionAdmissionRejectionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BasisKindDivergence => "basis_kind_divergence",
            Self::BasisResolutionFailure => "basis_resolution_failure",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBridgeSubscription {
    admitted_subscription_identity: BridgeAdmittedSubscriptionIdentity,
    declaration: BridgeSubscriptionDeclaration,
    basis_binding: ValidatedSubscriptionBasisBinding,
    signal_strategy: BridgeSignalStrategyDescriptor,
    counters: BridgeSubscriptionCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl AdmittedBridgeSubscription {
    pub(crate) fn admit(
        runtime: &RuntimeBridge,
        declaration: &BridgeSubscriptionDeclaration,
        basis_request: BridgeSubscriptionBasisRequest,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, BridgeSubscriptionAdmissionRejection> {
        let requested_basis_kind = basis_request.basis_kind();
        if !family_supports_basis_kind(declaration.requested_family_kind(), requested_basis_kind) {
            return Err(BridgeSubscriptionAdmissionRejection::basis_kind_divergence(
                declaration,
                requested_basis_kind,
            ));
        }

        let basis_binding = ValidatedSubscriptionBasisBinding::bind(
            runtime,
            declaration,
            &basis_request,
            execution,
        )
        .map_err(|failure| {
            BridgeSubscriptionAdmissionRejection::basis_resolution_failure(
                declaration,
                requested_basis_kind,
                failure.kind(),
            )
        })?;
        let signal_strategy = BridgeSignalStrategyDescriptor::lower(declaration, &basis_binding);

        let canonical_basis = Arc::<str>::from(format!(
            "bridge-admitted-subscription|declaration={}|basis={}|strategy={}",
            declaration.digest(),
            basis_binding.digest(),
            signal_strategy.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Ok(Self {
            admitted_subscription_identity: BridgeAdmittedSubscriptionIdentity::admit_bridge_owned(
                format!("bridge-admitted-subscription-id:sha256:{digest:x}"),
            ),
            declaration: declaration.clone(),
            basis_binding,
            signal_strategy,
            counters: BridgeSubscriptionCounters::from_admitted_subscription(),
            canonical_basis,
            digest: Arc::from(format!("bridge-admitted-subscription:sha256:{digest:x}")),
        })
    }

    pub fn admitted_subscription_identity(&self) -> &BridgeAdmittedSubscriptionIdentity {
        &self.admitted_subscription_identity
    }

    pub fn declaration(&self) -> &BridgeSubscriptionDeclaration {
        &self.declaration
    }

    pub fn basis_binding(&self) -> &ValidatedSubscriptionBasisBinding {
        &self.basis_binding
    }

    pub fn signal_strategy(&self) -> &BridgeSignalStrategyDescriptor {
        &self.signal_strategy
    }

    pub fn counters(&self) -> &BridgeSubscriptionCounters {
        &self.counters
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

fn family_supports_basis_kind(
    family_kind: BridgeSubscriptionDeclarationFamilyKind,
    basis_kind: BridgeSubscriptionBasisKind,
) -> bool {
    match family_kind {
        BridgeSubscriptionDeclarationFamilyKind::DetailExact => matches!(
            basis_kind,
            BridgeSubscriptionBasisKind::Snapshot | BridgeSubscriptionBasisKind::BranchHead
        ),
        BridgeSubscriptionDeclarationFamilyKind::CollectionMembership => matches!(
            basis_kind,
            BridgeSubscriptionBasisKind::Snapshot | BridgeSubscriptionBasisKind::BranchHead
        ),
    }
}
