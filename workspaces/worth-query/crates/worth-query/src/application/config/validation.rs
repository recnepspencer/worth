use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::hash_parts;

use super::families::{
    ConfigurationAdmissionFailureClass, WorthQueryConfigSectionFamily, WorthQuerySubsystemOwner,
};
use super::root::WorthQueryConfig;
use super::sections::{
    WorthQueryQueryConfig, WorthQueryRelationalConfig, WorthQueryRuntimeBridgeConfig,
    WorthQuerySignalConfig, WorthQueryStoreConfig,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryConfigCounters {
    config_validation_count: usize,
    config_section_resolution_count: usize,
    config_validation_denial_count: usize,
}

impl WorthQueryConfigCounters {
    pub fn config_validation_count(&self) -> usize {
        self.config_validation_count
    }

    pub fn config_section_resolution_count(&self) -> usize {
        self.config_section_resolution_count
    }

    pub fn config_validation_denial_count(&self) -> usize {
        self.config_validation_denial_count
    }

    fn successful_validation() -> Self {
        Self {
            config_validation_count: 1,
            config_section_resolution_count: 5,
            config_validation_denial_count: 0,
        }
    }

    fn denied_validation() -> Self {
        Self {
            config_validation_count: 1,
            config_section_resolution_count: 5,
            config_validation_denial_count: 1,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigurationAdmissionError {
    failure_class: ConfigurationAdmissionFailureClass,
    section: Option<WorthQueryConfigSectionFamily>,
    counters: WorthQueryConfigCounters,
    reason: &'static str,
}

impl ConfigurationAdmissionError {
    pub(crate) fn identity_denied(
        cause: IdentityDerivationDenial,
        counters: WorthQueryConfigCounters,
    ) -> Self {
        Self {
            failure_class: ConfigurationAdmissionFailureClass::EvidenceIdentityDerivationDenied(
                cause,
            ),
            section: None,
            counters,
            reason: "application support identity exceeds its canonical digest limit",
        }
    }

    fn new(
        failure_class: ConfigurationAdmissionFailureClass,
        section: Option<WorthQueryConfigSectionFamily>,
        reason: &'static str,
    ) -> Self {
        Self {
            failure_class,
            section,
            counters: WorthQueryConfigCounters::denied_validation(),
            reason,
        }
    }

    pub fn failure_class(&self) -> ConfigurationAdmissionFailureClass {
        self.failure_class
    }

    pub fn section(&self) -> Option<WorthQueryConfigSectionFamily> {
        self.section
    }

    pub fn counters(&self) -> &WorthQueryConfigCounters {
        &self.counters
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConfigSectionResolution {
    section: WorthQueryConfigSectionFamily,
    owner: WorthQuerySubsystemOwner,
    enabled: bool,
    config_digest: String,
}

impl WorthQueryConfigSectionResolution {
    fn new(
        section: WorthQueryConfigSectionFamily,
        owner: WorthQuerySubsystemOwner,
        enabled: bool,
        config_digest: String,
    ) -> Self {
        Self {
            section,
            owner,
            enabled,
            config_digest,
        }
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedWorthQueryConfig {
    raw: WorthQueryConfig,
    query_resolution: WorthQueryConfigSectionResolution,
    relational_resolution: WorthQueryConfigSectionResolution,
    signal_resolution: WorthQueryConfigSectionResolution,
    runtime_bridge_resolution: WorthQueryConfigSectionResolution,
    store_resolution: WorthQueryConfigSectionResolution,
    counters: WorthQueryConfigCounters,
    validated_digest: String,
}

impl ValidatedWorthQueryConfig {
    pub(super) fn new(raw: WorthQueryConfig) -> Result<Self, ConfigurationAdmissionError> {
        let config_digest = raw.digest();
        let query_resolution = WorthQueryConfigSectionResolution::new(
            WorthQueryConfigSectionFamily::Query,
            WorthQuerySubsystemOwner::Query,
            raw.query().runtime_backed_reads_enabled(),
            config_digest.clone(),
        );
        let relational_resolution = WorthQueryConfigSectionResolution::new(
            WorthQueryConfigSectionFamily::Relational,
            WorthQuerySubsystemOwner::Relational,
            raw.relational().workflow_orchestration_enabled()
                || raw.relational().historical_evaluation_enabled(),
            config_digest.clone(),
        );
        let signal_resolution = WorthQueryConfigSectionResolution::new(
            WorthQueryConfigSectionFamily::Signal,
            WorthQuerySubsystemOwner::Signal,
            raw.signal().live_promotion_enabled(),
            config_digest.clone(),
        );
        let runtime_bridge_resolution = WorthQueryConfigSectionResolution::new(
            WorthQueryConfigSectionFamily::RuntimeBridge,
            WorthQuerySubsystemOwner::RuntimeBridge,
            raw.runtime_bridge().preview_session_enabled(),
            config_digest.clone(),
        );
        let store_resolution = WorthQueryConfigSectionResolution::new(
            WorthQueryConfigSectionFamily::Store,
            WorthQuerySubsystemOwner::Store,
            raw.store().durable_artifacts_enabled(),
            config_digest.clone(),
        );

        if !raw.query().runtime_backed_reads_enabled()
            && (raw.signal().live_promotion_enabled()
                || raw.runtime_bridge().preview_session_enabled()
                || raw.relational().workflow_orchestration_enabled()
                || raw.relational().historical_evaluation_enabled())
        {
            return Err(ConfigurationAdmissionError::new(
                ConfigurationAdmissionFailureClass::MissingRequiredSection,
                Some(WorthQueryConfigSectionFamily::Query),
                "query section must remain enabled when signal, runtime bridge, or relational runtime-backed sections are enabled",
            ));
        }

        if raw.store().durable_artifacts_enabled()
            && !raw.relational().historical_evaluation_enabled()
        {
            return Err(ConfigurationAdmissionError::new(
                ConfigurationAdmissionFailureClass::ContradictorySectionPosture,
                Some(WorthQueryConfigSectionFamily::Store),
                "store-backed durability cannot be enabled before relational historical evaluation is enabled",
            ));
        }

        if raw.store().durable_artifacts_enabled() {
            return Err(ConfigurationAdmissionError::new(
                ConfigurationAdmissionFailureClass::DeferredStoreBackedSection,
                Some(WorthQueryConfigSectionFamily::Store),
                "store-backed durable artifacts remain deferred debt until later milestones close",
            ));
        }

        let validated_digest = hash_parts(&[
            format!("config:{config_digest}"),
            format!("query:{}", query_resolution.enabled()),
            format!("relational:{}", relational_resolution.enabled()),
            format!("signal:{}", signal_resolution.enabled()),
            format!("runtime_bridge:{}", runtime_bridge_resolution.enabled()),
            format!("store:{}", store_resolution.enabled()),
        ]);

        Ok(Self {
            raw,
            query_resolution,
            relational_resolution,
            signal_resolution,
            runtime_bridge_resolution,
            store_resolution,
            counters: WorthQueryConfigCounters::successful_validation(),
            validated_digest,
        })
    }

    pub fn raw(&self) -> &WorthQueryConfig {
        &self.raw
    }

    pub fn query(&self) -> &WorthQueryQueryConfig {
        self.raw.query()
    }

    pub fn relational(&self) -> &WorthQueryRelationalConfig {
        self.raw.relational()
    }

    pub fn signal(&self) -> &WorthQuerySignalConfig {
        self.raw.signal()
    }

    pub fn runtime_bridge(&self) -> &WorthQueryRuntimeBridgeConfig {
        self.raw.runtime_bridge()
    }

    pub fn store(&self) -> &WorthQueryStoreConfig {
        self.raw.store()
    }

    pub fn resolve_section(
        &self,
        section: WorthQueryConfigSectionFamily,
    ) -> WorthQueryConfigSectionResolution {
        match section {
            WorthQueryConfigSectionFamily::Query => self.query_resolution.clone(),
            WorthQueryConfigSectionFamily::Relational => self.relational_resolution.clone(),
            WorthQueryConfigSectionFamily::Signal => self.signal_resolution.clone(),
            WorthQueryConfigSectionFamily::RuntimeBridge => self.runtime_bridge_resolution.clone(),
            WorthQueryConfigSectionFamily::Store => self.store_resolution.clone(),
        }
    }

    pub fn counters(&self) -> &WorthQueryConfigCounters {
        &self.counters
    }

    pub fn validated_digest(&self) -> &str {
        &self.validated_digest
    }
}
