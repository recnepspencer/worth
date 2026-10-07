use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryWorkspace;

use super::{
    WorthQueryDomainHandleDenial, WorthQueryDomainHandleDenialKind,
    WorthQueryInstalledDomainAuthorityWitness,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledDomainExecutionDriftKind {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    DomainNotInstalled,
    ForeignRuntime,
    StaleInstallation,
    PackageMeaningChanged,
}

impl WorthQueryInstalledDomainExecutionDriftKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Handle(_) => "handle-denied",
            Self::DomainNotInstalled => "domain-not-installed",
            Self::ForeignRuntime => "foreign-runtime",
            Self::StaleInstallation => "stale-installation",
            Self::PackageMeaningChanged => "package-meaning-changed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledDomainExecutionNextAction {
    InstallDomainPackage,
    UseOwningRuntime,
    RebindCurrentInstallation,
    ReconcilePackageMeaning,
}

impl WorthQueryInstalledDomainExecutionNextAction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InstallDomainPackage => "install-domain-package",
            Self::UseOwningRuntime => "use-owning-runtime",
            Self::RebindCurrentInstallation => "rebind-current-installation",
            Self::ReconcilePackageMeaning => "reconcile-package-meaning",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainExecutionDriftCounters {
    planning_attempts: u64,
    lower_runtime_attempts: u64,
    execution_attempts: u64,
}

impl WorthQueryInstalledDomainExecutionDriftCounters {
    pub(super) const fn denied_before_work() -> Self {
        Self {
            planning_attempts: 0,
            lower_runtime_attempts: 0,
            execution_attempts: 0,
        }
    }

    pub const fn planning_attempts(self) -> u64 {
        self.planning_attempts
    }

    pub const fn lower_runtime_attempts(self) -> u64 {
        self.lower_runtime_attempts
    }

    pub const fn execution_attempts(self) -> u64 {
        self.execution_attempts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainExecutionDrift {
    kind: WorthQueryInstalledDomainExecutionDriftKind,
    next_action: WorthQueryInstalledDomainExecutionNextAction,
    counters: WorthQueryInstalledDomainExecutionDriftCounters,
    retained_authority: WorthQueryInstalledDomainAuthorityWitness,
    current_authority: Option<WorthQueryInstalledDomainAuthorityWitness>,
    drift_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryInstalledDomainExecutionDrift {
    pub(crate) fn closed(
        witness: &WorthQueryInstalledDomainAuthorityWitness,
        denial: worth_query_execution::facade::primary_graph::WorthQueryHandleDenial,
    ) -> Self {
        Self::new(
            witness,
            None,
            WorthQueryInstalledDomainExecutionDriftKind::Handle(denial),
            WorthQueryInstalledDomainExecutionNextAction::UseOwningRuntime,
        )
    }
    pub(crate) fn validate<D: 'static>(
        witness: &WorthQueryInstalledDomainAuthorityWitness,
        workspace: &WorthQueryWorkspace,
    ) -> Result<(), Self> {
        workspace
            .validate_installed_domain_witness::<D>(witness)
            .map_err(|denial| Self::from_handle_denial(witness, denial))
    }

    pub(crate) fn validate_current(
        witness: &WorthQueryInstalledDomainAuthorityWitness,
    ) -> Result<(), Self> {
        if witness.authority().is_current_installation_generation() {
            Ok(())
        } else {
            Err(Self::stale_installation(witness))
        }
    }

    pub(crate) fn validate_retained_for_current(
        retained: &WorthQueryInstalledDomainAuthorityWitness,
        current: &WorthQueryInstalledDomainAuthorityWitness,
    ) -> Result<(), Self> {
        if retained.authority().runtime_authority() != current.authority().runtime_authority() {
            return Err(Self::new(
                retained,
                Some(current),
                WorthQueryInstalledDomainExecutionDriftKind::ForeignRuntime,
                WorthQueryInstalledDomainExecutionNextAction::UseOwningRuntime,
            ));
        }
        if retained.package_identity() != current.package_identity() {
            return Err(Self::new(
                retained,
                Some(current),
                WorthQueryInstalledDomainExecutionDriftKind::PackageMeaningChanged,
                WorthQueryInstalledDomainExecutionNextAction::ReconcilePackageMeaning,
            ));
        }
        if retained.authority().installation_generation()
            != current.authority().installation_generation()
            || !retained.authority().is_current_installation_generation()
            || !current.authority().is_current_installation_generation()
        {
            return Err(Self::new(
                retained,
                Some(current),
                WorthQueryInstalledDomainExecutionDriftKind::StaleInstallation,
                WorthQueryInstalledDomainExecutionNextAction::RebindCurrentInstallation,
            ));
        }
        Ok(())
    }

    pub(crate) fn stale_installation(witness: &WorthQueryInstalledDomainAuthorityWitness) -> Self {
        Self::new(
            witness,
            None,
            WorthQueryInstalledDomainExecutionDriftKind::StaleInstallation,
            WorthQueryInstalledDomainExecutionNextAction::RebindCurrentInstallation,
        )
    }

    fn from_handle_denial(
        witness: &WorthQueryInstalledDomainAuthorityWitness,
        denial: WorthQueryDomainHandleDenial,
    ) -> Self {
        let (kind, next_action) = match denial.kind() {
            WorthQueryDomainHandleDenialKind::DomainNotInstalled => (
                WorthQueryInstalledDomainExecutionDriftKind::DomainNotInstalled,
                WorthQueryInstalledDomainExecutionNextAction::InstallDomainPackage,
            ),
            WorthQueryDomainHandleDenialKind::ForeignRuntime => (
                WorthQueryInstalledDomainExecutionDriftKind::ForeignRuntime,
                WorthQueryInstalledDomainExecutionNextAction::UseOwningRuntime,
            ),
            WorthQueryDomainHandleDenialKind::StaleInstallationGeneration => (
                WorthQueryInstalledDomainExecutionDriftKind::StaleInstallation,
                WorthQueryInstalledDomainExecutionNextAction::RebindCurrentInstallation,
            ),
            WorthQueryDomainHandleDenialKind::PackageIdentityChanged => (
                WorthQueryInstalledDomainExecutionDriftKind::PackageMeaningChanged,
                WorthQueryInstalledDomainExecutionNextAction::ReconcilePackageMeaning,
            ),
        };
        Self::new(witness, None, kind, next_action)
    }

    fn new(
        retained_authority: &WorthQueryInstalledDomainAuthorityWitness,
        current_authority: Option<&WorthQueryInstalledDomainAuthorityWitness>,
        kind: WorthQueryInstalledDomainExecutionDriftKind,
        next_action: WorthQueryInstalledDomainExecutionNextAction,
    ) -> Self {
        let mut drift_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::InstalledDomainExecution)
                .field_shape(WorthQueryEvidenceTag::new("outcome"), "authority-drift")
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("next_action"),
                    next_action.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("retained_authority"),
                    retained_authority.witness_identity(),
                );
        if let Some(current_authority) = current_authority {
            drift_identity = drift_identity.field_evidence_identity(
                WorthQueryEvidenceTag::new("current_authority"),
                current_authority.witness_identity(),
            );
        }
        Self {
            kind,
            next_action,
            counters: WorthQueryInstalledDomainExecutionDriftCounters::denied_before_work(),
            retained_authority: retained_authority.clone(),
            current_authority: current_authority.cloned(),
            drift_identity: drift_identity.seal(),
        }
    }

    pub fn kind(&self) -> WorthQueryInstalledDomainExecutionDriftKind {
        self.kind
    }

    pub fn next_action(&self) -> WorthQueryInstalledDomainExecutionNextAction {
        self.next_action
    }

    pub fn counters(&self) -> WorthQueryInstalledDomainExecutionDriftCounters {
        self.counters
    }

    pub fn retained_authority(&self) -> &WorthQueryInstalledDomainAuthorityWitness {
        &self.retained_authority
    }

    pub fn current_authority(&self) -> Option<&WorthQueryInstalledDomainAuthorityWitness> {
        self.current_authority.as_ref()
    }

    pub fn drift_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.drift_identity
    }
}

impl std::fmt::Display for WorthQueryInstalledDomainExecutionDrift {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "installed domain authority drift: {}; next action: {}",
            self.kind.as_str(),
            self.next_action.as_str()
        )
    }
}

impl std::error::Error for WorthQueryInstalledDomainExecutionDrift {}
