use crate::application::{
    WorthQueryApplicationFacade, WorthQueryDeclarationEntryContributionCategoryFamily,
    WorthQueryDomainEntryMarker, WorthQueryDomainEntrySupportSnapshot,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryDomainDeclarationFamilyDefinition, WorthQueryDomainGraphReadOperationDefinition,
    WorthQueryDomainIdentityDeclaration, WorthQueryDomainInvariantDefinition,
    WorthQueryDomainOperationDefinitionRecord, WorthQueryDomainOperationGraphParticipationRecord,
    WorthQueryDomainPackageIdentity, WorthQueryValidatedDomainPackage,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainPackageAdmissionDenialKind {
    InvalidSupportProfileIdentity,
    InvalidConfigurationProfileIdentity,
    ConflictingProfileRow,
    UnsupportedCapability,
    DeferredCapability,
    DisabledConfiguration,
    DeferredOperatingRequirement,
    UnsupportedOperatingRequirement,
    UnsupportedArtifactVersion,
    RetiredArtifactVersion,
    ArtifactMigrationRequired,
    AmbiguousArtifactMigration,
    DeferredArtifactComparator,
    UnsupportedArtifactComparator,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainPackageAdmissionDenial {
    identity_derivation_denial: Option<IdentityDerivationDenial>,
    kind: WorthQueryDomainPackageAdmissionDenialKind,
    subject: String,
}

impl WorthQueryDomainPackageAdmissionDenial {
    pub fn identity_derivation_denial(&self) -> Option<&IdentityDerivationDenial> {
        self.identity_derivation_denial.as_ref()
    }

    pub(crate) fn new(
        kind: WorthQueryDomainPackageAdmissionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            identity_derivation_denial: None,
            kind,
            subject: subject.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryDomainPackageAdmissionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryDomainPackageAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.subject)
    }
}

impl std::error::Error for WorthQueryDomainPackageAdmissionDenial {}
use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDomainOperatingRequirement,
};

pub(crate) struct WorthQueryAdmittedDomainPackage<D: WorthQueryDomainEntryMarker> {
    pub(crate) marker: D,
    pub(crate) identity: WorthQueryDomainIdentityDeclaration<D>,
    pub(crate) package_identity: WorthQueryDomainPackageIdentity,
    pub(crate) admission_identity: WorthQueryEvidenceIdentity,
    pub(crate) portable_package:
        worth_query_installation::facade::WorthQueryAdmittedPortableDomainPackage,
    pub(crate) support_snapshot: WorthQueryDomainEntrySupportSnapshot,
    pub(crate) required_capabilities: Vec<WorthQueryCapabilityFamily>,
    pub(crate) required_configuration: Vec<WorthQueryConfigSectionFamily>,
    pub(crate) operating_requirements: Vec<WorthQueryDomainOperatingRequirement>,
    pub(crate) invariant_definitions: Vec<WorthQueryDomainInvariantDefinition>,
    pub(crate) graph_read_operations: Vec<WorthQueryDomainGraphReadOperationDefinition>,
    pub(crate) declaration_families: Vec<WorthQueryDomainDeclarationFamilyDefinition>,
    pub(crate) domain_operations: Vec<WorthQueryDomainOperationDefinitionRecord>,
    pub(crate) operation_graph_participations:
        Vec<WorthQueryDomainOperationGraphParticipationRecord>,
    pub(crate) operation_required_domains:
        Vec<super::WorthQueryDomainOperationRequiredDomainRecord>,
    pub(crate) contribution_policy: Vec<WorthQueryDeclarationEntryContributionCategoryFamily>,
}

#[cfg(test)]
pub(crate) fn admit_domain_package<D: WorthQueryDomainEntryMarker>(
    package: WorthQueryValidatedDomainPackage<D>,
) -> Result<WorthQueryAdmittedDomainPackage<D>, WorthQueryDomainPackageAdmissionDenial> {
    admit_domain_package_with_artifact_support(
        package,
        &super::WorthQueryArtifactInstallationSupport::default(),
    )
}

pub(crate) fn admit_domain_package_with_artifact_support<D: WorthQueryDomainEntryMarker>(
    package: WorthQueryValidatedDomainPackage<D>,
    artifact_support: &super::WorthQueryArtifactInstallationSupport,
) -> Result<WorthQueryAdmittedDomainPackage<D>, WorthQueryDomainPackageAdmissionDenial> {
    let facade = WorthQueryApplicationFacade::runtime_backed_default();
    let support_matrix = facade.support_matrix();
    let support_snapshot =
        WorthQueryDomainEntrySupportSnapshot::from_support_report(facade.support_report()?)?;
    let portable_package = super::admission_profile::admit_portable_package(
        package.portable_package,
        &package.required_capabilities,
        &package.required_configuration,
        &package.operating_requirements,
        &facade,
        artifact_support,
    )?;

    let admission_identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::DomainPackageAdmission)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("package"),
                package.package_identity.evidence_identity(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("support"),
                support_matrix.support_matrix_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("configuration"),
                facade.validated_config().validated_digest(),
            )
            .field_digest(
                WorthQueryEvidenceTag::new("portable_admission"),
                portable_package.admission_identity().digest(),
            )
            .seal();

    Ok(WorthQueryAdmittedDomainPackage {
        marker: package.marker,
        identity: package.identity,
        package_identity: package.package_identity,
        admission_identity,
        portable_package,
        support_snapshot,
        required_capabilities: package.required_capabilities,
        required_configuration: package.required_configuration,
        operating_requirements: package.operating_requirements,
        invariant_definitions: package.invariant_definitions,
        graph_read_operations: package.graph_read_operations,
        declaration_families: package.declaration_families,
        domain_operations: package.domain_operations,
        operation_graph_participations: package.operation_graph_participations,
        operation_required_domains: package.operation_required_domains,
        contribution_policy: package.contribution_policy,
    })
}

impl From<IdentityDerivationDenial> for WorthQueryDomainPackageAdmissionDenial {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self {
            kind: WorthQueryDomainPackageAdmissionDenialKind::CanonicalDigestSlotRejected,
            subject: format!("{:?}", cause.scope),
            identity_derivation_denial: Some(cause),
        }
    }
}
