use crate::schema::data::{
    FreeFormSchemaDiffIntent, HistoricalInterpretationSensitivity, ProposedSchemaTransition,
    SchemaBridgeabilityClassification, SchemaContinuationAdmissionObservation,
    SchemaContinuationClassification, SchemaDiffAtom, SchemaDiffDetail,
    SchemaReconciliationClassification, SchemaReconciliationPolicy, SchemaStratum,
    SchemaSubscriberImpact, SubscriberBoundaryVisibility, ValidatedSchemaTransition,
};

pub fn classify_schema_transition(
    proposed: ProposedSchemaTransition,
    policy: Option<SchemaReconciliationPolicy>,
) -> ValidatedSchemaTransition {
    let classifications = classify_transition_atoms(&proposed.diff_atoms);
    ValidatedSchemaTransition {
        proposed,
        continuation_admission_observation: continuation_admission_observation(
            classifications.continuation,
            classifications.reconciliation,
        ),
        reconciliation: classifications.reconciliation,
        continuation: classifications.continuation,
        bridgeability: bridgeability_after_policy(
            policy,
            classifications.continuation,
            classifications.bridgeability,
        ),
    }
}

#[derive(Debug, Clone, Copy)]
struct TransitionClassifications {
    reconciliation: SchemaReconciliationClassification,
    continuation: SchemaContinuationClassification,
    bridgeability: SchemaBridgeabilityClassification,
}

fn classify_transition_atoms(diff_atoms: &[SchemaDiffAtom]) -> TransitionClassifications {
    let mut classifications = TransitionClassifications {
        reconciliation: SchemaReconciliationClassification::Additive,
        continuation: SchemaContinuationClassification::ContinueUnchanged,
        bridgeability: SchemaBridgeabilityClassification::Transparent,
    };

    for atom in diff_atoms {
        classifications.reconciliation = max_reconciliation_classification(
            classifications.reconciliation,
            classify_reconciliation(atom),
        );
        classifications.continuation = max_continuation_classification(
            classifications.continuation,
            classify_continuation(atom),
        );
        classifications.bridgeability = max_bridgeability_classification(
            classifications.bridgeability,
            classify_bridgeability(atom),
        );
    }

    force_rejected_continuity_when_reconciliation_denies(classifications)
}

fn force_rejected_continuity_when_reconciliation_denies(
    mut classifications: TransitionClassifications,
) -> TransitionClassifications {
    if matches!(
        classifications.reconciliation,
        SchemaReconciliationClassification::TypeContinuityDenied
            | SchemaReconciliationClassification::StructuralContinuityDenied
    ) {
        classifications.continuation = SchemaContinuationClassification::Rejected;
        classifications.bridgeability = SchemaBridgeabilityClassification::Rejected;
    }

    if (classifications.continuation == SchemaContinuationClassification::Rejected
        || classifications.bridgeability == SchemaBridgeabilityClassification::Rejected)
        && classifications.reconciliation == SchemaReconciliationClassification::Additive
    {
        classifications.reconciliation = max_reconciliation_classification(
            classifications.reconciliation,
            SchemaReconciliationClassification::StructuralContinuityDenied,
        );
    }

    classifications
}

fn continuation_admission_observation(
    continuation: SchemaContinuationClassification,
    reconciliation: SchemaReconciliationClassification,
) -> SchemaContinuationAdmissionObservation {
    if continuation == SchemaContinuationClassification::Rejected
        && matches!(
            reconciliation,
            SchemaReconciliationClassification::TypeContinuityDenied
                | SchemaReconciliationClassification::StructuralContinuityDenied
        )
    {
        SchemaContinuationAdmissionObservation::RejectedInAllLayers
    } else {
        SchemaContinuationAdmissionObservation::NonRejectedInAtLeastOneLayer
    }
}

fn bridgeability_after_policy(
    policy: Option<SchemaReconciliationPolicy>,
    continuation: SchemaContinuationClassification,
    bridgeability: SchemaBridgeabilityClassification,
) -> SchemaBridgeabilityClassification {
    if is_contract_upgrade_policy(policy)
        && continuation == SchemaContinuationClassification::ContinueWithContractUpgrade
    {
        SchemaBridgeabilityClassification::ContractUpgradeOnly
    } else {
        bridgeability
    }
}

pub(super) fn classify_reconciliation(atom: &SchemaDiffAtom) -> SchemaReconciliationClassification {
    match &atom.detail {
        SchemaDiffDetail::AddedField { .. }
        | SchemaDiffDetail::EnumDomainExpanded { .. }
        | SchemaDiffDetail::ProjectionContractChanged { .. }
        | SchemaDiffDetail::SubscriberContractChanged { .. } => {
            SchemaReconciliationClassification::Additive
        }
        SchemaDiffDetail::RemovedField { .. } => SchemaReconciliationClassification::Narrowing,
        SchemaDiffDetail::TypeChanged { .. } => {
            SchemaReconciliationClassification::TypeContinuityDenied
        }
        SchemaDiffDetail::InvariantContractChanged { .. } => {
            if atom.strata.contains(&SchemaStratum::BehavioralSemantics)
                || atom
                    .strata
                    .contains(&SchemaStratum::EntityIdentitySemantics)
                || atom.strata.contains(&SchemaStratum::LineageSemantics)
            {
                SchemaReconciliationClassification::StructuralContinuityDenied
            } else {
                SchemaReconciliationClassification::Additive
            }
        }
        SchemaDiffDetail::FreeText {
            declared_intent, ..
        } => match declared_intent {
            FreeFormSchemaDiffIntent::Additive => SchemaReconciliationClassification::Additive,
            FreeFormSchemaDiffIntent::StructuralContinuityDenied => {
                SchemaReconciliationClassification::StructuralContinuityDenied
            }
        },
    }
}

pub(super) fn classify_continuation(atom: &SchemaDiffAtom) -> SchemaContinuationClassification {
    match atom.subscriber_impact {
        SchemaSubscriberImpact::None => SchemaContinuationClassification::ContinueUnchanged,
        SchemaSubscriberImpact::ConsumableSurfaceChanged => {
            if atom.historical_interpretation == HistoricalInterpretationSensitivity::NotSensitive
                && atom.boundary_visibility
                    == SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable
            {
                SchemaContinuationClassification::ContinueWithVisibleBridge
            } else {
                SchemaContinuationClassification::RequireRenegotiation
            }
        }
        SchemaSubscriberImpact::ContractUpgradeRequired => {
            SchemaContinuationClassification::ContinueWithContractUpgrade
        }
        SchemaSubscriberImpact::RenegotiationRequired => {
            SchemaContinuationClassification::RequireRenegotiation
        }
    }
}

pub(super) fn classify_bridgeability(atom: &SchemaDiffAtom) -> SchemaBridgeabilityClassification {
    match classify_continuation(atom) {
        SchemaContinuationClassification::ContinueUnchanged
        | SchemaContinuationClassification::ContinueWithTransparentBridge => {
            SchemaBridgeabilityClassification::Transparent
        }
        SchemaContinuationClassification::ContinueWithVisibleBridge => {
            SchemaBridgeabilityClassification::SubscriberVisible
        }
        SchemaContinuationClassification::ContinueWithContractUpgrade => {
            SchemaBridgeabilityClassification::ContractUpgradeOnly
        }
        SchemaContinuationClassification::RequireRenegotiation => {
            SchemaBridgeabilityClassification::RenegotiationOnly
        }
        SchemaContinuationClassification::Rejected => SchemaBridgeabilityClassification::Rejected,
    }
}

pub fn is_narrowing(atom: &SchemaDiffAtom) -> bool {
    matches!(atom.detail, SchemaDiffDetail::RemovedField { .. })
}

pub fn is_contract_upgrade_policy(policy: Option<SchemaReconciliationPolicy>) -> bool {
    matches!(
        policy,
        Some(SchemaReconciliationPolicy::RequireExplicitProjection)
    )
}

pub(super) fn max_reconciliation_classification(
    current: SchemaReconciliationClassification,
    candidate: SchemaReconciliationClassification,
) -> SchemaReconciliationClassification {
    use SchemaReconciliationClassification::*;
    match (current, candidate) {
        (StructuralContinuityDenied, _) | (_, StructuralContinuityDenied) => {
            StructuralContinuityDenied
        }
        (TypeContinuityDenied, _) | (_, TypeContinuityDenied) => TypeContinuityDenied,
        (Narrowing, _) | (_, Narrowing) => Narrowing,
        _ => Additive,
    }
}

pub(super) fn max_continuation_classification(
    current: SchemaContinuationClassification,
    candidate: SchemaContinuationClassification,
) -> SchemaContinuationClassification {
    use SchemaContinuationClassification::*;
    match (current, candidate) {
        (Rejected, _) | (_, Rejected) => Rejected,
        (RequireRenegotiation, _) | (_, RequireRenegotiation) => RequireRenegotiation,
        (ContinueWithContractUpgrade, _) | (_, ContinueWithContractUpgrade) => {
            ContinueWithContractUpgrade
        }
        (ContinueWithVisibleBridge, _) | (_, ContinueWithVisibleBridge) => {
            ContinueWithVisibleBridge
        }
        (ContinueWithTransparentBridge, _) | (_, ContinueWithTransparentBridge) => {
            ContinueWithTransparentBridge
        }
        _ => ContinueUnchanged,
    }
}

pub(super) fn max_bridgeability_classification(
    current: SchemaBridgeabilityClassification,
    candidate: SchemaBridgeabilityClassification,
) -> SchemaBridgeabilityClassification {
    use SchemaBridgeabilityClassification::*;
    match (current, candidate) {
        (Rejected, _) | (_, Rejected) => Rejected,
        (RenegotiationOnly, _) | (_, RenegotiationOnly) => RenegotiationOnly,
        (ContractUpgradeOnly, _) | (_, ContractUpgradeOnly) => ContractUpgradeOnly,
        (SubscriberVisible, _) | (_, SubscriberVisible) => SubscriberVisible,
        _ => Transparent,
    }
}
