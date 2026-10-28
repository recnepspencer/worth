use super::report::FoundationalPerformanceProductionReadinessReport;
use super::vocabulary::{
    FoundationalPerformanceCertifiedSurface, FoundationalPerformanceCompileFailBoundary,
    FoundationalPerformanceRuntimeAdoptionPressure, FoundationalPerformanceRuntimeAssumption,
    FoundationalPerformanceRuntimeNonAssumption, FoundationalPerformanceSyntheticRuntimePressure,
    FoundationalPerformanceWORTHProofApi, FoundationalPerformanceWORTHProofForbiddenSurface,
    FoundationalPerformanceWORTHProofSurface,
};
use crate::performance::FoundationalPerformanceMilestone8PhaseGate;
use crate::performance_api::FoundationalPerformancePublicLane;

impl FoundationalPerformanceProductionReadinessReport {
    pub fn passes_readiness_checklist(&self) -> bool {
        self.has_all_certified_surfaces()
            && self.has_evidence_for_each_certified_surface()
            && self.has_all_synthetic_pressures()
            && self.has_all_compile_fail_boundaries()
            && self.has_all_required_worth_proof_surfaces()
            && self.has_named_worth_proof_api_appendix()
            && self.has_all_forbidden_worth_proof_surfaces()
            && self.has_runtime_assumption_boundary()
            && self.has_named_residual_debt()
            && self.has_runtime_adoption_pressure_boundary()
            && self.has_linear_phase_gates()
            && self.has_exact_public_surface_inventory()
            && self.has_exact_documentation_surface_inventory()
            && self.has_exact_public_surface_documentation_coverage()
    }

    fn has_all_certified_surfaces(&self) -> bool {
        exact_inventory_matches(
            self.certified_surfaces(),
            &[
                FoundationalPerformanceCertifiedSurface::PrimitiveAndCategoryLaw,
                FoundationalPerformanceCertifiedSurface::ClaimBoundaryAndEvidenceStrengthLaw,
                FoundationalPerformanceCertifiedSurface::LayoutIntentAndRepresentationFreedom,
                FoundationalPerformanceCertifiedSurface::PolicyAdmissionAndBudgetLaw,
                FoundationalPerformanceCertifiedSurface::CanonicalBundleAndCounterReceiptLaw,
                FoundationalPerformanceCertifiedSurface::ReportAttachmentAndMaterializationLaw,
                FoundationalPerformanceCertifiedSurface::CertifiedBundleAndReadmissionLaw,
            ],
        )
    }

    fn has_evidence_for_each_certified_surface(&self) -> bool {
        self.certified_surfaces().iter().all(|surface| {
            self.certified_surface_evidence()
                .iter()
                .filter(|evidence| evidence.surface() == *surface)
                .count()
                == 1
        })
    }

    fn has_all_synthetic_pressures(&self) -> bool {
        exact_inventory_matches(
            self.synthetic_pressures(),
            &[
            FoundationalPerformanceSyntheticRuntimePressure::PrimitiveFamilyNonSubstitution,
            FoundationalPerformanceSyntheticRuntimePressure::ClaimStrengthAndLaneCollapseRejection,
            FoundationalPerformanceSyntheticRuntimePressure::RepresentationEquivalenceOverclaimRejection,
            FoundationalPerformanceSyntheticRuntimePressure::PreExecutionMasqueradeRejection,
            FoundationalPerformanceSyntheticRuntimePressure::CanonicalCounterLoweringRejection,
            FoundationalPerformanceSyntheticRuntimePressure::HiddenSupportExpansionRejection,
            FoundationalPerformanceSyntheticRuntimePressure::CertifiedProofLaneBoundary,
            FoundationalPerformanceSyntheticRuntimePressure::GroupedStrongerLaneBoundary,
            ],
        )
    }

    fn has_all_compile_fail_boundaries(&self) -> bool {
        exact_inventory_matches(
            self.compile_fail_boundaries(),
            &[
            FoundationalPerformanceCompileFailBoundary::PrimitiveFamiliesAndCommonPathBoundaries,
            FoundationalPerformanceCompileFailBoundary::ClaimLaneBoundaries,
            FoundationalPerformanceCompileFailBoundary::LayoutAttachmentBoundaries,
            FoundationalPerformanceCompileFailBoundary::PolicyPreExecutionBoundaries,
            FoundationalPerformanceCompileFailBoundary::BundleAndCounterReceiptLoweringBoundaries,
            FoundationalPerformanceCompileFailBoundary::ReportMaterializationBoundaries,
            FoundationalPerformanceCompileFailBoundary::CertifiedBundleAndReadmissionProofLane,
            FoundationalPerformanceCompileFailBoundary::PerformanceReadinessRequiresCertifiedArtifact,
            FoundationalPerformanceCompileFailBoundary::PerformanceReadinessAuthorityCannotBeMinted,
            FoundationalPerformanceCompileFailBoundary::GroupedStrongerLaneRequiresCertifiedReadiness,
            ],
        )
    }

    fn has_all_required_worth_proof_surfaces(&self) -> bool {
        exact_inventory_matches(
            self.worth_proof_required_surfaces(),
            &[
                FoundationalPerformanceWORTHProofSurface::ProductionReadinessCertificationArtifact,
                FoundationalPerformanceWORTHProofSurface::AuthorityWitness,
                FoundationalPerformanceWORTHProofSurface::ProofFromAuthorityWitness,
                FoundationalPerformanceWORTHProofSurface::ArtifactWithProofsAndCurrentBasis,
            ],
        )
    }

    fn has_named_worth_proof_api_appendix(&self) -> bool {
        exact_inventory_matches(
            self.worth_proof_api_appendix(),
            &[
                FoundationalPerformanceWORTHProofApi::AuthorityWitnessFromAuthorityMarker,
                FoundationalPerformanceWORTHProofApi::ProofFromAuthorityWitness,
                FoundationalPerformanceWORTHProofApi::ArtifactWithProofsAndCurrentBasis,
            ],
        )
    }

    fn has_all_forbidden_worth_proof_surfaces(&self) -> bool {
        exact_inventory_matches(
            self.worth_proof_forbidden_surfaces(),
            &[
            FoundationalPerformanceWORTHProofForbiddenSurface::PlainPerformanceVocabulary,
            FoundationalPerformanceWORTHProofForbiddenSurface::PlainPerformanceLowerLaneArtifacts,
            FoundationalPerformanceWORTHProofForbiddenSurface::PlainPerformanceReportPlanningVocabulary,
            ],
        )
    }

    fn has_runtime_assumption_boundary(&self) -> bool {
        let assumptions: std::collections::BTreeSet<_> =
            self.assumptions().iter().copied().collect();
        let non_assumptions: std::collections::BTreeSet<_> =
            self.non_assumptions().iter().copied().collect();

        assumptions
            == std::collections::BTreeSet::from([
                FoundationalPerformanceRuntimeAssumption::WORTHProofAuthorityLaneRemainsAvailable,
                FoundationalPerformanceRuntimeAssumption::ProfileLawRemainsAuthorityForReportElision,
                FoundationalPerformanceRuntimeAssumption::PhaseEvidencePathsRemainOwnedWithinFoundational,
            ])
            && assumptions.len() == self.assumptions().len()
            && non_assumptions
                == std::collections::BTreeSet::from([
                    FoundationalPerformanceRuntimeNonAssumption::WorkspaceWideTelemetryEngineIsOwnedHere,
                ])
            && non_assumptions.len() == self.non_assumptions().len()
    }

    fn has_named_residual_debt(&self) -> bool {
        let residual_debt: std::collections::BTreeSet<_> =
            self.residual_debt().iter().copied().collect();

        residual_debt.is_empty() && self.residual_debt().is_empty()
    }

    fn has_runtime_adoption_pressure_boundary(&self) -> bool {
        let expected_pressures = [
                FoundationalPerformanceRuntimeAdoptionPressure::CrossCrateMeaningParityMatrix,
                FoundationalPerformanceRuntimeAdoptionPressure::CertifiedBundleSourceCompatibilityMatrix,
            ];
        let expected_evidence = std::collections::BTreeSet::from([
            (
                FoundationalPerformanceRuntimeAdoptionPressure::CrossCrateMeaningParityMatrix,
                "tests/worth_foundational/fast/certification/performance/runtime_parity.rs",
            ),
            (
                FoundationalPerformanceRuntimeAdoptionPressure::CertifiedBundleSourceCompatibilityMatrix,
                "tests/worth_foundational/fast/certification/performance/runtime_parity.rs",
            ),
        ]);
        let actual_evidence: std::collections::BTreeSet<_> = self
            .runtime_adoption_pressure_evidence()
            .iter()
            .map(|row| (row.pressure(), row.evidence_path()))
            .collect();

        exact_inventory_matches(self.runtime_adoption_pressures(), &expected_pressures)
            && actual_evidence == expected_evidence
            && actual_evidence.len() == self.runtime_adoption_pressure_evidence().len()
    }

    fn has_linear_phase_gates(&self) -> bool {
        self.phase_gates()
            .iter()
            .map(|evidence| evidence.gate())
            .eq([
                FoundationalPerformanceMilestone8PhaseGate::PrimitiveAndCategoryLaw,
                FoundationalPerformanceMilestone8PhaseGate::ClaimBoundaryAndEvidenceStrengthLaw,
                FoundationalPerformanceMilestone8PhaseGate::LayoutIntentAccessAndAllocationLaw,
                FoundationalPerformanceMilestone8PhaseGate::RuntimePolicyBudgetAndFallbackLaw,
                FoundationalPerformanceMilestone8PhaseGate::CanonicalBasisCounterAndComparisonLaw,
                FoundationalPerformanceMilestone8PhaseGate::AttachmentMaterializationAndBundleLaw,
                FoundationalPerformanceMilestone8PhaseGate::ProductionReadiness,
                FoundationalPerformanceMilestone8PhaseGate::FeatureDocsCrateDocIntegrationAndPublicationClosure,
            ])
    }

    fn has_exact_public_surface_inventory(&self) -> bool {
        let actual_entries: std::collections::BTreeSet<_> = self
            .public_surface_inventory()
            .iter()
            .map(|entry| {
                (
                    entry.path(),
                    entry.lane(),
                    entry.teaches(),
                    entry.does_not_hide(),
                )
            })
            .collect();
        let expected_entries = std::collections::BTreeSet::from([
            (
                "worth_foundational::performance_api::common_path",
                FoundationalPerformancePublicLane::CommonPath,
                "common-path performance claim authoring, layout intent definition, and primitive legality entrypoints",
                "lower-lane canonical lowering, explicit receipts, or stronger readiness proof",
            ),
            (
                "worth_foundational::performance_api::lower_lane::basis",
                FoundationalPerformancePublicLane::LowerLane,
                "inspectable canonical bundle, canonical-basis preparation, digest-ready lowering, contract name, counter spec, and comparison vocabulary",
                "common-path claim authoring or stronger proof-bearing certification",
            ),
            (
                "worth_foundational::performance_api::lower_lane::policy",
                FoundationalPerformancePublicLane::LowerLane,
                "inspectable budget and policy-admission receipt vocabulary",
                "executed counter-backed truth",
            ),
            (
                "worth_foundational::performance_api::lower_lane::receipts",
                FoundationalPerformancePublicLane::LowerLane,
                "inspectable counter-backed execution receipt vocabulary",
                "support/report materialization or stronger readiness proof",
            ),
            (
                "worth_foundational::performance_api::lower_lane::reports",
                FoundationalPerformancePublicLane::LowerLane,
                "inspectable attachment targets, report requests, report plans, and explicit materialization vocabulary",
                "common-path claim authoring or stronger readiness certification",
            ),
            (
                "worth_foundational::performance_api::lower_lane",
                FoundationalPerformancePublicLane::LowerLane,
                "grouped lower-lane performance lowering and inspection topology",
                "common-path authoring or stronger readiness certification",
            ),
            (
                "worth_foundational::performance_api::stronger_lane",
                FoundationalPerformancePublicLane::StrongerLane,
                "grouped stronger lane for certified performance bundles, trust-boundary readmission, and readiness certification",
                "common-path authoring or lower-lane inspection",
            ),
            (
                "worth_foundational::performance_api::stronger_lane::certified",
                FoundationalPerformancePublicLane::StrongerLane,
                "proof-bearing certified performance bundles and trust-boundary readmission over current-basis hot-path receipts and support-expansion reports",
                "plain lower-lane receipt/report inspection or readiness-only certification",
            ),
            (
                "worth_foundational::performance_api::stronger_lane::readiness",
                FoundationalPerformancePublicLane::StrongerLane,
                "production-readiness certification and proof-bearing readiness requirement",
                "plain readiness report or certified bundle proof progression",
            ),
        ]);

        actual_entries == expected_entries
            && actual_entries.len() == self.public_surface_inventory().len()
    }

    fn has_exact_documentation_surface_inventory(&self) -> bool {
        let docs: std::collections::BTreeSet<_> = self
            .documentation_surface_inventory()
            .iter()
            .copied()
            .collect();

        docs == std::collections::BTreeSet::from([
            "docs/README.md",
            "docs/performance/README.md",
            "docs/performance/common-performance-claims-and-layout-intent.md",
            "docs/performance/policy-admission-receipts.md",
            "docs/performance/canonical-bundles-and-comparison.md",
            "docs/performance/counter-backed-performance-receipts.md",
            "docs/performance/performance-report-planning-and-materialization.md",
            "docs/performance/certified-and-readmitted-performance-bundles.md",
            "docs/performance/grouped-public-lanes-and-stronger-readiness.md",
            "docs/performance/performance-production-readiness.md",
        ]) && docs.len() == self.documentation_surface_inventory().len()
    }

    fn has_exact_public_surface_documentation_coverage(&self) -> bool {
        let surface_paths: std::collections::BTreeSet<_> = self
            .public_surface_inventory()
            .iter()
            .map(|entry| entry.path())
            .collect();
        let documented_surface_paths: std::collections::BTreeSet<_> = self
            .public_surface_documentation_coverage()
            .iter()
            .map(|row| row.public_surface_path())
            .collect();
        let documentation_paths: std::collections::BTreeSet<_> = self
            .documentation_surface_inventory()
            .iter()
            .copied()
            .collect();

        surface_paths == documented_surface_paths
            && self.public_surface_documentation_coverage().len() == documented_surface_paths.len()
            && self
                .public_surface_documentation_coverage()
                .iter()
                .all(|row| documentation_paths.contains(row.primary_documentation_path()))
    }
}

fn exact_inventory_matches<T>(actual: &[T], expected: &[T]) -> bool
where
    T: Copy + Ord,
{
    let actual_set: std::collections::BTreeSet<_> = actual.iter().copied().collect();
    let expected_set: std::collections::BTreeSet<_> = expected.iter().copied().collect();

    actual.len() == expected.len() && actual_set == expected_set
}

#[cfg(test)]
mod tests {
    use crate::performance::readiness::foundational_performance_milestone8_readiness_report;

    #[test]
    fn readiness_checklist_rejects_public_surface_inventory_drift() {
        let report = foundational_performance_milestone8_readiness_report()
            .clone()
            .with_public_surface_inventory(vec![]);

        assert!(!report.passes_readiness_checklist());
    }

    #[test]
    fn readiness_checklist_rejects_runtime_adoption_evidence_drift() {
        let readiness = foundational_performance_milestone8_readiness_report();
        let duplicated_row = readiness.runtime_adoption_pressure_evidence()[0];
        let report = readiness
            .clone()
            .with_runtime_adoption_pressure_evidence(vec![duplicated_row, duplicated_row]);

        assert!(!report.passes_readiness_checklist());
    }
}
