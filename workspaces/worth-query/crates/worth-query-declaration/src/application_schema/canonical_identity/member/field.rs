use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::ApplicationSchemaMember;
use crate::application_schema::{ApplicationCanonicalValueContract, ApplicationFieldPresence};
use worth_foundational::facade::ScalarAspectType;
mod value_shape;

pub(super) fn append_schema_field(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    let ApplicationSchemaMember::Field {
        entity,
        aspect,
        field,
        presence,
        scalar_family,
        value_type,
        canonical,
        unit,
        frame,
        writable,
        equality_queryable,
        unique,
    } = member
    else {
        unreachable!("field lowering requires a field member")
    };
    basis.text(format!("{prefix}.kind"), "field");
    basis.text(format!("{prefix}.entity"), entity);
    basis.text(format!("{prefix}.aspect"), aspect);
    append_field_contract(
        basis,
        prefix,
        FieldContract {
            field,
            presence: *presence,
            scalar_family: *scalar_family,
            value_type,
            canonical: canonical.as_ref(),
            unit: unit.as_deref(),
            frame: frame.as_deref(),
            writable: *writable,
            equality_queryable: *equality_queryable,
            unique: *unique,
        },
    );
}

pub(super) struct FieldContract<'a> {
    pub field: &'a str,
    pub presence: ApplicationFieldPresence,
    pub scalar_family: ScalarAspectType,
    pub value_type: &'a str,
    pub canonical: Option<&'a ApplicationCanonicalValueContract>,
    pub unit: Option<&'a str>,
    pub frame: Option<&'a str>,
    pub writable: bool,
    pub equality_queryable: bool,
    pub unique: bool,
}

pub(super) fn append_field_contract(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    field: FieldContract<'_>,
) {
    basis.text(format!("{prefix}.field"), field.field);
    basis.text(
        format!("{prefix}.presence"),
        field.presence.canonical_name(),
    );
    basis.text(
        format!("{prefix}.scalar-family"),
        field.scalar_family.canonical_name(),
    );
    basis.text(format!("{prefix}.value-type"), field.value_type);
    if let Some(contract) = field.canonical {
        value_shape::append_contract(basis, prefix, contract);
    }
    basis.optional_text(format!("{prefix}.unit"), field.unit);
    basis.optional_text(format!("{prefix}.frame"), field.frame);
    basis.bool(format!("{prefix}.writable"), field.writable);
    basis.bool(
        format!("{prefix}.equality-queryable"),
        field.equality_queryable,
    );
    basis.bool(format!("{prefix}.unique"), field.unique);
}
