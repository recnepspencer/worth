use crate::application::config::{
    ValidatedWorthQueryConfig, WorthQueryConfigSectionFamily, WorthQuerySubsystemOwner,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryCapabilityFamily {
    QueryRead,
    QueryComposition,
    QueryContext,
    IdentityEvolution,
    LiveQuery,
    PreviewSession,
    WorkflowOrchestration,
    HistoricalEvaluation,
    DurableArtifacts,
}

impl WorthQueryCapabilityFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueryRead => "query_read",
            Self::QueryComposition => "query_composition",
            Self::QueryContext => "query_context",
            Self::IdentityEvolution => "identity_evolution",
            Self::LiveQuery => "live_query",
            Self::PreviewSession => "preview_session",
            Self::WorkflowOrchestration => "workflow_orchestration",
            Self::HistoricalEvaluation => "historical_evaluation",
            Self::DurableArtifacts => "durable_artifacts",
        }
    }

    pub fn config_section(&self) -> WorthQueryConfigSectionFamily {
        match self {
            Self::QueryRead
            | Self::QueryComposition
            | Self::QueryContext
            | Self::IdentityEvolution => WorthQueryConfigSectionFamily::Query,
            Self::LiveQuery => WorthQueryConfigSectionFamily::Signal,
            Self::PreviewSession => WorthQueryConfigSectionFamily::RuntimeBridge,
            Self::WorkflowOrchestration | Self::HistoricalEvaluation => {
                WorthQueryConfigSectionFamily::Relational
            }
            Self::DurableArtifacts => WorthQueryConfigSectionFamily::Store,
        }
    }

    pub(crate) fn satisfies_operation_requirement(
        self,
        requirement: worth_query_installation::facade::WorthQueryOperationCapabilityRequirement,
    ) -> bool {
        use worth_query_installation::facade::WorthQueryOperationCapabilityRequirement as Requirement;

        matches!(
            (self, requirement),
            (Self::QueryRead, Requirement::QueryRead)
                | (Self::QueryComposition, Requirement::QueryComposition)
                | (Self::QueryContext, Requirement::QueryContext)
                | (Self::IdentityEvolution, Requirement::IdentityEvolution)
                | (Self::LiveQuery, Requirement::LiveQuery)
                | (Self::PreviewSession, Requirement::PreviewSession)
                | (
                    Self::WorkflowOrchestration,
                    Requirement::WorkflowOrchestration
                )
                | (
                    Self::HistoricalEvaluation,
                    Requirement::HistoricalEvaluation
                )
                | (Self::DurableArtifacts, Requirement::DurableArtifacts)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCapabilityStatus {
    Admitted,
    DeferredDebt,
    Unsupported,
}

pub type WorthQueryCapabilitySupportStatus = WorthQueryCapabilityStatus;

impl WorthQueryCapabilityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::DeferredDebt => "deferred_debt",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCapabilityDescriptor {
    family: WorthQueryCapabilityFamily,
    status: WorthQueryCapabilityStatus,
    owner: WorthQuerySubsystemOwner,
    config_section: WorthQueryConfigSectionFamily,
    reason: &'static str,
}

impl WorthQueryCapabilityDescriptor {
    pub(crate) fn new(
        family: WorthQueryCapabilityFamily,
        status: WorthQueryCapabilityStatus,
        owner: WorthQuerySubsystemOwner,
        reason: &'static str,
    ) -> Self {
        Self {
            family,
            status,
            owner,
            config_section: family.config_section(),
            reason,
        }
    }

    pub fn family(&self) -> WorthQueryCapabilityFamily {
        self.family
    }

    pub fn status(&self) -> WorthQueryCapabilityStatus {
        self.status
    }

    pub fn owner(&self) -> WorthQuerySubsystemOwner {
        self.owner
    }

    pub fn config_section(&self) -> WorthQueryConfigSectionFamily {
        self.config_section
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCapabilityRegistry {
    descriptors: Vec<WorthQueryCapabilityDescriptor>,
    registry_digest: String,
}

impl WorthQueryCapabilityRegistry {
    pub(crate) fn from_validated_config(
        config: &ValidatedWorthQueryConfig,
    ) -> Result<Self, IdentityDerivationDenial> {
        let descriptors = vec![
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::QueryRead,
                if config.query().runtime_backed_reads_enabled() {
                    WorthQueryCapabilityStatus::Admitted
                } else {
                    WorthQueryCapabilityStatus::Unsupported
                },
                WorthQuerySubsystemOwner::Query,
                if config.query().runtime_backed_reads_enabled() {
                    "runtime-backed query execution is admitted through the daily-driver facade"
                } else {
                    "runtime-backed query execution is disabled by the query config section"
                },
            ),
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::QueryComposition,
                if config.query().runtime_backed_reads_enabled() {
                    WorthQueryCapabilityStatus::Admitted
                } else {
                    WorthQueryCapabilityStatus::Unsupported
                },
                WorthQuerySubsystemOwner::Query,
                if config.query().runtime_backed_reads_enabled() {
                    "query composition is admitted through the query config section"
                } else {
                    "query composition is disabled by the query config section"
                },
            ),
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::QueryContext,
                if config.query().runtime_backed_reads_enabled() {
                    WorthQueryCapabilityStatus::Admitted
                } else {
                    WorthQueryCapabilityStatus::Unsupported
                },
                WorthQuerySubsystemOwner::Query,
                if config.query().runtime_backed_reads_enabled() {
                    "query context binding is admitted through the query config section"
                } else {
                    "query context binding is disabled by the query config section"
                },
            ),
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::IdentityEvolution,
                if config.query().runtime_backed_reads_enabled() {
                    WorthQueryCapabilityStatus::Admitted
                } else {
                    WorthQueryCapabilityStatus::Unsupported
                },
                WorthQuerySubsystemOwner::Query,
                if config.query().runtime_backed_reads_enabled() {
                    "identity evolution is admitted through the query config section"
                } else {
                    "identity evolution is disabled by the query config section"
                },
            ),
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::LiveQuery,
                if config.query().runtime_backed_reads_enabled()
                    && config.signal().live_promotion_enabled()
                {
                    WorthQueryCapabilityStatus::Admitted
                } else {
                    WorthQueryCapabilityStatus::Unsupported
                },
                WorthQuerySubsystemOwner::Signal,
                if config.signal().live_promotion_enabled() {
                    "live query promotion is admitted through the signal config section"
                } else {
                    "live query promotion is disabled by the signal config section"
                },
            ),
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::PreviewSession,
                if config.query().runtime_backed_reads_enabled()
                    && config.runtime_bridge().preview_session_enabled()
                {
                    WorthQueryCapabilityStatus::Admitted
                } else {
                    WorthQueryCapabilityStatus::Unsupported
                },
                WorthQuerySubsystemOwner::RuntimeBridge,
                if config.runtime_bridge().preview_session_enabled() {
                    "preview session composition is admitted through the runtime bridge config section"
                } else {
                    "preview session composition is disabled by the runtime bridge config section"
                },
            ),
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::WorkflowOrchestration,
                if config.query().runtime_backed_reads_enabled()
                    && config.relational().workflow_orchestration_enabled()
                {
                    WorthQueryCapabilityStatus::Admitted
                } else {
                    WorthQueryCapabilityStatus::Unsupported
                },
                WorthQuerySubsystemOwner::Relational,
                if config.relational().workflow_orchestration_enabled() {
                    "workflow orchestration is admitted through the relational config section"
                } else {
                    "workflow orchestration is disabled by the relational config section"
                },
            ),
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::HistoricalEvaluation,
                if config.query().runtime_backed_reads_enabled()
                    && config.relational().historical_evaluation_enabled()
                {
                    WorthQueryCapabilityStatus::Admitted
                } else {
                    WorthQueryCapabilityStatus::Unsupported
                },
                WorthQuerySubsystemOwner::Relational,
                if config.relational().historical_evaluation_enabled() {
                    "historical evaluation is admitted through the relational config section"
                } else {
                    "historical evaluation is disabled by the relational config section"
                },
            ),
            WorthQueryCapabilityDescriptor::new(
                WorthQueryCapabilityFamily::DurableArtifacts,
                WorthQueryCapabilityStatus::DeferredDebt,
                WorthQuerySubsystemOwner::Store,
                "durable artifacts remain store-gated debt until the later durability milestones close",
            ),
        ];

        let descriptor_identities = descriptors
            .iter()
            .map(capability_descriptor_identity)
            .collect::<Vec<_>>();
        let registry_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
                .field_shape(WorthQueryEvidenceTag::new("role"), "capability-registry")
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("descriptors"),
                    descriptor_identities.iter().map(String::as_str),
                )
                .seal()?
                .as_str()
                .to_string();

        Ok(Self {
            descriptors,
            registry_digest,
        })
    }

    pub fn descriptors(&self) -> &[WorthQueryCapabilityDescriptor] {
        &self.descriptors
    }

    pub fn descriptor(
        &self,
        family: WorthQueryCapabilityFamily,
    ) -> Option<&WorthQueryCapabilityDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.family == family)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn capability_descriptor_identity(
    descriptor: &WorthQueryCapabilityDescriptor,
) -> Result<String, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ApplicationSupportReport)
            .field_shape(WorthQueryEvidenceTag::new("role"), "capability-descriptor")
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                descriptor.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("status"),
                descriptor.status().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("owner"),
                descriptor.owner().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("config_section"),
                descriptor.config_section().as_str(),
            )
            .field_value(WorthQueryEvidenceTag::new("reason"), descriptor.reason())
            .seal()?
            .as_str()
            .to_string(),
    )
}
