use crate::application_naming::{ErasedApplicationGroupNameScheme, ErasedApplicationNameScheme};
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationNameScheme,
) {
    basis.text(format!("{prefix}.kind"), "application-name-scheme");
    append_scheme_fields(basis, prefix, definition);
}

pub(super) fn append_group(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationGroupNameScheme,
) {
    basis.text(format!("{prefix}.kind"), "application-group-name-scheme");
    basis.text(
        format!("{prefix}.continuity-rule"),
        definition.continuity_rule().canonical_name(),
    );
    basis.optional_text(format!("{prefix}.anchor-role"), definition.anchor_role());
    basis.optional_text(
        format!("{prefix}.boundary-role"),
        definition.boundary_role(),
    );
    append_scheme_fields(basis, prefix, definition.name_scheme());
}

fn append_scheme_fields(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationNameScheme,
) {
    basis.text(format!("{prefix}.scheme-id"), definition.scheme_id());
    basis.text(
        format!("{prefix}.subject-kind"),
        definition.subject_kind().canonical_name(),
    );
    basis.text(format!("{prefix}.subject-name"), definition.subject_name());
    basis.bool(
        format!("{prefix}.split-capable"),
        definition.split_capable(),
    );
    basis.optional_text(
        format!("{prefix}.split-discriminator"),
        definition.split_discriminator_identity(),
    );
    basis.bool(
        format!("{prefix}.require-authored-occurrence"),
        definition.require_authored_occurrence(),
    );
    // Absent unless set, so every parent-inclusive scheme keeps its identity.
    if definition.structural_identity() {
        basis.bool(format!("{prefix}.structural-identity"), true);
    }
    for (index, role) in definition.role_order().iter().enumerate() {
        basis.text(format!("{prefix}.role[{index}]"), role);
    }
    for (index, dependency) in definition.role_dependencies().iter().enumerate() {
        basis.text(
            format!("{prefix}.dependency[{index}].role"),
            dependency.role(),
        );
        basis.text(
            format!("{prefix}.dependency[{index}].depends-on"),
            dependency.depends_on(),
        );
    }
    for (index, role) in definition
        .discriminator_referenced_roles()
        .iter()
        .enumerate()
    {
        basis.text(format!("{prefix}.discriminator-role[{index}]"), role);
    }
}
