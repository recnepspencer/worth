//! Canonical created-row meaning, preserving the order and names of older initializer forms.
use crate::application_schema::{
    canonical_basis::ApplicationSchemaCanonicalBasis,
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyCreation,
};
#[cfg(test)]
mod tests;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    creation: &ApplicationOperationBodyCreation,
) {
    if !creation.children().is_empty() {
        basis.usize(
            format!("{prefix}.created.children"),
            creation.children().len(),
        );
        for (index, child) in creation.children().iter().enumerate() {
            let child_prefix = format!("{prefix}.created.child[{index}]");
            basis.text(format!("{child_prefix}.parent"), &child.parent_key);
            append(basis, &child_prefix, &child.creation);
        }
    }
    if creation.reuses_current_if_unchanged() {
        basis.text(
            format!("{prefix}.created.reuse-current"),
            "complete-content-v1",
        );
    }
    if !creation.links().is_empty() {
        basis.usize(format!("{prefix}.created.links"), creation.links().len());
        for (index, link) in creation.links().iter().enumerate() {
            basis.text(
                format!("{prefix}.created.link[{index}]"),
                &link.canonical_record(),
            );
        }
    }
    basis.text(format!("{prefix}.created.entity"), creation.entity());
    basis.text(format!("{prefix}.created.key"), creation.key());
    if let Some(bound) = creation.subject_impact_bound() {
        basis.usize(
            format!("{prefix}.created.subject-impact-bound"),
            bound.get(),
        );
    }
    if let Some(enrollment) = creation.enrollment() {
        basis.text(
            format!("{prefix}.created.workflow-enrollment"),
            &enrollment.canonical_record(),
        );
    }
    if let Some(lifecycle) = creation.signing_lifecycle() {
        basis.text(
            format!("{prefix}.created.signing-lifecycle"),
            &lifecycle.canonical_record(),
        );
    }
    if let Some(signing) = creation.signing() {
        basis.text(
            format!("{prefix}.created.signing"),
            &signing.canonical_record(),
        );
    }
    basis.usize(format!("{prefix}.created.fields"), creation.fields().len());
    for (index, step) in creation.fields().iter().enumerate() {
        let field = format!("{prefix}.created.field[{index}]");
        basis.text(
            format!("{field}.kind"),
            match step {
                Field::Literal { .. } => "literal",
                Field::Input { .. } => "input",
                Field::Computed { .. } => "computed-value",
                Field::OptionalInput { .. } => "optional-input",
                Field::ScopeField { .. } => "scope-field",
                Field::RelatedField { .. } => "related-field",
                Field::PresentScopeField { .. } => "present-scope-field",
                Field::CapturedScopeField { .. } => "classified-scope-field",
                Field::CapturedPathField { .. } => "classified-path-field",
                Field::SigningProvenance { .. } => "signing-provenance",
                Field::PrincipalIdentity { .. } => "principal-identity",
            },
        );
        basis.text(format!("{field}.entity"), step.field().entity());
        basis.text(format!("{field}.aspect"), step.field().aspect());
        basis.text(format!("{field}.field"), step.field().field());
        basis.text(format!("{field}.binding"), step.binding().as_str());
        match step {
            Field::CapturedPathField {
                source,
                scalar_family,
                role,
                target_presence,
                ..
            } => {
                basis.text(format!("{field}.path-source"), &source.canonical_record());
                basis.text(
                    format!("{field}.source-family"),
                    scalar_family.canonical_name(),
                );
                basis.text(format!("{field}.capture-role"), role.canonical_name());
                basis.text(
                    format!("{field}.target-presence"),
                    target_presence.canonical_name(),
                );
            }
            Field::Computed { value, .. } => basis.text(format!("{field}.computed-value"), value),
            Field::Literal { value, .. } => basis.value(
                format!("{field}.value"),
                worth_foundational::facade::canonical_basis_value_for_aspect_value(value),
            ),
            Field::Input {
                member,
                scalar_family,
                ..
            }
            | Field::OptionalInput {
                member,
                scalar_family,
                ..
            } => {
                basis.text(format!("{field}.input-member"), member);
                basis.text(
                    format!("{field}.input-family"),
                    scalar_family.canonical_name(),
                );
            }
            Field::ScopeField {
                source,
                scalar_family,
                ..
            }
            | Field::PresentScopeField {
                source,
                scalar_family,
                ..
            }
            | Field::CapturedScopeField {
                source,
                scalar_family,
                ..
            } => {
                basis.text(format!("{field}.source.entity"), source.entity());
                basis.text(format!("{field}.source.aspect"), source.aspect());
                basis.text(format!("{field}.source.field"), source.field());
                basis.text(
                    format!("{field}.source-family"),
                    scalar_family.canonical_name(),
                );
                if let Field::CapturedScopeField {
                    source_presence,
                    role,
                    ..
                } = step
                {
                    basis.text(
                        format!("{field}.source-presence"),
                        source_presence.canonical_name(),
                    );
                    basis.text(format!("{field}.capture-role"), role.canonical_name());
                }
            }
            Field::RelatedField {
                source,
                scalar_family,
                ..
            } => {
                basis.text(
                    format!("{field}.related-source"),
                    &source.canonical_record(),
                );
                basis.text(
                    format!("{field}.source-family"),
                    scalar_family.canonical_name(),
                );
            }
            Field::SigningProvenance { .. } => {}
            Field::PrincipalIdentity {
                source,
                scalar_family,
                mutation_binding,
                principal_binding,
                ..
            } => {
                basis.text(
                    format!("{field}.principal.mutation-binding"),
                    mutation_binding,
                );
                basis.text(format!("{field}.principal.binding"), principal_binding);
                basis.text(format!("{field}.principal.entity"), source.entity());
                basis.text(format!("{field}.principal.aspect"), source.aspect());
                basis.text(format!("{field}.principal.field"), source.field());
                basis.text(
                    format!("{field}.principal.family"),
                    scalar_family.canonical_name(),
                );
            }
        }
    }
}
