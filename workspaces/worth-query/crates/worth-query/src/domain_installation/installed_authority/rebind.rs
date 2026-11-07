use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryDomainPackageIdentity, WorthQueryInstalledDomainAuthority,
    WorthQueryInstalledDomainAuthorityWitness, WorthQueryInstalledDomainExecutionDriftCounters,
    WorthQueryInstalledDomainHandle,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainRebindRequest<D> {
    prior: WorthQueryInstalledDomainAuthorityWitness,
    marker: PhantomData<fn() -> D>,
}

impl<D> WorthQueryDomainRebindRequest<D> {
    pub(crate) fn new(prior: WorthQueryInstalledDomainAuthorityWitness) -> Self {
        Self {
            prior,
            marker: PhantomData,
        }
    }

    pub fn prior_witness(&self) -> &WorthQueryInstalledDomainAuthorityWitness {
        &self.prior
    }

    pub(crate) fn into_prior(self) -> WorthQueryInstalledDomainAuthorityWitness {
        self.prior
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainRebindDenialKind {
    DomainNotInstalled,
    PackageMeaningChanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainRebindNextAction {
    InstallDomainPackage,
    ReconcilePackageMeaning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainRebindDenial {
    kind: WorthQueryDomainRebindDenialKind,
    next_action: WorthQueryDomainRebindNextAction,
    prior_package_identity: WorthQueryDomainPackageIdentity,
    current_package_identity: Option<WorthQueryDomainPackageIdentity>,
    counters: WorthQueryInstalledDomainExecutionDriftCounters,
    denial_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryDomainRebindDenial {
    pub(crate) fn domain_not_installed(prior: &WorthQueryInstalledDomainAuthorityWitness) -> Self {
        Self::new(
            WorthQueryDomainRebindDenialKind::DomainNotInstalled,
            WorthQueryDomainRebindNextAction::InstallDomainPackage,
            prior.package_identity().clone(),
            None,
        )
    }

    pub(crate) fn package_meaning_changed(
        prior: &WorthQueryInstalledDomainAuthorityWitness,
        current: &WorthQueryInstalledDomainAuthority,
    ) -> Self {
        Self::new(
            WorthQueryDomainRebindDenialKind::PackageMeaningChanged,
            WorthQueryDomainRebindNextAction::ReconcilePackageMeaning,
            prior.package_identity().clone(),
            Some(current.package_identity().clone()),
        )
    }

    fn new(
        kind: WorthQueryDomainRebindDenialKind,
        next_action: WorthQueryDomainRebindNextAction,
        prior_package_identity: WorthQueryDomainPackageIdentity,
        current_package_identity: Option<WorthQueryDomainPackageIdentity>,
    ) -> Self {
        let counters = WorthQueryInstalledDomainExecutionDriftCounters::denied_before_work();
        let mut denial_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainRebind)
                .field_shape(WorthQueryEvidenceTag::new("outcome"), "denied")
                .field_shape(
                    WorthQueryEvidenceTag::new("kind"),
                    match kind {
                        WorthQueryDomainRebindDenialKind::DomainNotInstalled => {
                            "domain-not-installed"
                        }
                        WorthQueryDomainRebindDenialKind::PackageMeaningChanged => {
                            "package-meaning-changed"
                        }
                    },
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("prior_package"),
                    prior_package_identity.evidence_identity(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("planning_attempts"),
                    counters.planning_attempts().to_string(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("lower_runtime_attempts"),
                    counters.lower_runtime_attempts().to_string(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("execution_attempts"),
                    counters.execution_attempts().to_string(),
                );
        if let Some(current_package_identity) = current_package_identity.as_ref() {
            denial_identity = denial_identity.field_evidence_identity(
                WorthQueryEvidenceTag::new("current_package"),
                current_package_identity.evidence_identity(),
            );
        }
        let denial_identity = denial_identity.seal().expect("fixed rebind grammar: enum labels, three counters and at most two sealed package identities");
        Self {
            kind,
            next_action,
            prior_package_identity,
            current_package_identity,
            counters,
            denial_identity,
        }
    }

    pub fn kind(&self) -> WorthQueryDomainRebindDenialKind {
        self.kind
    }

    pub fn next_action(&self) -> WorthQueryDomainRebindNextAction {
        self.next_action
    }

    pub fn prior_package_identity(&self) -> &WorthQueryDomainPackageIdentity {
        &self.prior_package_identity
    }

    pub fn current_package_identity(&self) -> Option<&WorthQueryDomainPackageIdentity> {
        self.current_package_identity.as_ref()
    }

    pub fn counters(&self) -> WorthQueryInstalledDomainExecutionDriftCounters {
        self.counters
    }

    pub fn denial_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.denial_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainRebindReceipt {
    prior_authority: Arc<WorthQueryInstalledDomainAuthority>,
    current_authority: Arc<WorthQueryInstalledDomainAuthority>,
    prior_witness_identity: WorthQueryEvidenceIdentity,
    current_witness_identity: WorthQueryEvidenceIdentity,
    package_identity: WorthQueryDomainPackageIdentity,
    semantic_equivalence_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryDomainRebindReceipt {
    pub(crate) fn new(
        prior: &WorthQueryInstalledDomainAuthorityWitness,
        current: &WorthQueryInstalledDomainAuthorityWitness,
    ) -> Self {
        let package_identity = current.package_identity().clone();
        let semantic_equivalence_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainRebind)
                .field_shape(
                    WorthQueryEvidenceTag::new("claim"),
                    "package-semantic-equivalence",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("package"),
                    package_identity.evidence_identity(),
                )
                .seal().expect("fixed rebind grammar: enum labels, three counters and at most two sealed package identities");
        let receipt_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainRebind)
                .field_shape(WorthQueryEvidenceTag::new("outcome"), "rebound")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("prior"),
                    prior.witness_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("current"),
                    current.witness_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("semantic_equivalence"),
                    &semantic_equivalence_identity,
                )
                .seal().expect("fixed rebind grammar: enum labels, three counters and at most two sealed package identities");
        Self {
            prior_authority: prior.authority_arc(),
            current_authority: current.authority_arc(),
            prior_witness_identity: prior.witness_identity().clone(),
            current_witness_identity: current.witness_identity().clone(),
            package_identity,
            semantic_equivalence_identity,
            receipt_identity,
        }
    }

    pub fn prior_witness_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.prior_witness_identity
    }

    pub fn current_witness_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.current_witness_identity
    }

    pub fn package_identity(&self) -> &WorthQueryDomainPackageIdentity {
        &self.package_identity
    }

    pub fn semantic_equivalence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.semantic_equivalence_identity
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub(crate) fn binds_authorities(
        &self,
        prior: &Arc<WorthQueryInstalledDomainAuthority>,
        current: &Arc<WorthQueryInstalledDomainAuthority>,
    ) -> bool {
        Arc::ptr_eq(&self.prior_authority, prior) && Arc::ptr_eq(&self.current_authority, current)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReboundDomainHandle<D> {
    handle: WorthQueryInstalledDomainHandle<D>,
    receipt: WorthQueryDomainRebindReceipt,
}

impl<D> WorthQueryReboundDomainHandle<D> {
    pub(crate) fn new(
        handle: WorthQueryInstalledDomainHandle<D>,
        receipt: WorthQueryDomainRebindReceipt,
    ) -> Self {
        Self { handle, receipt }
    }

    pub fn handle(&self) -> &WorthQueryInstalledDomainHandle<D> {
        &self.handle
    }

    pub fn receipt(&self) -> &WorthQueryDomainRebindReceipt {
        &self.receipt
    }

    pub fn into_handle(self) -> WorthQueryInstalledDomainHandle<D> {
        self.handle
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryInstalledDomainHandle<D>,
        WorthQueryDomainRebindReceipt,
    ) {
        (self.handle, self.receipt)
    }
}
