pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::claim_receipt_evidence_boundary_surface,
    worth_foundational::claim_support_only_boundary_surface,
    worth_foundational::foundational_boundary_canonical_basis_entries,
    worth_foundational::plan_artifact_boundary_bundle,
    worth_foundational::plan_descriptive_boundary_materialization,
    worth_foundational::prepare_materialized_boundary_artifact_for_canonical_basis,
    worth_foundational::prepare_materialized_boundary_bundle_for_canonical_basis,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalBasisDomain,
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind,
    worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue,
    worth_foundational::CanonicalComparisonOutcome,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalBoundaryArtifactSurface,
    worth_foundational::FoundationalBoundaryMaterializationSeam,
    worth_foundational::FoundationalBoundaryMaterializationSource,
    worth_foundational::FoundationalBoundaryReceiptSurface,
    worth_foundational::FoundationalBoundaryReportSurface,
    worth_foundational::FoundationalBoundarySummarySurface,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

pub(crate) use {
    crate::certification::ordinary_context::boundary_artifacts::support::exact_compare,
    crate::certification::ordinary_context::boundary_artifacts::support::materialize_projection_artifact,
    crate::certification::ordinary_context::boundary_artifacts::support::materialized_profile,
    crate::certification::ordinary_context::boundary_artifacts::support::version,
};

pub(crate) fn named_entry<'a>(
    entries: &'a [CanonicalBasisEntry],
    name: &str,
) -> &'a CanonicalBasisEntry {
    entries
        .iter()
        .find(|entry| {
            entry.locus()
                == &worth_foundational::CanonicalBasisLocus::Named(name.to_string().into())
        })
        .unwrap_or_else(|| panic!("missing basis entry {name}"))
}
