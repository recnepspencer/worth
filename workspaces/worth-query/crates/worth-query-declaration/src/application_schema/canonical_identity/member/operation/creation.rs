//! Canonical created-row meaning, preserving the order and names of older initializer forms.
use crate::application_schema::{
    canonical_basis::ApplicationSchemaCanonicalBasis,
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyCreation,
};

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    creation: &ApplicationOperationBodyCreation,
) {
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
                Field::OptionalInput { .. } => "optional-input",
                Field::ScopeField { .. } => "scope-field",
                Field::PresentScopeField { .. } => "present-scope-field",
                Field::SigningProvenance { .. } => "signing-provenance",
                Field::PrincipalIdentity { .. } => "principal-identity",
            },
        );
        basis.text(format!("{field}.entity"), step.field().entity());
        basis.text(format!("{field}.aspect"), step.field().aspect());
        basis.text(format!("{field}.field"), step.field().field());
        basis.text(format!("{field}.binding"), step.binding().as_str());
        match step {
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
            } => {
                basis.text(format!("{field}.source.entity"), source.entity());
                basis.text(format!("{field}.source.aspect"), source.aspect());
                basis.text(format!("{field}.source.field"), source.field());
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
