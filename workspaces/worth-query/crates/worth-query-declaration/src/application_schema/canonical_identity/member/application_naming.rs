use crate::application_naming::ErasedApplicationNameScheme;
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationNameScheme,
) {
    basis.text(format!("{prefix}.kind"), "application-name-scheme");
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
