use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeCrossingClassification {
    CanonicalLowerRuntimeReuse,
    QueryBoundaryAdapter,
    CompatibilityDebtLane,
    DeferredNeighbor,
    ForbiddenDuplicate,
}

impl WorthQueryLowerRuntimeCrossingClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalLowerRuntimeReuse => "canonical-lower-runtime-reuse",
            Self::QueryBoundaryAdapter => "query-boundary-adapter",
            Self::CompatibilityDebtLane => "compatibility-debt-lane",
            Self::DeferredNeighbor => "deferred-neighbor",
            Self::ForbiddenDuplicate => "forbidden-duplicate",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeRouteKind {
    RoutePlanning,
    ReadmissionHandoff,
}

impl WorthQueryLowerRuntimeRouteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RoutePlanning => "route-planning",
            Self::ReadmissionHandoff => "readmission-handoff",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeAuthorityOwner {
    Query,
    RuntimeBridge,
    Relational,
    Signal,
    Store,
}

impl WorthQueryLowerRuntimeAuthorityOwner {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::RuntimeBridge => "runtime-bridge",
            Self::Relational => "relational",
            Self::Signal => "signal",
            Self::Store => "store",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeArtifactStrength {
    WeakUnitReturn,
    WeakStringToken,
    TypedReceipt,
    TypedEnvelope,
    TypedAuthoritativeArtifact,
    DerivedAggregateArtifact,
}

impl WorthQueryLowerRuntimeArtifactStrength {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WeakUnitReturn => "weak-unit-return",
            Self::WeakStringToken => "weak-string-token",
            Self::TypedReceipt => "typed-receipt",
            Self::TypedEnvelope => "typed-envelope",
            Self::TypedAuthoritativeArtifact => "typed-authoritative-artifact",
            Self::DerivedAggregateArtifact => "derived-aggregate-artifact",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeSeamKey {
    ComposeRead,
    ExecuteReadFamily,
    ExecuteReadFamilyInBasisContext,
    ExecuteRuntimeCurrentReadGraph,
    ExecuteRuntimeBasisContextReadGraph,
    PublicLiveViewDeclaration,
    LiveViewSchemaAdmission,
    LiveViewSourceDeclaration,
    RuntimeLiveInstallationOrchestration,
    RuntimeIntentAuthorityAdapter,
    SubscriptionActivation,
    SubscriptionContinuity,
    PreviewBasisAdmission,
    BasisReadmissionFromTruthViewEvidence,
    BasisReadmissionFromSubscriptionEvidence,
    HistoricalBridgeLowering,
    EffectBackedRelationalMutation,
    EffectBackedRelationalMerge,
    EffectBackedBridgeWriteback,
    WriteAuthorityBackendExecution,
    SignalInvalidationRouting,
    IntentRuntimeExecution,
    ProjectionSourceIntakeFromQueryReceipts,
    ProjectionSourceIntakeFromRelationalArtifacts,
    ProjectionSourceIntakeFromBridgeArtifacts,
    CausalBridgeMaterialization,
    FrontierEvidenceIntake,
    RuntimeBackendBoundaryModules,
    FrontierSignalAdapterModule,
    EffectExecutionBridgeModule,
    RuntimeIntentModule,
    HistoricalBridgeLoweringModule,
    ProjectionConsumptionSourceModule,
    CausalBuilderBridgeModule,
    StoreBackedRouteParityNeighbor,
    DurableRouteReplayNeighbor,
    PersistedBoundaryExecutionReceiptNeighbor,
    RestartStableBoundaryEnvelopeReloadNeighbor,
    TemporalQueryBasisRoutingNeighbor,
    AsyncResourceRoutingNeighbor,
    MixedTruthTimeAsyncRoutingNeighbor,
    FinalDeferredCertificationClosureNeighbor,
}

impl WorthQueryLowerRuntimeSeamKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ComposeRead => "compose-read",
            Self::ExecuteReadFamily => "execute-read-family",
            Self::ExecuteReadFamilyInBasisContext => "execute-read-family-in-basis-context",
            Self::ExecuteRuntimeCurrentReadGraph => "execute-runtime-current-read-graph",
            Self::ExecuteRuntimeBasisContextReadGraph => "execute-runtime-basis-context-read-graph",
            Self::PublicLiveViewDeclaration => "public-live-view-declaration",
            Self::LiveViewSchemaAdmission => "live-view-schema-admission",
            Self::LiveViewSourceDeclaration => "live-view-source-declaration",
            Self::RuntimeLiveInstallationOrchestration => "runtime-live-installation-orchestration",
            Self::RuntimeIntentAuthorityAdapter => "runtime-intent-authority-adapter",
            Self::SubscriptionActivation => "subscription-activation",
            Self::SubscriptionContinuity => "subscription-continuity",
            Self::PreviewBasisAdmission => "preview-basis-admission",
            Self::BasisReadmissionFromTruthViewEvidence => {
                "basis-readmission-from-truth-view-evidence"
            }
            Self::BasisReadmissionFromSubscriptionEvidence => {
                "basis-readmission-from-subscription-evidence"
            }
            Self::HistoricalBridgeLowering => "historical-bridge-lowering",
            Self::EffectBackedRelationalMutation => "effect-backed-relational-mutation",
            Self::EffectBackedRelationalMerge => "effect-backed-relational-merge",
            Self::EffectBackedBridgeWriteback => "effect-backed-bridge-writeback",
            Self::WriteAuthorityBackendExecution => "write-authority-backend-execution",
            Self::SignalInvalidationRouting => "signal-invalidation-routing",
            Self::IntentRuntimeExecution => "intent-runtime-execution",
            Self::ProjectionSourceIntakeFromQueryReceipts => {
                "projection-source-intake-from-query-receipts"
            }
            Self::ProjectionSourceIntakeFromRelationalArtifacts => {
                "projection-source-intake-from-relational-artifacts"
            }
            Self::ProjectionSourceIntakeFromBridgeArtifacts => {
                "projection-source-intake-from-bridge-artifacts"
            }
            Self::CausalBridgeMaterialization => "causal-bridge-materialization",
            Self::FrontierEvidenceIntake => "frontier-evidence-intake",
            Self::RuntimeBackendBoundaryModules => "runtime-backend-boundary-modules",
            Self::FrontierSignalAdapterModule => "frontier-signal-adapter-module",
            Self::EffectExecutionBridgeModule => "effect-execution-bridge-module",
            Self::RuntimeIntentModule => "runtime-intent-module",
            Self::HistoricalBridgeLoweringModule => "historical-bridge-lowering-module",
            Self::ProjectionConsumptionSourceModule => "projection-consumption-source-module",
            Self::CausalBuilderBridgeModule => "causal-builder-bridge-module",
            Self::StoreBackedRouteParityNeighbor => "store-backed-route-parity-neighbor",
            Self::DurableRouteReplayNeighbor => "durable-route-replay-neighbor",
            Self::PersistedBoundaryExecutionReceiptNeighbor => {
                "persisted-boundary-execution-receipt-neighbor"
            }
            Self::RestartStableBoundaryEnvelopeReloadNeighbor => {
                "restart-stable-boundary-envelope-reload-neighbor"
            }
            Self::TemporalQueryBasisRoutingNeighbor => "temporal-query-basis-routing-neighbor",
            Self::AsyncResourceRoutingNeighbor => "async-resource-routing-neighbor",
            Self::MixedTruthTimeAsyncRoutingNeighbor => "mixed-truth-time-async-routing-neighbor",
            Self::FinalDeferredCertificationClosureNeighbor => {
                "final-deferred-certification-closure-neighbor"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeCrossingRow {
    seam_key: WorthQueryLowerRuntimeSeamKey,
    capability_label: &'static str,
    concrete_seam: &'static str,
    classification: WorthQueryLowerRuntimeCrossingClassification,
    route_kind: WorthQueryLowerRuntimeRouteKind,
    lower_runtime_owner: WorthQueryLowerRuntimeAuthorityOwner,
    current_artifact_strength: WorthQueryLowerRuntimeArtifactStrength,
    required_action: &'static str,
}

impl WorthQueryLowerRuntimeCrossingRow {
    pub(crate) const fn new(
        seam_key: WorthQueryLowerRuntimeSeamKey,
        capability_label: &'static str,
        concrete_seam: &'static str,
        classification: WorthQueryLowerRuntimeCrossingClassification,
        route_kind: WorthQueryLowerRuntimeRouteKind,
        lower_runtime_owner: WorthQueryLowerRuntimeAuthorityOwner,
        current_artifact_strength: WorthQueryLowerRuntimeArtifactStrength,
        required_action: &'static str,
    ) -> Self {
        Self {
            seam_key,
            capability_label,
            concrete_seam,
            classification,
            route_kind,
            lower_runtime_owner,
            current_artifact_strength,
            required_action,
        }
    }

    pub fn seam_key(&self) -> WorthQueryLowerRuntimeSeamKey {
        self.seam_key
    }

    pub fn capability_label(&self) -> &'static str {
        self.capability_label
    }

    pub fn concrete_seam(&self) -> &'static str {
        self.concrete_seam
    }

    pub fn classification(&self) -> WorthQueryLowerRuntimeCrossingClassification {
        self.classification
    }

    pub fn route_kind(&self) -> WorthQueryLowerRuntimeRouteKind {
        self.route_kind
    }

    pub fn lower_runtime_owner(&self) -> WorthQueryLowerRuntimeAuthorityOwner {
        self.lower_runtime_owner
    }

    pub fn current_artifact_strength(&self) -> WorthQueryLowerRuntimeArtifactStrength {
        self.current_artifact_strength
    }

    pub fn required_action(&self) -> &'static str {
        self.required_action
    }

    pub fn row_digest(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.row_identity()?.as_str().to_string())
    }

    fn row_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_crossing_row_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("seam"), self.seam_key.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("capability"),
            self.capability_label,
        )
        .field_value(WorthQueryEvidenceTag::new("seam_path"), self.concrete_seam)
        .field_shape(
            WorthQueryEvidenceTag::new("classification"),
            self.classification.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("route_kind"),
            self.route_kind.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("owner"),
            self.lower_runtime_owner.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("artifact"),
            self.current_artifact_strength.as_str(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("required_action"),
            self.required_action,
        )
        .seal()?)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeCrossingInventory {
    rows: &'static [WorthQueryLowerRuntimeCrossingRow],
}

impl WorthQueryLowerRuntimeCrossingInventory {
    pub(crate) const fn new(rows: &'static [WorthQueryLowerRuntimeCrossingRow]) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &'static [WorthQueryLowerRuntimeCrossingRow] {
        self.rows
    }

    pub fn inventory_digest(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        let row_identities = self
            .rows
            .iter()
            .map(WorthQueryLowerRuntimeCrossingRow::row_identity)
            .collect::<Vec<_>>();
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_crossing_inventory_v1",
        )
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), &row_identities)
        .seal()?)
    }

    pub fn classification_digest(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        let classification_identities = self
            .rows
            .iter()
            .map(|row| {
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
                )
                .field_shape(WorthQueryEvidenceTag::new("seam"), row.seam_key().as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("classification"),
                    row.classification().as_str(),
                )
                .seal()
            })
            .collect::<Vec<_>>();
        Ok(WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "lower_runtime_crossing_classification_v1",
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("rows"),
            &classification_identities,
        )
        .seal()?)
    }
}
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
