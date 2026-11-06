use super::registry::{
    WorthQueryCapabilityFamily, WorthQueryCapabilityRegistry, WorthQueryCapabilityStatus,
};
use super::WorthQuerySupportMatrix;
use crate::application::config::{
    ConfigurationAdmissionError, ValidatedWorthQueryConfig, WorthQueryConfig,
    WorthQueryConfigSectionFamily, WorthQueryConfigSectionResolution, WorthQuerySubsystemOwner,
};
use crate::composition::{
    runtime_backed_query_composition_support_profile, QueryCompositionSupportProfile,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::identity_evolution::{
    runtime_backed_direct_identity_evolution_support_profile, IdentityEvolutionSupportProfile,
};
use crate::query_context::{
    runtime_backed_narrow_query_context_support_profile, QueryContextSupportProfile,
};

pub type WorthQueryQueryContextSupportProfile = QueryContextSupportProfile;
pub type WorthQueryIdentityEvolutionSupportProfile = IdentityEvolutionSupportProfile;
pub type WorthQueryQueryCompositionSupportProfile = QueryCompositionSupportProfile;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySupportReportCounters {
    support_report_generation_count: usize,
}

impl WorthQuerySupportReportCounters {
    fn generated_once() -> Self {
        Self {
            support_report_generation_count: 1,
        }
    }

    pub fn support_report_generation_count(&self) -> usize {
        self.support_report_generation_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySupportSectionPosture {
    section: WorthQueryConfigSectionFamily,
    owner: WorthQuerySubsystemOwner,
    enabled: bool,
    config_digest: String,
    posture_identity: WorthQueryEvidenceIdentity,
}

impl WorthQuerySupportSectionPosture {
    fn from_resolution(
        resolution: WorthQueryConfigSectionResolution,
    ) -> Result<Self, IdentityDerivationDenial> {
        let section = resolution.section();
        let owner = resolution.owner();
        let enabled = resolution.enabled();
        let config_digest = resolution.config_digest().to_string();
        let posture_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::ApplicationSupportSectionPosture,
        )
        .field_shape(WorthQueryEvidenceTag::new("section"), section.as_str())
        .field_shape(WorthQueryEvidenceTag::new("owner"), owner.as_str())
        .field_bool(WorthQueryEvidenceTag::new("enabled"), enabled)
        .field_value(
            WorthQueryEvidenceTag::new("config_digest"),
            config_digest.clone(),
        )
        .seal()?;
        Ok(Self {
            section,
            owner,
            enabled,
            config_digest,
            posture_identity,
        })
    }

    pub fn section(&self) -> WorthQueryConfigSectionFamily {
        self.section
    }

    pub fn owner(&self) -> WorthQuerySubsystemOwner {
        self.owner
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn config_digest(&self) -> &str {
        &self.config_digest
    }

    pub fn posture_digest(&self) -> &str {
        self.posture_identity.as_str()
    }

    pub fn posture_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.posture_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySupportReport {
    support_matrix: WorthQuerySupportMatrix,
    admitted_capability_count: usize,
    deferred_capability_count: usize,
    unsupported_capability_count: usize,
    admitted_capability_families: Vec<WorthQueryCapabilityFamily>,
    deferred_capability_families: Vec<WorthQueryCapabilityFamily>,
    unsupported_capability_families: Vec<WorthQueryCapabilityFamily>,
    section_postures: Vec<WorthQuerySupportSectionPosture>,
    query_context_support_profile: Option<WorthQueryQueryContextSupportProfile>,
    query_composition_support_profile: Option<WorthQueryQueryCompositionSupportProfile>,
    identity_evolution_support_profile: Option<WorthQueryIdentityEvolutionSupportProfile>,
    validated_config_digest: String,
    counters: WorthQuerySupportReportCounters,
    report_identity: WorthQueryEvidenceIdentity,
}

impl WorthQuerySupportReport {
    pub fn from_config(config: WorthQueryConfig) -> Result<Self, ConfigurationAdmissionError> {
        let validated = config.validate()?;
        let registry = WorthQueryCapabilityRegistry::from_validated_config(&validated);
        let matrix = WorthQuerySupportMatrix::new(registry);
        Ok(Self::from_validated_config_and_matrix(&validated, matrix))
    }

    pub fn runtime_backed_default() -> Self {
        Self::from_config(WorthQueryConfig::runtime_backed_default())
            .expect("runtime-backed Query support configuration must be valid")
    }

    pub(crate) fn from_validated_config_and_matrix(
        config: &ValidatedWorthQueryConfig,
        support_matrix: WorthQuerySupportMatrix,
    ) -> Result<Self, IdentityDerivationDenial> {
        let admitted_capability_families = support_matrix
            .capability_registry()
            .descriptors()
            .iter()
            .filter(|descriptor| descriptor.status() == WorthQueryCapabilityStatus::Admitted)
            .map(|descriptor| descriptor.family())
            .collect::<Vec<_>>();
        let deferred_capability_families = support_matrix
            .capability_registry()
            .descriptors()
            .iter()
            .filter(|descriptor| descriptor.status() == WorthQueryCapabilityStatus::DeferredDebt)
            .map(|descriptor| descriptor.family())
            .collect::<Vec<_>>();
        let unsupported_capability_families = support_matrix
            .capability_registry()
            .descriptors()
            .iter()
            .filter(|descriptor| descriptor.status() == WorthQueryCapabilityStatus::Unsupported)
            .map(|descriptor| descriptor.family())
            .collect::<Vec<_>>();
        let admitted_capability_count = admitted_capability_families.len();
        let deferred_capability_count = deferred_capability_families.len();
        let unsupported_capability_count = unsupported_capability_families.len();
        let section_postures = vec![
            WorthQuerySupportSectionPosture::from_resolution(
                config.resolve_section(WorthQueryConfigSectionFamily::Query),
            ),
            WorthQuerySupportSectionPosture::from_resolution(
                config.resolve_section(WorthQueryConfigSectionFamily::Relational),
            ),
            WorthQuerySupportSectionPosture::from_resolution(
                config.resolve_section(WorthQueryConfigSectionFamily::Signal),
            ),
            WorthQuerySupportSectionPosture::from_resolution(
                config.resolve_section(WorthQueryConfigSectionFamily::RuntimeBridge),
            ),
            WorthQuerySupportSectionPosture::from_resolution(
                config.resolve_section(WorthQueryConfigSectionFamily::Store),
            ),
        ];
        let query_composition_support_profile = support_matrix
            .descriptor(WorthQueryCapabilityFamily::QueryComposition)
            .filter(|descriptor| descriptor.status() == WorthQueryCapabilityStatus::Admitted)
            .map(|_| runtime_backed_query_composition_support_profile());
        let query_context_support_profile = support_matrix
            .descriptor(WorthQueryCapabilityFamily::QueryContext)
            .filter(|descriptor| descriptor.status() == WorthQueryCapabilityStatus::Admitted)
            .map(|_| runtime_backed_narrow_query_context_support_profile())?;
        let identity_evolution_support_profile = support_matrix
            .descriptor(WorthQueryCapabilityFamily::IdentityEvolution)
            .filter(|descriptor| descriptor.status() == WorthQueryCapabilityStatus::Admitted)
            .map(|_| runtime_backed_direct_identity_evolution_support_profile());
        let validated_config_digest = config.validated_digest().to_string();
        let counters = WorthQuerySupportReportCounters::generated_once();
        let report_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
                .field_value(
                    WorthQueryEvidenceTag::new("support_matrix_digest"),
                    support_matrix.support_matrix_digest(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("validated_config_digest"),
                    validated_config_digest.clone(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("admitted_capability_count"),
                    admitted_capability_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("deferred_capability_count"),
                    deferred_capability_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("unsupported_capability_count"),
                    unsupported_capability_count,
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("admitted_capability_family"),
                    admitted_capability_families
                        .iter()
                        .map(WorthQueryCapabilityFamily::as_str),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("deferred_capability_family"),
                    deferred_capability_families
                        .iter()
                        .map(WorthQueryCapabilityFamily::as_str),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("unsupported_capability_family"),
                    unsupported_capability_families
                        .iter()
                        .map(WorthQueryCapabilityFamily::as_str),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("section_posture_digest"),
                    section_postures
                        .iter()
                        .map(WorthQuerySupportSectionPosture::posture_digest),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("query_composition_profile_digest"),
                    query_composition_support_profile
                        .as_ref()
                        .map(WorthQueryQueryCompositionSupportProfile::profile_digest),
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("query_context_profile_digest"),
                    query_context_support_profile
                        .as_ref()
                        .map(WorthQueryQueryContextSupportProfile::profile_digest)?,
                )
                .optional_value(
                    WorthQueryEvidenceTag::new("identity_evolution_profile_digest"),
                    identity_evolution_support_profile
                        .as_ref()
                        .map(WorthQueryIdentityEvolutionSupportProfile::profile_digest),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("support_report_generation_count"),
                    counters.support_report_generation_count(),
                )
                .seal()?;

        Ok(Self {
            support_matrix,
            admitted_capability_count,
            deferred_capability_count,
            unsupported_capability_count,
            admitted_capability_families,
            deferred_capability_families,
            unsupported_capability_families,
            section_postures,
            query_context_support_profile,
            query_composition_support_profile,
            identity_evolution_support_profile,
            validated_config_digest,
            counters,
            report_identity,
        })
    }

    pub fn support_matrix(&self) -> &WorthQuerySupportMatrix {
        &self.support_matrix
    }

    pub fn admitted_capability_count(&self) -> usize {
        self.admitted_capability_count
    }

    pub fn deferred_capability_count(&self) -> usize {
        self.deferred_capability_count
    }

    pub fn unsupported_capability_count(&self) -> usize {
        self.unsupported_capability_count
    }

    pub fn admitted_capability_families(&self) -> &[WorthQueryCapabilityFamily] {
        &self.admitted_capability_families
    }

    pub fn deferred_capability_families(&self) -> &[WorthQueryCapabilityFamily] {
        &self.deferred_capability_families
    }

    pub fn unsupported_capability_families(&self) -> &[WorthQueryCapabilityFamily] {
        &self.unsupported_capability_families
    }

    pub fn section_postures(&self) -> &[WorthQuerySupportSectionPosture] {
        &self.section_postures
    }

    pub fn query_context_support_profile(&self) -> Option<&WorthQueryQueryContextSupportProfile> {
        self.query_context_support_profile.as_ref()
    }

    pub fn query_composition_support_profile(
        &self,
    ) -> Option<&WorthQueryQueryCompositionSupportProfile> {
        self.query_composition_support_profile.as_ref()
    }

    pub fn identity_evolution_support_profile(
        &self,
    ) -> Option<&WorthQueryIdentityEvolutionSupportProfile> {
        self.identity_evolution_support_profile.as_ref()
    }

    pub fn validated_config_digest(&self) -> &str {
        &self.validated_config_digest
    }

    pub fn counters(&self) -> &WorthQuerySupportReportCounters {
        &self.counters
    }

    pub fn report_digest(&self) -> &str {
        self.report_identity.as_str()
    }

    pub fn report_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.report_identity
    }
}
