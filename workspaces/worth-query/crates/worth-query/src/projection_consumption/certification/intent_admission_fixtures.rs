use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::source::{
    ProjectionSourceCapabilityProfile, ProjectionSourceExecutionPosture,
};
use crate::projection_consumption::{
    declare_projection_consumption, ProjectMaterializedFacts, ProjectionConsumptionBindingContext,
    ProjectionConsumptionDeclaration, ProjectionConsumptionSource, ProjectionSourceFamily,
};
use worth_foundational::facade::{AspectKey, FieldKey};

pub(crate) fn intent_admission_admitted_projection_declaration(
) -> Result<ProjectionConsumptionDeclaration, IdentityDerivationDenial> {
    Ok(declaration(
        ProjectionSourceFamily::QueryReadReceipt,
        ProjectionSourceCapabilityProfile::QueryReadReceipt {
            execution_posture: ProjectionSourceExecutionPosture::Current,
        },
        "query-read:certification-admitted",
    )?)
}

pub(crate) fn intent_admission_warning_projection_declaration(
) -> Result<ProjectionConsumptionDeclaration, IdentityDerivationDenial> {
    Ok(declaration(
        ProjectionSourceFamily::QueryContextExecution,
        ProjectionSourceCapabilityProfile::QueryContextExecution {
            execution_posture: ProjectionSourceExecutionPosture::Current,
        },
        "query-context:certification-warning",
    )?)
}

fn declaration(
    family: ProjectionSourceFamily,
    profile: ProjectionSourceCapabilityProfile,
    source_identity: &str,
) -> Result<ProjectionConsumptionDeclaration, IdentityDerivationDenial> {
    Ok(declare_projection_consumption(
        ProjectionConsumptionSource::intent_admission_certification(
            family,
            profile,
            Some("query-digest".to_string()),
            Some("basis-digest".to_string()),
            Some("result-digest".to_string()),
            Some("shape-digest".to_string()),
            source_identity,
            Vec::new(),
        )?,
        projection_binding(source_identity),
        ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                FieldKey::new("field").expect("projection fact field segment should admit"),
                FieldKey::new("visible").expect("projection fact field segment should admit"),
            ]),
        ),
    )
    .expect("intent-admission projection declaration should build"))
}

fn projection_binding(source_identity: &str) -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::intent_admission_certification_binding(
        "shape-digest",
        "query-digest",
        "shape-digest",
        source_identity,
        "narrowed-shape-digest",
        "policy-digest",
        "tenant-schema-digest",
        vec![
            crate::authorized_projection::AuthorizedProjectionFieldPath::from_native_keys(
                AspectKey::new("field").expect("certification aspect key should be foundational"),
                FieldKey::new("visible").expect("certification field key should be foundational"),
            ),
        ],
    )
}
