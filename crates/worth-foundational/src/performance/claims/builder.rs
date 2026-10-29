use crate::performance::legality::{
    evaluate_performance_primitive_legality, FoundationalPerformancePrimitiveLegalityDenial,
};
use crate::performance::primitives::{
    FoundationalPerformanceAccessPatternPosture, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceWorkClass,
};

use super::context::FoundationalPerformanceObservationContext;
use super::types::{
    FoundationalAuthoritativePerformanceClaim, FoundationalPerformanceClaimPayload,
    FoundationalPolicyAdmissionPerformanceClaim, FoundationalReplayMaterializationPerformanceClaim,
    FoundationalSupportDerivedPerformanceClaim,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundationalPerformanceClaimConstructionDenial {
    MissingBoundary,
    MissingEvidenceStrength,
    MissingBreadthLocality,
    MissingAccessPattern,
    MissingExecutionTemperature,
    MissingFreshnessRetention,
    MissingFallbackDebt,
    MissingIncludedWorkDisclosure,
    MissingExcludedWorkDisclosure,
    MissingObservationContext,
    ObservationWorkRequiresActiveDisposition,
    OverlappingIncludedAndExcludedWorkDisclosure,
    BoundaryNotAllowedForClaimFamily,
    EvidenceStrengthNotAllowedForClaimFamily,
    PrimitiveLegality(FoundationalPerformancePrimitiveLegalityDenial),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FoundationalPerformanceClaimAuthoringFrontDoor;

#[derive(Debug, Clone, Default)]
struct FoundationalPerformanceClaimBuilderState {
    boundary: Option<FoundationalPerformanceBoundary>,
    evidence_strength: Option<FoundationalPerformanceEvidenceStrength>,
    breadth_locality: Option<FoundationalPerformanceBreadthLocalityPosture>,
    access_pattern: Option<FoundationalPerformanceAccessPatternPosture>,
    execution_temperature: Option<FoundationalPerformanceExecutionTemperature>,
    freshness_retention: Option<FoundationalPerformanceFreshnessRetentionPosture>,
    fallback_debt: Option<FoundationalPerformanceFallbackDebtPosture>,
    included_work: Vec<FoundationalPerformanceWorkClass>,
    excluded_work: Vec<FoundationalPerformanceWorkClass>,
    observation_context: Option<FoundationalPerformanceObservationContext>,
}

#[derive(Debug, Clone, Default)]
pub struct FoundationalAuthoritativePerformanceClaimBuilder {
    state: FoundationalPerformanceClaimBuilderState,
}

#[derive(Debug, Clone, Default)]
pub struct FoundationalSupportDerivedPerformanceClaimBuilder {
    state: FoundationalPerformanceClaimBuilderState,
}

#[derive(Debug, Clone, Default)]
pub struct FoundationalReplayMaterializationPerformanceClaimBuilder {
    state: FoundationalPerformanceClaimBuilderState,
}

#[derive(Debug, Clone, Default)]
pub struct FoundationalPolicyAdmissionPerformanceClaimBuilder {
    state: FoundationalPerformanceClaimBuilderState,
}

impl FoundationalPerformanceClaimAuthoringFrontDoor {
    pub fn authoritative_execution(self) -> FoundationalAuthoritativePerformanceClaimBuilder {
        FoundationalAuthoritativePerformanceClaimBuilder::default()
    }

    pub fn support_derived(self) -> FoundationalSupportDerivedPerformanceClaimBuilder {
        FoundationalSupportDerivedPerformanceClaimBuilder::default()
    }

    pub fn replay_or_materialization(
        self,
    ) -> FoundationalReplayMaterializationPerformanceClaimBuilder {
        FoundationalReplayMaterializationPerformanceClaimBuilder::default()
    }

    pub fn policy_admission(self) -> FoundationalPolicyAdmissionPerformanceClaimBuilder {
        FoundationalPolicyAdmissionPerformanceClaimBuilder::default()
    }
}

macro_rules! claim_builder_methods {
    ($builder:ident) => {
        impl $builder {
            pub fn boundary(mut self, boundary: FoundationalPerformanceBoundary) -> Self {
                self.state.boundary = Some(boundary);
                self
            }

            pub fn evidence_strength(
                mut self,
                evidence_strength: FoundationalPerformanceEvidenceStrength,
            ) -> Self {
                self.state.evidence_strength = Some(evidence_strength);
                self
            }

            pub fn breadth_locality(
                mut self,
                breadth_locality: FoundationalPerformanceBreadthLocalityPosture,
            ) -> Self {
                self.state.breadth_locality = Some(breadth_locality);
                self
            }

            pub fn access_pattern(
                mut self,
                access_pattern: FoundationalPerformanceAccessPatternPosture,
            ) -> Self {
                self.state.access_pattern = Some(access_pattern);
                self
            }

            pub fn execution_temperature(
                mut self,
                execution_temperature: FoundationalPerformanceExecutionTemperature,
            ) -> Self {
                self.state.execution_temperature = Some(execution_temperature);
                self
            }

            pub fn freshness_retention(
                mut self,
                freshness_retention: FoundationalPerformanceFreshnessRetentionPosture,
            ) -> Self {
                self.state.freshness_retention = Some(freshness_retention);
                self
            }

            pub fn fallback_debt(
                mut self,
                fallback_debt: FoundationalPerformanceFallbackDebtPosture,
            ) -> Self {
                self.state.fallback_debt = Some(fallback_debt);
                self
            }

            pub fn include_work(mut self, work_class: FoundationalPerformanceWorkClass) -> Self {
                self.state.included_work.push(work_class);
                self
            }

            pub fn exclude_work(mut self, work_class: FoundationalPerformanceWorkClass) -> Self {
                self.state.excluded_work.push(work_class);
                self
            }

            pub fn observation_context(
                mut self,
                context: FoundationalPerformanceObservationContext,
            ) -> Self {
                self.state.observation_context = Some(context);
                self
            }
        }
    };
}

claim_builder_methods!(FoundationalAuthoritativePerformanceClaimBuilder);
claim_builder_methods!(FoundationalSupportDerivedPerformanceClaimBuilder);
claim_builder_methods!(FoundationalReplayMaterializationPerformanceClaimBuilder);
claim_builder_methods!(FoundationalPolicyAdmissionPerformanceClaimBuilder);

impl FoundationalAuthoritativePerformanceClaimBuilder {
    pub fn finish(
        self,
    ) -> Result<
        FoundationalAuthoritativePerformanceClaim,
        FoundationalPerformanceClaimConstructionDenial,
    > {
        finalize_claim(
            self.state,
            &[
                FoundationalPerformanceBoundary::AuthoritativeExecution,
                FoundationalPerformanceBoundary::MaintenanceExecution,
                FoundationalPerformanceBoundary::Publication,
                FoundationalPerformanceBoundary::Delivery,
                FoundationalPerformanceBoundary::RetentionCompaction,
            ],
            &[FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt],
            FoundationalAuthoritativePerformanceClaim,
        )
    }
}

impl FoundationalSupportDerivedPerformanceClaimBuilder {
    pub fn finish(
        self,
    ) -> Result<
        FoundationalSupportDerivedPerformanceClaim,
        FoundationalPerformanceClaimConstructionDenial,
    > {
        finalize_claim(
            self.state,
            &[FoundationalPerformanceBoundary::SupportAssembly],
            &[
                FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim,
                FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim,
            ],
            FoundationalSupportDerivedPerformanceClaim,
        )
    }
}

impl FoundationalReplayMaterializationPerformanceClaimBuilder {
    pub fn finish(
        self,
    ) -> Result<
        FoundationalReplayMaterializationPerformanceClaim,
        FoundationalPerformanceClaimConstructionDenial,
    > {
        finalize_claim(
            self.state,
            &[
                FoundationalPerformanceBoundary::BoundaryMaterialization,
                FoundationalPerformanceBoundary::ReplayReconstruction,
                FoundationalPerformanceBoundary::RestoreRecovery,
            ],
            &[
                FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim,
                FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim,
            ],
            FoundationalReplayMaterializationPerformanceClaim,
        )
    }
}

impl FoundationalPolicyAdmissionPerformanceClaimBuilder {
    pub fn finish(
        self,
    ) -> Result<
        FoundationalPolicyAdmissionPerformanceClaim,
        FoundationalPerformanceClaimConstructionDenial,
    > {
        finalize_claim(
            self.state,
            &[
                FoundationalPerformanceBoundary::AuthoritativeExecution,
                FoundationalPerformanceBoundary::MaintenancePlanning,
                FoundationalPerformanceBoundary::MaintenanceExecution,
                FoundationalPerformanceBoundary::Publication,
                FoundationalPerformanceBoundary::Delivery,
                FoundationalPerformanceBoundary::RetentionCompaction,
                FoundationalPerformanceBoundary::RestoreRecovery,
            ],
            &[
                FoundationalPerformanceEvidenceStrength::CompileTimeContract,
                FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission,
                FoundationalPerformanceEvidenceStrength::ExplicitDebtDeferredClaim,
            ],
            FoundationalPolicyAdmissionPerformanceClaim,
        )
    }
}

fn finalize_claim<T>(
    state: FoundationalPerformanceClaimBuilderState,
    allowed_boundaries: &[FoundationalPerformanceBoundary],
    allowed_strengths: &[FoundationalPerformanceEvidenceStrength],
    build: impl FnOnce(FoundationalPerformanceClaimPayload) -> T,
) -> Result<T, FoundationalPerformanceClaimConstructionDenial> {
    finalize_claim_core(state, allowed_boundaries, allowed_strengths).map(build)
}

/// One compiled claim check. The claim constructor is the shell's only generic call.
#[inline(never)]
fn finalize_claim_core(
    mut state: FoundationalPerformanceClaimBuilderState,
    allowed_boundaries: &[FoundationalPerformanceBoundary],
    allowed_strengths: &[FoundationalPerformanceEvidenceStrength],
) -> Result<FoundationalPerformanceClaimPayload, FoundationalPerformanceClaimConstructionDenial> {
    let boundary = state
        .boundary
        .ok_or(FoundationalPerformanceClaimConstructionDenial::MissingBoundary)?;
    if !allowed_boundaries.contains(&boundary) {
        return Err(
            FoundationalPerformanceClaimConstructionDenial::BoundaryNotAllowedForClaimFamily,
        );
    }

    let evidence_strength = state
        .evidence_strength
        .ok_or(FoundationalPerformanceClaimConstructionDenial::MissingEvidenceStrength)?;
    if !allowed_strengths.contains(&evidence_strength) {
        return Err(
            FoundationalPerformanceClaimConstructionDenial::EvidenceStrengthNotAllowedForClaimFamily,
        );
    }

    let breadth_locality = state
        .breadth_locality
        .ok_or(FoundationalPerformanceClaimConstructionDenial::MissingBreadthLocality)?;
    let access_pattern = state
        .access_pattern
        .ok_or(FoundationalPerformanceClaimConstructionDenial::MissingAccessPattern)?;
    let execution_temperature = state
        .execution_temperature
        .ok_or(FoundationalPerformanceClaimConstructionDenial::MissingExecutionTemperature)?;
    let freshness_retention = state
        .freshness_retention
        .ok_or(FoundationalPerformanceClaimConstructionDenial::MissingFreshnessRetention)?;
    let fallback_debt = state
        .fallback_debt
        .ok_or(FoundationalPerformanceClaimConstructionDenial::MissingFallbackDebt)?;

    canonicalize_work_classes(&mut state.included_work);
    canonicalize_work_classes(&mut state.excluded_work);

    if state.included_work.is_empty() {
        return Err(FoundationalPerformanceClaimConstructionDenial::MissingIncludedWorkDisclosure);
    }
    if state.excluded_work.is_empty() {
        return Err(FoundationalPerformanceClaimConstructionDenial::MissingExcludedWorkDisclosure);
    }
    if state
        .included_work
        .iter()
        .any(|work_class| state.excluded_work.contains(work_class))
    {
        return Err(
            FoundationalPerformanceClaimConstructionDenial::OverlappingIncludedAndExcludedWorkDisclosure,
        );
    }

    let includes_optional_observation =
        state.included_work.iter().any(is_optional_observation_work);
    if includes_optional_observation {
        let Some(context) = state.observation_context.as_ref() else {
            return Err(FoundationalPerformanceClaimConstructionDenial::MissingObservationContext);
        };
        if !context.disposition().is_active() {
            return Err(
                FoundationalPerformanceClaimConstructionDenial::ObservationWorkRequiresActiveDisposition,
            );
        }
    }

    evaluate_performance_primitive_legality(
        boundary,
        evidence_strength,
        execution_temperature,
        freshness_retention,
        fallback_debt,
        &state.included_work,
        &state.excluded_work,
    )
    .map_err(FoundationalPerformanceClaimConstructionDenial::PrimitiveLegality)?;

    Ok(FoundationalPerformanceClaimPayload {
        boundary,
        evidence_strength,
        breadth_locality,
        access_pattern,
        execution_temperature,
        freshness_retention,
        fallback_debt,
        included_work: state.included_work,
        excluded_work: state.excluded_work,
        observation_context: state.observation_context,
    })
}

fn is_optional_observation_work(work: &FoundationalPerformanceWorkClass) -> bool {
    matches!(
        work,
        FoundationalPerformanceWorkClass::StructuralCounterCapture
            | FoundationalPerformanceWorkClass::DiagnosticFactCapture
            | FoundationalPerformanceWorkClass::DescriptiveLineageRecordMaintenance
            | FoundationalPerformanceWorkClass::ProvenanceFactCapture
            | FoundationalPerformanceWorkClass::ReplaySidecarMaintenance
    )
}

fn canonicalize_work_classes(work_classes: &mut Vec<FoundationalPerformanceWorkClass>) {
    work_classes.sort();
    work_classes.dedup();
}
