use super::super::super::admission::QuerySubscriptionAdmissionArtifact;
use super::super::super::bridge_lowering::BridgeSubscriptionLoweringPlan;
use super::super::super::declaration::QuerySubscriptionDeclarationArtifact;
use super::super::super::evidence_identities::typed_identity_drift;
use super::super::super::support::QuerySubscriptionSupportReport;
use super::super::bundle::{
    QuerySubscriptionDiagnosticBundleError, QuerySubscriptionDiagnosticBundleErrorKind,
    QuerySubscriptionDiagnosticFailure,
};
use super::super::context::QuerySubscriptionDiagnosticSelectionContext;
use super::super::stage::{QuerySubscriptionDiagnosticEvidence, QuerySubscriptionDiagnosticStage};
use super::identity::{stage_evidence_from_state, trace_from_stage_evidence};
use super::validation::{failure_is_selection_stage, validate_denied_selection_context};
use super::vocabulary::QuerySubscriptionDiagnosticTrace;

pub fn trace_denied_query_subscription_diagnostics(
    selection_context: &QuerySubscriptionDiagnosticSelectionContext,
    declaration: Option<&QuerySubscriptionDeclarationArtifact>,
    lowering: Option<&BridgeSubscriptionLoweringPlan>,
    admission: Option<&QuerySubscriptionAdmissionArtifact>,
    support: Option<&QuerySubscriptionSupportReport>,
    failure: QuerySubscriptionDiagnosticFailure,
) -> Result<QuerySubscriptionDiagnosticTrace, QuerySubscriptionDiagnosticBundleError> {
    validate_denied_selection_context(
        selection_context,
        failure.stage(),
        &failure,
        declaration.is_some() || lowering.is_some() || admission.is_some() || support.is_some(),
    )?;

    if selection_context.selection().is_some() && failure_is_selection_stage(*failure.stage()) {
        return Err(QuerySubscriptionDiagnosticBundleError::new(
                QuerySubscriptionDiagnosticBundleErrorKind::SelectionContextMismatch,
                "diagnostic trace assembly may not bind an admitted family selection context to a family-selection denial",
                &[
                    format!("selection_context:{}", selection_context.context_projection().label()),
                    format!("failure_stage:{}", failure.stage().as_str()),
                ],
            ));
    }

    if let Some(declaration) = declaration
        && selection_context
            .selection()
            .map(|selection| selection.family())
            != Some(declaration.family())
    {
        return Err(QuerySubscriptionDiagnosticBundleError::new(
                QuerySubscriptionDiagnosticBundleErrorKind::DeclarationSourceMismatch,
                "diagnostic trace assembly requires declaration and family selection to preserve the same query subscription family",
                &[
                    format!(
                        "selection_family:{}",
                        selection_context.query_family_label()
                    ),
                    format!("declaration_family:{}", declaration.family().as_str()),
                ],
            ));
    }
    if let (Some(declaration), Some(lowering)) = (declaration, lowering)
        && typed_identity_drift(
            declaration.declaration_identity(),
            lowering.query_declaration_identity(),
        )
    {
        return Err(QuerySubscriptionDiagnosticBundleError::new(
                QuerySubscriptionDiagnosticBundleErrorKind::BridgeLoweringSourceMismatch,
                "diagnostic trace assembly requires bridge lowering to bind the same declaration artifact",
                &[
                    format!("declaration:{}", declaration.declaration_projection().label()),
                    format!("lowering:{}", lowering.query_declaration_projection().label()),
                ],
            ));
    }
    if let (Some(declaration), Some(admission)) = (declaration, admission)
        && typed_identity_drift(
            declaration.declaration_identity(),
            admission.query_declaration_identity(),
        )
    {
        return Err(QuerySubscriptionDiagnosticBundleError::new(
            QuerySubscriptionDiagnosticBundleErrorKind::AdmissionSourceMismatch,
            "diagnostic trace assembly requires admission to preserve declaration identity",
            &[
                format!(
                    "declaration:{}",
                    declaration.declaration_projection().label()
                ),
                format!(
                    "admission:{}",
                    admission.query_declaration_projection().label()
                ),
            ],
        ));
    }
    if let (Some(declaration), Some(support)) = (declaration, support)
        && typed_identity_drift(
            declaration.declaration_identity(),
            support.support_subject().declaration_identity(),
        )
    {
        return Err(QuerySubscriptionDiagnosticBundleError::new(
            QuerySubscriptionDiagnosticBundleErrorKind::SupportSourceMismatch,
            "diagnostic trace assembly requires support reporting to preserve declaration identity",
            &[
                format!(
                    "declaration:{}",
                    declaration.declaration_projection().label()
                ),
                format!(
                    "support_declaration:{}",
                    support.support_subject().declaration_projection().label()
                ),
            ],
        ));
    }

    let failure_source_identity = failure.source_identity();
    let failure_counter_identity = failure.counter_identity();

    let mut stage_evidence = if let Some(selection) = selection_context.selection() {
        vec![QuerySubscriptionDiagnosticEvidence::admitted(
            QuerySubscriptionDiagnosticStage::FamilySelection,
            format!(
                "query subscription family {} selected through canonical admission classification",
                selection.family().as_str()
            ),
            selection.equivalence_basis().evidence_identity(),
            &selection.counters().evidence_identity(),
        )]
    } else {
        vec![stage_evidence_from_state(
            *failure.stage(),
            false,
            failure.reason().to_string(),
            failure_source_identity,
            failure_counter_identity,
        )]
    };

    if selection_context.selection().is_some() {
        if let Some(declaration) = declaration {
            let declaration_counters = declaration.counters().evidence_identity();
            let stage = if *failure.stage() == QuerySubscriptionDiagnosticStage::Declaration
                || *failure.stage() == QuerySubscriptionDiagnosticStage::DeliveryIntent
            {
                *failure.stage()
            } else {
                QuerySubscriptionDiagnosticStage::Declaration
            };
            let admitted = stage != *failure.stage();
            stage_evidence.push(stage_evidence_from_state(
                stage,
                admitted,
                if admitted {
                    format!(
                        "query subscription declaration {} preserved canonical family semantics",
                        declaration.family().as_str()
                    )
                } else {
                    failure.reason().to_string()
                },
                if admitted {
                    declaration.declaration_identity()
                } else {
                    failure_source_identity
                },
                if admitted {
                    &declaration_counters
                } else {
                    failure_counter_identity
                },
            ));
        } else if matches!(
            failure.stage(),
            QuerySubscriptionDiagnosticStage::Declaration
                | QuerySubscriptionDiagnosticStage::DeliveryIntent
        ) {
            stage_evidence.push(stage_evidence_from_state(
                *failure.stage(),
                false,
                failure.reason().to_string(),
                failure_source_identity,
                failure_counter_identity,
            ));
        }
    }

    if let Some(lowering) = lowering {
        let lowering_counters = lowering.counters().evidence_identity();
        let bridge_failure = matches!(
            failure.stage(),
            QuerySubscriptionDiagnosticStage::BridgeFamilyLowering
                | QuerySubscriptionDiagnosticStage::BridgeSliceLowering
                | QuerySubscriptionDiagnosticStage::BasisBinding
        );
        stage_evidence.push(stage_evidence_from_state(
            if bridge_failure {
                *failure.stage()
            } else {
                QuerySubscriptionDiagnosticStage::BridgeFamilyLowering
            },
            !bridge_failure,
            if bridge_failure {
                failure.reason().to_string()
            } else {
                format!(
                    "bridge lowering admitted family {} for canonical query declaration",
                    lowering.bridge_family().as_str()
                )
            },
            if bridge_failure {
                failure_source_identity
            } else {
                lowering.bridge_declaration_identity()
            },
            if bridge_failure {
                failure_counter_identity
            } else {
                &lowering_counters
            },
        ));
    }

    if let Some(admission) = admission {
        let admission_counters = admission.counters().evidence_identity();
        let admission_failure = matches!(
            failure.stage(),
            QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission
                | QuerySubscriptionDiagnosticStage::AdmissionBudget
                | QuerySubscriptionDiagnosticStage::DurableReloadOverclaim
                | QuerySubscriptionDiagnosticStage::ActiveLifecycleAllocation
                | QuerySubscriptionDiagnosticStage::ActivationReadiness
        );
        stage_evidence.push(stage_evidence_from_state(
            if admission_failure {
                *failure.stage()
            } else {
                QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission
            },
            !admission_failure,
            if admission_failure {
                failure.reason().to_string()
            } else {
                "runtime-backed subscription admission preserved declaration, bridge, basis, and signal identity".to_string()
            },
            if admission_failure {
                failure_source_identity
            } else {
                admission.evidence_identity()
            },
            if admission_failure {
                failure_counter_identity
            } else {
                &admission_counters
            },
        ));
    }

    if let Some(support) = support {
        let support_counters = support.counters().evidence_identity();
        let support_failure =
            *failure.stage() == QuerySubscriptionDiagnosticStage::SupportReporting;
        stage_evidence.push(stage_evidence_from_state(
            QuerySubscriptionDiagnosticStage::SupportReporting,
            !support_failure,
            if support_failure {
                failure.reason().to_string()
            } else {
                format!(
                    "support reporting certified {} for subject class {}",
                    support.support_posture().as_str(),
                    support.support_subject().support_class().as_str()
                )
            },
            if support_failure {
                failure_source_identity
            } else {
                support.report_identity()
            },
            if support_failure {
                failure_counter_identity
            } else {
                &support_counters
            },
        ));
    } else if *failure.stage() == QuerySubscriptionDiagnosticStage::SupportReporting {
        stage_evidence.push(stage_evidence_from_state(
            QuerySubscriptionDiagnosticStage::SupportReporting,
            false,
            failure.reason().to_string(),
            failure_source_identity,
            failure_counter_identity,
        ));
    }

    if *failure.stage() == QuerySubscriptionDiagnosticStage::Certification {
        stage_evidence.push(stage_evidence_from_state(
            QuerySubscriptionDiagnosticStage::Certification,
            false,
            failure.reason().to_string(),
            failure_source_identity,
            failure_counter_identity,
        ));
    }

    Ok(trace_from_stage_evidence(stage_evidence))
}
