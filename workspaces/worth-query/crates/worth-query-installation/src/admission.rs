use std::collections::{BTreeMap, BTreeSet};

use worth_proof::{
    Admitted, AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    CurrentValidity, FreshnessScopedBasis, Recipe, Unresolved,
};

use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use crate::package::WorthQueryValidatedPortableDomainPackage;
use worth_foundational::facade::{CanonicalDigestDerivationDenial, CanonicalDigestId};

mod artifact_support;
mod identity;
mod meaning;
mod requirements;

pub use artifact_support::WorthQueryArtifactVersionSupport;
use identity::admission_identity;
pub use identity::WorthQueryInstallationAdmissionIdentity;
use meaning::{admission_meaning, WorthQueryInstallationAdmissionMeaning};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstallationSupportStatus {
    Admitted,
    Deferred,
    Unsupported,
}

impl WorthQueryInstallationSupportStatus {
    pub(super) fn canonical_part(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Deferred => "deferred",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstallationAdmissionProfile {
    support_identity: String,
    configuration_identity: String,
    capability_statuses: BTreeMap<String, WorthQueryInstallationSupportStatus>,
    configuration_statuses: BTreeMap<String, bool>,
    operating_statuses: BTreeMap<String, WorthQueryInstallationSupportStatus>,
    artifact_version_statuses: BTreeMap<(String, u32, u32), WorthQueryArtifactVersionSupport>,
    artifact_comparator_statuses: BTreeMap<String, WorthQueryInstallationSupportStatus>,
    conflicting_rows: BTreeSet<String>,
}

impl WorthQueryInstallationAdmissionProfile {
    pub fn new(
        support_identity: impl Into<String>,
        configuration_identity: impl Into<String>,
    ) -> Self {
        Self {
            support_identity: support_identity.into(),
            configuration_identity: configuration_identity.into(),
            capability_statuses: BTreeMap::new(),
            configuration_statuses: BTreeMap::new(),
            operating_statuses: BTreeMap::new(),
            artifact_version_statuses: BTreeMap::new(),
            artifact_comparator_statuses: BTreeMap::new(),
            conflicting_rows: BTreeSet::new(),
        }
    }

    pub fn capability(
        mut self,
        family: impl Into<String>,
        status: WorthQueryInstallationSupportStatus,
    ) -> Self {
        let family = family.into();
        retain_profile_row(
            &mut self.capability_statuses,
            &mut self.conflicting_rows,
            "capability",
            family,
            status,
        );
        self
    }

    pub fn configuration(mut self, section: impl Into<String>, enabled: bool) -> Self {
        let section = section.into();
        retain_profile_row(
            &mut self.configuration_statuses,
            &mut self.conflicting_rows,
            "configuration",
            section,
            enabled,
        );
        self
    }

    pub fn operating_requirement(
        mut self,
        requirement: impl Into<String>,
        status: WorthQueryInstallationSupportStatus,
    ) -> Self {
        let requirement = requirement.into();
        retain_profile_row(
            &mut self.operating_statuses,
            &mut self.conflicting_rows,
            "operating",
            requirement,
            status,
        );
        self
    }

    pub fn admit(
        &self,
        package: WorthQueryValidatedPortableDomainPackage,
    ) -> Result<WorthQueryAdmittedPortableDomainPackage, WorthQueryInstallationAdmissionDenial>
    {
        self.validate_profile_identity_and_conflicts()?;
        self.admit_declared_requirements(&package)?;
        self.admit_artifact_contracts(&package)?;

        let (admission_identity, admission_work) =
            admission_identity(&package, self).map_err(map_admission_canonical_denial)?;
        let canonical_work = package.canonical_work().combine(admission_work);
        let admission_meaning = admission_meaning(&package, self);
        let basis = WorthQueryInstallationAdmissionBasis {
            package_identity: *package.identity().digest(),
            support_identity: self.support_identity.clone(),
            configuration_identity: self.configuration_identity.clone(),
            admission_identity: admission_identity.clone(),
        };
        let resolved = Recipe::<Unresolved, _>::new(package).resolve_with_authority(
            basis,
            AuthorityWitness::from_authority_marker(InstallationProfileResolutionAuthority {
                _private: (),
            }),
        );
        let lowered = resolved.lower_with_capability(CapabilityWitness::from_capability_marker(
            InstallationSupportMatchedCapability { _private: () },
        ));
        let recipe = lowered.admit_with_authority(AuthorityWitness::from_authority_marker(
            InstallationAdmissionAuthority { _private: () },
        ));
        Ok(WorthQueryAdmittedPortableDomainPackage {
            recipe,
            admission_identity,
            admission_meaning,
            canonical_work,
        })
    }
}

#[derive(Clone, Debug)]
pub struct WorthQueryAdmittedPortableDomainPackage {
    recipe: AdmittedPortablePackageRecipe,
    admission_identity: WorthQueryInstallationAdmissionIdentity,
    admission_meaning: WorthQueryInstallationAdmissionMeaning,
    canonical_work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryAdmittedPortableDomainPackage {
    pub fn package(&self) -> &WorthQueryValidatedPortableDomainPackage {
        self.recipe.payload()
    }

    pub fn admission_identity(&self) -> &WorthQueryInstallationAdmissionIdentity {
        &self.admission_identity
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.canonical_work
    }

    pub(crate) fn has_same_admission_authority(&self, other: &Self) -> bool {
        self.admission_meaning == other.admission_meaning
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorthQueryInstallationAdmissionBasis {
    package_identity: CanonicalDigestId,
    support_identity: String,
    configuration_identity: String,
    admission_identity: WorthQueryInstallationAdmissionIdentity,
}

struct InstallationProfileResolutionAuthority {
    _private: (),
}
impl AuthorityMarker for InstallationProfileResolutionAuthority {}

struct InstallationSupportMatchedCapability {
    _private: (),
}
impl CapabilityMarker for InstallationSupportMatchedCapability {}

struct InstallationAdmissionAuthority {
    _private: (),
}
impl AuthorityMarker for InstallationAdmissionAuthority {}

type AdmittedPortablePackageRecipe = Recipe<
    Admitted,
    WorthQueryValidatedPortableDomainPackage,
    FreshnessScopedBasis<
        CurrentValidity,
        worth_proof::AssumptionBasis<WorthQueryInstallationAdmissionBasis>,
    >,
>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInstallationAdmissionDenialKind {
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

trait AdmissionProfileKey: Ord + Clone {
    fn profile_subject(&self) -> String;
}

impl AdmissionProfileKey for String {
    fn profile_subject(&self) -> String {
        self.clone()
    }
}

impl AdmissionProfileKey for (String, u32, u32) {
    fn profile_subject(&self) -> String {
        format!("{}:{}:{}", self.0, self.1, self.2)
    }
}

fn retain_profile_row<K: AdmissionProfileKey, T: Clone + Eq>(
    rows: &mut BTreeMap<K, T>,
    conflicts: &mut BTreeSet<String>,
    dimension: &str,
    subject: K,
    value: T,
) {
    if rows
        .insert(subject.clone(), value.clone())
        .is_some_and(|prior| prior != value)
    {
        conflicts.insert(format!("{dimension}:{}", subject.profile_subject()));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstallationAdmissionDenial {
    kind: WorthQueryInstallationAdmissionDenialKind,
    subject: String,
}

impl std::fmt::Display for WorthQueryInstallationAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "package installation admission denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryInstallationAdmissionDenial {}

impl WorthQueryInstallationAdmissionDenial {
    pub fn kind(&self) -> WorthQueryInstallationAdmissionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    fn canonical(kind: WorthQueryInstallationAdmissionDenialKind) -> Self {
        Self {
            kind,
            subject: "installation-admission-canonical-identity".to_string(),
        }
    }
}

fn map_admission_canonical_denial(
    denial: CanonicalDigestDerivationDenial,
) -> WorthQueryInstallationAdmissionDenial {
    let kind = match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { .. } => {
            WorthQueryInstallationAdmissionDenialKind::CanonicalEntryBudgetExceeded
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. } => {
            WorthQueryInstallationAdmissionDenialKind::CanonicalEncodedByteBudgetExceeded
        }
        _ => WorthQueryInstallationAdmissionDenialKind::CanonicalDigestSlotRejected,
    };
    WorthQueryInstallationAdmissionDenial::canonical(kind)
}
