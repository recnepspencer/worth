use crate::basis_lifecycle::admit_basis_capability;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::ProjectionConsumptionEligibility;

use super::{
    WorthQueryAdmittedIntentPlan, WorthQueryAdmittedIntentPlanCore,
    WorthQueryAuthoritativeIntentExecutionPlan, WorthQueryBasisObservationPlan,
    WorthQueryEffectTriggeredIntentExecutionPlan, WorthQueryProjectionConsumptionPlan,
};
use crate::intent_admission::{
    WorthQueryAuthoritativeMutationBatchExecutionPlan,
    WorthQueryAuthoritativeMutationExecutionPlan, WorthQueryDerivedInspectionExecutionPlan,
    WorthQueryDerivedMaterializationExecutionPlan, WorthQueryExistingTruthProbeExecutionPlan,
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionEligibility,
    WorthQueryIntentAdmissionExecutionSeam, WorthQueryIntentAdmissionFamily,
    WorthQueryLiveReadExecutionPlan, WorthQueryReadExecutionPlan,
    WorthQueryUnifiedInspectionExecutionPlan,
};

impl WorthQueryAuthoritativeIntentExecutionPlan {
    pub(crate) fn from_eligibility(
        eligibility: WorthQueryIntentAdmissionEligibility,
        execution_seam: WorthQueryIntentAdmissionExecutionSeam,
    ) -> Self {
        Self {
            inner: WorthQueryAdmittedIntentPlanCore::from_eligibility(
                eligibility,
                Some(execution_seam),
            ),
        }
    }
}

impl WorthQueryEffectTriggeredIntentExecutionPlan {
    pub(crate) fn from_eligibility(
        eligibility: WorthQueryIntentAdmissionEligibility,
        execution_seam: WorthQueryIntentAdmissionExecutionSeam,
    ) -> Self {
        Self {
            inner: WorthQueryAdmittedIntentPlanCore::from_eligibility(
                eligibility,
                Some(execution_seam),
            ),
        }
    }
}

impl WorthQueryBasisObservationPlan {
    pub(crate) fn from_eligibility(eligibility: WorthQueryIntentAdmissionEligibility) -> Self {
        let normalized = eligibility
            .request()
            .basis_observation()
            .expect("basis observation plan requires basis payload")
            .clone();
        let admitted = admit_basis_capability(
            crate::basis_lifecycle::evaluate_basis_observation_eligibility(normalized)
                .expect("basis observation eligibility should admit before plan construction"),
        );
        Self {
            inner: WorthQueryAdmittedIntentPlanCore::from_eligibility(eligibility, None),
            capability: admitted,
        }
    }
}

impl WorthQueryProjectionConsumptionPlan {
    pub(crate) fn from_eligibility(
        eligibility: WorthQueryIntentAdmissionEligibility,
    ) -> Result<Self, IdentityDerivationDenial> {
        let declaration = eligibility
            .request()
            .projection_consumption_declaration()
            .expect("projection plan requires projection declaration");
        let (admitted, warnings) =
            match crate::projection_consumption::evaluate_projection_consumption_eligibility(
                declaration,
            )? {
                ProjectionConsumptionEligibility::Admitted(admitted) => (admitted, None),
                ProjectionConsumptionEligibility::AdmittedWithWarnings(admitted, warnings) => {
                    (admitted, Some(warnings))
                }
                other => panic!(
                    "projection plan requires admitted projection eligibility, got {other:?}"
                ),
            };
        Ok(Self {
            inner: WorthQueryAdmittedIntentPlanCore::from_eligibility(eligibility, None),
            admitted,
            warnings,
        })
    }
}

impl WorthQueryAdmittedIntentPlan {
    pub(crate) fn from_eligibility(
        eligibility: WorthQueryIntentAdmissionEligibility,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(match eligibility.request().family() {
            WorthQueryIntentAdmissionFamily::AuthoritativeUserIntent => Self::Authoritative(
                WorthQueryAuthoritativeIntentExecutionPlan::from_eligibility(
                    eligibility,
                    WorthQueryIntentAdmissionExecutionSeam::BackendIntentAuthorityRoute,
                ),
            ),
            WorthQueryIntentAdmissionFamily::EffectTriggeredWriteIntent => Self::EffectTriggered(
                WorthQueryEffectTriggeredIntentExecutionPlan::from_eligibility(
                    eligibility,
                    WorthQueryIntentAdmissionExecutionSeam::BackendIntentAuthorityRoute,
                ),
            ),
            WorthQueryIntentAdmissionFamily::AuthoritativeMutationIntent => {
                match eligibility.request().entrypoint() {
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteScalarWrite => {
                        Self::AuthoritativeMutation(
                            WorthQueryAuthoritativeMutationExecutionPlan::from_eligibility(
                                eligibility,
                                WorthQueryIntentAdmissionExecutionSeam::BackendWriteAuthorityRoute,
                            ),
                        )
                    }
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteBatchWrite => {
                        Self::AuthoritativeMutationBatch(
                            WorthQueryAuthoritativeMutationBatchExecutionPlan::from_eligibility(
                                eligibility,
                                WorthQueryIntentAdmissionExecutionSeam::BackendWriteAuthorityRoute,
                            ),
                        )
                    }
                    other => panic!(
                        "authoritative mutation family cannot mint admitted plan for {other:?}"
                    ),
                }
            }
            WorthQueryIntentAdmissionFamily::ReadExecutionIntent => {
                match eligibility.request().entrypoint() {
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteLiveRead => {
                        Self::LiveReadExecution(WorthQueryLiveReadExecutionPlan::from_eligibility(
                            eligibility,
                            WorthQueryIntentAdmissionExecutionSeam::QueryRuntimeReadExecutionRoute,
                        ))
                    }
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamily
                    | WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamilyInBasisContext => {
                        Self::ReadExecution(WorthQueryReadExecutionPlan::from_eligibility(
                            eligibility,
                            WorthQueryIntentAdmissionExecutionSeam::QueryRuntimeReadExecutionRoute,
                        ))
                    }
                    other => {
                        panic!("read execution family cannot mint admitted plan for {other:?}")
                    }
                }
            }
            WorthQueryIntentAdmissionFamily::InspectionMaterializationIntent => {
                match eligibility.request().entrypoint() {
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedMaterialization => {
                        Self::DerivedMaterialization(
                            WorthQueryDerivedMaterializationExecutionPlan::from_eligibility(
                                eligibility,
                                WorthQueryIntentAdmissionExecutionSeam::QueryRuntimeInspectionMaterializationRoute,
                            ),
                        )
                    }
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteDerivedInspection => {
                        Self::DerivedInspection(
                            WorthQueryDerivedInspectionExecutionPlan::from_eligibility(
                                eligibility,
                                WorthQueryIntentAdmissionExecutionSeam::QueryRuntimeInspectionMaterializationRoute,
                            ),
                        )
                    }
                    WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteUnifiedInspection => {
                        Self::UnifiedInspection(
                            WorthQueryUnifiedInspectionExecutionPlan::from_eligibility(
                                eligibility,
                                WorthQueryIntentAdmissionExecutionSeam::QueryRuntimeInspectionMaterializationRoute,
                            ),
                        )
                    }
                    other => panic!(
                        "inspection-materialization family cannot mint admitted plan for {other:?}"
                    ),
                }
            }
            WorthQueryIntentAdmissionFamily::LowerRuntimeCapabilityRoutingIntent => {
                Self::ExistingTruthProbeRouting(
                    WorthQueryExistingTruthProbeExecutionPlan::from_eligibility(
                        eligibility,
                        WorthQueryIntentAdmissionExecutionSeam::BackendExistingTruthProbeRoute,
                    ),
                )
            }
            WorthQueryIntentAdmissionFamily::BasisUseIntent => Self::BasisObservation(
                WorthQueryBasisObservationPlan::from_eligibility(eligibility),
            ),
            WorthQueryIntentAdmissionFamily::ProjectionConsumptionIntent => {
                Self::ProjectionConsumption(WorthQueryProjectionConsumptionPlan::from_eligibility(
                    eligibility,
                )?)
            }
        })
    }
}
