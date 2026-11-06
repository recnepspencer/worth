use super::identity::{compose_transition_rule_digest, compose_transition_rules_digest};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionDeferredNeighborFamily {
    DurableReceiptReloadAndStoreParity,
    PortableReceiptExport,
}

impl ProjectionConsumptionDeferredNeighborFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DurableReceiptReloadAndStoreParity => "durable_receipt_reload_and_store_parity",
            Self::PortableReceiptExport => "portable_receipt_export",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionTransitionKind {
    InspectReceipt,
    DeriveEnvelope,
    DiscoverSupport,
    ReloadPersistedReceipt,
    ExportPortableReceipt,
}

impl ProjectionConsumptionTransitionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InspectReceipt => "inspect_receipt",
            Self::DeriveEnvelope => "derive_envelope",
            Self::DiscoverSupport => "discover_support",
            Self::ReloadPersistedReceipt => "reload_persisted_receipt",
            Self::ExportPortableReceipt => "export_portable_receipt",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionTransitionPosture {
    Implemented,
    Deferred,
}

impl ProjectionConsumptionTransitionPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Implemented => "implemented",
            Self::Deferred => "deferred",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionTransitionRule {
    kind: ProjectionConsumptionTransitionKind,
    posture: ProjectionConsumptionTransitionPosture,
    detail: &'static str,
    deferred_neighbor: Option<ProjectionConsumptionDeferredNeighborFamily>,
    rule_digest: String,
}

impl ProjectionConsumptionTransitionRule {
    fn new(
        kind: ProjectionConsumptionTransitionKind,
        posture: ProjectionConsumptionTransitionPosture,
        detail: &'static str,
        deferred_neighbor: Option<ProjectionConsumptionDeferredNeighborFamily>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let rule_digest = compose_transition_rule_digest(kind, posture, detail, deferred_neighbor)?;
        Ok(Self {
            kind,
            posture,
            detail,
            deferred_neighbor,
            rule_digest,
        })
    }

    pub fn kind(&self) -> ProjectionConsumptionTransitionKind {
        self.kind
    }

    pub fn posture(&self) -> ProjectionConsumptionTransitionPosture {
        self.posture
    }

    pub fn detail(&self) -> &str {
        self.detail
    }

    pub fn deferred_neighbor(&self) -> Option<ProjectionConsumptionDeferredNeighborFamily> {
        self.deferred_neighbor
    }

    pub fn rule_digest(&self) -> &str {
        &self.rule_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionTransitionRules {
    rules: Vec<ProjectionConsumptionTransitionRule>,
    rules_digest: String,
}

impl ProjectionConsumptionTransitionRules {
    pub(crate) fn current_phase_five_surface() -> Result<Self, IdentityDerivationDenial> {
        let rules = vec![
            ProjectionConsumptionTransitionRule::new(
                ProjectionConsumptionTransitionKind::InspectReceipt,
                ProjectionConsumptionTransitionPosture::Implemented,
                "the receipt is the canonical operational artifact for consumed-fact inspection",
                None,
            ),
            ProjectionConsumptionTransitionRule::new(
                ProjectionConsumptionTransitionKind::DeriveEnvelope,
                ProjectionConsumptionTransitionPosture::Implemented,
                "the self-describing projection-consumption envelope derives directly from the receipt",
                None,
            ),
            ProjectionConsumptionTransitionRule::new(
                ProjectionConsumptionTransitionKind::DiscoverSupport,
                ProjectionConsumptionTransitionPosture::Implemented,
                "the receipt exposes support posture and deferred-neighbor summary for downstream inspection",
                None,
            ),
            ProjectionConsumptionTransitionRule::new(
                ProjectionConsumptionTransitionKind::ReloadPersistedReceipt,
                ProjectionConsumptionTransitionPosture::Deferred,
                "durable receipt reload remains deferred to later store-backed milestones",
                Some(ProjectionConsumptionDeferredNeighborFamily::DurableReceiptReloadAndStoreParity),
            ),
            ProjectionConsumptionTransitionRule::new(
                ProjectionConsumptionTransitionKind::ExportPortableReceipt,
                ProjectionConsumptionTransitionPosture::Deferred,
                "portable receipt export remains deferred to later store-backed milestones",
                Some(ProjectionConsumptionDeferredNeighborFamily::PortableReceiptExport),
            ),
        ];
        let rules_digest = compose_transition_rules_digest(
            &rules
                .iter()
                .map(|rule| rule.rule_digest().to_string())
                .collect::<Vec<_>>(),
        )?;
        Ok(Self {
            rules,
            rules_digest,
        })
    }

    pub fn rules(&self) -> &[ProjectionConsumptionTransitionRule] {
        &self.rules
    }

    pub fn rules_digest(&self) -> &str {
        &self.rules_digest
    }
}
