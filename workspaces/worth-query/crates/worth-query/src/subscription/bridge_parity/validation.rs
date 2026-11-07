use crate::subscription::activation::SubscriptionActivationInput;
use crate::subscription::bridge_lowering::BridgeSubscriptionLoweringPlan;
use crate::subscription::declaration::QuerySubscriptionDeclarationArtifact;
use crate::subscription::evidence_identities::typed_identity_drift;
use crate::subscription::family::QuerySubscriptionFamily;
use crate::subscription::validation_evidence::{
    validation_label_list_evidence_identity, validation_role_evidence_identity,
    validation_shape_role_evidence_identity,
};

use super::support::{
    QuerySubscriptionBridgeParityClass, QuerySubscriptionBridgeParityCounters,
    QuerySubscriptionBridgeParityError, QuerySubscriptionBridgeParityFailure,
    QuerySubscriptionBridgeParityFailureKind,
};
use super::witness::QuerySubscriptionManualBridgeWitness;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(test)]
pub(super) struct CanonicalBridgeParitySemantics {
    pub(super) query_family_label: String,
    pub(super) declaration_family_label: String,
    pub(super) bridge_family_label: String,
    pub(super) bridge_slice_labels: Vec<String>,
    pub(super) basis_posture_label: String,
    pub(super) signal_strategy_class_label: String,
}

impl CanonicalBridgeParitySemantics {
    #[cfg(test)]
    pub(super) fn from_authoritative_sources(
        declaration: &QuerySubscriptionDeclarationArtifact,
        lowering: &BridgeSubscriptionLoweringPlan,
    ) -> Self {
        Self {
            query_family_label: declaration.family().as_str().to_string(),
            declaration_family_label: declaration.family().as_str().to_string(),
            bridge_family_label: lowering.bridge_family().as_str().to_string(),
            bridge_slice_labels: lowering
                .bridge_slices()
                .iter()
                .map(|slice| slice.as_str().to_string())
                .collect(),
            basis_posture_label: declaration.basis_posture().as_str().to_string(),
            signal_strategy_class_label: lowering
                .signal_strategy_request()
                .request_kind()
                .as_str()
                .to_string(),
        }
    }
}

#[cfg(test)]
pub(super) fn validate_parity_sources(
    declaration: &QuerySubscriptionDeclarationArtifact,
    lowering: &BridgeSubscriptionLoweringPlan,
    activation: &SubscriptionActivationInput,
    witness: &QuerySubscriptionManualBridgeWitness,
    semantics: &CanonicalBridgeParitySemantics,
) -> Result<(), QuerySubscriptionBridgeParityError> {
    if typed_identity_drift(
        declaration.declaration_identity(),
        witness.query_declaration_identity(),
    ) || typed_identity_drift(
        lowering.query_declaration_identity(),
        witness.query_declaration_identity(),
    ) || typed_identity_drift(
        activation.query_declaration_identity(),
        witness.query_declaration_identity(),
    ) {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::DeclarationMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires declaration, lowering, activation, and witness to preserve canonical declaration identity",
                witness.query_declaration_identity().clone(),
                &[
                    validation_role_evidence_identity(
                        "declaration",
                        declaration.declaration_identity(),
                    ),
                    validation_role_evidence_identity(
                        "lowering",
                        lowering.query_declaration_identity(),
                    ),
                    validation_role_evidence_identity(
                        "activation",
                        activation.query_declaration_identity(),
                    ),
                    validation_role_evidence_identity(
                        "witness",
                        witness.query_declaration_identity(),
                    ),
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    if typed_identity_drift(
        activation.evidence_identity(),
        witness.activation_identity(),
    ) {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::ActivationMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires activation and witness to preserve the same runtime activation identity",
                witness.activation_identity().clone(),
                &[
                    validation_role_evidence_identity("activation", activation.evidence_identity()),
                    validation_role_evidence_identity("witness", witness.activation_identity()),
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    if declaration.family().as_str() != witness.query_family_label()
        || declaration.family().as_str() != witness.declaration_family_label()
        || semantics.query_family_label != witness.query_family_label()
        || semantics.declaration_family_label != witness.declaration_family_label()
    {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::DeclarationMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires witness family labels to preserve canonical query and declaration family semantics",
                witness.query_declaration_identity().clone(),
                &[
                    validation_shape_role_evidence_identity(
                        "declaration_family",
                        declaration.family().as_str(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                    validation_shape_role_evidence_identity(
                        "witness_query_family",
                        witness.query_family_label(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                    validation_shape_role_evidence_identity(
                        "witness_declaration_family",
                        witness.declaration_family_label(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    if typed_identity_drift(
        lowering.bridge_declaration_identity(),
        witness.bridge_declaration_identity(),
    ) || typed_identity_drift(
        activation.bridge_declaration_identity(),
        witness.bridge_declaration_identity(),
    ) {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::BridgeMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires lowering, activation, and witness to preserve bridge declaration identity",
                witness.bridge_declaration_identity().clone(),
                &[
                    validation_role_evidence_identity(
                        "lowering",
                        lowering.bridge_declaration_identity(),
                    ),
                    validation_role_evidence_identity(
                        "activation",
                        activation.bridge_declaration_identity(),
                    ),
                    validation_role_evidence_identity(
                        "witness",
                        witness.bridge_declaration_identity(),
                    ),
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    if lowering.bridge_family().as_str() != witness.bridge_family_label()
        || semantics.bridge_family_label != witness.bridge_family_label()
        || semantics.bridge_slice_labels.as_slice() != witness.bridge_slice_labels()
    {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::BridgeMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires witness bridge family and slice labels to preserve canonical lowering semantics",
                witness.bridge_declaration_identity().clone(),
                &[
                    validation_shape_role_evidence_identity(
                        "lowering_bridge_family",
                        lowering.bridge_family().as_str(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                    validation_shape_role_evidence_identity(
                        "witness_bridge_family",
                        witness.bridge_family_label(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                    validation_label_list_evidence_identity(
                        "lowering_bridge_slices",
                        &semantics.bridge_slice_labels,
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                    validation_label_list_evidence_identity(
                        "witness_bridge_slices",
                        witness.bridge_slice_labels(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    if typed_identity_drift(
        lowering.basis_request().evidence_identity(),
        witness.basis_binding_identity(),
    ) || typed_identity_drift(
        activation.basis_binding_identity(),
        witness.basis_binding_identity(),
    ) {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::BasisMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires lowering, activation, and witness to preserve basis request identity",
                witness.basis_binding_identity().clone(),
                &[
                    validation_role_evidence_identity(
                        "lowering",
                        lowering.basis_request().evidence_identity(),
                    ),
                    validation_role_evidence_identity(
                        "activation",
                        activation.basis_binding_identity(),
                    ),
                    validation_role_evidence_identity(
                        "witness",
                        witness.basis_binding_identity(),
                    ),
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    if declaration.basis_posture().as_str() != witness.basis_posture_label()
        || semantics.basis_posture_label != witness.basis_posture_label()
    {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::BasisMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires witness basis posture labels to preserve canonical declaration semantics",
                witness.basis_binding_identity().clone(),
                &[
                    validation_shape_role_evidence_identity(
                        "declaration_basis",
                        declaration.basis_posture().as_str(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                    validation_shape_role_evidence_identity(
                        "witness_basis",
                        witness.basis_posture_label(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    if typed_identity_drift(
        lowering.signal_strategy_request().evidence_identity(),
        witness.signal_strategy_identity(),
    ) || typed_identity_drift(
        activation.signal_strategy_identity(),
        witness.signal_strategy_identity(),
    ) {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::SignalStrategyMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires lowering, activation, and witness to preserve signal strategy identity",
                witness.signal_strategy_identity().clone(),
                &[
                    validation_role_evidence_identity(
                        "lowering",
                        lowering.signal_strategy_request().evidence_identity(),
                    ),
                    validation_role_evidence_identity(
                        "activation",
                        activation.signal_strategy_identity(),
                    ),
                    validation_role_evidence_identity(
                        "witness",
                        witness.signal_strategy_identity(),
                    ),
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    if lowering.signal_strategy_request().request_kind().as_str()
        != witness.signal_strategy_class_label()
        || semantics.signal_strategy_class_label != witness.signal_strategy_class_label()
    {
        return Err(QuerySubscriptionBridgeParityError::new(
            QuerySubscriptionBridgeParityFailure::new(
                QuerySubscriptionBridgeParityFailureKind::SignalStrategyMismatch,
                QuerySubscriptionBridgeParityClass::DeniedSourceMismatch,
                "bridge parity explanation requires witness signal strategy labels to preserve canonical lowering semantics",
                witness.signal_strategy_identity().clone(),
                &[
                    validation_shape_role_evidence_identity(
                        "lowering_signal_strategy",
                        lowering.signal_strategy_request().request_kind().as_str(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                    validation_shape_role_evidence_identity(
                        "witness_signal_strategy",
                        witness.signal_strategy_class_label(),
                    ).map_err(|cause| QuerySubscriptionBridgeParityError::identity_refused(cause, witness.evidence_identity().clone()))?,
                ],
            ),
            QuerySubscriptionBridgeParityCounters::denied(),
        ));
    }

    Ok(())
}

#[cfg(test)]
pub(super) fn parity_class_for_family(
    family: &QuerySubscriptionFamily,
) -> QuerySubscriptionBridgeParityClass {
    match family {
        QuerySubscriptionFamily::InspectorDetailExact
        | QuerySubscriptionFamily::GroupedCollectionMembership
        | QuerySubscriptionFamily::BoundedMaterialization => {
            QuerySubscriptionBridgeParityClass::FamilyDistinctBridgeShared
        }
        QuerySubscriptionFamily::DetailExact | QuerySubscriptionFamily::CollectionMembership => {
            QuerySubscriptionBridgeParityClass::ExactParity
        }
    }
}
