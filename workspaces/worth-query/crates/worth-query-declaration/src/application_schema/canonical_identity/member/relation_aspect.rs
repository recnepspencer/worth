use super::field::{append_field_contract, FieldContract};
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::{
    ApplicationRelationFieldDefinition, ErasedApplicationRelationAspectDefinition,
};

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationRelationAspectDefinition,
) {
    let aspect = definition.parts();
    basis.text(format!("{prefix}.kind"), "relation-aspect");
    basis.text(format!("{prefix}.relation"), &aspect.relation);
    basis.text(format!("{prefix}.from"), &aspect.from);
    basis.text(format!("{prefix}.to"), &aspect.to);
    basis.text(format!("{prefix}.aspect"), &aspect.aspect);
    basis.u64(format!("{prefix}.identity"), aspect.identity.0);
    basis.u64(format!("{prefix}.revision"), aspect.revision.0);
    basis.text(
        format!("{prefix}.absence"),
        match aspect.absence {
            worth_foundational::facade::AbsenceLaw::Required => "required",
            worth_foundational::facade::AbsenceLaw::Optional => "optional",
            worth_foundational::facade::AbsenceLaw::Defaulted => "defaulted",
        },
    );
    basis.usize(format!("{prefix}.field-count"), aspect.fields.len());
    for (index, field) in aspect.fields.iter().enumerate() {
        append_relation_field(
            basis,
            &format!("{prefix}.field[{index}]"),
            &aspect.relation,
            &aspect.aspect,
            field,
        );
    }
}

pub(in crate::application_schema::canonical_identity) fn append_relation_field(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    relation: &str,
    aspect: &str,
    field: &ApplicationRelationFieldDefinition,
) {
    basis.text(format!("{prefix}.kind"), "relation-field");
    basis.text(format!("{prefix}.relation"), relation);
    basis.text(format!("{prefix}.aspect"), aspect);
    append_field_contract(
        basis,
        prefix,
        FieldContract {
            field: &field.field,
            presence: field.presence,
            scalar_family: field.scalar_family,
            value_type: &field.value_type,
            canonical: field.canonical.as_ref(),
            unit: field.unit.as_deref(),
            frame: field.frame.as_deref(),
            writable: field.writable,
            equality_queryable: false,
            unique: false,
        },
    );
}
