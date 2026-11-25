use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaStratum {
    StructuralShape,
    ValueDomain,
    EntityIdentitySemantics,
    CorrespondenceSemantics,
    LineageSemantics,
    BehavioralSemantics,
    PublicationContract,
    SubscriberContract,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HistoricalInterpretationSensitivity {
    NotSensitive = 0,
    SensitiveToValueMeaning = 1,
    SensitiveToLegalityMeaning = 2,
    SensitiveToIdentityMeaning = 3,
    SensitiveToPublicationMeaning = 4,
    SensitiveToDerivedMeaning = 5,
}

impl HistoricalInterpretationSensitivity {
    pub const fn sensitivity_rank(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaPublicationImpact {
    None,
    ObservableSurfaceChanged,
    PatchEncodingChanged,
    ProjectionContractChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaSubscriberImpact {
    None,
    ConsumableSurfaceChanged,
    ContractUpgradeRequired,
    RenegotiationRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SubscriberBoundaryVisibility {
    NotVisible,
    VisibleSemanticallyIgnorable,
    VisibleRequiresContractUptake,
}

pub const fn default_boundary_visibility_for_subscriber_impact(
    subscriber_impact: SchemaSubscriberImpact,
) -> SubscriberBoundaryVisibility {
    match subscriber_impact {
        SchemaSubscriberImpact::ContractUpgradeRequired => {
            SubscriberBoundaryVisibility::VisibleRequiresContractUptake
        }
        _ => SubscriberBoundaryVisibility::NotVisible,
    }
}

pub const fn default_boundary_visibility_for_continuation(
    continuation: SchemaContinuationClassification,
) -> SubscriberBoundaryVisibility {
    match continuation {
        SchemaContinuationClassification::ContinueWithContractUpgrade => {
            SubscriberBoundaryVisibility::VisibleRequiresContractUptake
        }
        _ => SubscriberBoundaryVisibility::NotVisible,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FreeFormSchemaDiffIntent {
    Additive,
    StructuralContinuityDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaTransitionBarrier {
    ConstructionBarrier,
    ValidationBarrier,
    LoweringBarrier,
    ExecutionBarrier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaReconciliationClassification {
    Additive,
    Narrowing,
    TypeContinuityDenied,
    StructuralContinuityDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaBridgeabilityClassification {
    Transparent,
    SubscriberVisible,
    ContractUpgradeOnly,
    RenegotiationOnly,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaContinuationClassification {
    ContinueUnchanged,
    ContinueWithTransparentBridge,
    ContinueWithVisibleBridge,
    ContinueWithContractUpgrade,
    RequireRenegotiation,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaContinuationAdmissionObservation {
    RejectedInAllLayers,
    NonRejectedInAtLeastOneLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaReconciliationPolicy {
    RejectLossyNarrowing,
    PreserveInformation,
    PreserveTargetContract,
    PreserveSourceContract,
    PermitLossyNarrowingWithAnnotation,
    RequireExplicitProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaReconciliationOrderingMode {
    CanonicalizedPair,
    ExplicitDirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SchemaLineageOrderingSemantics {
    SymmetricResult,
    DirectionSensitiveResult,
}
