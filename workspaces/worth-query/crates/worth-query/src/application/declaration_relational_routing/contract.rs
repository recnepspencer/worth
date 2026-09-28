use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily, WorthQueryDeclarationAspectContract,
    WorthQueryDeclarationAspectCoverage, WorthQueryDeclarationAspectFit,
    WorthQueryDeclarationAuthorityAspectMismatch, WorthQueryDeclarationInput,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};

const WORKFLOW_ONLY: &[WorthQueryCapabilityFamily] =
    &[WorthQueryCapabilityFamily::WorkflowOrchestration];
const HISTORY_ONLY: &[WorthQueryCapabilityFamily] =
    &[WorthQueryCapabilityFamily::HistoricalEvaluation];
const RELATIONAL_SECTION_ONLY: &[WorthQueryConfigSectionFamily] =
    &[WorthQueryConfigSectionFamily::Relational];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationRelationalTruthClaim {
    AuthoritativeCurrentTruth,
    Identity,
    Lineage,
    HistoricalTruth,
    InvariantTruth,
    GroupedTruth,
    StrategyTruth,
}

impl WorthQueryDeclarationRelationalTruthClaim {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AuthoritativeCurrentTruth => "authoritative_current_truth",
            Self::Identity => "identity",
            Self::Lineage => "lineage",
            Self::HistoricalTruth => "historical_truth",
            Self::InvariantTruth => "invariant_truth",
            Self::GroupedTruth => "grouped_truth",
            Self::StrategyTruth => "strategy_truth",
        }
    }
}

/// The relational authority a declaration's truth routes through.
///
/// "Relational" names the truth source, not the owning crate: `GroupedTruth`
/// and `BridgeSource` classify truth that Relational decides, even though the
/// surfaces that carry it live in `worth-runtime-bridge`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationRelationalAuthorityFamily {
    Runtime,
    History,
    /// Grouped projections over Relational snapshot reads.
    GroupedTruth,
    CommitStrategies,
    /// The Bridge's adapter over Relational committed changes.
    BridgeSource,
}

impl WorthQueryDeclarationRelationalAuthorityFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::History => "history",
            Self::GroupedTruth => "grouped_truth",
            Self::CommitStrategies => "commit_strategies",
            Self::BridgeSource => "bridge_source",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeclarationRelationalTruthContract {
    truth_claim: WorthQueryDeclarationRelationalTruthClaim,
    authority_family: WorthQueryDeclarationRelationalAuthorityFamily,
    required_capability_families: &'static [WorthQueryCapabilityFamily],
    required_config_sections: &'static [WorthQueryConfigSectionFamily],
    required_aspects: WorthQueryDeclarationAspectContract,
    reason: &'static str,
}

impl WorthQueryDeclarationRelationalTruthContract {
    pub fn authoritative_current_truth() -> Self {
        Self {
            truth_claim: WorthQueryDeclarationRelationalTruthClaim::AuthoritativeCurrentTruth,
            authority_family: WorthQueryDeclarationRelationalAuthorityFamily::Runtime,
            required_capability_families: WORKFLOW_ONLY,
            required_config_sections: RELATIONAL_SECTION_ONLY,
            required_aspects: WorthQueryDeclarationAspectContract::empty(),
            reason: "the declaration lowers into authoritative relational runtime truth",
        }
    }

    pub fn grouped_truth() -> Self {
        Self {
            truth_claim: WorthQueryDeclarationRelationalTruthClaim::GroupedTruth,
            authority_family: WorthQueryDeclarationRelationalAuthorityFamily::GroupedTruth,
            required_capability_families: WORKFLOW_ONLY,
            required_config_sections: RELATIONAL_SECTION_ONLY,
            required_aspects: WorthQueryDeclarationAspectContract::empty(),
            reason: "the declaration lowers into grouped relational truth materialization",
        }
    }

    pub fn historical_truth() -> Self {
        Self {
            truth_claim: WorthQueryDeclarationRelationalTruthClaim::HistoricalTruth,
            authority_family: WorthQueryDeclarationRelationalAuthorityFamily::History,
            required_capability_families: HISTORY_ONLY,
            required_config_sections: RELATIONAL_SECTION_ONLY,
            required_aspects: WorthQueryDeclarationAspectContract::empty(),
            reason: "the declaration lowers into relational historical truth evaluation",
        }
    }

    pub fn strategy_truth() -> Self {
        Self {
            truth_claim: WorthQueryDeclarationRelationalTruthClaim::StrategyTruth,
            authority_family: WorthQueryDeclarationRelationalAuthorityFamily::CommitStrategies,
            required_capability_families: WORKFLOW_ONLY,
            required_config_sections: RELATIONAL_SECTION_ONLY,
            required_aspects: WorthQueryDeclarationAspectContract::empty(),
            reason: "the declaration lowers into relational commit strategy authority",
        }
    }

    pub fn bridge_source_current_truth() -> Self {
        Self {
            truth_claim: WorthQueryDeclarationRelationalTruthClaim::AuthoritativeCurrentTruth,
            authority_family: WorthQueryDeclarationRelationalAuthorityFamily::BridgeSource,
            required_capability_families: WORKFLOW_ONLY,
            required_config_sections: RELATIONAL_SECTION_ONLY,
            required_aspects: WorthQueryDeclarationAspectContract::empty(),
            reason: "the declaration lowers into a bridge-consumable relational source binding",
        }
    }

    pub fn truth_claim(&self) -> WorthQueryDeclarationRelationalTruthClaim {
        self.truth_claim
    }

    pub fn authority_family(&self) -> WorthQueryDeclarationRelationalAuthorityFamily {
        self.authority_family
    }

    pub fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        self.required_capability_families
    }

    pub fn required_config_sections(&self) -> &'static [WorthQueryConfigSectionFamily] {
        self.required_config_sections
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }

    pub fn required_aspects(&self) -> &WorthQueryDeclarationAspectContract {
        &self.required_aspects
    }

    pub fn with_required_aspects(
        mut self,
        required_aspects: WorthQueryDeclarationAspectContract,
    ) -> Self {
        self.required_aspects = required_aspects;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDeclarationRelationalTruthRoutingSupportStatus {
    Admitted,
    Unsupported,
    InvalidContext,
}

impl WorthQueryDeclarationRelationalTruthRoutingSupportStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Unsupported => "unsupported",
            Self::InvalidContext => "invalid_context",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeclarationRelationalRoutingSupportRow {
    truth_claim: WorthQueryDeclarationRelationalTruthClaim,
    authority_family: WorthQueryDeclarationRelationalAuthorityFamily,
    required_aspect_slice: WorthQueryDeclarationAspectContract,
    available_aspect_slice: WorthQueryDeclarationAspectCoverage,
    aspect_fit: WorthQueryDeclarationAspectFit,
    aspect_mismatch: Option<WorthQueryDeclarationAuthorityAspectMismatch>,
    status: WorthQueryDeclarationRelationalTruthRoutingSupportStatus,
    reason: &'static str,
}

impl WorthQueryDeclarationRelationalRoutingSupportRow {
    pub(crate) fn new(
        truth_claim: WorthQueryDeclarationRelationalTruthClaim,
        authority_family: WorthQueryDeclarationRelationalAuthorityFamily,
        required_aspect_slice: WorthQueryDeclarationAspectContract,
        available_aspect_slice: WorthQueryDeclarationAspectCoverage,
        aspect_fit: WorthQueryDeclarationAspectFit,
        aspect_mismatch: Option<WorthQueryDeclarationAuthorityAspectMismatch>,
        status: WorthQueryDeclarationRelationalTruthRoutingSupportStatus,
        reason: &'static str,
    ) -> Self {
        Self {
            truth_claim,
            authority_family,
            required_aspect_slice,
            available_aspect_slice,
            aspect_fit,
            aspect_mismatch,
            status,
            reason,
        }
    }

    pub fn truth_claim(&self) -> WorthQueryDeclarationRelationalTruthClaim {
        self.truth_claim
    }

    pub fn authority_family(&self) -> WorthQueryDeclarationRelationalAuthorityFamily {
        self.authority_family
    }

    pub fn required_aspect_slice(&self) -> &WorthQueryDeclarationAspectContract {
        &self.required_aspect_slice
    }

    pub fn available_aspect_slice(&self) -> &WorthQueryDeclarationAspectCoverage {
        &self.available_aspect_slice
    }

    pub fn aspect_fit(&self) -> WorthQueryDeclarationAspectFit {
        self.aspect_fit
    }

    pub fn aspect_mismatch(&self) -> Option<WorthQueryDeclarationAuthorityAspectMismatch> {
        self.aspect_mismatch
    }

    pub fn status(&self) -> WorthQueryDeclarationRelationalTruthRoutingSupportStatus {
        self.status
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeclarationRelationalRoutingSupportReport<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    declaration_family_key: &'static str,
    rows: Vec<WorthQueryDeclarationRelationalRoutingSupportRow>,
    support_digest: String,
    _marker: std::marker::PhantomData<(D, I)>,
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    WorthQueryDeclarationRelationalRoutingSupportReport<D, I>
{
    pub(crate) fn new(
        declaration_family_key: &'static str,
        rows: Vec<WorthQueryDeclarationRelationalRoutingSupportRow>,
        support_digest: String,
    ) -> Self {
        Self {
            declaration_family_key,
            rows,
            support_digest,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn declaration_family_key(&self) -> &'static str {
        self.declaration_family_key
    }

    pub fn rows(&self) -> &[WorthQueryDeclarationRelationalRoutingSupportRow] {
        &self.rows
    }

    pub fn support_digest(&self) -> &str {
        &self.support_digest
    }
}

pub(crate) fn derive_relational_routing_support_report<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
) -> WorthQueryDeclarationRelationalRoutingSupportReport<D, I> {
    crate::application::worth_query_relational_routing_support_from_entry_readiness::<D, C, I>(
        handle,
    )
}
