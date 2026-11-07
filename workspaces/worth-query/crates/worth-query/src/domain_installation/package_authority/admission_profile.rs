use worth_query_installation::facade::{
    WorthQueryAdmittedPortableDomainPackage, WorthQueryInstallationAdmissionDenial,
    WorthQueryInstallationAdmissionDenialKind, WorthQueryInstallationAdmissionProfile,
    WorthQueryInstallationSupportStatus, WorthQueryValidatedPortableDomainPackage,
};

use crate::application::{
    WorthQueryApplicationFacade, WorthQueryCapabilityFamily, WorthQueryCapabilityStatus,
    WorthQueryConfigSectionFamily, WorthQueryDomainEntrySupportSnapshot,
    WorthQueryDomainOperatingRequirement,
};
use crate::runtime::WorthQueryRuntimeFamilySupportStatus;

use super::{WorthQueryDomainPackageAdmissionDenial, WorthQueryDomainPackageAdmissionDenialKind};

pub(super) fn admit_portable_package(
    package: WorthQueryValidatedPortableDomainPackage,
    required_capabilities: &[WorthQueryCapabilityFamily],
    required_configuration: &[WorthQueryConfigSectionFamily],
    operating_requirements: &[WorthQueryDomainOperatingRequirement],
    facade: &WorthQueryApplicationFacade,
    artifact_support: &super::WorthQueryArtifactInstallationSupport,
) -> Result<WorthQueryAdmittedPortableDomainPackage, WorthQueryDomainPackageAdmissionDenial> {
    let support_matrix = facade.support_matrix();
    let mut profile = WorthQueryInstallationAdmissionProfile::new(
        support_matrix.support_matrix_digest(),
        facade.validated_config().validated_digest(),
    );
    for capability in required_capabilities {
        let status = support_matrix
            .descriptor(*capability)
            .map(|descriptor| descriptor.status())
            .unwrap_or(WorthQueryCapabilityStatus::Unsupported);
        profile = profile.capability(capability.as_str(), capability_status(status));
    }
    for section in required_configuration {
        profile = profile.configuration(
            section.as_str(),
            facade
                .validated_config()
                .resolve_section(*section)
                .enabled(),
        );
    }
    let support_snapshot =
        WorthQueryDomainEntrySupportSnapshot::from_support_report(facade.support_report()?)?;
    for requirement in operating_requirements {
        let status = match support_snapshot.operating_requirement_status(*requirement) {
            Some(WorthQueryRuntimeFamilySupportStatus::Supported) => {
                WorthQueryInstallationSupportStatus::Admitted
            }
            Some(WorthQueryRuntimeFamilySupportStatus::DeferredDebt) => {
                WorthQueryInstallationSupportStatus::Deferred
            }
            Some(WorthQueryRuntimeFamilySupportStatus::Unsupported) | None => {
                WorthQueryInstallationSupportStatus::Unsupported
            }
        };
        profile = profile.operating_requirement(requirement.as_str(), status);
    }
    profile = artifact_support.apply(profile);
    profile.admit(package).map_err(map_admission_denial)
}

fn capability_status(status: WorthQueryCapabilityStatus) -> WorthQueryInstallationSupportStatus {
    match status {
        WorthQueryCapabilityStatus::Admitted => WorthQueryInstallationSupportStatus::Admitted,
        WorthQueryCapabilityStatus::DeferredDebt => WorthQueryInstallationSupportStatus::Deferred,
        WorthQueryCapabilityStatus::Unsupported => WorthQueryInstallationSupportStatus::Unsupported,
    }
}

fn map_admission_denial(
    denial: WorthQueryInstallationAdmissionDenial,
) -> WorthQueryDomainPackageAdmissionDenial {
    let kind = match denial.kind() {
        WorthQueryInstallationAdmissionDenialKind::InvalidSupportProfileIdentity => {
            WorthQueryDomainPackageAdmissionDenialKind::InvalidSupportProfileIdentity
        }
        WorthQueryInstallationAdmissionDenialKind::InvalidConfigurationProfileIdentity => {
            WorthQueryDomainPackageAdmissionDenialKind::InvalidConfigurationProfileIdentity
        }
        WorthQueryInstallationAdmissionDenialKind::ConflictingProfileRow => {
            WorthQueryDomainPackageAdmissionDenialKind::ConflictingProfileRow
        }
        WorthQueryInstallationAdmissionDenialKind::UnsupportedCapability => {
            WorthQueryDomainPackageAdmissionDenialKind::UnsupportedCapability
        }
        WorthQueryInstallationAdmissionDenialKind::DeferredCapability => {
            WorthQueryDomainPackageAdmissionDenialKind::DeferredCapability
        }
        WorthQueryInstallationAdmissionDenialKind::DisabledConfiguration => {
            WorthQueryDomainPackageAdmissionDenialKind::DisabledConfiguration
        }
        WorthQueryInstallationAdmissionDenialKind::DeferredOperatingRequirement => {
            WorthQueryDomainPackageAdmissionDenialKind::DeferredOperatingRequirement
        }
        WorthQueryInstallationAdmissionDenialKind::UnsupportedOperatingRequirement => {
            WorthQueryDomainPackageAdmissionDenialKind::UnsupportedOperatingRequirement
        }
        WorthQueryInstallationAdmissionDenialKind::UnsupportedArtifactVersion => {
            WorthQueryDomainPackageAdmissionDenialKind::UnsupportedArtifactVersion
        }
        WorthQueryInstallationAdmissionDenialKind::RetiredArtifactVersion => {
            WorthQueryDomainPackageAdmissionDenialKind::RetiredArtifactVersion
        }
        WorthQueryInstallationAdmissionDenialKind::ArtifactMigrationRequired => {
            WorthQueryDomainPackageAdmissionDenialKind::ArtifactMigrationRequired
        }
        WorthQueryInstallationAdmissionDenialKind::AmbiguousArtifactMigration => {
            WorthQueryDomainPackageAdmissionDenialKind::AmbiguousArtifactMigration
        }
        WorthQueryInstallationAdmissionDenialKind::DeferredArtifactComparator => {
            WorthQueryDomainPackageAdmissionDenialKind::DeferredArtifactComparator
        }
        WorthQueryInstallationAdmissionDenialKind::UnsupportedArtifactComparator => {
            WorthQueryDomainPackageAdmissionDenialKind::UnsupportedArtifactComparator
        }
        WorthQueryInstallationAdmissionDenialKind::CanonicalEntryBudgetExceeded => {
            WorthQueryDomainPackageAdmissionDenialKind::CanonicalEntryBudgetExceeded
        }
        WorthQueryInstallationAdmissionDenialKind::CanonicalEncodedByteBudgetExceeded => {
            WorthQueryDomainPackageAdmissionDenialKind::CanonicalEncodedByteBudgetExceeded
        }
        WorthQueryInstallationAdmissionDenialKind::CanonicalDigestSlotRejected => {
            WorthQueryDomainPackageAdmissionDenialKind::CanonicalDigestSlotRejected
        }
    };
    WorthQueryDomainPackageAdmissionDenial::new(kind, denial.subject())
}
