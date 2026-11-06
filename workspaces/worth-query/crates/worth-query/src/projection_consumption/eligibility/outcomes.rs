use super::*;

pub(super) fn deferred_projection_consumption(
    declaration: &ProjectionConsumptionDeclaration,
    reason: DeferredProjectionConsumptionReason,
    counters: ProjectionConsumptionEligibilityCounters,
) -> Result<ProjectionConsumptionEligibility, IdentityDerivationDenial> {
    Ok(ProjectionConsumptionEligibility::Deferred(DeferredProjectionConsumption {
        declaration_digest: declaration.declaration_digest().to_string(),
        source_family: declaration.source().family(),
        reason,
        counters,
        trace: ProjectionConsumptionEligibilityTrace {
            rule_label: "fact_family_deferred_for_source",
            explanation:
                "the requested fact family remains deferred for this source family in the current milestone slice",
        },
        failure_digest: compose_eligibility_deferred_failure_digest(
            declaration.declaration_digest(),
        )?,
    }))
}

pub(super) fn source_mismatched_projection_consumption(
    declaration: &ProjectionConsumptionDeclaration,
    requested_fact_kind: ProjectionFactKind,
    counters: ProjectionConsumptionEligibilityCounters,
) -> Result<ProjectionConsumptionEligibility, IdentityDerivationDenial> {
    Ok(ProjectionConsumptionEligibility::SourceMismatch(
        SourceMismatchedProjectionConsumption {
            declaration_digest: declaration.declaration_digest().to_string(),
            source_family: declaration.source().family(),
            requested_fact_kind,
            counters,
            trace: ProjectionConsumptionEligibilityTrace {
                rule_label: "source_family_does_not_prove_fact_family",
                explanation: "the named source family does not prove the requested fact family",
            },
            failure_digest: compose_eligibility_source_mismatch_failure_digest(
                declaration.declaration_digest(),
                declaration.source().family(),
                requested_fact_kind,
            )?,
        },
    ))
}

pub(super) fn denied_projection_consumption(
    declaration: &ProjectionConsumptionDeclaration,
    reason: ProjectionConsumptionDenialReason,
    counters: ProjectionConsumptionEligibilityCounters,
) -> Result<ProjectionConsumptionEligibility, IdentityDerivationDenial> {
    Ok(ProjectionConsumptionEligibility::Denied(DeniedProjectionConsumption {
        declaration_digest: declaration.declaration_digest().to_string(),
        reason,
        counters,
        trace: ProjectionConsumptionEligibilityTrace {
            rule_label: "authorized_projection_visibility_denial",
            explanation:
                "the requested field-backed fact is not visible in the bound authorized projection",
        },
        failure_digest: compose_eligibility_denied_failure_digest(declaration.declaration_digest())?,
    }))
}

pub(super) fn admitted_projection_consumption(
    declaration: &ProjectionConsumptionDeclaration,
    counters: ProjectionConsumptionEligibilityCounters,
    warnings: Vec<ProjectionConsumptionWarningKind>,
) -> Result<ProjectionConsumptionEligibility, IdentityDerivationDenial> {
    let admitted = AdmittedProjectionConsumption {
        declaration: declaration.clone(),
        declaration_digest: declaration.declaration_digest().to_string(),
        query_digest: declaration.source().query_digest().unwrap_or("").to_string(),
        basis_digest: declaration.source().basis_digest().unwrap_or("").to_string(),
        result_shape_digest: declaration.binding().result_shape_digest().to_string(),
        authorized_projection_identity: declaration
            .binding()
            .authorized_projection_identity()
            .to_string(),
        warning_kinds: warnings.clone(),
        counters: counters.clone(),
        trace: ProjectionConsumptionEligibilityTrace {
            rule_label: "all_requested_fact_families_admitted",
            explanation:
                "every requested fact family remained within the supported and visible surface for the declared source family",
        },
        eligibility_digest: compose_eligibility_admitted_digest(
            declaration.declaration_digest(),
            warnings.len(),
        )?,
    };
    Ok(if warnings.is_empty() {
        ProjectionConsumptionEligibility::Admitted(admitted)
    } else {
        let warning_digest = compose_eligibility_warning_kinds_digest(&warnings)?;
        ProjectionConsumptionEligibility::AdmittedWithWarnings(
            admitted,
            ProjectionConsumptionWarnings {
                warning_kinds: warnings,
                warning_digest,
            },
        )
    })
}
