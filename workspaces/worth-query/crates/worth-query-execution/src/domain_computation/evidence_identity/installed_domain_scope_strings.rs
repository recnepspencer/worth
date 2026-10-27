use super::scope::WorthQueryEvidenceScope;

macro_rules! installed_domain_evidence_scopes {
    () => {
        WorthQueryEvidenceScope::DomainPackageIdentity
            | WorthQueryEvidenceScope::DomainPackageValidation
            | WorthQueryEvidenceScope::DomainPackageAdmission
            | WorthQueryEvidenceScope::DomainInstallation
            | WorthQueryEvidenceScope::InstalledDomainHandle
            | WorthQueryEvidenceScope::InstalledDomainDeclarationContext
            | WorthQueryEvidenceScope::InstalledDomainWorld
            | WorthQueryEvidenceScope::InstalledDomainContributionTarget
            | WorthQueryEvidenceScope::InstalledDomainContribution
            | WorthQueryEvidenceScope::InstalledDomainExecution
            | WorthQueryEvidenceScope::InstalledDomainExecutionIndex
            | WorthQueryEvidenceScope::InstalledDomainSubstrateProvenance
            | WorthQueryEvidenceScope::InstalledDomainRebind
    };
}

pub(super) use installed_domain_evidence_scopes;

pub(super) fn installed_domain_evidence_scope_as_str(
    scope: WorthQueryEvidenceScope,
) -> &'static str {
    match scope {
        WorthQueryEvidenceScope::DomainPackageIdentity => "domain-package-identity",
        WorthQueryEvidenceScope::DomainPackageValidation => "domain-package-validation",
        WorthQueryEvidenceScope::DomainPackageAdmission => "domain-package-admission",
        WorthQueryEvidenceScope::DomainInstallation => "domain-installation",
        WorthQueryEvidenceScope::InstalledDomainHandle => "installed-domain-handle",
        WorthQueryEvidenceScope::InstalledDomainDeclarationContext => {
            "installed-domain-declaration-context"
        }
        WorthQueryEvidenceScope::InstalledDomainWorld => "installed-domain-world",
        WorthQueryEvidenceScope::InstalledDomainContributionTarget => {
            "installed-domain-contribution-target"
        }
        WorthQueryEvidenceScope::InstalledDomainContribution => "installed-domain-contribution",
        WorthQueryEvidenceScope::InstalledDomainExecution => "installed-domain-execution",
        WorthQueryEvidenceScope::InstalledDomainExecutionIndex => {
            "installed-domain-execution-index"
        }
        WorthQueryEvidenceScope::InstalledDomainSubstrateProvenance => {
            "installed-domain-substrate-provenance"
        }
        WorthQueryEvidenceScope::InstalledDomainRebind => "installed-domain-rebind",
        _ => unreachable!("installed-domain scope router received an unrelated scope"),
    }
}
