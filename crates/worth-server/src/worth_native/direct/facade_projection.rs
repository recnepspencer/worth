use worth_proof::TransitionOutcome;

use crate::{
    declaration_intake::WorthServerNamedLiveProjectionExecutionError,
    WorthServerAdmittedDirectDeclaration, WorthServerDirectProjectionRequest,
    WorthServerOperationFamily, WorthServerQueryHandoffOperation, WorthServerResponseInput,
};

use super::{
    WorthServerDirectContextArtifact, WorthServerDirectProjection, WorthServerDirectRemaskPosture,
    WorthServerWorthNativeDirectFacade,
};

impl WorthServerWorthNativeDirectFacade {
    pub fn project(
        &self,
        declaration: &WorthServerAdmittedDirectDeclaration,
        request: &WorthServerDirectProjectionRequest,
    ) -> super::WorthServerDirectProjectionOutcome {
        if let Err(denial) =
            self.admit_operation_family(WorthServerOperationFamily::QueryDirectProjection)
        {
            return self.operation_denial_outcome(denial);
        }
        let observed_basis_digest = match declaration.subscription_basis_digest() {
            Ok(value) => value,
            Err(error) => return self.runtime_error_outcome(error),
        };
        let operation_name = declaration.declaration_binding_label().to_string();
        let handoff_target = declaration.declaration_binding_label().to_string();
        let operation_request = match self.admit_shared_read_operation_request(
            WorthServerOperationFamily::QueryDirectProjection,
            &operation_name,
            &observed_basis_digest,
        ) {
            Ok(value) => value,
            Err(denial) => return TransitionOutcome::Denied(denial),
        };
        match self.prepare_declared_plan(
            operation_request,
            WorthServerQueryHandoffOperation::direct_projection(&handoff_target),
        ) {
            Ok(plan) => {
                let plan_proof = plan.proof();
                let handoff = plan.into_query_handoff();
                let projection_attempt = match declaration.consume_named_live_projection(request) {
                    Ok(projection_attempt) => projection_attempt,
                    Err(WorthServerNamedLiveProjectionExecutionError::Runtime(error)) => {
                        return self.runtime_error_outcome(*error);
                    }
                    Err(WorthServerNamedLiveProjectionExecutionError::Consumption(path_error)) => {
                        return self.projection_error_outcome(*path_error);
                    }
                };
                let support_posture = handoff.support_posture().clone();
                let workspace_name = handoff.workspace().name().to_string();
                let handoff_digest = handoff.canonical_digest().to_string();
                let response_envelope = self
                    .responses
                    .shape_with_defaults(WorthServerResponseInput::query_handoff_success(handoff));
                match projection_attempt {
                    worth_query::facade::foundation::ProjectionAuthorityOutcome::Admitted(authority) => {
                        let direct_context = WorthServerDirectContextArtifact::new(
                            self.admission.request_context(),
                            &support_posture,
                            &response_envelope,
                            authority.contract().basis_digest(),
                            WorthServerDirectRemaskPosture::from_materialized_fact_posture(
                                authority.receipt().materialized_fact_posture(),
                            ),
                        );
                        WorthServerDirectProjection::from_authority(
                            super::WorthServerDirectProjectionParts {
                            plan_proof: plan_proof.clone(),
                            support_posture,
                            workspace_name,
                            handoff_digest,
                            direct_context,
                            authority: *authority,
                            warning_kinds: Vec::new(),
                            response_envelope,
                            },
                        ).map_or_else(|cause| TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(crate::WorthServerQueryHandoffDenialCode::EvidenceIdentityDerivationDenied(cause), self.admission.request_context().diagnostics_profile(), "projection envelope identity exceeded canonical digest bounds")), TransitionOutcome::Success)
                    }
                    worth_query::facade::foundation::ProjectionAuthorityOutcome::AdmittedWithWarnings(
                        authority,
                        warnings,
                    ) => {
                        let direct_context = WorthServerDirectContextArtifact::new(
                            self.admission.request_context(),
                            &support_posture,
                            &response_envelope,
                            authority.contract().basis_digest(),
                            WorthServerDirectRemaskPosture::from_materialized_fact_posture(
                                authority.receipt().materialized_fact_posture(),
                            ),
                        );
                        WorthServerDirectProjection::from_authority(
                            super::WorthServerDirectProjectionParts {
                            plan_proof,
                            support_posture,
                            workspace_name,
                            handoff_digest,
                            direct_context,
                            authority: *authority,
                            warning_kinds: warnings.warning_kinds().to_vec(),
                            response_envelope,
                            },
                        ).map_or_else(|cause| TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(crate::WorthServerQueryHandoffDenialCode::EvidenceIdentityDerivationDenied(cause), self.admission.request_context().diagnostics_profile(), "projection envelope identity exceeded canonical digest bounds")), TransitionOutcome::Success)
                    }
                    worth_query::facade::foundation::ProjectionAuthorityOutcome::ConsumptionDenied(denied) => {
                        TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                            crate::WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionDenied,
                            self.admission.request_context().diagnostics_profile(),
                            format!("{:?}", denied.reason()),
                        ))
                    }
                    worth_query::facade::foundation::ProjectionAuthorityOutcome::Deferred(deferred) => {
                        TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                            crate::WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionDeferred,
                            self.admission.request_context().diagnostics_profile(),
                            format!("{:?}", deferred.reason()),
                        ))
                    }
                    worth_query::facade::foundation::ProjectionAuthorityOutcome::SourceMismatch(
                        mismatch,
                    ) => TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                        crate::WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionSourceMismatch,
                        self.admission.request_context().diagnostics_profile(),
                        format!(
                            "{:?} unsupported for {:?}",
                            mismatch.requested_fact_kind(),
                            mismatch.source_family()
                        ),
                    )),
                    worth_query::facade::foundation::ProjectionAuthorityOutcome::AuthorityDenied(denied) => {
                        TransitionOutcome::Denied(crate::WorthServerQueryHandoffDenial::new(
                            crate::WorthServerQueryHandoffDenialCode::ProjectionFactConsumptionDenied,
                            self.admission.request_context().diagnostics_profile(),
                            format!("projection authority denied: {:?}", denied.kind()),
                        ))
                    }
                }
            }
            Err(denial) => TransitionOutcome::Denied(denial),
        }
    }
}
